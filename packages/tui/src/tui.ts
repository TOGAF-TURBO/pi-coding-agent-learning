/**
 * @fileoverview TUI — 最小化终端 UI 引擎，基于差异渲染。
 *
 * 核心概念：
 *   - Component：渲染单元，通过 render() 返回字符串数组
 *   - TUI 类：管理组件树、输入处理、差异计算和终端输出
 *   - 差异渲染：仅重绘变化的行，最小化终端 I/O
 *   - 输入处理：将终端原始输入解码为按键事件
 *   - Kitty 图片协议：支持终端内嵌图片（用于 Read 工具的图片输出）
 *   - Overlay：浮层组件（如选择器、对话框）
 */

import * as fs from "node:fs";
import * as os from "node:os";
import * as path from "node:path";
import { performance } from "node:perf_hooks";
import { isKeyRelease, matchesKey } from "./keys.js";
import type { Terminal } from "./terminal.js";
import { deleteKittyImage, getCapabilities, isImageLine, setCellDimensions } from "./terminal-image.js";
import { extractSegments, normalizeTerminalOutput, sliceByColumn, sliceWithWidth, visibleWidth } from "./utils.js";

// Kitty 图形协议的转义序列前缀，用于在终端输出中嵌入图片
const KITTY_SEQUENCE_PREFIX = "\x1b_G";

/**
 * 从单行文本中提取 Kitty 图片协议的图片 ID
 * 解析格式为 \x1b_G...;i=<id>;... 的转义序列，提取其中的 i 参数值
 * @param line - 要解析的行文本
 * @returns 提取到的图片 ID 数组（通常 0 或 1 个），ID 无效或不存在时返回空数组
 */
function extractKittyImageIds(line: string): number[] {
	const sequenceStart = line.indexOf(KITTY_SEQUENCE_PREFIX);
	if (sequenceStart === -1) return [];

	const paramsStart = sequenceStart + KITTY_SEQUENCE_PREFIX.length;
	const paramsEnd = line.indexOf(";", paramsStart);
	if (paramsEnd === -1) return [];

	const params = line.slice(paramsStart, paramsEnd);
	for (const param of params.split(",")) {
		const [key, value] = param.split("=", 2);
		if (key !== "i" || value === undefined) continue;
		const id = Number(value);
		if (Number.isInteger(id) && id > 0 && id <= 0xffffffff) {
			return [id];
		}
	}
	return [];
}

/**
 * 组件接口 —— TUI 渲染的基本单元，所有 UI 组件必须实现此接口。
 * 提供三个核心方法：render（渲染为文本行）、handleInput（处理键盘输入）、invalidate（清除缓存）。
 *
 * Component interface - all components must implement this
 */
export interface Component {
	/**
	 * 将组件渲染为指定视口宽度的文本行数组
	 * @param width - 当前视口宽度（列数）
	 * @returns 字符串数组，每个元素代表一行，每行的可见宽度不能超过 width
	 *
	 * Render the component to lines for the given viewport width
	 * @param width - Current viewport width
	 * @returns Array of strings, each representing a line
	 */
	render(width: number): string[];

	/**
	 * 可选的键盘输入处理器，组件获得焦点时由 TUI 调用
	 *
	 * Optional handler for keyboard input when component has focus
	 */
	handleInput?(data: string): void;

	/**
	 * 是否接收按键释放事件（Kitty 协议）。默认为 false，TUI 会过滤掉释放事件
	 *
	 * If true, component receives key release events (Kitty protocol).
	 * Default is false - release events are filtered out.
	 */
	wantsKeyRelease?: boolean;

	/**
	 * 清除缓存的渲染状态。主题变更或组件需要从头重绘时由 TUI 调用
	 *
	 * Invalidate any cached rendering state.
	 * Called when theme changes or when component needs to re-render from scratch.
	 */
	invalidate(): void;
}

/**
 * 输入监听器函数类型
 * @param data - 原始输入数据
 * @returns 消费标记（阻止后续处理）和/或修改后的数据，返回 undefined 表示不做任何处理
 */
type InputListenerResult = { consume?: boolean; data?: string } | undefined;
type InputListener = (data: string) => InputListenerResult;

/**
 * 可获取焦点的组件接口。
 * 组件获得焦点时，应在渲染输出中输出 CURSOR_MARKER 标记光标位置。
 * TUI 会找到此标记并将硬件光标定位到该位置，以便输入法候选窗口正确对齐。
 *
 * Interface for components that can receive focus and display a hardware cursor.
 * When focused, the component should emit CURSOR_MARKER at the cursor position
 * in its render output. TUI will find this marker and position the hardware
 * cursor there for proper IME candidate window positioning.
 */
export interface Focusable {
	/** 焦点状态标志，由 TUI 在焦点变更时设置。为 true 时组件应在渲染输出中输出 CURSOR_MARKER */
	/** Set by TUI when focus changes. Component should emit CURSOR_MARKER when true. */
	focused: boolean;
}

/**
 * 类型守卫：判断组件是否实现了 Focusable 接口（即可接收焦点）
 * @param component - 待检查的组件
 * @returns 如果组件非 null 且包含 focused 属性则返回 true
 *
 * Type guard to check if a component implements Focusable
 */
export function isFocusable(component: Component | null): component is Component & Focusable {
	return component !== null && "focused" in component;
}

/**
 * 光标位置标记 —— APC（Application Program Command）序列。
 * 零宽度转义序列，终端会忽略其显示。
 * 组件获得焦点时在光标位置输出此标记，TUI 查找并剔除后将硬件光标定位到该位置。
 *
 * Cursor position marker - APC (Application Program Command) sequence.
 * This is a zero-width escape sequence that terminals ignore.
 * Components emit this at the cursor position when focused.
 * TUI finds and strips this marker, then positions the hardware cursor there.
 */
export const CURSOR_MARKER = "\x1b_pi:c\x07";

export { visibleWidth };

/**
 * 浮层定位锚点，决定浮层组件在终端中的对齐方式
 * Anchor position for overlays
 */
export type OverlayAnchor =
	| "center"
	| "top-left"
	| "top-right"
	| "bottom-left"
	| "bottom-right"
	| "top-center"
	| "bottom-center"
	| "left-center"
	| "right-center";

/**
 * 浮层边距配置，控制浮层与终端边缘的距离
 * Margin configuration for overlays
 */
export interface OverlayMargin {
	top?: number;
	right?: number;
	bottom?: number;
	left?: number;
}

/** 尺寸值类型：绝对值（number）或百分比（如 "50%"） */
/** Value that can be absolute (number) or percentage (string like "50%") */
export type SizeValue = number | `${number}%`;

/**
 * 将尺寸值（绝对值或百分比字符串）解析为绝对像素值
 * @param value - 要解析的尺寸值，undefined 时返回 undefined
 * @param referenceSize - 参考尺寸（如终端宽度或高度）
 * @returns 解析后的绝对值，无效格式时返回 undefined
 */
/** Parse a SizeValue into absolute value given a reference size */
function parseSizeValue(value: SizeValue | undefined, referenceSize: number): number | undefined {
	if (value === undefined) return undefined;
	if (typeof value === "number") return value;
	// 解析百分比字符串如 "50%"
	// Parse percentage string like "50%"
	const match = value.match(/^(\d+(?:\.\d+)?)%$/);
	if (match) {
		return Math.floor((referenceSize * parseFloat(match[1])) / 100);
	}
	return undefined;
}

/**
 * 检测当前是否运行在 Termux 环境中
 * Termux 在软键盘弹出/收起时会改变终端高度，需要特殊处理以避免全量重绘
 */
function isTermuxSession(): boolean {
	return Boolean(process.env.TERMUX_VERSION);
}

/**
 * 浮层定位和尺寸配置选项。
 * 尺寸和位置均支持绝对值和百分比字符串（如 "50%"）。
 *
 * Options for overlay positioning and sizing.
 * Values can be absolute numbers or percentage strings (e.g., "50%").
 */
export interface OverlayOptions {
	// === 尺寸 ===
	// === Sizing ===
	/** 浮层宽度（列数），或终端宽度的百分比（如 "50%"） */
	/** Width in columns, or percentage of terminal width (e.g., "50%") */
	width?: SizeValue;
	/** 最小宽度（列数） */
	/** Minimum width in columns */
	minWidth?: number;
	/** 最大高度（行数），或终端高度的百分比（如 "50%"）。超过时内容被截断 */
	/** Maximum height in rows, or percentage of terminal height (e.g., "50%") */
	maxHeight?: SizeValue;

	// === 定位 - 基于锚点 ===
	// === Positioning - anchor-based ===
	/** 定位锚点（默认 'center'） */
	/** Anchor point for positioning (default: 'center') */
	anchor?: OverlayAnchor;
	/** 相对于锚点的水平偏移（正值为右） */
	/** Horizontal offset from anchor position (positive = right) */
	offsetX?: number;
	/** 相对于锚点的垂直偏移（正值为下） */
	/** Vertical offset from anchor position (positive = down) */
	offsetY?: number;

	// === 定位 - 百分比或绝对值 ===
	// === Positioning - percentage or absolute ===
	/** 行位置：绝对行号，或百分比（如 "25%" = 距顶部 25%） */
	/** Row position: absolute number, or percentage (e.g., "25%" = 25% from top) */
	row?: SizeValue;
	/** 列位置：绝对列号，或百分比（如 "50%" = 水平居中） */
	/** Column position: absolute number, or percentage (e.g., "50%" = centered horizontally) */
	col?: SizeValue;

	// === 终端边缘边距 ===
	// === Margin from terminal edges ===
	/** 终端边缘的边距。数字类型应用到四边 */
	/** Margin from terminal edges. Number applies to all sides. */
	margin?: OverlayMargin | number;

	// === 可见性 ===
	// === Visibility ===
	/**
	 * 根据终端尺寸控制浮层的可见性。
	 * 提供此函数时，仅当返回 true 时浮层才显示。
	 * 每次渲染周期以当前终端尺寸调用。
	 *
	 * Control overlay visibility based on terminal dimensions.
	 * If provided, overlay is only rendered when this returns true.
	 * Called each render cycle with current terminal dimensions.
	 */
	visible?: (termWidth: number, termHeight: number) => boolean;
	/** 是否为非捕获模式。为 true 时浮层显示但不抢占键盘焦点 */
	/** If true, don't capture keyboard focus when shown */
	nonCapturing?: boolean;
}

/**
 * showOverlay 返回的浮层控制句柄，用于控制浮层的显示/隐藏、焦点管理
 *
 * Handle returned by showOverlay for controlling the overlay
 */
export interface OverlayHandle {
	/** 永久移除浮层（不可再次显示） */
	/** Permanently remove the overlay (cannot be shown again) */
	hide(): void;
	/** 临时隐藏或显示浮层 */
	/** Temporarily hide or show the overlay */
	setHidden(hidden: boolean): void;
	/** 检查浮层是否被临时隐藏 */
	/** Check if overlay is temporarily hidden */
	isHidden(): boolean;
	/** 聚焦到此浮层并将其置于视觉顶层 */
	/** Focus this overlay and bring it to the visual front */
	focus(): void;
	/** 释放焦点到上一个焦点目标 */
	/** Release focus to the previous target */
	unfocus(): void;
	/** 检查此浮层当前是否拥有焦点 */
	/** Check if this overlay currently has focus */
	isFocused(): boolean;
}

/**
 * 容器组件 —— 可包含子组件的组件。
 * 采用扁平化渲染，将子组件的渲染输出按顺序拼接为一个整体。
 *
 * Container - a component that contains other components
 */
export class Container implements Component {
	children: Component[] = [];

	/**
	 * 添加一个子组件到容器中
	 * @param component - 要添加的组件
	 */
	addChild(component: Component): void {
		this.children.push(component);
	}

	/**
	 * 从容器中移除指定子组件
	 * @param component - 要移除的组件
	 */
	removeChild(component: Component): void {
		const index = this.children.indexOf(component);
		if (index !== -1) {
			this.children.splice(index, 1);
		}
	}

	/**
	 * 清空所有子组件
	 */
	clear(): void {
		this.children = [];
	}

	/**
	 * 清空所有子组件的缓存渲染状态
	 */
	invalidate(): void {
		for (const child of this.children) {
			child.invalidate?.();
		}
	}

	/**
	 * 渲染容器：按顺序渲染所有子组件并将输出拼接为单一线条数组
	 * @param width - 视口宽度
	 * @returns 所有子组件渲染输出的拼接结果
	 */
	render(width: number): string[] {
		const lines: string[] = [];
		for (const child of this.children) {
			const childLines = child.render(width);
			for (const line of childLines) {
				lines.push(line);
			}
		}
		return lines;
	}
}

/**
 * TUI 引擎 —— 基于差异渲染的终端 UI 核心类。
 *
 * 核心职责：
 *   - 组件树管理（继承自 Container，可包含子组件）
 *   - 浮层系统（模态对话框、选择器等叠加组件）
 *   - 差异渲染（仅重绘变化的行，最小化终端 I/O）
 *   - 输入分发（将终端原始输入路由到焦点组件或输入监听器）
 *   - 硬件光标定位（支持 IME 输入法的候选窗口对齐）
 *   - Kitty 图片协议支持（内嵌图片的渲染与清理）
 *
 * TUI - Main class for managing terminal UI with differential rendering
 */
export class TUI extends Container {
	public terminal: Terminal;
	// 上一帧渲染的行内容，用于差异计算
	private previousLines: string[] = [];
	// 上一帧渲染中的 Kitty 图片 ID 集合，用于清理
	private previousKittyImageIds = new Set<number>();
	private previousWidth = 0;
	private previousHeight = 0;
	// 当前拥有键盘焦点的组件
	private focusedComponent: Component | null = null;
	// 输入监听器集合，在将按键分发给焦点组件前先过一遍监听器
	private inputListeners = new Set<InputListener>();

	/** 全局调试快捷键（Shift+Ctrl+D）的回调。在输入分发到焦点组件之前触发 */
	/** Global callback for debug key (Shift+Ctrl+D). Called before input is forwarded to focused component. */
	public onDebug?: () => void;
	// 是否已请求渲染，用于去重避免多次同时请求
	private renderRequested = false;
	// 延迟渲染的定时器，用于帧率控制和批量处理
	private renderTimer: NodeJS.Timeout | undefined;
	private lastRenderAt = 0;
	// 最小渲染间隔（毫秒），约 60fps，防止过高的渲染频率
	private static readonly MIN_RENDER_INTERVAL_MS = 16;
	// 逻辑光标行（内容末尾），用于视口计算
	private cursorRow = 0; // Logical cursor row (end of rendered content)
	// 硬件光标行（终端实际光标位置），因 IME 定位可能不同于逻辑光标
	private hardwareCursorRow = 0; // Actual terminal cursor row (may differ due to IME positioning)
	// 是否展示硬件光标（通过 PI_HARDWARE_CURSOR 环境变量控制，用于输入法候选窗口对齐）
	private showHardwareCursor = process.env.PI_HARDWARE_CURSOR === "1";
	// 内容缩减时是否清空空白行（默认关闭），通过 PI_CLEAR_ON_SHRINK 环境变量控制
	private clearOnShrink = process.env.PI_CLEAR_ON_SHRINK === "1"; // Clear empty rows when content shrinks (default: off)
	// 终端工作区域的最大渲染行数（历史高水位，用于判断是否需要清屏）
	private maxLinesRendered = 0; // Track terminal's working area (max lines ever rendered)
	// 上一帧的视口顶部行号，用于尺寸变化时的光标位置校正
	private previousViewportTop = 0; // Track previous viewport top for resize-aware cursor moves
	// 全量重绘计数器
	private fullRedrawCount = 0;
	// 是否已停止
	private stopped = false;

	// 浮层栈，管理模态组件（按 focusOrder 排序，值越大越在上层）
	// Overlay stack for modal components rendered on top of base content
	private focusOrderCounter = 0;
	private overlayStack: {
		component: Component;
		options?: OverlayOptions;
		// 浮层显示前的焦点组件，用于隐藏/移除后恢复焦点
		preFocus: Component | null;
		hidden: boolean;
		focusOrder: number;
	}[] = [];

	constructor(terminal: Terminal, showHardwareCursor?: boolean) {
		super();
		this.terminal = terminal;
		if (showHardwareCursor !== undefined) {
			this.showHardwareCursor = showHardwareCursor;
		}
	}

	/** 全量重绘次数，用于性能监控 */
	get fullRedraws(): number {
		return this.fullRedrawCount;
	}

	getShowHardwareCursor(): boolean {
		return this.showHardwareCursor;
	}

	/**
	 * 设置是否展示硬件光标
	 * 硬件光标用于 IME（输入法）候选窗口的位置对齐
	 * @param enabled - 是否启用
	 */
	setShowHardwareCursor(enabled: boolean): void {
		if (this.showHardwareCursor === enabled) return;
		this.showHardwareCursor = enabled;
		if (!enabled) {
			// 关闭时立即隐藏光标
			this.terminal.hideCursor();
		}
		this.requestRender();
	}

	getClearOnShrink(): boolean {
		return this.clearOnShrink;
	}

	/**
	 * 设置内容缩减时是否触发全量重绘以清除空白行。
	 * 启用时（默认），内容减少时会全量重绘以清除残留的旧行；
	 * 禁用时，旧行会保留在终端中（减少重绘次数，适合较慢的终端）。
	 *
	 * Set whether to trigger full re-render when content shrinks.
	 * When true (default), empty rows are cleared when content shrinks.
	 * When false, empty rows remain (reduces redraws on slower terminals).
	 */
	setClearOnShrink(enabled: boolean): void {
		this.clearOnShrink = enabled;
	}

	/**
	 * 设置键盘焦点组件
	 * 将旧焦点组件的 focused 标志设为 false，新组件的设为 true
	 * @param component - 要聚焦的组件，null 表示清除焦点
	 */
	setFocus(component: Component | null): void {
		// 清除旧焦点组件的状态标志
		// Clear focused flag on old component
		if (isFocusable(this.focusedComponent)) {
			this.focusedComponent.focused = false;
		}

		this.focusedComponent = component;

		// 设置新焦点组件的状态标志
		// Set focused flag on new component
		if (isFocusable(component)) {
			component.focused = true;
		}
	}

	/**
	 * 显示一个浮层组件，支持可配置的定位和尺寸。
	 * 返回控制句柄用于管理浮层的可见性和焦点状态。
	 *
	 * Show an overlay component with configurable positioning and sizing.
	 * Returns a handle to control the overlay's visibility.
	 */
	showOverlay(component: Component, options?: OverlayOptions): OverlayHandle {
		const entry = {
			component,
			options,
			preFocus: this.focusedComponent,
			hidden: false,
			focusOrder: ++this.focusOrderCounter,
		};
		this.overlayStack.push(entry);
		// 仅当浮层可见且非捕获模式时才抢占焦点
		// Only focus if overlay is actually visible
		if (!options?.nonCapturing && this.isOverlayVisible(entry)) {
			this.setFocus(component);
		}
		this.terminal.hideCursor();
		this.requestRender();

		// 返回浮层控制句柄，每个方法通过闭包捕获 entry 实现状态管理
		// Return handle for controlling this overlay
		return {
			hide: () => {
				const index = this.overlayStack.indexOf(entry);
				if (index !== -1) {
					this.overlayStack.splice(index, 1);
					// 如果此浮层拥有焦点，则将焦点归还给下一个可见浮层或 preFocus
					// Restore focus if this overlay had focus
					if (this.focusedComponent === component) {
						const topVisible = this.getTopmostVisibleOverlay();
						this.setFocus(topVisible?.component ?? entry.preFocus);
					}
					// 所有浮层移除后恢复光标隐藏状态
					if (this.overlayStack.length === 0) this.terminal.hideCursor();
					this.requestRender();
				}
			},
			setHidden: (hidden: boolean) => {
				if (entry.hidden === hidden) return;
				entry.hidden = hidden;
				// 显示/隐藏时自动处理焦点转移
				// Update focus when hiding/showing
				if (hidden) {
					// 隐藏时：如果此浮层有焦点，则移交给下一个可见浮层
					// If this overlay had focus, move focus to next visible or preFocus
					if (this.focusedComponent === component) {
						const topVisible = this.getTopmostVisibleOverlay();
						this.setFocus(topVisible?.component ?? entry.preFocus);
					}
				} else {
					// 显示时：如果可见且非捕获模式，则恢复焦点
					// Restore focus to this overlay when showing (if it's actually visible)
					if (!options?.nonCapturing && this.isOverlayVisible(entry)) {
						entry.focusOrder = ++this.focusOrderCounter;
						this.setFocus(component);
					}
				}
				this.requestRender();
			},
			isHidden: () => entry.hidden,
			focus: () => {
				if (!this.overlayStack.includes(entry) || !this.isOverlayVisible(entry)) return;
				if (this.focusedComponent !== component) {
					this.setFocus(component);
				}
				// 更新 focusOrder 将浮层置于渲染栈顶层
				entry.focusOrder = ++this.focusOrderCounter;
				this.requestRender();
			},
			unfocus: () => {
				if (this.focusedComponent !== component) return;
				const topVisible = this.getTopmostVisibleOverlay();
				this.setFocus(topVisible && topVisible !== entry ? topVisible.component : entry.preFocus);
				this.requestRender();
			},
			isFocused: () => this.focusedComponent === component,
		};
	}

	/**
	 * 隐藏最顶部的浮层并恢复上一个焦点组件
	 *
	 * Hide the topmost overlay and restore previous focus.
	 */
	hideOverlay(): void {
		const overlay = this.overlayStack.pop();
		if (!overlay) return;
		if (this.focusedComponent === overlay.component) {
			// 查找下一个可见浮层，或回退到 preFocus
			// Find topmost visible overlay, or fall back to preFocus
			const topVisible = this.getTopmostVisibleOverlay();
			this.setFocus(topVisible?.component ?? overlay.preFocus);
		}
		// 所有浮层移除后恢复光标隐藏状态
		if (this.overlayStack.length === 0) this.terminal.hideCursor();
		this.requestRender();
	}

	/** 检查是否有任何可见的浮层 */
	/** Check if there are any visible overlays */
	hasOverlay(): boolean {
		return this.overlayStack.some((o) => this.isOverlayVisible(o));
	}

	/** 检查某个浮层条目当前是否可见（未被隐藏且通过了 visible 回调） */
	/** Check if an overlay entry is currently visible */
	private isOverlayVisible(entry: (typeof this.overlayStack)[number]): boolean {
		if (entry.hidden) return false;
		// 通过 visible() 回调根据终端尺寸动态判断可见性
		if (entry.options?.visible) {
			return entry.options.visible(this.terminal.columns, this.terminal.rows);
		}
		return true;
	}

	/** 查找最顶部的可见且可捕获焦点的浮层（跳过 nonCapturing 浮层） */
	/** Find the topmost visible capturing overlay, if any */
	private getTopmostVisibleOverlay(): (typeof this.overlayStack)[number] | undefined {
		for (let i = this.overlayStack.length - 1; i >= 0; i--) {
			// 跳过非捕获模式浮层（它们不参与焦点管理）
			if (this.overlayStack[i].options?.nonCapturing) continue;
			if (this.isOverlayVisible(this.overlayStack[i])) {
				return this.overlayStack[i];
			}
		}
		return undefined;
	}

	/**
	 * 清空所有组件（包括浮层）的缓存渲染状态
	 */
	override invalidate(): void {
		super.invalidate();
		for (const overlay of this.overlayStack) overlay.component.invalidate?.();
	}

	/**
	 * 启动 TUI 引擎
	 * 流程：启动终端输入监听 -> 隐藏光标 -> 查询终端像素尺寸 -> 请求首帧渲染
	 */
	start(): void {
		this.stopped = false;
		this.terminal.start(
			(data) => this.handleInput(data),
			() => this.requestRender(),
		);
		this.terminal.hideCursor();
		// 查询终端字符单元像素尺寸（用于图片渲染）
		this.queryCellSize();
		this.requestRender();
	}

	/**
	 * 添加一个输入监听器
	 * 监听器在焦点组件之前处理输入，可以消费事件或修改输入数据
	 * @param listener - 输入监听函数
	 * @returns 用于移除监听器的函数
	 */
	addInputListener(listener: InputListener): () => void {
		this.inputListeners.add(listener);
		return () => {
			this.inputListeners.delete(listener);
		};
	}

	/**
	 * 移除指定输入监听器
	 * @param listener - 要移除的监听函数
	 */
	removeInputListener(listener: InputListener): void {
		this.inputListeners.delete(listener);
	}

	/**
	 * 查询终端字符单元像素尺寸
	 * 仅在支持图片的终端上执行（像素尺寸仅用于图片渲染），结果通过 setCellDimensions 保存
	 */
	private queryCellSize(): void {
		// 仅当终端支持图片显示时才查询（像素尺寸仅用于图片渲染）
		// Only query if terminal supports images (cell size is only used for image rendering)
		if (!getCapabilities().images) {
			return;
		}
		// 发送 CSI 16 t 查询终端字符单元的像素尺寸，响应格式：CSI 6 ; height ; width t
		// Query terminal for cell size in pixels: CSI 16 t
		// Response format: CSI 6 ; height ; width t
		this.terminal.write("\x1b[16t");
	}

	/**
	 * 停止 TUI 引擎
	 * 流程：清除渲染定时器 -> 移动光标到内容末尾（防止退出时覆盖残留内容）-> 显示光标 -> 停止终端
	 */
	stop(): void {
		this.stopped = true;
		if (this.renderTimer) {
			clearTimeout(this.renderTimer);
			this.renderTimer = undefined;
		}
		// 将光标移到内容末尾，防止退出时出现覆盖/残留伪影
		// Move cursor to the end of the content to prevent overwriting/artifacts on exit
		if (this.previousLines.length > 0) {
			const targetRow = this.previousLines.length; // 最后一行之后的位置 // Line after the last content
			const lineDiff = targetRow - this.hardwareCursorRow;
			if (lineDiff > 0) {
				this.terminal.write(`\x1b[${lineDiff}B`);
			} else if (lineDiff < 0) {
				this.terminal.write(`\x1b[${-lineDiff}A`);
			}
			this.terminal.write("\r\n");
		}

		this.terminal.showCursor();
		this.terminal.stop();
	}

	/**
	 * 请求渲染（异步，通过 process.nextTick 批量处理）。
	 * 多次快速连续调用会被合并为一次渲染，受 MIN_RENDER_INTERVAL_MS 帧率限制。
	 * @param force - 强制定时全量重绘（清除 previousLines 等状态，触发完整渲染）
	 */
	requestRender(force = false): void {
		if (force) {
			// 强制定制：清除所有历史状态，触发全量重绘
			this.previousLines = [];
			this.previousWidth = -1; // -1 触发 widthChanged 标志，强制清屏 // -1 triggers widthChanged, forcing a full clear
			this.previousHeight = -1; // -1 触发 heightChanged 标志 // -1 triggers heightChanged, forcing a full clear
			this.cursorRow = 0;
			this.hardwareCursorRow = 0;
			this.maxLinesRendered = 0;
			this.previousViewportTop = 0;
			if (this.renderTimer) {
				clearTimeout(this.renderTimer);
				this.renderTimer = undefined;
			}
			this.renderRequested = true;
			// 强制渲染使用 process.nextTick 同步调度，不经过帧率限制
			process.nextTick(() => {
				if (this.stopped || !this.renderRequested) {
					return;
				}
				this.renderRequested = false;
				this.lastRenderAt = performance.now();
				this.doRender();
			});
			return;
		}
		// 非强制：去重后延迟批量化调度
		if (this.renderRequested) return;
		this.renderRequested = true;
		process.nextTick(() => this.scheduleRender());
	}

	/**
	 * 调度渲染，遵循帧率限制（约 60fps）
	 * 如果距上次渲染不足 MIN_RENDER_INTERVAL_MS，则设置定时器延迟到帧周期；
	 * 如果已过帧周期，则立即执行渲染
	 */
	private scheduleRender(): void {
		if (this.stopped || this.renderTimer || !this.renderRequested) {
			return;
		}
		const elapsed = performance.now() - this.lastRenderAt;
		// 计算距下一帧的等待时间
		const delay = Math.max(0, TUI.MIN_RENDER_INTERVAL_MS - elapsed);
		this.renderTimer = setTimeout(() => {
			this.renderTimer = undefined;
			if (this.stopped || !this.renderRequested) {
				return;
			}
			this.renderRequested = false;
			this.lastRenderAt = performance.now();
			this.doRender();
			// 如果渲染期间又有新的渲染请求，则继续调度
			if (this.renderRequested) {
				this.scheduleRender();
			}
		}, delay);
	}

	/**
	 * 处理终端原始输入数据
	 * 流程：输入监听器链条 -> 终端像素尺寸响应处理 -> 全局调试快捷键 -> 焦点可见性验证 -> 分发给焦点组件
	 */
	private handleInput(data: string): void {
		// 第一阶段：通过输入监听器链条处理
		// 监听器可以消费事件（阻止后续处理）、修改输入数据
		if (this.inputListeners.size > 0) {
			let current = data;
			for (const listener of this.inputListeners) {
				const result = listener(current);
				if (result?.consume) {
					// 事件已被消费，停止处理
					return;
				}
				if (result?.data !== undefined) {
					current = result.data;
				}
			}
			// 如果监听器将数据清空，停止处理
			if (current.length === 0) {
				return;
			}
			data = current;
		}

		// 处理终端像素尺寸查询的响应（不阻塞其他输入）
		// Consume terminal cell size responses without blocking unrelated input.
		if (this.consumeCellSizeResponse(data)) {
			return;
		}

		// 全局调试快捷键（Shift+Ctrl+D），优先于焦点组件处理
		// Global debug key handler (Shift+Ctrl+D)
		if (matchesKey(data, "shift+ctrl+d") && this.onDebug) {
			this.onDebug();
			return;
		}

		// 验证焦点组件对应的浮层是否仍然可见（可能因终端尺寸变化而不可见）
		// If focused component is an overlay, verify it's still visible
		// (visibility can change due to terminal resize or visible() callback)
		const focusedOverlay = this.overlayStack.find((o) => o.component === this.focusedComponent);
		if (focusedOverlay && !this.isOverlayVisible(focusedOverlay)) {
			// 焦点浮层已不可见，重定向到下一个可见浮层
			// Focused overlay is no longer visible, redirect to topmost visible overlay
			const topVisible = this.getTopmostVisibleOverlay();
			if (topVisible) {
				this.setFocus(topVisible.component);
			} else {
				// 没有可见浮层，恢复到浮层显示前的焦点
				// No visible overlays, restore to preFocus
				this.setFocus(focusedOverlay.preFocus);
			}
		}

		// 将输入分发给焦点组件
		// Pass input to focused component (including Ctrl+C)
		// The focused component can decide how to handle Ctrl+C
		if (this.focusedComponent?.handleInput) {
			// 过滤按键释放事件（除非组件明确表示需要接收）
			// 这样做是为了让大部分编辑器组件只处理按下事件，减少重复处理
			// Filter out key release events unless component opts in
			if (isKeyRelease(data) && !this.focusedComponent.wantsKeyRelease) {
				return;
			}
			this.focusedComponent.handleInput(data);
			this.requestRender();
		}
	}

	/**
	 * 处理终端字符单元像素尺寸响应
	 * 响应格式：ESC [ 6 ; height ; width t
	 * @returns 如果数据匹配并成功处理，返回 true
	 */
	private consumeCellSizeResponse(data: string): boolean {
		// 匹配 CSI 6 ; 高度 ; 宽度 t 格式的终端响应
		// Response format: ESC [ 6 ; height ; width t
		const match = data.match(/^\x1b\[6;(\d+);(\d+)t$/);
		if (!match) {
			return false;
		}

		const heightPx = parseInt(match[1], 10);
		const widthPx = parseInt(match[2], 10);
		if (heightPx <= 0 || widthPx <= 0) {
			return true;
		}

		setCellDimensions({ widthPx, heightPx });
		// 尺寸变更后需要清除所有组件的缓存，使图片以正确的尺寸重新渲染
		// Invalidate all components so images re-render with correct dimensions.
		this.invalidate();
		this.requestRender();
		return true;
	}

	/**
	 * 从浮层配置中解析布局参数（宽度、行、列、最大高度）
	 * @param options - 浮层配置选项
	 * @param overlayHeight - 浮层渲染后的实际高度
	 * @param termWidth - 终端宽度
	 * @param termHeight - 终端高度
	 * @returns 解析后的 { width, row, col, maxHeight }
	 *
	 * Resolve overlay layout from options.
	 * Returns { width, row, col, maxHeight } for rendering.
	 */
	private resolveOverlayLayout(
		options: OverlayOptions | undefined,
		overlayHeight: number,
		termWidth: number,
		termHeight: number,
	): { width: number; row: number; col: number; maxHeight: number | undefined } {
		const opt = options ?? {};

		// 解析边距并限制为非负值，防止浮层超出终端边界
		// Parse margin (clamp to non-negative)
		const margin =
			typeof opt.margin === "number"
				? { top: opt.margin, right: opt.margin, bottom: opt.margin, left: opt.margin }
				: (opt.margin ?? {});
		const marginTop = Math.max(0, margin.top ?? 0);
		const marginRight = Math.max(0, margin.right ?? 0);
		const marginBottom = Math.max(0, margin.bottom ?? 0);
		const marginLeft = Math.max(0, margin.left ?? 0);

		// 计算扣除边距后的可用空间
		// Available space after margins
		const availWidth = Math.max(1, termWidth - marginLeft - marginRight);
		const availHeight = Math.max(1, termHeight - marginTop - marginBottom);

		// === 解析宽度：优先 options.width，否则默认取 min(80, 可用宽度) ===
		// === Resolve width ===
		let width = parseSizeValue(opt.width, termWidth) ?? Math.min(80, availWidth);
		// 应用最小宽度限制
		// Apply minWidth
		if (opt.minWidth !== undefined) {
			width = Math.max(width, opt.minWidth);
		}
		// 限制到可用空间内
		// Clamp to available space
		width = Math.max(1, Math.min(width, availWidth));

		// === 解析最大高度 ===
		// === Resolve maxHeight ===
		let maxHeight = parseSizeValue(opt.maxHeight, termHeight);
		// 限制到可用空间内
		// Clamp to available space
		if (maxHeight !== undefined) {
			maxHeight = Math.max(1, Math.min(maxHeight, availHeight));
		}

		// 有效浮层高度（可能被 maxHeight 截断）
		// Effective overlay height (may be clamped by maxHeight)
		const effectiveHeight = maxHeight !== undefined ? Math.min(overlayHeight, maxHeight) : overlayHeight;

		// === 解析垂直位置 ===
		// === Resolve position ===
		let row: number;
		let col: number;

		// 垂直位置：优先 row，否则通过锚点计算
		if (opt.row !== undefined) {
			if (typeof opt.row === "string") {
				// 百分比格式：0% = 距顶部，100% = 距底部（浮层保持在可用区域内）
				// Percentage: 0% = top, 100% = bottom (overlay stays within bounds)
				const match = opt.row.match(/^(\d+(?:\.\d+)?)%$/);
				if (match) {
					const maxRow = Math.max(0, availHeight - effectiveHeight);
					const percent = parseFloat(match[1]) / 100;
					row = marginTop + Math.floor(maxRow * percent);
				} else {
					// 无效格式，回退到居中
					// Invalid format, fall back to center
					row = this.resolveAnchorRow("center", effectiveHeight, availHeight, marginTop);
				}
			} else {
				// 绝对行位置
				// Absolute row position
				row = opt.row;
			}
		} else {
			// 基于锚点计算（默认居中）
			// Anchor-based (default: center)
			const anchor = opt.anchor ?? "center";
			row = this.resolveAnchorRow(anchor, effectiveHeight, availHeight, marginTop);
		}

		// 水平位置：优先 col，否则通过锚点计算
		if (opt.col !== undefined) {
			if (typeof opt.col === "string") {
				// 百分比格式：0% = 贴左，100% = 贴右（浮层保持在可用区域内）
				// Percentage: 0% = left, 100% = right (overlay stays within bounds)
				const match = opt.col.match(/^(\d+(?:\.\d+)?)%$/);
				if (match) {
					const maxCol = Math.max(0, availWidth - width);
					const percent = parseFloat(match[1]) / 100;
					col = marginLeft + Math.floor(maxCol * percent);
				} else {
					// 无效格式，回退到居中
					// Invalid format, fall back to center
					col = this.resolveAnchorCol("center", width, availWidth, marginLeft);
				}
			} else {
				// 绝对列位置
				// Absolute column position
				col = opt.col;
			}
		} else {
			// 基于锚点计算（默认居中）
			// Anchor-based (default: center)
			const anchor = opt.anchor ?? "center";
			col = this.resolveAnchorCol(anchor, width, availWidth, marginLeft);
		}

		// 应用偏移量
		// Apply offsets
		if (opt.offsetY !== undefined) row += opt.offsetY;
		if (opt.offsetX !== undefined) col += opt.offsetX;

		// 限制到终端边界内（遵守边距）
		// Clamp to terminal bounds (respecting margins)
		row = Math.max(marginTop, Math.min(row, termHeight - marginBottom - effectiveHeight));
		col = Math.max(marginLeft, Math.min(col, termWidth - marginRight - width));

		return { width, row, col, maxHeight };
	}

	/**
	 * 根据锚点计算浮层的起始行
	 * @param anchor - 定位锚点
	 * @param height - 浮层高度
	 * @param availHeight - 可用高度
	 * @param marginTop - 顶部边距
	 */
	private resolveAnchorRow(anchor: OverlayAnchor, height: number, availHeight: number, marginTop: number): number {
		switch (anchor) {
			case "top-left":
			case "top-center":
			case "top-right":
				return marginTop;
			case "bottom-left":
			case "bottom-center":
			case "bottom-right":
				return marginTop + availHeight - height;
			case "left-center":
			case "center":
			case "right-center":
				return marginTop + Math.floor((availHeight - height) / 2);
		}
	}

	/**
	 * 根据锚点计算浮层的起始列
	 * @param anchor - 定位锚点
	 * @param width - 浮层宽度
	 * @param availWidth - 可用宽度
	 * @param marginLeft - 左边距
	 */
	private resolveAnchorCol(anchor: OverlayAnchor, width: number, availWidth: number, marginLeft: number): number {
		switch (anchor) {
			case "top-left":
			case "left-center":
			case "bottom-left":
				return marginLeft;
			case "top-right":
			case "right-center":
			case "bottom-right":
				return marginLeft + availWidth - width;
			case "top-center":
			case "center":
			case "bottom-center":
				return marginLeft + Math.floor((availWidth - width) / 2);
		}
	}

	/**
	 * 将所有可见浮层合成到基础内容行中
	 * 按 focusOrder 排序（值大的在上层），依次叠加浮层内容到对应位置
	 *
	 * Composite all overlays into content lines (sorted by focusOrder, higher = on top).
	 */
	private compositeOverlays(lines: string[], termWidth: number, termHeight: number): string[] {
		if (this.overlayStack.length === 0) return lines;
		const result = [...lines];

		// 预渲染所有可见浮层并计算位置
		// Pre-render all visible overlays and calculate positions
		const rendered: { overlayLines: string[]; row: number; col: number; w: number }[] = [];
		let minLinesNeeded = result.length;

		const visibleEntries = this.overlayStack.filter((e) => this.isOverlayVisible(e));
		// 按 focusOrder 升序排列，值小的先渲染作为底层
		visibleEntries.sort((a, b) => a.focusOrder - b.focusOrder);
		for (const entry of visibleEntries) {
			const { component, options } = entry;

			// 先以 height=0 获取宽度和 maxHeight（这两个参数不依赖浮层高度）
			// Get layout with height=0 first to determine width and maxHeight
			// (width and maxHeight don't depend on overlay height)
			const { width, maxHeight } = this.resolveOverlayLayout(options, 0, termWidth, termHeight);

			// 以计算出的宽度渲染组件
			// Render component at calculated width
			let overlayLines = component.render(width);

			// 如果指定了最大高度，则截断输出
			// Apply maxHeight if specified
			if (maxHeight !== undefined && overlayLines.length > maxHeight) {
				overlayLines = overlayLines.slice(0, maxHeight);
			}

			// 以实际高度获取最终行/列位置
			// Get final row/col with actual overlay height
			const { row, col } = this.resolveOverlayLayout(options, overlayLines.length, termWidth, termHeight);

			rendered.push({ overlayLines, row, col, w: width });
			minLinesNeeded = Math.max(minLinesNeeded, row + overlayLines.length);
		}

		// 将结果至少扩展到终端高度，以便浮层位置基于屏幕坐标计算。
		// 不使用 maxLinesRendered：历史高水位会导致终端宽度增大时产生自增强膨胀。
		// Pad to at least terminal height so overlays have screen-relative positions.
		// Excludes maxLinesRendered: the historical high-water mark caused self-reinforcing
		// inflation that pushed content into scrollback on terminal widen.
		const workingHeight = Math.max(result.length, termHeight, minLinesNeeded);

		// 如果基础内容不够长，用空行填充到工作区域高度
		// Extend result with empty lines if content is too short for overlay placement or working area
		while (result.length < workingHeight) {
			result.push("");
		}

		// 可见视口的起始行
		const viewportStart = Math.max(0, workingHeight - termHeight);

		// 将每个浮层内容合成到基础行中
		// Composite each overlay
		for (const { overlayLines, row, col, w } of rendered) {
			for (let i = 0; i < overlayLines.length; i++) {
				const idx = viewportStart + row + i;
				if (idx >= 0 && idx < result.length) {
					// 防御性剪裁：将浮层行限制在声明宽度内（组件应已遵守 width，此处为安全兜底）
					// Defensive: truncate overlay line to declared width before compositing
					// (components should already respect width, but this ensures it)
					const truncatedOverlayLine =
						visibleWidth(overlayLines[i]) > w ? sliceByColumn(overlayLines[i], 0, w, true) : overlayLines[i];
					result[idx] = this.compositeLineAt(result[idx], truncatedOverlayLine, col, w, termWidth);
				}
			}
		}

		return result;
	}

	// 段落重置序列：重置样式 + 关闭超链接，确保每行结束时样式不会跨行泄漏
	private static readonly SEGMENT_RESET = "\x1b[0m\x1b]8;;\x07";

	/**
	 * 对每行应用样式重置序列
	 * 非图片行在末尾追加重置序列，防止样式泄漏到下一行
	 */
	private applyLineResets(lines: string[]): string[] {
		const reset = TUI.SEGMENT_RESET;
		for (let i = 0; i < lines.length; i++) {
			const line = lines[i];
			// 图片行不需要追加重置序列（会破坏图片显示）
			if (!isImageLine(line)) {
				lines[i] = normalizeTerminalOutput(line) + reset;
			}
		}
		return lines;
	}

	/**
	 * 从渲染行集合中收集所有 Kitty 图片 ID
	 * @returns 所有图片 ID 的集合
	 */
	private collectKittyImageIds(lines: string[]): Set<number> {
		const ids = new Set<number>();
		for (const line of lines) {
			for (const id of extractKittyImageIds(line)) {
				ids.add(id);
			}
		}
		return ids;
	}

	/**
	 * 批量删除指定 ID 的 Kitty 图片
	 * @param ids - 要删除的图片 ID 集合
	 * @returns 用于写入终端的删除指令字符串
	 */
	private deleteKittyImages(ids: Iterable<number>): string {
		let buffer = "";
		for (const id of ids) {
			buffer += deleteKittyImage(id);
		}
		return buffer;
	}

	/**
	 * 扩展差异渲染的变更行范围，以包含变化区域内的 Kitty 图片行
	 * 图片所在行即使内容未变，也需要重新输出以触发重绘
	 */
	private expandLastChangedForKittyImages(firstChanged: number, lastChanged: number): number {
		let expandedLastChanged = lastChanged;
		for (let i = firstChanged; i < this.previousLines.length; i++) {
			if (extractKittyImageIds(this.previousLines[i]).length > 0) {
				expandedLastChanged = Math.max(expandedLastChanged, i);
			}
		}
		return expandedLastChanged;
	}

	/**
	 * 删除变更区域内上帧已渲染的 Kitty 图片
	 * 在刷新输出前，需要先删除旧图片避免残留
	 */
	private deleteChangedKittyImages(firstChanged: number, lastChanged: number): string {
		if (firstChanged < 0 || lastChanged < firstChanged) return "";

		const ids = new Set<number>();
		const maxLine = Math.min(lastChanged, this.previousLines.length - 1);
		for (let i = firstChanged; i <= maxLine; i++) {
			for (const id of extractKittyImageIds(this.previousLines[i] ?? "")) {
				ids.add(id);
			}
		}

		return this.deleteKittyImages(ids);
	}

	/**
	 * 将浮层行合成到基础行的指定列位置。单次遍历优化。
	 * 分为三段：基础行前半部分、浮层内容、基础行后半部分，分别处理宽度对齐
	 *
	 * Splice overlay content into a base line at a specific column. Single-pass optimized.
	 */
	private compositeLineAt(
		baseLine: string,
		overlayLine: string,
		startCol: number,
		overlayWidth: number,
		totalWidth: number,
	): string {
		// 图片行不可合成覆盖
		if (isImageLine(baseLine)) return baseLine;

		// 单次遍历基础行，提取前段和后段
		// Single pass through baseLine extracts both before and after segments
		const afterStart = startCol + overlayWidth;
		const base = extractSegments(baseLine, startCol, afterStart, totalWidth - afterStart, true);

		// 提取浮层内容并跟踪宽度
		// Extract overlay with width tracking (strict=true to exclude wide chars at boundary)
		const overlay = sliceWithWidth(overlayLine, 0, overlayWidth, true);

		// 用空格填充各段到目标宽度
		// Pad segments to target widths
		const beforePad = Math.max(0, startCol - base.beforeWidth);
		const overlayPad = Math.max(0, overlayWidth - overlay.width);
		const actualBeforeWidth = Math.max(startCol, base.beforeWidth);
		const actualOverlayWidth = Math.max(overlayWidth, overlay.width);
		const afterTarget = Math.max(0, totalWidth - actualBeforeWidth - actualOverlayWidth);
		const afterPad = Math.max(0, afterTarget - base.afterWidth);

		// 拼接结果：前段 + 填充空格 + 样式重置 + 浮层内容 + 样式重置 + 后段 + 填充空格
		// Compose result
		const r = TUI.SEGMENT_RESET;
		const result =
			base.before +
			" ".repeat(beforePad) +
			r +
			overlay.text +
			" ".repeat(overlayPad) +
			r +
			base.after +
			" ".repeat(afterPad);

		// 关键安全兜底：始终验证并截断到终端宽度。
		// 宽度跟踪可能因以下原因与实际可见宽度产生偏差：
		// - 复杂的 ANSI/OSC 序列（超链接、颜色）
		// - 宽字符位于段落边界
		// - 段落提取的边缘情况
		// 超宽将导致 TUI 崩溃，此处的截断是最后一道防线。
		//
		// CRITICAL: Always verify and truncate to terminal width.
		// This is the final safeguard against width overflow which would crash the TUI.
		// Width tracking can drift from actual visible width due to:
		// - Complex ANSI/OSC sequences (hyperlinks, colors)
		// - Wide characters at segment boundaries
		// - Edge cases in segment extraction
		const resultWidth = visibleWidth(result);
		if (resultWidth <= totalWidth) {
			return result;
		}
		// 使用严格模式截断，确保不超出终端宽度
		// Truncate with strict=true to ensure we don't exceed totalWidth
		return sliceByColumn(result, 0, totalWidth, true);
	}

	/**
	 * 从渲染行中查找并提取光标位置。
	 * 搜索 CURSOR_MARKER，计算其所在行列，并从输出中剔除标记。
	 * 只扫描底部 terminal height 行（可见视口），忽略视口上方的内容。
	 * @param lines - 要搜索的渲染行
	 * @param height - 终端高度（可见视口大小）
	 * @returns 光标位置 { row, col }，未找到标记时返回 null
	 *
	 * Find and extract cursor position from rendered lines.
	 * Searches for CURSOR_MARKER, calculates its position, and strips it from the output.
	 * Only scans the bottom terminal height lines (visible viewport).
	 * @param lines - Rendered lines to search
	 * @param height - Terminal height (visible viewport size)
	 * @returns Cursor position { row, col } or null if no marker found
	 */
	private extractCursorPosition(lines: string[], height: number): { row: number; col: number } | null {
		// 只扫描底部 `height` 行（可见视口），忽略视口上方不显示的内容
		// Only scan the bottom `height` lines (visible viewport)
		const viewportTop = Math.max(0, lines.length - height);
		for (let row = lines.length - 1; row >= viewportTop; row--) {
			const line = lines[row];
			const markerIndex = line.indexOf(CURSOR_MARKER);
			if (markerIndex !== -1) {
				// 计算标记前的可见宽度作为列号
				// Calculate visual column (width of text before marker)
				const beforeMarker = line.slice(0, markerIndex);
				const col = visibleWidth(beforeMarker);

				// 从行中剔除光标标记
				// Strip marker from the line
				lines[row] = line.slice(0, markerIndex) + line.slice(markerIndex + CURSOR_MARKER.length);

				return { row, col };
			}
		}
		return null;
	}

	/**
	 * 执行实际渲染 —— TUI 引擎的核心方法。
	 *
	 * 流程：
	 *   1. 检测终端尺寸变化（宽度/高度）
	 *   2. 渲染所有组件获取新内容行
	 *   3. 合成浮层内容
	 *   4. 提取光标位置并应用样式重置
	 *   5. 决定渲染策略：全量渲染或差异渲染
	 *   6. 输出到终端
	 */
	private doRender(): void {
		if (this.stopped) return;
		const width = this.terminal.columns;
		const height = this.terminal.rows;
		// 检测终端尺寸变化
		const widthChanged = this.previousWidth !== 0 && this.previousWidth !== width;
		const heightChanged = this.previousHeight !== 0 && this.previousHeight !== height;
		const previousBufferLength = this.previousHeight > 0 ? this.previousViewportTop + this.previousHeight : height;
		let prevViewportTop = heightChanged ? Math.max(0, previousBufferLength - height) : this.previousViewportTop;
		let viewportTop = prevViewportTop;
		let hardwareCursorRow = this.hardwareCursorRow;
		const computeLineDiff = (targetRow: number): number => {
			// 计算光标从当前屏幕位置到目标屏幕位置需要移动的行数
			const currentScreenRow = hardwareCursorRow - prevViewportTop;
			const targetScreenRow = targetRow - viewportTop;
			return targetScreenRow - currentScreenRow;
		};

		// 第一步：渲染所有组件获取新内容行
		// Render all components to get new lines
		let newLines = this.render(width);

		// 第二步：将浮层合成到渲染行中（在差异比较之前）
		// Composite overlays into the rendered lines (before differential compare)
		if (this.overlayStack.length > 0) {
			newLines = this.compositeOverlays(newLines, width, height);
		}

		// 第三步：提取光标位置（必须在样式重置之前，标记需要先被找到）
		// Extract cursor position before applying line resets (marker must be found first)
		const cursorPos = this.extractCursorPosition(newLines, height);

		// 第四步：应用样式重置序列
		newLines = this.applyLineResets(newLines);

		// 全量渲染辅助函数：清除回滚缓冲区和视口，渲染所有新行
		// Helper to clear scrollback and viewport and render all new lines
		const fullRender = (clear: boolean): void => {
			this.fullRedrawCount += 1;
			// 开始同步输出（保证终端原子性渲染）
			let buffer = "\x1b[?2026h"; // Begin synchronized output
			if (clear) {
				// 删除旧图片 + 清屏 + 归位 + 清除回滚缓冲区
				buffer += this.deleteKittyImages(this.previousKittyImageIds);
				buffer += "\x1b[2J\x1b[H\x1b[3J"; // Clear screen, home, then clear scrollback
			}
			for (let i = 0; i < newLines.length; i++) {
				if (i > 0) buffer += "\r\n";
				buffer += newLines[i];
			}
			// 结束同步输出
			buffer += "\x1b[?2026l"; // End synchronized output
			this.terminal.write(buffer);
			this.cursorRow = Math.max(0, newLines.length - 1);
			this.hardwareCursorRow = this.cursorRow;
			// 清屏时重置历史高水位，否则跟踪增长
			// Reset max lines when clearing, otherwise track growth
			if (clear) {
				this.maxLinesRendered = newLines.length;
			} else {
				this.maxLinesRendered = Math.max(this.maxLinesRendered, newLines.length);
			}
			const bufferLength = Math.max(height, newLines.length);
			this.previousViewportTop = Math.max(0, bufferLength - height);
			this.positionHardwareCursor(cursorPos, newLines.length);
			this.previousLines = newLines;
			this.previousKittyImageIds = this.collectKittyImageIds(newLines);
			this.previousWidth = width;
			this.previousHeight = height;
		};

		// 调试日志：记录触发全量渲染的原因
		const debugRedraw = process.env.PI_DEBUG_REDRAW === "1";
		const logRedraw = (reason: string): void => {
			if (!debugRedraw) return;
			const logPath = path.join(os.homedir(), ".pi", "agent", "pi-debug.log");
			const msg = `[${new Date().toISOString()}] fullRender: ${reason} (prev=${this.previousLines.length}, new=${newLines.length}, height=${height})\n`;
			fs.appendFileSync(logPath, msg);
		};

		// 策略判断：选择全量渲染还是差异渲染

		// 首次渲染：直接输出所有内容不清屏（假设终端为空）
		// First render - just output everything without clearing (assumes clean screen)
		if (this.previousLines.length === 0 && !widthChanged && !heightChanged) {
			logRedraw("first render");
			fullRender(false);
			return;
		}

		// 宽度变化：因自动换行完全改变，必须全量重绘
		// Width changes always need a full re-render because wrapping changes.
		if (widthChanged) {
			logRedraw(`terminal width changed (${this.previousWidth} -> ${width})`);
			fullRender(true);
			return;
		}

		// 高度变化：通常全量重绘保持视口对齐。
		// 但 Termux 环境下软键盘弹出/收起会导致高度变化，此时全量重绘将重放整个历史，
		// 所以 Termux 中跳过全量重绘。
		// Height changes normally need a full re-render to keep the visible viewport aligned,
		// but Termux changes height when the software keyboard shows or hides.
		// In that environment, a full redraw causes the entire history to replay on every toggle.
		if (heightChanged && !isTermuxSession()) {
			logRedraw(`terminal height changed (${this.previousHeight} -> ${height})`);
			fullRender(true);
			return;
		}

		// 内容缩减且无浮层时：全量重绘以清除残留的空白行（浮层需要填充空间，跳过此优化）
		// 可通过 setClearOnShrink() 或 PI_CLEAR_ON_SHRINK=0 环境变量配置
		// Content shrunk below the working area and no overlays - re-render to clear empty rows
		// (overlays need the padding, so only do this when no overlays are active)
		// Configurable via setClearOnShrink() or PI_CLEAR_ON_SHRINK=0 env var
		if (this.clearOnShrink && newLines.length < this.maxLinesRendered && this.overlayStack.length === 0) {
			logRedraw(`clearOnShrink (maxLinesRendered=${this.maxLinesRendered})`);
			fullRender(true);
			return;
		}

		// === 差异渲染路径 ===
		// 计算第一行和最后一行变更的位置
		// Find first and last changed lines
		let firstChanged = -1;
		let lastChanged = -1;
		const maxLines = Math.max(newLines.length, this.previousLines.length);
		for (let i = 0; i < maxLines; i++) {
			const oldLine = i < this.previousLines.length ? this.previousLines[i] : "";
			const newLine = i < newLines.length ? newLines[i] : "";

			if (oldLine !== newLine) {
				if (firstChanged === -1) {
					firstChanged = i;
				}
				lastChanged = i;
			}
		}
		const appendedLines = newLines.length > this.previousLines.length;
		if (appendedLines) {
			if (firstChanged === -1) {
				firstChanged = this.previousLines.length;
			}
			lastChanged = newLines.length - 1;
		}
		if (firstChanged !== -1) {
			// 扩展变更范围以包含 Kitty 图片行
			lastChanged = this.expandLastChangedForKittyImages(firstChanged, lastChanged);
		}
		// 纯追加模式（第一行变更正好是旧内容的下一行）
		const appendStart = appendedLines && firstChanged === this.previousLines.length && firstChanged > 0;

		// 无变更：只需更新硬件光标位置
		// No changes - but still need to update hardware cursor position if it moved
		if (firstChanged === -1) {
			this.positionHardwareCursor(cursorPos, newLines.length);
			this.previousViewportTop = prevViewportTop;
			this.previousHeight = height;
			return;
		}

		// 所有变更都是删除行（没有新行需要渲染，只需清除旧行）
		// All changes are in deleted lines (nothing to render, just clear)
		if (firstChanged >= newLines.length) {
			if (this.previousLines.length > newLines.length) {
				let buffer = "\x1b[?2026h";
				buffer += this.deleteChangedKittyImages(firstChanged, lastChanged);
				// 移动到新内容末尾（空内容时限制为 0）
				// Move to end of new content (clamp to 0 for empty content)
				const targetRow = Math.max(0, newLines.length - 1);
				if (targetRow < prevViewportTop) {
					logRedraw(`deleted lines moved viewport up (${targetRow} < ${prevViewportTop})`);
					fullRender(true);
					return;
				}
				const lineDiff = computeLineDiff(targetRow);
				if (lineDiff > 0) buffer += `\x1b[${lineDiff}B`;
				else if (lineDiff < 0) buffer += `\x1b[${-lineDiff}A`;
				buffer += "\r";
				// 清除多余行（不触发滚动）
				// Clear extra lines without scrolling
				const extraLines = this.previousLines.length - newLines.length;
				if (extraLines > height) {
					logRedraw(`extraLines > height (${extraLines} > ${height})`);
					fullRender(true);
					return;
				}
				if (extraLines > 0) {
					buffer += "\x1b[1B";
				}
				for (let i = 0; i < extraLines; i++) {
					buffer += "\r\x1b[2K";
					if (i < extraLines - 1) buffer += "\x1b[1B";
				}
				if (extraLines > 0) {
					buffer += `\x1b[${extraLines}A`;
				}
				buffer += "\x1b[?2026l";
				this.terminal.write(buffer);
				this.cursorRow = targetRow;
				this.hardwareCursorRow = targetRow;
			}
			this.positionHardwareCursor(cursorPos, newLines.length);
			this.previousLines = newLines;
			this.previousKittyImageIds = this.collectKittyImageIds(newLines);
			this.previousWidth = width;
			this.previousHeight = height;
			this.previousViewportTop = prevViewportTop;
			return;
		}

		// 差异渲染只能操作实际可见的范围。
		// 如果第一行变更在视口之前，无法进行差异渲染，必须全量重绘。
		// Differential rendering can only touch what was actually visible.
		// If the first changed line is above the previous viewport, we need a full redraw.
		if (firstChanged < prevViewportTop) {
			logRedraw(`firstChanged < viewportTop (${firstChanged} < ${prevViewportTop})`);
			fullRender(true);
			return;
		}

		// 执行差异渲染：从第一行变更到末尾
		// Render from first changed line to end
		// Build buffer with all updates wrapped in synchronized output
		let buffer = "\x1b[?2026h"; // 开始同步输出 // Begin synchronized output
		buffer += this.deleteChangedKittyImages(firstChanged, lastChanged);
		const prevViewportBottom = prevViewportTop + height - 1;
		// 追加模式时先定位到上一行（然后输出 \r\n 开始新行）
		const moveTargetRow = appendStart ? firstChanged - 1 : firstChanged;
		if (moveTargetRow > prevViewportBottom) {
			// 目标行在视口下方：先将光标移到屏幕底部，再通过 \r\n 滚动到目标行
			const currentScreenRow = Math.max(0, Math.min(height - 1, hardwareCursorRow - prevViewportTop));
			const moveToBottom = height - 1 - currentScreenRow;
			if (moveToBottom > 0) {
				buffer += `\x1b[${moveToBottom}B`;
			}
			const scroll = moveTargetRow - prevViewportBottom;
			buffer += "\r\n".repeat(scroll);
			prevViewportTop += scroll;
			viewportTop += scroll;
			hardwareCursorRow = moveTargetRow;
		}

		// 移动光标到第一行变更（使用 hardwareCursorRow 计算实际位置）
		// Move cursor to first changed line (use hardwareCursorRow for actual position)
		const lineDiff = computeLineDiff(moveTargetRow);
		if (lineDiff > 0) {
			buffer += `\x1b[${lineDiff}B`; // 下移 // Move down
		} else if (lineDiff < 0) {
			buffer += `\x1b[${-lineDiff}A`; // 上移 // Move up
		}

		buffer += appendStart ? "\r\n" : "\r"; // 移至第 0 列 // Move to column 0

		// 只渲染变更行（firstChanged 到 lastChanged），不是所有行到末尾。
		// 这样在只有单行变化时（如旋转动画）能减少闪烁。
		// Only render changed lines (firstChanged to lastChanged), not all lines to end
		// This reduces flicker when only a single line changes (e.g., spinner animation)
		const renderEnd = Math.min(lastChanged, newLines.length - 1);
		for (let i = firstChanged; i <= renderEnd; i++) {
			if (i > firstChanged) buffer += "\r\n";
			buffer += "\x1b[2K"; // 清除当前行 // Clear current line
			const line = newLines[i];
			const isImage = isImageLine(line);
			if (!isImage && visibleWidth(line) > width) {
				// 超宽是致命错误：记录所有行到崩溃日志文件，停止 TUI，抛出异常
				// Log all lines to crash file for debugging
				const crashLogPath = path.join(os.homedir(), ".pi", "agent", "pi-crash.log");
				const crashData = [
					`Crash at ${new Date().toISOString()}`,
					`Terminal width: ${width}`,
					`Line ${i} visible width: ${visibleWidth(line)}`,
					"",
					"=== All rendered lines ===",
					...newLines.map((l, idx) => `[${idx}] (w=${visibleWidth(l)}) ${l}`),
					"",
				].join("\n");
				fs.mkdirSync(path.dirname(crashLogPath), { recursive: true });
				fs.writeFileSync(crashLogPath, crashData);

				// 停止 TUI 后再抛出异常
				// Clean up terminal state before throwing
				this.stop();

				const errorMsg = [
					`Rendered line ${i} exceeds terminal width (${visibleWidth(line)} > ${width}).`,
					"",
					"This is likely caused by a custom TUI component not truncating its output.",
					"Use visibleWidth() to measure and truncateToWidth() to truncate lines.",
					"",
					`Debug log written to: ${crashLogPath}`,
				].join("\n");
				throw new Error(errorMsg);
			}
			buffer += line;
		}

		// 跟踪渲染后光标结束位置
		// Track where cursor ended up after rendering
		let finalCursorRow = renderEnd;

		// 如果之前行数更多，清除多余行并将光标移回
		// If we had more lines before, clear them and move cursor back
		if (this.previousLines.length > newLines.length) {
			// 先移到新内容末尾（如果渲染停止点在此之前）
			// Move to end of new content first if we stopped before it
			if (renderEnd < newLines.length - 1) {
				const moveDown = newLines.length - 1 - renderEnd;
				buffer += `\x1b[${moveDown}B`;
				finalCursorRow = newLines.length - 1;
			}
			const extraLines = this.previousLines.length - newLines.length;
			for (let i = newLines.length; i < this.previousLines.length; i++) {
				buffer += "\r\n\x1b[2K";
			}
			// 将光标移回新内容末尾
			// Move cursor back to end of new content
			buffer += `\x1b[${extraLines}A`;
		}

		buffer += "\x1b[?2026l"; // 结束同步输出 // End synchronized output

		// 调试模式：将渲染详情写入日志文件
		if (process.env.PI_TUI_DEBUG === "1") {
			const debugDir = "/tmp/tui";
			fs.mkdirSync(debugDir, { recursive: true });
			const debugPath = path.join(debugDir, `render-${Date.now()}-${Math.random().toString(36).slice(2)}.log`);
			const debugData = [
				`firstChanged: ${firstChanged}`,
				`viewportTop: ${viewportTop}`,
				`cursorRow: ${this.cursorRow}`,
				`height: ${height}`,
				`lineDiff: ${lineDiff}`,
				`hardwareCursorRow: ${hardwareCursorRow}`,
				`renderEnd: ${renderEnd}`,
				`finalCursorRow: ${finalCursorRow}`,
				`cursorPos: ${JSON.stringify(cursorPos)}`,
				`newLines.length: ${newLines.length}`,
				`previousLines.length: ${this.previousLines.length}`,
				"",
				"=== newLines ===",
				JSON.stringify(newLines, null, 2),
				"",
				"=== previousLines ===",
				JSON.stringify(this.previousLines, null, 2),
				"",
				"=== buffer ===",
				JSON.stringify(buffer),
			].join("\n");
			fs.writeFileSync(debugPath, debugData);
		}

		// 一次性写入整个缓冲区到终端（原子性输出）
		// Write entire buffer at once
		this.terminal.write(buffer);

		// 更新光标位置供下一次渲染使用
		// Track cursor position for next render
		// cursorRow 跟踪内容末尾（用于视口计算）
		// hardwareCursorRow 跟踪终端实际光标位置（用于光标移动）
		// cursorRow tracks end of content (for viewport calculation)
		// hardwareCursorRow tracks actual terminal cursor position (for movement)
		this.cursorRow = Math.max(0, newLines.length - 1);
		this.hardwareCursorRow = finalCursorRow;
		// 跟踪终端工作区域（只增长不缩减，除非清屏）
		// Track terminal's working area (grows but doesn't shrink unless cleared)
		this.maxLinesRendered = Math.max(this.maxLinesRendered, newLines.length);
		this.previousViewportTop = Math.max(prevViewportTop, finalCursorRow - height + 1);

		// 定位硬件光标以支持 IME 输入法候选窗口对齐
		// Position hardware cursor for IME
		this.positionHardwareCursor(cursorPos, newLines.length);

		this.previousLines = newLines;
		this.previousKittyImageIds = this.collectKittyImageIds(newLines);
		this.previousWidth = width;
		this.previousHeight = height;
	}

	/**
	 * 定位硬件光标，用于 IME（输入法）候选窗口的位置对齐。
	 * 根据从渲染输出中提取的光标位置，计算并发送转义序列将终端光标移动到目标位置。
	 * @param cursorPos - 从渲染输出中提取的光标位置，null 表示无光标则隐藏
	 * @param totalLines - 渲染总行数
	 *
	 * Position the hardware cursor for IME candidate window.
	 * @param cursorPos The cursor position extracted from rendered output, or null
	 * @param totalLines Total number of rendered lines
	 */
	private positionHardwareCursor(cursorPos: { row: number; col: number } | null, totalLines: number): void {
		if (!cursorPos || totalLines <= 0) {
			this.terminal.hideCursor();
			return;
		}

		// 将光标位置限制到有效范围内
		// Clamp cursor position to valid range
		const targetRow = Math.max(0, Math.min(cursorPos.row, totalLines - 1));
		const targetCol = Math.max(0, cursorPos.col);

		// 从当前位置移动到目标位置
		// Move cursor from current position to target
		const rowDelta = targetRow - this.hardwareCursorRow;
		let buffer = "";
		if (rowDelta > 0) {
			buffer += `\x1b[${rowDelta}B`; // 下移 // Move down
		} else if (rowDelta < 0) {
			buffer += `\x1b[${-rowDelta}A`; // 上移 // Move up
		}
		// 移到绝对列（终端列号从 1 开始）
		// Move to absolute column (1-indexed)
		buffer += `\x1b[${targetCol + 1}G`;

		if (buffer) {
			this.terminal.write(buffer);
		}

		this.hardwareCursorRow = targetRow;
		// 根据配置决定是否显示硬件光标
		if (this.showHardwareCursor) {
			this.terminal.showCursor();
		} else {
			this.terminal.hideCursor();
		}
	}
}
