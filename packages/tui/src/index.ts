/**
 * @fileoverview @earendil-works/pi-tui 包的公共 API 导出入口。
 *
 * TUI（Terminal User Interface）包提供终端用户界面的底层构建块：
 *
 * 导出内容分类：
 *   - TUI 核心：Container、TUI 主类、组件接口
 *   - 组件：Box、Editor、Image、Input、Loader、Markdown、SelectList 等
 *   - 键盘处理：Key 类、按键解析、Kitty 协议支持
 *   - 快捷键系统：KeybindingsManager、快捷键定义和配置
 *   - 自动补全：AutocompleteProvider、CombinedAutocompleteProvider
 *   - 终端图片：Kitty/iTerm2 协议、图片渲染和尺寸计算
 *   - 工具函数：文本截断、可见宽度计算、ANSI 文本换行
 */

// 自动补全支持
export {
	type AutocompleteItem,
	type AutocompleteProvider,
	type AutocompleteSuggestions,
	CombinedAutocompleteProvider,
	type SlashCommand,
} from "./autocomplete.js";
// 组件
export { Box } from "./components/box.js";
export { CancellableLoader } from "./components/cancellable-loader.js";
export { Editor, type EditorOptions, type EditorTheme } from "./components/editor.js";
export { Image, type ImageOptions, type ImageTheme } from "./components/image.js";
export { Input } from "./components/input.js";
export { Loader, type LoaderIndicatorOptions } from "./components/loader.js";
export { type DefaultTextStyle, Markdown, type MarkdownTheme } from "./components/markdown.js";
export {
	type SelectItem,
	SelectList,
	type SelectListLayoutOptions,
	type SelectListTheme,
	type SelectListTruncatePrimaryContext,
} from "./components/select-list.js";
export { type SettingItem, SettingsList, type SettingsListTheme } from "./components/settings-list.js";
export { Spacer } from "./components/spacer.js";
export { Text } from "./components/text.js";
export { TruncatedText } from "./components/truncated-text.js";
// 编辑器组件接口（用于自定义编辑器）
export type { EditorComponent } from "./editor-component.js";
// 模糊匹配
export { type FuzzyMatch, fuzzyFilter, fuzzyMatch } from "./fuzzy.js";
// 快捷键系统
export {
	getKeybindings,
	type Keybinding,
	type KeybindingConflict,
	type KeybindingDefinition,
	type KeybindingDefinitions,
	type Keybindings,
	type KeybindingsConfig,
	KeybindingsManager,
	setKeybindings,
	TUI_KEYBINDINGS,
} from "./keybindings.js";
// 键盘输入处理
export {
	decodeKittyPrintable,
	isKeyRelease,
	isKeyRepeat,
	isKittyProtocolActive,
	Key,
	type KeyEventType,
	type KeyId,
	matchesKey,
	parseKey,
	setKittyProtocolActive,
} from "./keys.js";
// 输入缓冲（用于批量分割）
export { StdinBuffer, type StdinBufferEventMap, type StdinBufferOptions } from "./stdin-buffer.js";
// 终端接口和实现
export { ProcessTerminal, type Terminal } from "./terminal.js";
// 终端图片支持（Kitty/iTerm2 协议、图片渲染、尺寸计算）
export {
	allocateImageId,
	type CellDimensions,
	calculateImageRows,
	deleteAllKittyImages,
	deleteKittyImage,
	detectCapabilities,
	encodeITerm2,
	encodeKitty,
	getCapabilities,
	getCellDimensions,
	getGifDimensions,
	getImageDimensions,
	getJpegDimensions,
	getPngDimensions,
	getWebpDimensions,
	hyperlink,
	type ImageDimensions,
	type ImageProtocol,
	type ImageRenderOptions,
	imageFallback,
	renderImage,
	resetCapabilitiesCache,
	setCapabilities,
	setCellDimensions,
	type TerminalCapabilities,
} from "./terminal-image.js";
// TUI 核心：容器、组件接口、主类
export {
	type Component,
	Container,
	CURSOR_MARKER,
	type Focusable,
	isFocusable,
	type OverlayAnchor,
	type OverlayHandle,
	type OverlayMargin,
	type OverlayOptions,
	type SizeValue,
	TUI,
} from "./tui.js";
// 工具函数
export { truncateToWidth, visibleWidth, wrapTextWithAnsi } from "./utils.js";
