/**
 * @fileoverview 编辑器组件 — 多行文本编辑器，TUI 引擎的核心交互入口。
 *
 * 作为交互模式下用户输入和编辑的主界面，基于差异渲染将光标状态映射到
 * 视觉行（Visual Line），在终端中用反向颜色高亮显示光标位置。
 *
 * 提供以下功能：
 *   - 多行文本编辑（插入、删除、换行）
 *   - 光标移动（字符按 grapheme 粒度、词、行、首尾、翻页）
 *   - Kill Ring（Emacs 风格的可累积剪贴板环）
 *   - 撤销/重做（UndoStack，支持鱼式按键合并）
 *   - 自动补全（通过 AutocompleteProvider 接口，支持斜杠命令和符号补全）
 *   - 粘贴折叠（大内容折叠为 [paste #N +M lines] 标记，逻辑光标按原子段处理）
 *   - 历史记录导航（上/下键浏览已提交的输入）
 *   - 字符跳转（Ctrl+F/B 触发，输入目标字符后定位）
 *   - 垂直滚动（编辑器区域为终端高度的 30%，超出部分可滚动）
 *   - Unicode 感知（使用 Intl.Segmenter 正确处理 grapheme cluster）
 *
 * 核心架构：
 *   逻辑行（EditorState.lines）→ 视觉行映射（wordWrapLine）→ 光标定位
 *   （buildVisualLineMap + moveToVisualLine）→ 差异渲染（render）。
 *   segmentWithMarkers 确保粘贴标记作为原子段参与所有编辑操作。
 *
 * @see 术语索引 — 编辑器组件 / Editor Component
 * @see 术语索引 — TUI 引擎 / Terminal UI Engine
 */
import type { AutocompleteProvider, AutocompleteSuggestions } from "../autocomplete.js";

import { getKeybindings } from "../keybindings.js";
import { decodePrintableKey, matchesKey } from "../keys.js";
import { KillRing } from "../kill-ring.js";
import { type Component, CURSOR_MARKER, type Focusable, type TUI } from "../tui.js";
import { UndoStack } from "../undo-stack.js";
import { getSegmenter, isPunctuationChar, isWhitespaceChar, truncateToWidth, visibleWidth } from "../utils.js";
import { SelectList, type SelectListLayoutOptions, type SelectListTheme } from "./select-list.js";

const baseSegmenter = getSegmenter();

/**
 * 粘贴标记正则（全局），匹配 `[paste #1 +123 lines]` 或 `[paste #2 1234 chars]`。
 * 用于在文本扫描中定位所有粘贴标记的位置范围，支撑 segmentWithMarkers 的原子段合并。
 */
const PASTE_MARKER_REGEX = /\[paste #(\d+)( (\+\d+ lines|\d+ chars))?\]/g;

/** 非全局版本，用于单段测试 —— isPasteMarker 使用它判定一个完整段是否为粘贴标记。 */
const PASTE_MARKER_SINGLE = /^\[paste #(\d+)( (\+\d+ lines|\d+ chars))?\]$/;

/** Check if a segment is a paste marker (i.e. was merged by segmentWithMarkers). */
function isPasteMarker(segment: string): boolean {
	return segment.length >= 10 && PASTE_MARKER_SINGLE.test(segment);
}

/**
 * 粘贴标记感知的文本分段器。
 *
 * 在 Intl.Segmenter 基础上，将位于有效粘贴标记范围内的 grapheme 合并为单个原子段。
 * 这确保粘贴标记 [paste #1 +123 lines] 在光标移动、删除、换行中始终作为不可分割的单元处理。
 * 仅当 validIds 非空且文本包含 "[paste #" 时才走慢路径，否则直接使用标准 Segmenter（快速路径）。
 *
 * @param text - 要分段的文本
 * @param validIds - 当前有效的粘贴 ID 集合（来自 pastes Map 的 key）
 */
function segmentWithMarkers(text: string, validIds: Set<number>): Iterable<Intl.SegmentData> {
	// Fast path: no paste markers in the text or no valid IDs.
	if (validIds.size === 0 || !text.includes("[paste #")) {
		return baseSegmenter.segment(text);
	}

	// Find all marker spans with valid IDs.
	const markers: Array<{ start: number; end: number }> = [];
	for (const m of text.matchAll(PASTE_MARKER_REGEX)) {
		const id = Number.parseInt(m[1]!, 10);
		if (!validIds.has(id)) continue;
		markers.push({ start: m.index, end: m.index + m[0].length });
	}
	if (markers.length === 0) {
		return baseSegmenter.segment(text);
	}

	// Build merged segment list.
	const baseSegments = baseSegmenter.segment(text);
	const result: Intl.SegmentData[] = [];
	let markerIdx = 0;

	for (const seg of baseSegments) {
		// Skip past markers that are entirely before this segment.
		while (markerIdx < markers.length && markers[markerIdx]!.end <= seg.index) {
			markerIdx++;
		}

		const marker = markerIdx < markers.length ? markers[markerIdx]! : null;

		if (marker && seg.index >= marker.start && seg.index < marker.end) {
			// This segment falls inside a marker.
			// If this is the first segment of the marker, emit a merged segment.
			if (seg.index === marker.start) {
				const markerText = text.slice(marker.start, marker.end);
				result.push({
					segment: markerText,
					index: marker.start,
					input: text,
				});
			}
			// Otherwise skip (already merged into the first segment).
		} else {
			result.push(seg);
		}
	}

	return result;
}

/**
 * Represents a chunk of text for word-wrap layout.
 * Tracks both the text content and its position in the original line.
 */
export interface TextChunk {
	text: string;
	startIndex: number;
	endIndex: number;
}

/**
 * Split a line into word-wrapped chunks.
 * Wraps at word boundaries when possible, falling back to character-level
 * wrapping for words longer than the available width.
 *
 * @param line - The text line to wrap
 * @param maxWidth - Maximum visible width per chunk
 * @param preSegmented - Optional pre-segmented graphemes (e.g. with paste-marker awareness).
 *                       When omitted the default Intl.Segmenter is used.
 * @returns Array of chunks with text and position information
 */
export function wordWrapLine(line: string, maxWidth: number, preSegmented?: Intl.SegmentData[]): TextChunk[] {
	if (!line || maxWidth <= 0) {
		return [{ text: "", startIndex: 0, endIndex: 0 }];
	}

	const lineWidth = visibleWidth(line);
	if (lineWidth <= maxWidth) {
		return [{ text: line, startIndex: 0, endIndex: line.length }];
	}

	const chunks: TextChunk[] = [];
	const segments = preSegmented ?? [...baseSegmenter.segment(line)];

	let currentWidth = 0;
	let chunkStart = 0;

	// Wrap opportunity: the position after the last whitespace before a non-whitespace
	// grapheme, i.e. where a line break is allowed.
	let wrapOppIndex = -1;
	let wrapOppWidth = 0;

	for (let i = 0; i < segments.length; i++) {
		const seg = segments[i]!;
		const grapheme = seg.segment;
		const gWidth = visibleWidth(grapheme);
		const charIndex = seg.index;
		const isWs = !isPasteMarker(grapheme) && isWhitespaceChar(grapheme);

		// Overflow check before advancing.
		if (currentWidth + gWidth > maxWidth) {
			if (wrapOppIndex >= 0 && currentWidth - wrapOppWidth + gWidth <= maxWidth) {
				// Backtrack to last wrap opportunity (the remaining content
				// plus the current grapheme still fits within maxWidth).
				chunks.push({ text: line.slice(chunkStart, wrapOppIndex), startIndex: chunkStart, endIndex: wrapOppIndex });
				chunkStart = wrapOppIndex;
				currentWidth -= wrapOppWidth;
			} else if (chunkStart < charIndex) {
				// No viable wrap opportunity: force-break at current position.
				// This also handles the case where backtracking to a word
				// boundary wouldn't help because the remaining content plus
				// the current grapheme (e.g. a wide character) still exceeds
				// maxWidth.
				chunks.push({ text: line.slice(chunkStart, charIndex), startIndex: chunkStart, endIndex: charIndex });
				chunkStart = charIndex;
				currentWidth = 0;
			}
			wrapOppIndex = -1;
		}

		if (gWidth > maxWidth) {
			// Single atomic segment wider than maxWidth (e.g. paste marker
			// in a narrow terminal). Re-wrap it at grapheme granularity.

			// The segment remains logically atomic for cursor
			// movement / editing — the split is purely visual for word-wrap layout.
			const subChunks = wordWrapLine(grapheme, maxWidth);
			for (let j = 0; j < subChunks.length - 1; j++) {
				const sc = subChunks[j]!;
				chunks.push({ text: sc.text, startIndex: charIndex + sc.startIndex, endIndex: charIndex + sc.endIndex });
			}
			const last = subChunks[subChunks.length - 1]!;
			chunkStart = charIndex + last.startIndex;
			currentWidth = visibleWidth(last.text);
			wrapOppIndex = -1;
			continue;
		}

		// Advance.
		currentWidth += gWidth;

		// Record wrap opportunity: whitespace followed by non-whitespace.
		// Multiple spaces join (no break between them); the break point is
		// after the last space before the next word.
		const next = segments[i + 1];
		if (isWs && next && (isPasteMarker(next.segment) || !isWhitespaceChar(next.segment))) {
			wrapOppIndex = next.index;
			wrapOppWidth = currentWidth;
		}
	}

	// Push final chunk.
	chunks.push({ text: line.slice(chunkStart), startIndex: chunkStart, endIndex: line.length });

	return chunks;
}

// Kitty CSI-u sequences for printable keys, including optional shifted/base codepoints.
/**
 * 编辑器内部状态 —— 逻辑行数组 + 光标位置。
 * 作为 UndoStack 的快照单元，每次变更前后 push/pop 此结构。
 */
interface EditorState {
	lines: string[];
	cursorLine: number;
	cursorCol: number;
}

/**
 * 渲染层布局行 —— 一行视觉文本 + 光标信息。
 * render() 将逻辑行展开为 LayoutLine[]，决定哪些行可见以及光标落在哪个视觉行。
 */
interface LayoutLine {
	text: string;
	hasCursor: boolean;
	cursorPos?: number;
}

export interface EditorTheme {
	borderColor: (str: string) => string;
	selectList: SelectListTheme;
}

export interface EditorOptions {
	paddingX?: number;
	autocompleteMaxVisible?: number;
}

/** 斜杠命令补全列表的布局参数：限制首列宽度范围，适配窄终端。 */
const SLASH_COMMAND_SELECT_LIST_LAYOUT: SelectListLayoutOptions = {
	minPrimaryColumnWidth: 12,
	maxPrimaryColumnWidth: 32,
};

/** @/# 符号补全的防抖延迟（毫秒），避免每次按键都触发昂贵的文件系统查询。 */
const ATTACHMENT_AUTOCOMPLETE_DEBOUNCE_MS = 20;

/**
 * 编辑器组件 — TUI 引擎的多行文本编辑器。
 *
 * 实现 Component 和 Focusable 接口，由交互模式的 InputArea 容器组件实例化。
 * 内部将逻辑行（EditorState.lines[]）通过 wordWrapLine 展开为视觉行布局，
 * 再计算滚动窗口和光标位置后输出 ANSI 转义序列。
 *
 * 输入处理 handleInput() 是事件分发的中央枢纽：
 *   按键 → 键绑定匹配 → 分支到光标移动 / 编辑操作 / 补全 / 提交。
 *
 * 支持的类型前缀补全通道：
 *   - "/" → 斜杠命令（首行有效）
 *   - "@" / "#" → 附件/文件符号补全（带防抖）
 *   - Tab → 强制触发文件路径补全
 */
export class Editor implements Component, Focusable {
	private state: EditorState = {
		lines: [""],
		cursorLine: 0,
		cursorCol: 0,
	};

	/** Focusable 接口 —— TUI 引擎在焦点变更时设置此标志 */
	focused: boolean = false;

	protected tui: TUI;
	private theme: EditorTheme;
	private paddingX: number = 0;

	/** 上一次渲染宽度，用于将逻辑光标映射到视觉行时的换行计算 */
	private lastWidth: number = 80;

	/** 当前滚动偏移量（视觉行索引偏移），render() 中根据光标位置动态调整 */
	private scrollOffset: number = 0;

	// 边框颜色（可由外部动态修改，如错误状态变红）
	public borderColor: (str: string) => string;

	// ── 自动补全子系统 ──
	/** 补全数据提供者，外部注入（交互模式中由 AutocompleteManager 桥接） */
	private autocompleteProvider?: AutocompleteProvider;
	/** 补全弹出列表（复用 SelectList 组件） */
	private autocompleteList?: SelectList;
	/** 补全状态：regular = 自动触发，force = Tab 强制触发，null = 关闭 */
	private autocompleteState: "regular" | "force" | null = null;
	/** 当前补全的前缀文本（用于匹配和替换） */
	private autocompletePrefix: string = "";
	/** 补全列表最大可见条目数（默认 5，范围 3-20） */
	private autocompleteMaxVisible: number = 5;
	/** 用于取消进行中的补全请求，防抖/文本变更时触发 */
	private autocompleteAbort?: AbortController;
	/** 符号补全（@/#）的防抖定时器引用 */
	private autocompleteDebounceTimer?: ReturnType<typeof setTimeout>;
	/** 排队中的补全请求 Promise 链，确保请求按顺序处理 */
	private autocompleteRequestTask: Promise<void> = Promise.resolve();
	/** 启动令牌 —— 递增使旧防抖/请求失效 */
	private autocompleteStartToken: number = 0;
	/** 请求序号 —— 递增后检查 isAutocompleteRequestCurrent 以丢弃过期结果 */
	private autocompleteRequestId: number = 0;

	// ── 粘贴折叠子系统 ──
	/** 粘贴 ID → 展开内容映射，大粘贴（>10行或>1000字符）折叠为标记后存储于此 */
	private pastes: Map<number, string> = new Map();
	/** 递增的粘贴序号，用于生成唯一的 [paste #N] 标记 */
	private pasteCounter: number = 0;

	// ── 括号粘贴模式缓冲 ──
	/** 括弧粘贴模式下累积的原始内容（终端以 \x1b[200~ ... \x1b[201~ 包裹的内容） */
	private pasteBuffer: string = "";
	/** 是否处于括弧粘贴模式中（收到 \x1b[200~ 后为 true） */
	private isInPaste: boolean = false;

	// ── 历史记录导航 ──
	/** 已提交的输入历史（上/下键浏览），最近提交的在索引 0 */
	private history: string[] = [];
	/** 历史浏览索引：-1 表示未浏览，0=最近，递增=更早 */
	private historyIndex: number = -1;

	// ── Kill Ring（Emacs 风格剪贴板环）──
	/** 可累积的剪贴板环，连续 kill 操作追加到同一入口 */
	private killRing = new KillRing();
	/**
	 * 上一次编辑操作类型 —— 用于确定：
	 * 1. kill 是否累积（连续 kill 追加到同一 ring 入口）
	 * 2. yank-pop 是否可用（仅在上次为 yank 时有效）
	 * 3. 输入合并（连续输入字符合并为一个 undo 单元）
	 */
	private lastAction: "kill" | "yank" | "type-word" | null = null;

	// ── 字符跳转模式 ──
	/** 跳转方向（forward / backward），非 null 时等待下一个字符作为跳转目标 */
	private jumpMode: "forward" | "backward" | null = null;

	// ── 垂直光标移动的粘性列 ──
	/** 记录用户期望的视觉列，使上/下移动时在短行也能保持列位置（sticky column） */
	private preferredVisualCol: number | null = null;

	/**
	 * 当光标被吸附到原子段（如粘贴标记）开头时，存储吸附前的 cursorCol，
	 * 以便下次垂直移动时可以正确解析视觉列位置。
	 */
	private snappedFromCursorCol: number | null = null;

	// ── 撤销/重做 ──
	/** 编辑器状态的快照栈，每次变更前 push 当前状态，undo 时 pop 恢复 */
	private undoStack = new UndoStack<EditorState>();

	/** 提交回调 —— 用户按 Enter 时调用，传入展开粘贴标记后的文本 */
	public onSubmit?: (text: string) => void;
	/** 变更回调 —— 文本每次修改后调用（包括撤销/重做/历史导航），传入当前文本 */
	public onChange?: (text: string) => void;
	/** 禁用提交 —— 为 true 时忽略 Enter 键（如 Agent 运行中防止误提交） */
	public disableSubmit: boolean = false;

	/**
	 * 构造编辑器实例。
	 *
	 * @param tui - TUI 引擎实例，用于 requestRender 和 terminal 尺寸查询
	 * @param theme - 编辑器主题（边框颜色 + SelectList 子主题）
	 * @param options - 可选配置：paddingX（水平内边距）、autocompleteMaxVisible（补全列表最大可见条目数）
	 */
	constructor(tui: TUI, theme: EditorTheme, options: EditorOptions = {}) {
		this.tui = tui;
		this.theme = theme;
		this.borderColor = theme.borderColor;
		const paddingX = options.paddingX ?? 0;
		this.paddingX = Number.isFinite(paddingX) ? Math.max(0, Math.floor(paddingX)) : 0;
		const maxVisible = options.autocompleteMaxVisible ?? 5;
		this.autocompleteMaxVisible = Number.isFinite(maxVisible) ? Math.max(3, Math.min(20, Math.floor(maxVisible))) : 5;
	}

	/** Set of currently valid paste IDs, for marker-aware segmentation. */
	private validPasteIds(): Set<number> {
		return new Set(this.pastes.keys());
	}

	/** Segment text with paste-marker awareness, only merging markers with valid IDs. */
	private segment(text: string): Iterable<Intl.SegmentData> {
		return segmentWithMarkers(text, this.validPasteIds());
	}

	getPaddingX(): number {
		return this.paddingX;
	}

	setPaddingX(padding: number): void {
		const newPadding = Number.isFinite(padding) ? Math.max(0, Math.floor(padding)) : 0;
		if (this.paddingX !== newPadding) {
			this.paddingX = newPadding;
			this.tui.requestRender();
		}
	}

	getAutocompleteMaxVisible(): number {
		return this.autocompleteMaxVisible;
	}

	setAutocompleteMaxVisible(maxVisible: number): void {
		const newMaxVisible = Number.isFinite(maxVisible) ? Math.max(3, Math.min(20, Math.floor(maxVisible))) : 5;
		if (this.autocompleteMaxVisible !== newMaxVisible) {
			this.autocompleteMaxVisible = newMaxVisible;
			this.tui.requestRender();
		}
	}

	setAutocompleteProvider(provider: AutocompleteProvider): void {
		this.cancelAutocomplete();
		this.autocompleteProvider = provider;
	}

	/**
	 * Add a prompt to history for up/down arrow navigation.
	 * Called after successful submission.
	 */
	addToHistory(text: string): void {
		const trimmed = text.trim();
		if (!trimmed) return;
		// Don't add consecutive duplicates
		if (this.history.length > 0 && this.history[0] === trimmed) return;
		this.history.unshift(trimmed);
		// Limit history size
		if (this.history.length > 100) {
			this.history.pop();
		}
	}

	private isEditorEmpty(): boolean {
		return this.state.lines.length === 1 && this.state.lines[0] === "";
	}

	private isOnFirstVisualLine(): boolean {
		const visualLines = this.buildVisualLineMap(this.lastWidth);
		const currentVisualLine = this.findCurrentVisualLine(visualLines);
		return currentVisualLine === 0;
	}

	private isOnLastVisualLine(): boolean {
		const visualLines = this.buildVisualLineMap(this.lastWidth);
		const currentVisualLine = this.findCurrentVisualLine(visualLines);
		return currentVisualLine === visualLines.length - 1;
	}

	/**
	 * 历史记录导航 —— 上/下键在已提交的输入历史中切换。
	 *
	 * @param direction - -1 = 向上（更早的历史），1 = 向下（更新的历史）
	 */
	private navigateHistory(direction: 1 | -1): void {
		this.lastAction = null;
		if (this.history.length === 0) return;

		const newIndex = this.historyIndex - direction; // Up(-1) increases index, Down(1) decreases
		if (newIndex < -1 || newIndex >= this.history.length) return;

		// Capture state when first entering history browsing mode
		if (this.historyIndex === -1 && newIndex >= 0) {
			this.pushUndoSnapshot();
		}

		this.historyIndex = newIndex;

		if (this.historyIndex === -1) {
			// Returned to "current" state - clear editor
			this.setTextInternal("");
		} else {
			this.setTextInternal(this.history[this.historyIndex] || "");
		}
	}

	/** Internal setText that doesn't reset history state - used by navigateHistory */
	private setTextInternal(text: string): void {
		const lines = text.split("\n");
		this.state.lines = lines.length === 0 ? [""] : lines;
		this.state.cursorLine = this.state.lines.length - 1;
		this.setCursorCol(this.state.lines[this.state.cursorLine]?.length || 0);
		// Reset scroll - render() will adjust to show cursor
		this.scrollOffset = 0;

		if (this.onChange) {
			this.onChange(this.getText());
		}
	}

	invalidate(): void {
		// 当前无需要失效的缓存状态
	}

	/**
	 * 渲染编辑器为 ANSI 转义序列字符串数组。
	 *
	 * 渲染流程：
	 *   1. 计算内容宽度（width - 2 * paddingX），为无内边距模式预留 1 列给光标
	 *   2. wordWrapLine 将逻辑行展开为 LayoutLine[]（layoutText）
	 *   3. 根据光标所在视觉行调整 scrollOffset，保持光标在可见区域内
	 *   4. 遍历可见的视觉行切片，在光标行插入反向颜色 ANSI 高亮
	 *   5. 顶部/底部分别渲染"↑ N more" / "↓ N more"滚动指示器
	 *   6. 如果补全列表激活，追加补全行
	 *
	 * 光标渲染策略：
	 *   - 光标在字符上 → 用 \x1b[7m 反向高亮该 grapheme
	 *   - 光标在行尾 → 高亮一个空格（反向颜色块）
	 *   - 光标溢出到内边距 → 截断右侧内边距 1 字符
	 *   - 失焦或补全显示中 → 不发射 CURSOR_MARKER
	 *
	 * @param width - 终端可用宽度（列数）
	 * @returns ANSI 转义字符串数组，每行一个元素
	 */
	render(width: number): string[] {
		const maxPadding = Math.max(0, Math.floor((width - 1) / 2));
		const paddingX = Math.min(this.paddingX, maxPadding);
		const contentWidth = Math.max(1, width - paddingX * 2);

		// Layout width: with padding the cursor can overflow into it,
		// without padding we reserve 1 column for the cursor.
		const layoutWidth = Math.max(1, contentWidth - (paddingX ? 0 : 1));

		// Store for cursor navigation (must match wrapping width)
		this.lastWidth = layoutWidth;

		const horizontal = this.borderColor("─");

		// Layout the text
		const layoutLines = this.layoutText(layoutWidth);

		// Calculate max visible lines: 30% of terminal height, minimum 5 lines
		const terminalRows = this.tui.terminal.rows;
		const maxVisibleLines = Math.max(5, Math.floor(terminalRows * 0.3));

		// Find the cursor line index in layoutLines
		let cursorLineIndex = layoutLines.findIndex((line) => line.hasCursor);
		if (cursorLineIndex === -1) cursorLineIndex = 0;

		// Adjust scroll offset to keep cursor visible
		if (cursorLineIndex < this.scrollOffset) {
			this.scrollOffset = cursorLineIndex;
		} else if (cursorLineIndex >= this.scrollOffset + maxVisibleLines) {
			this.scrollOffset = cursorLineIndex - maxVisibleLines + 1;
		}

		// Clamp scroll offset to valid range
		const maxScrollOffset = Math.max(0, layoutLines.length - maxVisibleLines);
		this.scrollOffset = Math.max(0, Math.min(this.scrollOffset, maxScrollOffset));

		// Get visible lines slice
		const visibleLines = layoutLines.slice(this.scrollOffset, this.scrollOffset + maxVisibleLines);

		const result: string[] = [];
		const leftPadding = " ".repeat(paddingX);
		const rightPadding = leftPadding;

		// Render top border (with scroll indicator if scrolled down)
		if (this.scrollOffset > 0) {
			const indicator = `─── ↑ ${this.scrollOffset} more `;
			const remaining = width - visibleWidth(indicator);
			if (remaining >= 0) {
				result.push(this.borderColor(indicator + "─".repeat(remaining)));
			} else {
				result.push(this.borderColor(truncateToWidth(indicator, width)));
			}
		} else {
			result.push(horizontal.repeat(width));
		}

		// Render each visible layout line
		// Emit hardware cursor marker only when focused and not showing autocomplete
		const emitCursorMarker = this.focused && !this.autocompleteState;

		for (const layoutLine of visibleLines) {
			let displayText = layoutLine.text;
			let lineVisibleWidth = visibleWidth(layoutLine.text);
			let cursorInPadding = false;

			// Add cursor if this line has it
			if (layoutLine.hasCursor && layoutLine.cursorPos !== undefined) {
				const before = displayText.slice(0, layoutLine.cursorPos);
				const after = displayText.slice(layoutLine.cursorPos);

				// Hardware cursor marker (zero-width, emitted before fake cursor for IME positioning)
				const marker = emitCursorMarker ? CURSOR_MARKER : "";

				if (after.length > 0) {
					// Cursor is on a character (grapheme) - replace it with highlighted version
					// Get the first grapheme from 'after'
					const afterGraphemes = [...this.segment(after)];
					const firstGrapheme = afterGraphemes[0]?.segment || "";
					const restAfter = after.slice(firstGrapheme.length);
					const cursor = `\x1b[7m${firstGrapheme}\x1b[0m`;
					displayText = before + marker + cursor + restAfter;
					// lineVisibleWidth stays the same - we're replacing, not adding
				} else {
					// Cursor is at the end - add highlighted space
					const cursor = "\x1b[7m \x1b[0m";
					displayText = before + marker + cursor;
					lineVisibleWidth = lineVisibleWidth + 1;
					// If cursor overflows content width into the padding, flag it
					if (lineVisibleWidth > contentWidth && paddingX > 0) {
						cursorInPadding = true;
					}
				}
			}

			// Calculate padding based on actual visible width
			const padding = " ".repeat(Math.max(0, contentWidth - lineVisibleWidth));
			const lineRightPadding = cursorInPadding ? rightPadding.slice(1) : rightPadding;

			// Render the line (no side borders, just horizontal lines above and below)
			result.push(`${leftPadding}${displayText}${padding}${lineRightPadding}`);
		}

		// Render bottom border (with scroll indicator if more content below)
		const linesBelow = layoutLines.length - (this.scrollOffset + visibleLines.length);
		if (linesBelow > 0) {
			const indicator = `─── ↓ ${linesBelow} more `;
			const remaining = width - visibleWidth(indicator);
			result.push(this.borderColor(indicator + "─".repeat(Math.max(0, remaining))));
		} else {
			result.push(horizontal.repeat(width));
		}

		// Add autocomplete list if active
		if (this.autocompleteState && this.autocompleteList) {
			const autocompleteResult = this.autocompleteList.render(contentWidth);
			for (const line of autocompleteResult) {
				const lineWidth = visibleWidth(line);
				const linePadding = " ".repeat(Math.max(0, contentWidth - lineWidth));
				result.push(`${leftPadding}${line}${linePadding}${rightPadding}`);
			}
		}

		return result;
	}

	/**
	 * 处理终端原始输入 — 编辑器的输入事件中央枢纽。
	 *
	 * 输入分发流程（按优先级）：
	 *   1. 字符跳转模式待命中 — 匹配下一个字符执行跳转
	 *   2. 括弧粘贴模式缓冲 — 累积到结束标记后调用 handlePaste
	 *   3. Ctrl+C — 透传给父组件
	 *   4. 撤销 (Ctrl+/) — undo()
	 *   5. 补全列表激活中 — 上下选择 / Tab确认 / Escape取消
	 *   6. Tab — 触发补全 (handleTabCompletion)
	 *   7. 删除操作 — deleteToLineEnd/deleteToLineStart/deleteWord/deleteChar
	 *   8. Kill Ring — yank(Ctrl+Y) / yankPop(Alt+Y)
	 *   9. 光标移动 — cursorUp/Down/Left/Right / cursorWord / cursorLine
	 *   10. 换行 — Enter / Shift+Enter / Ctrl+Enter
	 *   11. 提交 — Enter 匹配 submit 键绑定时
	 *   12. 翻页 — PageUp / PageDown
	 *   13. 字符跳转触发 — Ctrl+F / Ctrl+B
	 *   14. Shift+Space — 插入空格
	 *   15. 可打印字符 — insertCharacter()
	 *
	 * @param data - 终端发送的原始输入序列（ANSI 转义序列或 UTF-8 文本）
	 */
	handleInput(data: string): void {
		const kb = getKeybindings();

		// ── 字符跳转模式：等待用户在按 Ctrl+F/B 后输入目标字符 ──
		if (this.jumpMode !== null) {
			// 再次按下跳转热键 → 取消跳转模式
			if (kb.matches(data, "tui.editor.jumpForward") || kb.matches(data, "tui.editor.jumpBackward")) {
				this.jumpMode = null;
				return;
			}

			const printable = decodePrintableKey(data) ?? (data.charCodeAt(0) >= 32 ? data : undefined);
			if (printable !== undefined) {
				// 收到可打印字符 → 跳转到该字符的下一次出现
				const direction = this.jumpMode;
				this.jumpMode = null;
				this.jumpToChar(printable, direction);
				return;
			}

			// 收到控制字符 → 取消跳转，回退到正常处理
			this.jumpMode = null;
		}

		// ── 括弧粘贴模式：终端以 \x1b[200~ ... \x1b[201~ 包裹的粘贴内容 ──
		if (data.includes("\x1b[200~")) {
			this.isInPaste = true;
			this.pasteBuffer = "";
			data = data.replace("\x1b[200~", "");
		}

		if (this.isInPaste) {
			this.pasteBuffer += data;
			const endIndex = this.pasteBuffer.indexOf("\x1b[201~");
			if (endIndex !== -1) {
				const pasteContent = this.pasteBuffer.substring(0, endIndex);
				if (pasteContent.length > 0) {
					this.handlePaste(pasteContent);
				}
				this.isInPaste = false;
				// 处理 \x1b[201~ 之后的残留字符
				const remaining = this.pasteBuffer.substring(endIndex + 6);
				this.pasteBuffer = "";
				if (remaining.length > 0) {
					this.handleInput(remaining);
				}
				return;
			}
			return;
		}

		// Ctrl+C → 透传给父组件处理（退出/清空）
		if (kb.matches(data, "tui.input.copy")) {
			return;
		}

		// 撤销 (Ctrl+/)
		if (kb.matches(data, "tui.editor.undo")) {
			this.undo();
			return;
		}

		// ── 补全列表激活时的键处理 ──
		if (this.autocompleteState && this.autocompleteList) {
			if (kb.matches(data, "tui.select.cancel")) {
				this.cancelAutocomplete();
				return;
			}

			if (kb.matches(data, "tui.select.up") || kb.matches(data, "tui.select.down")) {
				this.autocompleteList.handleInput(data);
				return;
			}

			if (kb.matches(data, "tui.input.tab")) {
				const selected = this.autocompleteList.getSelectedItem();
				if (selected && this.autocompleteProvider) {
					this.pushUndoSnapshot();
					this.lastAction = null;
					const result = this.autocompleteProvider.applyCompletion(
						this.state.lines,
						this.state.cursorLine,
						this.state.cursorCol,
						selected,
						this.autocompletePrefix,
					);
					this.state.lines = result.lines;
					this.state.cursorLine = result.cursorLine;
					this.setCursorCol(result.cursorCol);
					this.cancelAutocomplete();
					if (this.onChange) this.onChange(this.getText());
				}
				return;
			}

			if (kb.matches(data, "tui.select.confirm")) {
				const selected = this.autocompleteList.getSelectedItem();
				if (selected && this.autocompleteProvider) {
					this.pushUndoSnapshot();
					this.lastAction = null;
					const result = this.autocompleteProvider.applyCompletion(
						this.state.lines,
						this.state.cursorLine,
						this.state.cursorCol,
						selected,
						this.autocompletePrefix,
					);
					this.state.lines = result.lines;
					this.state.cursorLine = result.cursorLine;
					this.setCursorCol(result.cursorCol);

					// 斜杠命令补全 → 确认后取消列表并回退到提交逻辑
					if (this.autocompletePrefix.startsWith("/")) {
						this.cancelAutocomplete();
					} else {
						this.cancelAutocomplete();
						if (this.onChange) this.onChange(this.getText());
						return;
					}
				}
			}
		}

		// Tab → 触发补全
		if (kb.matches(data, "tui.input.tab") && !this.autocompleteState) {
			this.handleTabCompletion();
			return;
		}

		// ── 删除操作 ──
		if (kb.matches(data, "tui.editor.deleteToLineEnd")) {
			this.deleteToEndOfLine();
			return;
		}
		if (kb.matches(data, "tui.editor.deleteToLineStart")) {
			this.deleteToStartOfLine();
			return;
		}
		if (kb.matches(data, "tui.editor.deleteWordBackward")) {
			this.deleteWordBackwards();
			return;
		}
		if (kb.matches(data, "tui.editor.deleteWordForward")) {
			this.deleteWordForward();
			return;
		}
		if (kb.matches(data, "tui.editor.deleteCharBackward") || matchesKey(data, "shift+backspace")) {
			this.handleBackspace();
			return;
		}
		if (kb.matches(data, "tui.editor.deleteCharForward") || matchesKey(data, "shift+delete")) {
			this.handleForwardDelete();
			return;
		}

		// ── Kill Ring 操作 (Ctrl+Y / Alt+Y) ──
		if (kb.matches(data, "tui.editor.yank")) {
			this.yank();
			return;
		}
		if (kb.matches(data, "tui.editor.yankPop")) {
			this.yankPop();
			return;
		}

		// ── 光标移动 ──
		if (kb.matches(data, "tui.editor.cursorLineStart")) {
			this.moveToLineStart();
			return;
		}
		if (kb.matches(data, "tui.editor.cursorLineEnd")) {
			this.moveToLineEnd();
			return;
		}
		if (kb.matches(data, "tui.editor.cursorWordLeft")) {
			this.moveWordBackwards();
			return;
		}
		if (kb.matches(data, "tui.editor.cursorWordRight")) {
			this.moveWordForwards();
			return;
		}

		// ── 换行（多行输入）──
		// 匹配 Ctrl+Enter (CSI-u 编码)、\x1b\r、\x1b[13;2~ (Shift+Enter 的 CSI 编码)
		// 以及纯 "\n"（单字符换行）
		if (
			kb.matches(data, "tui.input.newLine") ||
			(data.charCodeAt(0) === 10 && data.length > 1) ||
			data === "\x1b\r" ||
			data === "\x1b[13;2~" ||
			(data.length > 1 && data.includes("\x1b") && data.includes("\r")) ||
			(data === "\n" && data.length === 1)
		) {
			// 如果终端不支持 Shift+Enter 且当前行以 \ 结尾 → 删除 \ 后提交
			if (this.shouldSubmitOnBackslashEnter(data, kb)) {
				this.handleBackspace();
				this.submitValue();
				return;
			}
			this.addNewLine();
			return;
		}

		// ── 提交 (Enter) ──
		if (kb.matches(data, "tui.input.submit")) {
			if (this.disableSubmit) return;

			/**
			 * 折中方案：部分终端不支持 Shift+Enter，因此当光标前是 "\" 时
			 * 将 Enter 解释为"删除 \ 后换行"而非提交。
			 */
			const currentLine = this.state.lines[this.state.cursorLine] || "";
			if (this.state.cursorCol > 0 && currentLine[this.state.cursorCol - 1] === "\\") {
				this.handleBackspace();
				this.addNewLine();
				return;
			}

			this.submitValue();
			return;
		}

		// ── 方向键导航（含历史记录集成）──
		if (kb.matches(data, "tui.editor.cursorUp")) {
			// 编辑器为空时 → 浏览历史
			if (this.isEditorEmpty()) {
				this.navigateHistory(-1);
			} else if (this.historyIndex > -1 && this.isOnFirstVisualLine()) {
				// 在历史浏览模式已到顶 → 继续浏览更早历史
				this.navigateHistory(-1);
			} else if (this.isOnFirstVisualLine()) {
				// 已在文本顶部 → 跳转到行首
				this.moveToLineStart();
			} else {
				this.moveCursor(-1, 0);
			}
			return;
		}
		if (kb.matches(data, "tui.editor.cursorDown")) {
			if (this.historyIndex > -1 && this.isOnLastVisualLine()) {
				// 在历史浏览模式已到底 → 浏览更新历史
				this.navigateHistory(1);
			} else if (this.isOnLastVisualLine()) {
				// 已在文本底部 → 跳转到行尾
				this.moveToLineEnd();
			} else {
				this.moveCursor(1, 0);
			}
			return;
		}
		if (kb.matches(data, "tui.editor.cursorRight")) {
			this.moveCursor(0, 1);
			return;
		}
		if (kb.matches(data, "tui.editor.cursorLeft")) {
			this.moveCursor(0, -1);
			return;
		}

		// ── 翻页 ──
		if (kb.matches(data, "tui.editor.pageUp")) {
			this.pageScroll(-1);
			return;
		}
		if (kb.matches(data, "tui.editor.pageDown")) {
			this.pageScroll(1);
			return;
		}

		// ── 字符跳转触发 ──
		if (kb.matches(data, "tui.editor.jumpForward")) {
			this.jumpMode = "forward";
			return;
		}
		if (kb.matches(data, "tui.editor.jumpBackward")) {
			this.jumpMode = "backward";
			return;
		}

		// Shift+Space → 插入普通空格（部分终端 Shift+Space 发送不同序列）
		if (matchesKey(data, "shift+space")) {
			this.insertCharacter(" ");
			return;
		}

		// ── 可打印字符插入 ──
		const printable = decodePrintableKey(data);
		if (printable !== undefined) {
			this.insertCharacter(printable);
			return;
		}

		// ASCII 可打印字符回退（charCode >= 32）
		if (data.charCodeAt(0) >= 32) {
			this.insertCharacter(data);
		}
	}

	/**
	 * 将逻辑行展开为视觉行布局。
	 *
	 * 对每条逻辑行调用 wordWrapLine 进行断行，产生含光标位置的 LayoutLine[]。
	 * 光标落在哪个视觉行由 cursorCol 与 chunk 的 startIndex/endIndex 区间判定：
	 *   - 最后一个 chunk：光标位置 >= startIndex 即命中
	 *   - 非最后 chunk：光标必须在 [startIndex, endIndex) 范围内
	 *
	 * @param contentWidth - 视觉行可用宽度（列数）
	 * @returns 视觉行数组，含光标位置标注
	 */
	private layoutText(contentWidth: number): LayoutLine[] {
		const layoutLines: LayoutLine[] = [];

		if (this.state.lines.length === 0 || (this.state.lines.length === 1 && this.state.lines[0] === "")) {
			// Empty editor
			layoutLines.push({
				text: "",
				hasCursor: true,
				cursorPos: 0,
			});
			return layoutLines;
		}

		// Process each logical line
		for (let i = 0; i < this.state.lines.length; i++) {
			const line = this.state.lines[i] || "";
			const isCurrentLine = i === this.state.cursorLine;
			const lineVisibleWidth = visibleWidth(line);

			if (lineVisibleWidth <= contentWidth) {
				// Line fits in one layout line
				if (isCurrentLine) {
					layoutLines.push({
						text: line,
						hasCursor: true,
						cursorPos: this.state.cursorCol,
					});
				} else {
					layoutLines.push({
						text: line,
						hasCursor: false,
					});
				}
			} else {
				// Line needs wrapping - use word-aware wrapping
				const chunks = wordWrapLine(line, contentWidth, [...this.segment(line)]);

				for (let chunkIndex = 0; chunkIndex < chunks.length; chunkIndex++) {
					const chunk = chunks[chunkIndex];
					if (!chunk) continue;

					const cursorPos = this.state.cursorCol;
					const isLastChunk = chunkIndex === chunks.length - 1;

					// Determine if cursor is in this chunk
					// For word-wrapped chunks, we need to handle the case where
					// cursor might be in trimmed whitespace at end of chunk
					let hasCursorInChunk = false;
					let adjustedCursorPos = 0;

					if (isCurrentLine) {
						if (isLastChunk) {
							// Last chunk: cursor belongs here if >= startIndex
							hasCursorInChunk = cursorPos >= chunk.startIndex;
							adjustedCursorPos = cursorPos - chunk.startIndex;
						} else {
							// Non-last chunk: cursor belongs here if in range [startIndex, endIndex)
							// But we need to handle the visual position in the trimmed text
							hasCursorInChunk = cursorPos >= chunk.startIndex && cursorPos < chunk.endIndex;
							if (hasCursorInChunk) {
								adjustedCursorPos = cursorPos - chunk.startIndex;
								// Clamp to text length (in case cursor was in trimmed whitespace)
								if (adjustedCursorPos > chunk.text.length) {
									adjustedCursorPos = chunk.text.length;
								}
							}
						}
					}

					if (hasCursorInChunk) {
						layoutLines.push({
							text: chunk.text,
							hasCursor: true,
							cursorPos: adjustedCursorPos,
						});
					} else {
						layoutLines.push({
							text: chunk.text,
							hasCursor: false,
						});
					}
				}
			}
		}

		return layoutLines;
	}

	/**
	 * 获取当前文本（不含粘贴标记展开）。
	 * 粘贴标记保持为 [paste #N ...] 占位符，适合 UI 展示。
	 * 需要完整内容时使用 getExpandedText()。
	 *
	 * @returns 逻辑行以 \n 连接的文本
	 */
	getText(): string {
		return this.state.lines.join("\n");
	}

	/**
	 * 将粘贴标记展开为实际内容。
	 * 内部遍历 pastes Map，将每个 [paste #N ...] 替换为存储的原始文本。
	 * 用于提交（submitValue）和获取完整文本（getExpandedText）。
	 *
	 * @param text - 含粘贴标记的文本
	 * @returns 标记已展开的完整文本
	 */
	private expandPasteMarkers(text: string): string {
		let result = text;
		for (const [pasteId, pasteContent] of this.pastes) {
			const markerRegex = new RegExp(`\\[paste #${pasteId}( (\\+\\d+ lines|\\d+ chars))?\\]`, "g");
			result = result.replace(markerRegex, () => pasteContent);
		}
		return result;
	}

	/**
	 * Get text with paste markers expanded to their actual content.
	 * Use this when you need the full content (e.g., for external editor).
	 */
	getExpandedText(): string {
		return this.expandPasteMarkers(this.state.lines.join("\n"));
	}

	getLines(): string[] {
		return [...this.state.lines];
	}

	getCursor(): { line: number; col: number } {
		return { line: this.state.cursorLine, col: this.state.cursorCol };
	}

	/**
	 * 程序化设置编辑器文本。
	 *
	 * 取消补全、退出历史浏览、规范化行尾和 Tab → 4 空格。
	 * 如果内容发生变化则 pushUndoSnapshot，使程序化变更也可撤销。
	 *
	 * @param text - 新文本内容
	 */
	setText(text: string): void {
		this.cancelAutocomplete();
		this.lastAction = null;
		this.historyIndex = -1; // Exit history browsing mode
		const normalized = this.normalizeText(text);
		// Push undo snapshot if content differs (makes programmatic changes undoable)
		if (this.getText() !== normalized) {
			this.pushUndoSnapshot();
		}
		this.setTextInternal(normalized);
	}

	/**
	 * Insert text at the current cursor position.
	 * Used for programmatic insertion (e.g., clipboard image markers).
	 * This is atomic for undo - single undo restores entire pre-insert state.
	 */
	insertTextAtCursor(text: string): void {
		if (!text) return;
		this.cancelAutocomplete();
		this.pushUndoSnapshot();
		this.lastAction = null;
		this.historyIndex = -1;
		this.insertTextAtCursorInternal(text);
	}

	/**
	 * Normalize text for editor storage:
	 * - Normalize line endings (\r\n and \r -> \n)
	 * - Expand tabs to 4 spaces
	 */
	private normalizeText(text: string): string {
		return text.replace(/\r\n/g, "\n").replace(/\r/g, "\n").replace(/\t/g, "    ");
	}

	/**
	 * Internal text insertion at cursor. Handles single and multi-line text.
	 * Does not push undo snapshots or trigger autocomplete - caller is responsible.
	 * Normalizes line endings and calls onChange once at the end.
	 */
	private insertTextAtCursorInternal(text: string): void {
		if (!text) return;

		// Normalize line endings and tabs
		const normalized = this.normalizeText(text);
		const insertedLines = normalized.split("\n");

		const currentLine = this.state.lines[this.state.cursorLine] || "";
		const beforeCursor = currentLine.slice(0, this.state.cursorCol);
		const afterCursor = currentLine.slice(this.state.cursorCol);

		if (insertedLines.length === 1) {
			// Single line - insert at cursor position
			this.state.lines[this.state.cursorLine] = beforeCursor + normalized + afterCursor;
			this.setCursorCol(this.state.cursorCol + normalized.length);
		} else {
			// Multi-line insertion
			this.state.lines = [
				// All lines before current line
				...this.state.lines.slice(0, this.state.cursorLine),

				// The first inserted line merged with text before cursor
				beforeCursor + insertedLines[0],

				// All middle inserted lines
				...insertedLines.slice(1, -1),

				// The last inserted line with text after cursor
				insertedLines[insertedLines.length - 1] + afterCursor,

				// All lines after current line
				...this.state.lines.slice(this.state.cursorLine + 1),
			];

			this.state.cursorLine += insertedLines.length - 1;
			this.setCursorCol((insertedLines[insertedLines.length - 1] || "").length);
		}

		if (this.onChange) {
			this.onChange(this.getText());
		}
	}

	/**
	 * 插入单个字符。
	 *
	 * 合并策略（鱼式 undo）：
	 *   - 连续单词字符 → 合并为一个 undo 单元（撤销时一次删除整个词）
	 *   - 空白字符 → 在其之前 push snapshot，使 space 本身单独成 undo 单元
	 *   - skipUndoCoalescing = true 时跳过合并（用于原子操作如 handlePaste）
	 *
	 * @param char - 要插入的字符
	 * @param skipUndoCoalescing - 为 true 时不做 undo 合并（调用方负责 snapshot）
	 */
	private insertCharacter(char: string, skipUndoCoalescing?: boolean): void {
		this.historyIndex = -1; // Exit history browsing mode

		// Undo coalescing (fish-style):
		// - Consecutive word chars coalesce into one undo unit
		// - Space captures state before itself (so undo removes space+following word together)
		// - Each space is separately undoable
		// Skip coalescing when called from atomic operations (e.g., handlePaste)
		if (!skipUndoCoalescing) {
			if (isWhitespaceChar(char) || this.lastAction !== "type-word") {
				this.pushUndoSnapshot();
			}
			this.lastAction = "type-word";
		}

		const line = this.state.lines[this.state.cursorLine] || "";

		const before = line.slice(0, this.state.cursorCol);
		const after = line.slice(this.state.cursorCol);

		this.state.lines[this.state.cursorLine] = before + char + after;
		this.setCursorCol(this.state.cursorCol + char.length);

		if (this.onChange) {
			this.onChange(this.getText());
		}

		/**
		 * 自动补全触发条件检查（字符插入时）：
		 *   - "/" 在消息开头 → 斜杠命令补全
		 *   - "@" / "#" 在词边界（前面是空格或行首）→ 附件/文件补全
		 *   - 字母/数字在斜杠命令或 @/# 上下文中 → 更新补全列表
		 *   - 补全列表已激活 → 刷新列表（updateAutocomplete）
		 */
		if (!this.autocompleteState) {
			// "/" 在消息开头 → 斜杠命令补全
			if (char === "/" && this.isAtStartOfMessage()) {
				this.tryTriggerAutocomplete();
			}
			// "@" / "#" 在词边界（前面是空格、Tab 或行首）→ 附件/文件补全
			else if (char === "@" || char === "#") {
				const currentLine = this.state.lines[this.state.cursorLine] || "";
				const textBeforeCursor = currentLine.slice(0, this.state.cursorCol);
				const charBeforeSymbol = textBeforeCursor[textBeforeCursor.length - 2];
				if (textBeforeCursor.length === 1 || charBeforeSymbol === " " || charBeforeSymbol === "\t") {
					this.tryTriggerAutocomplete();
				}
			}
			// 在斜杠命令或 @/# 上下文中输入字母/数字 → 更新补全列表
			else if (/[a-zA-Z0-9.\-_]/.test(char)) {
				const currentLine = this.state.lines[this.state.cursorLine] || "";
				const textBeforeCursor = currentLine.slice(0, this.state.cursorCol);
				// 判断是否在斜杠命令上下文中
				if (this.isInSlashCommandContext(textBeforeCursor)) {
					this.tryTriggerAutocomplete();
				}
				// 判断是否在 @/# 符号补全上下文中
				else if (textBeforeCursor.match(/(?:^|[\s])[@#][^\s]*$/)) {
					this.tryTriggerAutocomplete();
				}
			}
		} else {
			// 补全列表已激活 → 实时刷新
			this.updateAutocomplete();
		}
	}

	/**
	 * 粘贴折叠处理 —— 将终端粘贴内容插入编辑器。
	 *
	 * 处理流程：
	 *   1. 归一化行尾 + Tab → 空格
	 *   2. 解码 CSI-u 编码的控制字节（限制在 tmux 弹出窗口等终端场景）
	 *   3. 过滤非打印字符（保留换行）
	 *   4. 判断是否为文件路径（以 / ~ . 开头），若是则在光标前补空格
	 *   5. 大粘贴检测：> 10 行或 > 1000 字符 → 折叠为 [paste #N +M lines] 标记
	 *      短粘贴 → 按普通文本原子插入
	 *
	 * 折叠原因：大段粘贴保持为标记可避免光标遍历大量文本时性能退化，
	 * 同时用户仍可通过提交时自动展开（getExpandedText）获得完整内容。
	 *
	 * @param pastedText - 终端括弧粘贴模式解包后的原始文本
	 */
	private handlePaste(pastedText: string): void {
		this.cancelAutocomplete();
		this.historyIndex = -1; // Exit history browsing mode
		this.lastAction = null;

		this.pushUndoSnapshot();

		// Some terminals (e.g. tmux popups with extended-keys-format=csi-u) re-encode
		// control bytes inside bracketed paste as CSI-u Ctrl+<letter> sequences
		// (ESC [ <codepoint> ; 5 u). Decode those back to their literal byte so the
		// per-char filter below preserves newlines instead of stripping ESC and
		// leaking the printable tail (e.g. "[106;5u") into the editor.
		const decodedText = pastedText.replace(/\x1b\[(\d+);5u/g, (match, code) => {
			const cp = Number(code);
			if (cp >= 97 && cp <= 122) return String.fromCharCode(cp - 96);
			if (cp >= 65 && cp <= 90) return String.fromCharCode(cp - 64);
			return match;
		});

		// Clean the pasted text: normalize line endings, expand tabs
		const cleanText = this.normalizeText(decodedText);

		// Filter out non-printable characters except newlines
		let filteredText = cleanText
			.split("")
			.filter((char) => char === "\n" || char.charCodeAt(0) >= 32)
			.join("");

		// If pasting a file path (starts with /, ~, or .) and the character before
		// the cursor is a word character, prepend a space for better readability
		if (/^[/~.]/.test(filteredText)) {
			const currentLine = this.state.lines[this.state.cursorLine] || "";
			const charBeforeCursor = this.state.cursorCol > 0 ? currentLine[this.state.cursorCol - 1] : "";
			if (charBeforeCursor && /\w/.test(charBeforeCursor)) {
				filteredText = ` ${filteredText}`;
			}
		}

		// Split into lines to check for large paste
		const pastedLines = filteredText.split("\n");

		// ── 大粘贴折叠：> 10 行或 > 1000 字符时折叠为标记 ──
		// 折叠原因是光标遍历大量文本时性能退化，同时保持编辑状态紧凑
		const totalChars = filteredText.length;
		if (pastedLines.length > 10 || totalChars > 1000) {
			// Store the paste and insert a marker
			this.pasteCounter++;
			const pasteId = this.pasteCounter;
			this.pastes.set(pasteId, filteredText);

			// Insert marker like "[paste #1 +123 lines]" or "[paste #1 1234 chars]"
			const marker =
				pastedLines.length > 10
					? `[paste #${pasteId} +${pastedLines.length} lines]`
					: `[paste #${pasteId} ${totalChars} chars]`;
			this.insertTextAtCursorInternal(marker);
			return;
		}

		if (pastedLines.length === 1) {
			// Single line - insert atomically (do not trigger autocomplete during paste)
			this.insertTextAtCursorInternal(filteredText);
			return;
		}

		// Multi-line paste - use direct state manipulation
		this.insertTextAtCursorInternal(filteredText);
	}

	/**
	 * 插入换行 —— 在光标位置拆分逻辑行为两行。
	 * 光标前文本保留在原行，光标后文本移到新行。
	 */
	private addNewLine(): void {
		this.cancelAutocomplete();
		this.historyIndex = -1; // Exit history browsing mode
		this.lastAction = null;

		this.pushUndoSnapshot();

		const currentLine = this.state.lines[this.state.cursorLine] || "";

		const before = currentLine.slice(0, this.state.cursorCol);
		const after = currentLine.slice(this.state.cursorCol);

		// Split current line
		this.state.lines[this.state.cursorLine] = before;
		this.state.lines.splice(this.state.cursorLine + 1, 0, after);

		// Move cursor to start of new line
		this.state.cursorLine++;
		this.setCursorCol(0);

		if (this.onChange) {
			this.onChange(this.getText());
		}
	}

	private shouldSubmitOnBackslashEnter(data: string, kb: ReturnType<typeof getKeybindings>): boolean {
		if (this.disableSubmit) return false;
		if (!matchesKey(data, "enter")) return false;
		const submitKeys = kb.getKeys("tui.input.submit");
		const hasShiftEnter = submitKeys.includes("shift+enter") || submitKeys.includes("shift+return");
		if (!hasShiftEnter) return false;

		const currentLine = this.state.lines[this.state.cursorLine] || "";
		return this.state.cursorCol > 0 && currentLine[this.state.cursorCol - 1] === "\\";
	}

	/**
	 * 提交编辑器内容 —— 展开粘贴标记后调用 onSubmit 回调。
	 *
	 * 重置所有内部状态：清空文本、清空粘贴映射、重置历史浏览索引、清空撤销栈。
	 * 先触发 onChange("") 再触发 onSubmit(result)。
	 */
	private submitValue(): void {
		this.cancelAutocomplete();
		const result = this.expandPasteMarkers(this.state.lines.join("\n")).trim();

		this.state = { lines: [""], cursorLine: 0, cursorCol: 0 };
		this.pastes.clear();
		this.pasteCounter = 0;
		this.historyIndex = -1;
		this.scrollOffset = 0;
		this.undoStack.clear();
		this.lastAction = null;

		if (this.onChange) this.onChange("");
		if (this.onSubmit) this.onSubmit(result);
	}

	/**
	 * 退格删除 —— grapheme 粒度删除光标前的字符。
	 *
	 * 行为：
	 *   - 有前导字符时：用 segment() 找到光标前的最后一个 grapheme 并删除
	 *   - 已在行首时：合并到上一逻辑行（删除隐式换行）
	 * 删除后更新补全列表或重新触发补全。
	 */
	private handleBackspace(): void {
		this.historyIndex = -1; // Exit history browsing mode
		this.lastAction = null;

		if (this.state.cursorCol > 0) {
			this.pushUndoSnapshot();

			// Delete grapheme before cursor (handles emojis, combining characters, etc.)
			const line = this.state.lines[this.state.cursorLine] || "";
			const beforeCursor = line.slice(0, this.state.cursorCol);

			// Find the last grapheme in the text before cursor
			const graphemes = [...this.segment(beforeCursor)];
			const lastGrapheme = graphemes[graphemes.length - 1];
			const graphemeLength = lastGrapheme ? lastGrapheme.segment.length : 1;

			const before = line.slice(0, this.state.cursorCol - graphemeLength);
			const after = line.slice(this.state.cursorCol);

			this.state.lines[this.state.cursorLine] = before + after;
			this.setCursorCol(this.state.cursorCol - graphemeLength);
		} else if (this.state.cursorLine > 0) {
			this.pushUndoSnapshot();

			// Merge with previous line
			const currentLine = this.state.lines[this.state.cursorLine] || "";
			const previousLine = this.state.lines[this.state.cursorLine - 1] || "";

			this.state.lines[this.state.cursorLine - 1] = previousLine + currentLine;
			this.state.lines.splice(this.state.cursorLine, 1);

			this.state.cursorLine--;
			this.setCursorCol(previousLine.length);
		}

		if (this.onChange) {
			this.onChange(this.getText());
		}

		// Update or re-trigger autocomplete after backspace
		if (this.autocompleteState) {
			this.updateAutocomplete();
		} else {
			// If autocomplete was cancelled (no matches), re-trigger if we're in a completable context
			const currentLine = this.state.lines[this.state.cursorLine] || "";
			const textBeforeCursor = currentLine.slice(0, this.state.cursorCol);
			// Slash command context
			if (this.isInSlashCommandContext(textBeforeCursor)) {
				this.tryTriggerAutocomplete();
			}
			// Symbol-based completion context like @ or #
			else if (textBeforeCursor.match(/(?:^|[\s])[@#][^\s]*$/)) {
				this.tryTriggerAutocomplete();
			}
		}
	}

	/**
	 * Set cursor column and clear preferredVisualCol.
	 * Use this for all non-vertical cursor movements to reset sticky column behavior.
	 */
	private setCursorCol(col: number): void {
		this.state.cursorCol = col;
		this.preferredVisualCol = null;
		this.snappedFromCursorCol = null;
	}

	/**
	 * 将光标移动到目标视觉行 —— 垂直光标移动的核心实现。
	 *
	 * 处理流程：
	 *   1. 解析当前视觉列（如果光标被吸附到原子段开头，用 snappedFromCursorCol 恢复原始位置）
	 *   2. 按视觉段末确定当前/目标行的最大视觉列
	 *   3. 调用 computeVerticalMoveColumn 计算目标视觉列
	 *   4. 设置 cursorLine 和 cursorCol
	 *   5. 检测原子段边界（粘贴标记等），吸附光标到段开头并存储 snappedFromCursorCol
	 *
	 * @param visualLines - buildVisualLineMap 的结果
	 * @param currentVisualLine - 当前视觉行索引
	 * @param targetVisualLine - 目标视觉行索引
	 */
	private moveToVisualLine(
		visualLines: Array<{ logicalLine: number; startCol: number; length: number }>,
		currentVisualLine: number,
		targetVisualLine: number,
	): void {
		const currentVL = visualLines[currentVisualLine];
		const targetVL = visualLines[targetVisualLine];
		if (!(currentVL && targetVL)) return;

		// When the cursor was snapped to a segment start, resolve the pre-snap
		// position against the VL it belongs to. This gives the correct visual
		// column even after a resize reshuffles VLs.
		let currentVisualCol: number;
		if (this.snappedFromCursorCol !== null) {
			const vlIndex = this.findVisualLineAt(visualLines, currentVL.logicalLine, this.snappedFromCursorCol);
			currentVisualCol = this.snappedFromCursorCol - visualLines[vlIndex].startCol;
		} else {
			currentVisualCol = this.state.cursorCol - currentVL.startCol;
		}

		// For non-last segments, clamp to length-1 to stay within the segment
		const isLastSourceSegment =
			currentVisualLine === visualLines.length - 1 ||
			visualLines[currentVisualLine + 1]?.logicalLine !== currentVL.logicalLine;
		const sourceMaxVisualCol = isLastSourceSegment ? currentVL.length : Math.max(0, currentVL.length - 1);

		const isLastTargetSegment =
			targetVisualLine === visualLines.length - 1 ||
			visualLines[targetVisualLine + 1]?.logicalLine !== targetVL.logicalLine;
		const targetMaxVisualCol = isLastTargetSegment ? targetVL.length : Math.max(0, targetVL.length - 1);

		const moveToVisualCol = this.computeVerticalMoveColumn(currentVisualCol, sourceMaxVisualCol, targetMaxVisualCol);

		// Set cursor position
		this.state.cursorLine = targetVL.logicalLine;
		const targetCol = targetVL.startCol + moveToVisualCol;
		const logicalLine = this.state.lines[targetVL.logicalLine] || "";
		this.state.cursorCol = Math.min(targetCol, logicalLine.length);

		// Snap cursor to atomic segment boundary (e.g. paste markers)
		// so the cursor never lands in the middle of a multi-grapheme unit.
		// Single-grapheme segments don't need snapping.
		const segments = [...this.segment(logicalLine)];
		for (const seg of segments) {
			if (seg.index > this.state.cursorCol) break;
			if (seg.segment.length <= 1) continue;
			if (this.state.cursorCol < seg.index + seg.segment.length) {
				const isContinuation = seg.index < targetVL.startCol;
				const isMovingDown = targetVisualLine > currentVisualLine;

				if (isContinuation && isMovingDown) {
					// The segment started on a previous visual line, and we
					// already visited it on the way down. Skip all remaining
					// continuation VLs and land on the first VL past it.
					const segEnd = seg.index + seg.segment.length;
					let next = targetVisualLine + 1;
					while (
						next < visualLines.length &&
						visualLines[next].logicalLine === targetVL.logicalLine &&
						visualLines[next].startCol < segEnd
					) {
						next++;
					}
					if (next < visualLines.length) {
						this.moveToVisualLine(visualLines, currentVisualLine, next);
						return;
					}
				}

				// Snap to the start of the segment so it gets highlighted.
				// Store the pre-snap position so the next vertical move can
				// resolve it to the correct visual column.
				this.snappedFromCursorCol = this.state.cursorCol;
				this.state.cursorCol = seg.index;
				return;
			}
		}

		// No snap occurred – we moved out of the atomic segment.
		this.snappedFromCursorCol = null;
	}

	/**
	 * 垂直光标移动的目标列计算（粘性列 / Sticky Column）。
	 *
	 * 核心问题：当用户上/下移动到较短行时，光标是否应保持在记忆的视觉列偏好位置？
	 * 答案取决于 preferredVisualCol 是否已设置、光标是否在行中部、目标行是否太短。
	 *
	 * 决策表（P=有偏好列, S=光标在行中, T=目标行太短, U=目标行比偏好列短）：
	 *
	 * | P | S | T | U | 策略                                                |
	 * |---|---|---|---| --------------------------------------------------- |
	 * | 0 | * | 0 | - | 无偏好 → 移动后取消偏好，保持在当前位置              |
	 * | 0 | * | 1 | - | 无偏好 → 行太短，记忆当前位置为偏好，吸附到行尾      |
	 * | 1 | 0 | 0 | 0 | 已吸附 → 目标能容纳偏好列，取消偏好，移到偏好列        |
	 * | 1 | 0 | 0 | 1 | 已吸附 → 目标更长了但仍不够，保持偏好，吸附到行尾      |
	 * | 1 | 0 | 1 | - | 已吸附 → 目标更短，保持偏好，吸附到行尾                |
	 * | 1 | 1 | 0 | - | 重新换行 → 目标足够，取消偏好，保持在当前位置          |
	 * | 1 | 1 | 1 | - | 重新换行 → 目标不够，记忆当前位置为偏好，吸附到行尾    |
	 *
	 * @param currentVisualCol - 当前视觉列位置
	 * @param sourceMaxVisualCol - 源行的最大视觉列
	 * @param targetMaxVisualCol - 目标行的最大视觉列
	 * @returns 应移动到的视觉列位置
	 */
	private computeVerticalMoveColumn(
		currentVisualCol: number,
		sourceMaxVisualCol: number,
		targetMaxVisualCol: number,
	): number {
		const hasPreferred = this.preferredVisualCol !== null; // P
		const cursorInMiddle = currentVisualCol < sourceMaxVisualCol; // S
		const targetTooShort = targetMaxVisualCol < currentVisualCol; // T

		if (!hasPreferred || cursorInMiddle) {
			if (targetTooShort) {
				// Cases 2 and 7
				this.preferredVisualCol = currentVisualCol;
				return targetMaxVisualCol;
			}

			// Cases 1 and 6
			this.preferredVisualCol = null;
			return currentVisualCol;
		}

		const targetCantFitPreferred = targetMaxVisualCol < this.preferredVisualCol!; // U
		if (targetTooShort || targetCantFitPreferred) {
			// Cases 4 and 5
			return targetMaxVisualCol;
		}

		// Case 3
		const result = this.preferredVisualCol!;
		this.preferredVisualCol = null;
		return result;
	}

	private moveToLineStart(): void {
		this.lastAction = null;
		this.setCursorCol(0);
	}

	private moveToLineEnd(): void {
		this.lastAction = null;
		const currentLine = this.state.lines[this.state.cursorLine] || "";
		this.setCursorCol(currentLine.length);
	}

	private deleteToStartOfLine(): void {
		this.historyIndex = -1; // Exit history browsing mode

		const currentLine = this.state.lines[this.state.cursorLine] || "";

		if (this.state.cursorCol > 0) {
			this.pushUndoSnapshot();

			// Calculate text to be deleted and save to kill ring (backward deletion = prepend)
			const deletedText = currentLine.slice(0, this.state.cursorCol);
			this.killRing.push(deletedText, { prepend: true, accumulate: this.lastAction === "kill" });
			this.lastAction = "kill";

			// Delete from start of line up to cursor
			this.state.lines[this.state.cursorLine] = currentLine.slice(this.state.cursorCol);
			this.setCursorCol(0);
		} else if (this.state.cursorLine > 0) {
			this.pushUndoSnapshot();

			// At start of line - merge with previous line, treating newline as deleted text
			this.killRing.push("\n", { prepend: true, accumulate: this.lastAction === "kill" });
			this.lastAction = "kill";

			const previousLine = this.state.lines[this.state.cursorLine - 1] || "";
			this.state.lines[this.state.cursorLine - 1] = previousLine + currentLine;
			this.state.lines.splice(this.state.cursorLine, 1);
			this.state.cursorLine--;
			this.setCursorCol(previousLine.length);
		}

		if (this.onChange) {
			this.onChange(this.getText());
		}
	}

	private deleteToEndOfLine(): void {
		this.historyIndex = -1; // Exit history browsing mode

		const currentLine = this.state.lines[this.state.cursorLine] || "";

		if (this.state.cursorCol < currentLine.length) {
			this.pushUndoSnapshot();

			// Calculate text to be deleted and save to kill ring (forward deletion = append)
			const deletedText = currentLine.slice(this.state.cursorCol);
			this.killRing.push(deletedText, { prepend: false, accumulate: this.lastAction === "kill" });
			this.lastAction = "kill";

			// Delete from cursor to end of line
			this.state.lines[this.state.cursorLine] = currentLine.slice(0, this.state.cursorCol);
		} else if (this.state.cursorLine < this.state.lines.length - 1) {
			this.pushUndoSnapshot();

			// At end of line - merge with next line, treating newline as deleted text
			this.killRing.push("\n", { prepend: false, accumulate: this.lastAction === "kill" });
			this.lastAction = "kill";

			const nextLine = this.state.lines[this.state.cursorLine + 1] || "";
			this.state.lines[this.state.cursorLine] = currentLine + nextLine;
			this.state.lines.splice(this.state.cursorLine + 1, 1);
		}

		if (this.onChange) {
			this.onChange(this.getText());
		}
	}

	private deleteWordBackwards(): void {
		this.historyIndex = -1; // Exit history browsing mode

		const currentLine = this.state.lines[this.state.cursorLine] || "";

		// If at start of line, behave like backspace at column 0 (merge with previous line)
		if (this.state.cursorCol === 0) {
			if (this.state.cursorLine > 0) {
				this.pushUndoSnapshot();

				// Treat newline as deleted text (backward deletion = prepend)
				this.killRing.push("\n", { prepend: true, accumulate: this.lastAction === "kill" });
				this.lastAction = "kill";

				const previousLine = this.state.lines[this.state.cursorLine - 1] || "";
				this.state.lines[this.state.cursorLine - 1] = previousLine + currentLine;
				this.state.lines.splice(this.state.cursorLine, 1);
				this.state.cursorLine--;
				this.setCursorCol(previousLine.length);
			}
		} else {
			this.pushUndoSnapshot();

			// Save lastAction before cursor movement (moveWordBackwards resets it)
			const wasKill = this.lastAction === "kill";

			const oldCursorCol = this.state.cursorCol;
			this.moveWordBackwards();
			const deleteFrom = this.state.cursorCol;
			this.setCursorCol(oldCursorCol);

			const deletedText = currentLine.slice(deleteFrom, this.state.cursorCol);
			this.killRing.push(deletedText, { prepend: true, accumulate: wasKill });
			this.lastAction = "kill";

			this.state.lines[this.state.cursorLine] =
				currentLine.slice(0, deleteFrom) + currentLine.slice(this.state.cursorCol);
			this.setCursorCol(deleteFrom);
		}

		if (this.onChange) {
			this.onChange(this.getText());
		}
	}

	private deleteWordForward(): void {
		this.historyIndex = -1; // Exit history browsing mode

		const currentLine = this.state.lines[this.state.cursorLine] || "";

		// If at end of line, merge with next line (delete the newline)
		if (this.state.cursorCol >= currentLine.length) {
			if (this.state.cursorLine < this.state.lines.length - 1) {
				this.pushUndoSnapshot();

				// Treat newline as deleted text (forward deletion = append)
				this.killRing.push("\n", { prepend: false, accumulate: this.lastAction === "kill" });
				this.lastAction = "kill";

				const nextLine = this.state.lines[this.state.cursorLine + 1] || "";
				this.state.lines[this.state.cursorLine] = currentLine + nextLine;
				this.state.lines.splice(this.state.cursorLine + 1, 1);
			}
		} else {
			this.pushUndoSnapshot();

			// Save lastAction before cursor movement (moveWordForwards resets it)
			const wasKill = this.lastAction === "kill";

			const oldCursorCol = this.state.cursorCol;
			this.moveWordForwards();
			const deleteTo = this.state.cursorCol;
			this.setCursorCol(oldCursorCol);

			const deletedText = currentLine.slice(this.state.cursorCol, deleteTo);
			this.killRing.push(deletedText, { prepend: false, accumulate: wasKill });
			this.lastAction = "kill";

			this.state.lines[this.state.cursorLine] =
				currentLine.slice(0, this.state.cursorCol) + currentLine.slice(deleteTo);
		}

		if (this.onChange) {
			this.onChange(this.getText());
		}
	}

	private handleForwardDelete(): void {
		this.historyIndex = -1; // Exit history browsing mode
		this.lastAction = null;

		const currentLine = this.state.lines[this.state.cursorLine] || "";

		if (this.state.cursorCol < currentLine.length) {
			this.pushUndoSnapshot();

			// Delete grapheme at cursor position (handles emojis, combining characters, etc.)
			const afterCursor = currentLine.slice(this.state.cursorCol);

			// Find the first grapheme at cursor
			const graphemes = [...this.segment(afterCursor)];
			const firstGrapheme = graphemes[0];
			const graphemeLength = firstGrapheme ? firstGrapheme.segment.length : 1;

			const before = currentLine.slice(0, this.state.cursorCol);
			const after = currentLine.slice(this.state.cursorCol + graphemeLength);
			this.state.lines[this.state.cursorLine] = before + after;
		} else if (this.state.cursorLine < this.state.lines.length - 1) {
			this.pushUndoSnapshot();

			// At end of line - merge with next line
			const nextLine = this.state.lines[this.state.cursorLine + 1] || "";
			this.state.lines[this.state.cursorLine] = currentLine + nextLine;
			this.state.lines.splice(this.state.cursorLine + 1, 1);
		}

		if (this.onChange) {
			this.onChange(this.getText());
		}

		// Update or re-trigger autocomplete after forward delete
		if (this.autocompleteState) {
			this.updateAutocomplete();
		} else {
			const currentLine = this.state.lines[this.state.cursorLine] || "";
			const textBeforeCursor = currentLine.slice(0, this.state.cursorCol);
			// Slash command context
			if (this.isInSlashCommandContext(textBeforeCursor)) {
				this.tryTriggerAutocomplete();
			}
			// Symbol-based completion context like @ or #
			else if (textBeforeCursor.match(/(?:^|[\s])[@#][^\s]*$/)) {
				this.tryTriggerAutocomplete();
			}
		}
	}

	/**
	 * 构建逻辑行 → 视觉行索引映射。
	 *
	 * 对每条逻辑行调用 wordWrapLine 将其断行为多个视觉行段，
	 * 每个段记录它在原始逻辑行中的起始列和长度。
	 * 空行单独占一个零长视觉行。
	 * 此映射是光标垂直移动和翻页的几何基础。
	 *
	 * @param width - 换行宽度（内容区列数）
	 * @returns 视觉行数组 [{ logicalLine, startCol, length }]
	 */
	private buildVisualLineMap(width: number): Array<{ logicalLine: number; startCol: number; length: number }> {
		const visualLines: Array<{ logicalLine: number; startCol: number; length: number }> = [];

		for (let i = 0; i < this.state.lines.length; i++) {
			const line = this.state.lines[i] || "";
			const lineVisWidth = visibleWidth(line);
			if (line.length === 0) {
				// Empty line still takes one visual line
				visualLines.push({ logicalLine: i, startCol: 0, length: 0 });
			} else if (lineVisWidth <= width) {
				visualLines.push({ logicalLine: i, startCol: 0, length: line.length });
			} else {
				// Line needs wrapping - use word-aware wrapping
				const chunks = wordWrapLine(line, width, [...this.segment(line)]);
				for (const chunk of chunks) {
					visualLines.push({
						logicalLine: i,
						startCol: chunk.startIndex,
						length: chunk.endIndex - chunk.startIndex,
					});
				}
			}
		}

		return visualLines;
	}

	/**
	 * 在视觉行映射中定位包含给定逻辑位置的视觉行索引。
	 *
	 * 对于逻辑行的最后一个视觉段，允许 cursor 落在末尾位置（offset === vl.length），
	 * 以便在行尾正常插入。其他段要求 offset < vl.length。
	 */
	private findVisualLineAt(
		visualLines: Array<{ logicalLine: number; startCol: number; length: number }>,
		line: number,
		col: number,
	): number {
		for (let i = 0; i < visualLines.length; i++) {
			const vl = visualLines[i];
			if (!vl || vl.logicalLine !== line) continue;
			const offset = col - vl.startCol;
			// Cursor is in this segment if it's within range. For the last
			// segment of a logical line, cursor can be at length (end position)
			const isLastSegmentOfLine = i === visualLines.length - 1 || visualLines[i + 1]?.logicalLine !== vl.logicalLine;
			if (offset >= 0 && (offset < vl.length || (isLastSegmentOfLine && offset === vl.length))) {
				return i;
			}
		}
		return visualLines.length - 1;
	}

	/**
	 * Find the visual line index for the current cursor position.
	 */
	private findCurrentVisualLine(
		visualLines: Array<{ logicalLine: number; startCol: number; length: number }>,
	): number {
		return this.findVisualLineAt(visualLines, this.state.cursorLine, this.state.cursorCol);
	}

	private moveCursor(deltaLine: number, deltaCol: number): void {
		this.lastAction = null;
		const visualLines = this.buildVisualLineMap(this.lastWidth);
		const currentVisualLine = this.findCurrentVisualLine(visualLines);

		if (deltaLine !== 0) {
			const targetVisualLine = currentVisualLine + deltaLine;

			if (targetVisualLine >= 0 && targetVisualLine < visualLines.length) {
				this.moveToVisualLine(visualLines, currentVisualLine, targetVisualLine);
			}
		}

		if (deltaCol !== 0) {
			const currentLine = this.state.lines[this.state.cursorLine] || "";

			if (deltaCol > 0) {
				// Moving right - move by one grapheme (handles emojis, combining characters, etc.)
				if (this.state.cursorCol < currentLine.length) {
					const afterCursor = currentLine.slice(this.state.cursorCol);
					const graphemes = [...this.segment(afterCursor)];
					const firstGrapheme = graphemes[0];
					this.setCursorCol(this.state.cursorCol + (firstGrapheme ? firstGrapheme.segment.length : 1));
				} else if (this.state.cursorLine < this.state.lines.length - 1) {
					// Wrap to start of next logical line
					this.state.cursorLine++;
					this.setCursorCol(0);
				} else {
					// At end of last line - can't move, but set preferredVisualCol for up/down navigation
					const currentVL = visualLines[currentVisualLine];
					if (currentVL) {
						this.preferredVisualCol = this.state.cursorCol - currentVL.startCol;
					}
				}
			} else {
				// Moving left - move by one grapheme (handles emojis, combining characters, etc.)
				if (this.state.cursorCol > 0) {
					const beforeCursor = currentLine.slice(0, this.state.cursorCol);
					const graphemes = [...this.segment(beforeCursor)];
					const lastGrapheme = graphemes[graphemes.length - 1];
					this.setCursorCol(this.state.cursorCol - (lastGrapheme ? lastGrapheme.segment.length : 1));
				} else if (this.state.cursorLine > 0) {
					// Wrap to end of previous logical line
					this.state.cursorLine--;
					const prevLine = this.state.lines[this.state.cursorLine] || "";
					this.setCursorCol(prevLine.length);
				}
			}
		}
	}

	/**
	 * Scroll by a page (direction: -1 for up, 1 for down).
	 * Moves cursor by the page size while keeping it in bounds.
	 */
	private pageScroll(direction: -1 | 1): void {
		this.lastAction = null;
		const terminalRows = this.tui.terminal.rows;
		const pageSize = Math.max(5, Math.floor(terminalRows * 0.3));

		const visualLines = this.buildVisualLineMap(this.lastWidth);
		const currentVisualLine = this.findCurrentVisualLine(visualLines);
		const targetVisualLine = Math.max(0, Math.min(visualLines.length - 1, currentVisualLine + direction * pageSize));

		this.moveToVisualLine(visualLines, currentVisualLine, targetVisualLine);
	}

	/**
	 * 向后（左）移动一个词。
	 *
	 * 按照 grapheme 粒度跳过三种连续的字符类：
	 *   1. 尾随空白 → 跳过
	 *   2. 粘贴标记 → 作为原子词跳过
	 *   3. 标点 → 跳过连续标点
	 *   4. 单词 → 跳过连续非空白/非标点字符
	 * 如果已在行首，跳到上一逻辑行的行尾。
	 */
	private moveWordBackwards(): void {
		this.lastAction = null;
		const currentLine = this.state.lines[this.state.cursorLine] || "";

		// If at start of line, move to end of previous line
		if (this.state.cursorCol === 0) {
			if (this.state.cursorLine > 0) {
				this.state.cursorLine--;
				const prevLine = this.state.lines[this.state.cursorLine] || "";
				this.setCursorCol(prevLine.length);
			}
			return;
		}

		const textBeforeCursor = currentLine.slice(0, this.state.cursorCol);
		const graphemes = [...this.segment(textBeforeCursor)];
		let newCol = this.state.cursorCol;

		// Skip trailing whitespace
		while (
			graphemes.length > 0 &&
			!isPasteMarker(graphemes[graphemes.length - 1]?.segment || "") &&
			isWhitespaceChar(graphemes[graphemes.length - 1]?.segment || "")
		) {
			newCol -= graphemes.pop()?.segment.length || 0;
		}

		if (graphemes.length > 0) {
			const lastGrapheme = graphemes[graphemes.length - 1]?.segment || "";
			if (isPasteMarker(lastGrapheme)) {
				// Paste marker is a single atomic word
				newCol -= graphemes.pop()?.segment.length || 0;
			} else if (isPunctuationChar(lastGrapheme)) {
				// Skip punctuation run
				while (
					graphemes.length > 0 &&
					isPunctuationChar(graphemes[graphemes.length - 1]?.segment || "") &&
					!isPasteMarker(graphemes[graphemes.length - 1]?.segment || "")
				) {
					newCol -= graphemes.pop()?.segment.length || 0;
				}
			} else {
				// Skip word run
				while (
					graphemes.length > 0 &&
					!isWhitespaceChar(graphemes[graphemes.length - 1]?.segment || "") &&
					!isPunctuationChar(graphemes[graphemes.length - 1]?.segment || "") &&
					!isPasteMarker(graphemes[graphemes.length - 1]?.segment || "")
				) {
					newCol -= graphemes.pop()?.segment.length || 0;
				}
			}
		}

		this.setCursorCol(newCol);
	}

	/**
	 * Yank (paste) the most recent kill ring entry at cursor position.
	 */
	private yank(): void {
		if (this.killRing.length === 0) return;

		this.pushUndoSnapshot();

		const text = this.killRing.peek()!;
		this.insertYankedText(text);

		this.lastAction = "yank";
	}

	/**
	 * Cycle through kill ring (only works immediately after yank or yank-pop).
	 * Replaces the last yanked text with the previous entry in the ring.
	 */
	private yankPop(): void {
		// Only works if we just yanked and have more than one entry
		if (this.lastAction !== "yank" || this.killRing.length <= 1) return;

		this.pushUndoSnapshot();

		// Delete the previously yanked text (still at end of ring before rotation)
		this.deleteYankedText();

		// Rotate the ring: move end to front
		this.killRing.rotate();

		// Insert the new most recent entry (now at end after rotation)
		const text = this.killRing.peek()!;
		this.insertYankedText(text);

		this.lastAction = "yank";
	}

	/**
	 * Insert text at cursor position (used by yank operations).
	 */
	private insertYankedText(text: string): void {
		this.historyIndex = -1; // Exit history browsing mode
		const lines = text.split("\n");

		if (lines.length === 1) {
			// Single line - insert at cursor
			const currentLine = this.state.lines[this.state.cursorLine] || "";
			const before = currentLine.slice(0, this.state.cursorCol);
			const after = currentLine.slice(this.state.cursorCol);
			this.state.lines[this.state.cursorLine] = before + text + after;
			this.setCursorCol(this.state.cursorCol + text.length);
		} else {
			// Multi-line insert
			const currentLine = this.state.lines[this.state.cursorLine] || "";
			const before = currentLine.slice(0, this.state.cursorCol);
			const after = currentLine.slice(this.state.cursorCol);

			// First line merges with text before cursor
			this.state.lines[this.state.cursorLine] = before + (lines[0] || "");

			// Insert middle lines
			for (let i = 1; i < lines.length - 1; i++) {
				this.state.lines.splice(this.state.cursorLine + i, 0, lines[i] || "");
			}

			// Last line merges with text after cursor
			const lastLineIndex = this.state.cursorLine + lines.length - 1;
			this.state.lines.splice(lastLineIndex, 0, (lines[lines.length - 1] || "") + after);

			// Update cursor position
			this.state.cursorLine = lastLineIndex;
			this.setCursorCol((lines[lines.length - 1] || "").length);
		}

		if (this.onChange) {
			this.onChange(this.getText());
		}
	}

	/**
	 * Delete the previously yanked text (used by yank-pop).
	 * The yanked text is derived from killRing[end] since it hasn't been rotated yet.
	 */
	private deleteYankedText(): void {
		const yankedText = this.killRing.peek();
		if (!yankedText) return;

		const yankLines = yankedText.split("\n");

		if (yankLines.length === 1) {
			// Single line - delete backward from cursor
			const currentLine = this.state.lines[this.state.cursorLine] || "";
			const deleteLen = yankedText.length;
			const before = currentLine.slice(0, this.state.cursorCol - deleteLen);
			const after = currentLine.slice(this.state.cursorCol);
			this.state.lines[this.state.cursorLine] = before + after;
			this.setCursorCol(this.state.cursorCol - deleteLen);
		} else {
			// Multi-line delete - cursor is at end of last yanked line
			const startLine = this.state.cursorLine - (yankLines.length - 1);
			const startCol = (this.state.lines[startLine] || "").length - (yankLines[0] || "").length;

			// Get text after cursor on current line
			const afterCursor = (this.state.lines[this.state.cursorLine] || "").slice(this.state.cursorCol);

			// Get text before yank start position
			const beforeYank = (this.state.lines[startLine] || "").slice(0, startCol);

			// Remove all lines from startLine to cursorLine and replace with merged line
			this.state.lines.splice(startLine, yankLines.length, beforeYank + afterCursor);

			// Update cursor
			this.state.cursorLine = startLine;
			this.setCursorCol(startCol);
		}

		if (this.onChange) {
			this.onChange(this.getText());
		}
	}

	/**
	 * 压入撤销快照 —— 将当前 EditorState 浅拷贝存入 UndoStack。
	 * 在每一个编辑操作（插入/删除/粘贴/补全确认）之前调用。
	 */
	private pushUndoSnapshot(): void {
		this.undoStack.push(this.state);
	}

	/**
	 * 撤销 —— 从 UndoStack 弹出最近快照并恢复到编辑器。
	 * 恢复后重置 lastAction、preferredVisualCol 并触发 onChange。
	 */
	private undo(): void {
		this.historyIndex = -1; // Exit history browsing mode
		const snapshot = this.undoStack.pop();
		if (!snapshot) return;
		Object.assign(this.state, snapshot);
		this.lastAction = null;
		this.preferredVisualCol = null;
		if (this.onChange) {
			this.onChange(this.getText());
		}
	}

	/**
	 * Jump to the first occurrence of a character in the specified direction.
	 * Multi-line search. Case-sensitive. Skips the current cursor position.
	 */
	private jumpToChar(char: string, direction: "forward" | "backward"): void {
		this.lastAction = null;
		const isForward = direction === "forward";
		const lines = this.state.lines;

		const end = isForward ? lines.length : -1;
		const step = isForward ? 1 : -1;

		for (let lineIdx = this.state.cursorLine; lineIdx !== end; lineIdx += step) {
			const line = lines[lineIdx] || "";
			const isCurrentLine = lineIdx === this.state.cursorLine;

			// Current line: start after/before cursor; other lines: search full line
			const searchFrom = isCurrentLine
				? isForward
					? this.state.cursorCol + 1
					: this.state.cursorCol - 1
				: undefined;

			const idx = isForward ? line.indexOf(char, searchFrom) : line.lastIndexOf(char, searchFrom);

			if (idx !== -1) {
				this.state.cursorLine = lineIdx;
				this.setCursorCol(idx);
				return;
			}
		}
		// No match found - cursor stays in place
	}

	/**
	 * 向前（右）移动一个词。
	 *
	 * 按 grapheme 粒度跳过三种字符类：
	 *   1. 前导空白 → 跳过
	 *   2. 粘贴标记 → 作为原子词跳过
	 *   3. 标点 → 跳过连续标点
	 *   4. 单词 → 跳过连续非空白非标点字符
	 * 如果已在行尾，跳到下一逻辑行的行首。
	 */
	private moveWordForwards(): void {
		this.lastAction = null;
		const currentLine = this.state.lines[this.state.cursorLine] || "";

		// If at end of line, move to start of next line
		if (this.state.cursorCol >= currentLine.length) {
			if (this.state.cursorLine < this.state.lines.length - 1) {
				this.state.cursorLine++;
				this.setCursorCol(0);
			}
			return;
		}

		const textAfterCursor = currentLine.slice(this.state.cursorCol);
		const segments = this.segment(textAfterCursor);
		const iterator = segments[Symbol.iterator]();
		let next = iterator.next();
		let newCol = this.state.cursorCol;

		// Skip leading whitespace
		while (!next.done && !isPasteMarker(next.value.segment) && isWhitespaceChar(next.value.segment)) {
			newCol += next.value.segment.length;
			next = iterator.next();
		}

		if (!next.done) {
			const firstGrapheme = next.value.segment;
			if (isPasteMarker(firstGrapheme)) {
				// Paste marker is a single atomic word
				newCol += firstGrapheme.length;
			} else if (isPunctuationChar(firstGrapheme)) {
				// Skip punctuation run
				while (!next.done && isPunctuationChar(next.value.segment) && !isPasteMarker(next.value.segment)) {
					newCol += next.value.segment.length;
					next = iterator.next();
				}
			} else {
				// Skip word run
				while (
					!next.done &&
					!isWhitespaceChar(next.value.segment) &&
					!isPunctuationChar(next.value.segment) &&
					!isPasteMarker(next.value.segment)
				) {
					newCol += next.value.segment.length;
					next = iterator.next();
				}
			}
		}

		this.setCursorCol(newCol);
	}

	// ── 自动补全子系统方法 ──

	/** 仅首行（cursorLine === 0）允许斜杠命令菜单 */
	private isSlashMenuAllowed(): boolean {
		return this.state.cursorLine === 0;
	}

	/** 光标前文本全为空白或仅有 "/" → 判定为消息开头（用于斜杠命令检测） */
	private isAtStartOfMessage(): boolean {
		if (!this.isSlashMenuAllowed()) return false;
		const currentLine = this.state.lines[this.state.cursorLine] || "";
		const beforeCursor = currentLine.slice(0, this.state.cursorCol);
		return beforeCursor.trim() === "" || beforeCursor.trim() === "/";
	}

	private isInSlashCommandContext(textBeforeCursor: string): boolean {
		return this.isSlashMenuAllowed() && textBeforeCursor.trimStart().startsWith("/");
	}

	// Autocomplete methods
	/**
	 * Find the best autocomplete item index for the given prefix.
	 * Returns -1 if no match is found.
	 *
	 * Match priority:
	 * 1. Exact match (prefix === item.value) -> always selected
	 * 2. Prefix match -> first item whose value starts with prefix
	 * 3. No match -> -1 (keep default highlight)
	 *
	 * Matching is case-sensitive and checks item.value only.
	 */
	private getBestAutocompleteMatchIndex(items: Array<{ value: string; label: string }>, prefix: string): number {
		if (!prefix) return -1;

		let firstPrefixIndex = -1;

		for (let i = 0; i < items.length; i++) {
			const value = items[i]!.value;
			if (value === prefix) {
				return i; // Exact match always wins
			}
			if (firstPrefixIndex === -1 && value.startsWith(prefix)) {
				firstPrefixIndex = i;
			}
		}

		return firstPrefixIndex;
	}

	private createAutocompleteList(
		prefix: string,
		items: Array<{ value: string; label: string; description?: string }>,
	): SelectList {
		const layout = prefix.startsWith("/") ? SLASH_COMMAND_SELECT_LIST_LAYOUT : undefined;
		return new SelectList(items, this.autocompleteMaxVisible, this.theme.selectList, layout);
	}

	private tryTriggerAutocomplete(explicitTab: boolean = false): void {
		this.requestAutocomplete({ force: false, explicitTab });
	}

	private handleTabCompletion(): void {
		if (!this.autocompleteProvider) return;

		const currentLine = this.state.lines[this.state.cursorLine] || "";
		const beforeCursor = currentLine.slice(0, this.state.cursorCol);

		if (this.isInSlashCommandContext(beforeCursor) && !beforeCursor.trimStart().includes(" ")) {
			this.handleSlashCommandCompletion();
		} else {
			this.forceFileAutocomplete(true);
		}
	}

	private handleSlashCommandCompletion(): void {
		this.requestAutocomplete({ force: false, explicitTab: true });
	}

	private forceFileAutocomplete(explicitTab: boolean = false): void {
		this.requestAutocomplete({ force: true, explicitTab });
	}

	/**
	 * 发起补全请求（带防抖和取消机制）。
	 *
	 * 请求生命周期：
	 *   1. cancelAutocompleteRequest() 中止前一个请求 + 递增 startToken
	 *   2. 根据上下文决定防抖延迟（斜杠命令 0ms，@/# 符号 20ms，Tab/Force 0ms）
	 *   3. 防抖或立即执行 startAutocompleteRequest
	 *   4. 异步请求完成后 isAutocompleteRequestCurrent 检查快照一致性，丢弃过期结果
	 *
	 * @param options.force - true = Tab 强制补全，null prefix 时触发文件补全
	 * @param options.explicitTab - true = 用户按 Tab，跳过防抖
	 */
	private requestAutocomplete(options: { force: boolean; explicitTab: boolean }): void {
		if (!this.autocompleteProvider) return;

		if (options.force) {
			const shouldTrigger =
				!this.autocompleteProvider.shouldTriggerFileCompletion ||
				this.autocompleteProvider.shouldTriggerFileCompletion(
					this.state.lines,
					this.state.cursorLine,
					this.state.cursorCol,
				);
			if (!shouldTrigger) {
				return;
			}
		}

		this.cancelAutocompleteRequest();
		const startToken = ++this.autocompleteStartToken;

		const debounceMs = this.getAutocompleteDebounceMs(options);
		if (debounceMs > 0) {
			this.autocompleteDebounceTimer = setTimeout(() => {
				this.autocompleteDebounceTimer = undefined;
				void this.startAutocompleteRequest(startToken, options);
			}, debounceMs);
			return;
		}

		void this.startAutocompleteRequest(startToken, options);
	}

	private async startAutocompleteRequest(
		startToken: number,
		options: { force: boolean; explicitTab: boolean },
	): Promise<void> {
		const previousTask = this.autocompleteRequestTask;
		this.autocompleteRequestTask = (async () => {
			await previousTask;
			if (startToken !== this.autocompleteStartToken || !this.autocompleteProvider) {
				return;
			}

			const controller = new AbortController();
			this.autocompleteAbort = controller;
			const requestId = ++this.autocompleteRequestId;
			const snapshotText = this.getText();
			const snapshotLine = this.state.cursorLine;
			const snapshotCol = this.state.cursorCol;

			await this.runAutocompleteRequest(requestId, controller, snapshotText, snapshotLine, snapshotCol, options);
		})();
		await this.autocompleteRequestTask;
	}

	private getAutocompleteDebounceMs(options: { force: boolean; explicitTab: boolean }): number {
		if (options.explicitTab || options.force) {
			return 0;
		}

		const currentLine = this.state.lines[this.state.cursorLine] || "";
		const textBeforeCursor = currentLine.slice(0, this.state.cursorCol);
		const isSymbolAutocompleteContext = /(?:^|[ \t])(?:@(?:"[^"]*|[^\s]*)|#[^\s]*)$/.test(textBeforeCursor);
		return isSymbolAutocompleteContext ? ATTACHMENT_AUTOCOMPLETE_DEBOUNCE_MS : 0;
	}

	private async runAutocompleteRequest(
		requestId: number,
		controller: AbortController,
		snapshotText: string,
		snapshotLine: number,
		snapshotCol: number,
		options: { force: boolean; explicitTab: boolean },
	): Promise<void> {
		if (!this.autocompleteProvider) return;

		const suggestions = await this.autocompleteProvider.getSuggestions(
			this.state.lines,
			this.state.cursorLine,
			this.state.cursorCol,
			{ signal: controller.signal, force: options.force },
		);

		if (!this.isAutocompleteRequestCurrent(requestId, controller, snapshotText, snapshotLine, snapshotCol)) {
			return;
		}

		this.autocompleteAbort = undefined;

		if (!suggestions || !Array.isArray(suggestions.items) || suggestions.items.length === 0) {
			this.cancelAutocomplete();
			this.tui.requestRender();
			return;
		}

		if (options.force && options.explicitTab && suggestions.items.length === 1) {
			const item = suggestions.items[0]!;
			this.pushUndoSnapshot();
			this.lastAction = null;
			const result = this.autocompleteProvider.applyCompletion(
				this.state.lines,
				this.state.cursorLine,
				this.state.cursorCol,
				item,
				suggestions.prefix,
			);
			this.state.lines = result.lines;
			this.state.cursorLine = result.cursorLine;
			this.setCursorCol(result.cursorCol);
			if (this.onChange) this.onChange(this.getText());
			this.tui.requestRender();
			return;
		}

		this.applyAutocompleteSuggestions(suggestions, options.force ? "force" : "regular");
		this.tui.requestRender();
	}

	private isAutocompleteRequestCurrent(
		requestId: number,
		controller: AbortController,
		snapshotText: string,
		snapshotLine: number,
		snapshotCol: number,
	): boolean {
		return (
			!controller.signal.aborted &&
			requestId === this.autocompleteRequestId &&
			this.getText() === snapshotText &&
			this.state.cursorLine === snapshotLine &&
			this.state.cursorCol === snapshotCol
		);
	}

	private applyAutocompleteSuggestions(suggestions: AutocompleteSuggestions, state: "regular" | "force"): void {
		this.autocompletePrefix = suggestions.prefix;
		this.autocompleteList = this.createAutocompleteList(suggestions.prefix, suggestions.items);

		const bestMatchIndex = this.getBestAutocompleteMatchIndex(suggestions.items, suggestions.prefix);
		if (bestMatchIndex >= 0) {
			this.autocompleteList.setSelectedIndex(bestMatchIndex);
		}

		this.autocompleteState = state;
	}

	private cancelAutocompleteRequest(): void {
		this.autocompleteStartToken += 1;
		if (this.autocompleteDebounceTimer) {
			clearTimeout(this.autocompleteDebounceTimer);
			this.autocompleteDebounceTimer = undefined;
		}
		this.autocompleteAbort?.abort();
		this.autocompleteAbort = undefined;
	}

	private clearAutocompleteUi(): void {
		this.autocompleteState = null;
		this.autocompleteList = undefined;
		this.autocompletePrefix = "";
	}

	private cancelAutocomplete(): void {
		this.cancelAutocompleteRequest();
		this.clearAutocompleteUi();
	}

	public isShowingAutocomplete(): boolean {
		return this.autocompleteState !== null;
	}

	private updateAutocomplete(): void {
		if (!this.autocompleteState || !this.autocompleteProvider) return;
		this.requestAutocomplete({ force: this.autocompleteState === "force", explicitTab: false });
	}
}
