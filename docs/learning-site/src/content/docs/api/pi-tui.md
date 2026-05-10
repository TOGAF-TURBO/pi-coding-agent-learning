---
title: pi-tui API 参考
---


- [setKittyProtocolActive](#gear-setkittyprotocolactive)
- [isKittyProtocolActive](#gear-iskittyprotocolactive)
- [Key.ctrl](#gear-key.ctrl)
- [isKeyRelease](#gear-iskeyrelease)
- [isKeyRepeat](#gear-iskeyrepeat)
- [matchesKey](#gear-matcheskey)
- [parseKey](#gear-parsekey)
- [decodeKittyPrintable](#gear-decodekittyprintable)
- [decodePrintableKey](#gear-decodeprintablekey)
- [getCellDimensions](#gear-getcelldimensions)
- [setCellDimensions](#gear-setcelldimensions)
- [detectCapabilities](#gear-detectcapabilities)
- [getCapabilities](#gear-getcapabilities)
- [resetCapabilitiesCache](#gear-resetcapabilitiescache)
- [setCapabilities](#gear-setcapabilities)
- [isImageLine](#gear-isimageline)
- [allocateImageId](#gear-allocateimageid)
- [encodeKitty](#gear-encodekitty)
- [deleteKittyImage](#gear-deletekittyimage)
- [deleteAllKittyImages](#gear-deleteallkittyimages)
- [encodeITerm2](#gear-encodeiterm2)
- [calculateImageRows](#gear-calculateimagerows)
- [getPngDimensions](#gear-getpngdimensions)
- [getJpegDimensions](#gear-getjpegdimensions)
- [getGifDimensions](#gear-getgifdimensions)
- [getWebpDimensions](#gear-getwebpdimensions)
- [getImageDimensions](#gear-getimagedimensions)
- [renderImage](#gear-renderimage)
- [hyperlink](#gear-hyperlink)
- [imageFallback](#gear-imagefallback)
- [getSegmenter](#gear-getsegmenter)
- [visibleWidth](#gear-visiblewidth)
- [normalizeTerminalOutput](#gear-normalizeterminaloutput)
- [extractAnsiCode](#gear-extractansicode)
- [wrapTextWithAnsi](#gear-wraptextwithansi)
- [isWhitespaceChar](#gear-iswhitespacechar)
- [isPunctuationChar](#gear-ispunctuationchar)
- [applyBackgroundToLine](#gear-applybackgroundtoline)
- [truncateToWidth](#gear-truncatetowidth)
- [sliceByColumn](#gear-slicebycolumn)
- [sliceWithWidth](#gear-slicewithwidth)
- [extractSegments](#gear-extractsegments)
- [isFocusable](#gear-isfocusable)
- [fuzzyMatch](#gear-fuzzymatch)
- [fuzzyFilter](#gear-fuzzyfilter)
- [setKeybindings](#gear-setkeybindings)
- [getKeybindings](#gear-getkeybindings)
- [wordWrapLine](#gear-wordwrapline)

### :gear: setKittyProtocolActive

Set the global Kitty keyboard protocol state.
Called by ProcessTerminal after detecting protocol support.

| Function | Type |
| ---------- | ---------- |
| `setKittyProtocolActive` | `(active: boolean) => void` |

### :gear: isKittyProtocolActive

Query whether Kitty keyboard protocol is currently active.

| Function | Type |
| ---------- | ---------- |
| `isKittyProtocolActive` | `() => boolean` |

### :gear: Key.ctrl

| Function | Type |
| ---------- | ---------- |
| `Key.ctrl` | `<K extends BaseKey>(key: K) => `ctrl+${K}`` |

### :gear: isKeyRelease

Check if the last parsed key event was a key release.
Only meaningful when Kitty keyboard protocol with flag 2 is active.

| Function | Type |
| ---------- | ---------- |
| `isKeyRelease` | `(data: string) => boolean` |

### :gear: isKeyRepeat

Check if the last parsed key event was a key repeat.
Only meaningful when Kitty keyboard protocol with flag 2 is active.

| Function | Type |
| ---------- | ---------- |
| `isKeyRepeat` | `(data: string) => boolean` |

### :gear: matchesKey

Match input data against a key identifier string.

Supported key identifiers:
- Single keys: "escape", "tab", "enter", "backspace", "delete", "home", "end", "space"
- Arrow keys: "up", "down", "left", "right"
- Ctrl combinations: "ctrl+c", "ctrl+z", etc.
- Shift combinations: "shift+tab", "shift+enter"
- Alt combinations: "alt+enter", "alt+backspace"
- Super combinations: "super+k", "super+enter"
- Combined modifiers: "shift+ctrl+p", "ctrl+alt+x", "ctrl+super+k"

Use the Key helper for autocomplete: Key.ctrl("c"), Key.escape, Key.ctrlShift("p"), Key.super("k")

| Function | Type |
| ---------- | ---------- |
| `matchesKey` | `(data: string, keyId: KeyId) => boolean` |

Parameters:

* `data`: - Raw input data from terminal
* `keyId`: - Key identifier (e.g., "ctrl+c", "escape", Key.ctrl("c"))


### :gear: parseKey

| Function | Type |
| ---------- | ---------- |
| `parseKey` | `(data: string) => string or undefined` |

### :gear: decodeKittyPrintable

Decode a Kitty CSI-u sequence into a printable character, if applicable.

When Kitty keyboard protocol flag 1 (disambiguate) is active, terminals send
CSI-u sequences for all keys, including plain printable characters. This
function extracts the printable character from such sequences.

Only accepts plain or Shift-modified keys. Rejects Ctrl, Alt, and unsupported
modifier combinations (those are handled by keybinding matching instead).
Prefers the shifted keycode when Shift is held and a shifted key is reported.

| Function | Type |
| ---------- | ---------- |
| `decodeKittyPrintable` | `(data: string) => string or undefined` |

Parameters:

* `data`: - Raw input data from terminal


Returns:

The printable character, or undefined if not a printable CSI-u sequence

### :gear: decodePrintableKey

| Function | Type |
| ---------- | ---------- |
| `decodePrintableKey` | `(data: string) => string or undefined` |

### :gear: getCellDimensions

| Function | Type |
| ---------- | ---------- |
| `getCellDimensions` | `() => CellDimensions` |

### :gear: setCellDimensions

| Function | Type |
| ---------- | ---------- |
| `setCellDimensions` | `(dims: CellDimensions) => void` |

### :gear: detectCapabilities

| Function | Type |
| ---------- | ---------- |
| `detectCapabilities` | `() => TerminalCapabilities` |

### :gear: getCapabilities

| Function | Type |
| ---------- | ---------- |
| `getCapabilities` | `() => TerminalCapabilities` |

### :gear: resetCapabilitiesCache

| Function | Type |
| ---------- | ---------- |
| `resetCapabilitiesCache` | `() => void` |

### :gear: setCapabilities

Override the cached capabilities. Useful in tests to exercise both code paths.

| Function | Type |
| ---------- | ---------- |
| `setCapabilities` | `(caps: TerminalCapabilities) => void` |

### :gear: isImageLine

| Function | Type |
| ---------- | ---------- |
| `isImageLine` | `(line: string) => boolean` |

### :gear: allocateImageId

Generate a random image ID for Kitty graphics protocol.
Uses random IDs to avoid collisions between different module instances
(e.g., main app vs extensions).

| Function | Type |
| ---------- | ---------- |
| `allocateImageId` | `() => number` |

### :gear: encodeKitty

| Function | Type |
| ---------- | ---------- |
| `encodeKitty` | `(base64Data: string, options?: { columns?: number or undefined; rows?: number or undefined; imageId?: number or undefined; moveCursor?: boolean or undefined; }) => string` |

### :gear: deleteKittyImage

Delete a Kitty graphics image by ID.
Uses uppercase 'I' to also free the image data.

| Function | Type |
| ---------- | ---------- |
| `deleteKittyImage` | `(imageId: number) => string` |

### :gear: deleteAllKittyImages

Delete all visible Kitty graphics images.
Uses uppercase 'A' to also free the image data.

| Function | Type |
| ---------- | ---------- |
| `deleteAllKittyImages` | `() => string` |

### :gear: encodeITerm2

| Function | Type |
| ---------- | ---------- |
| `encodeITerm2` | `(base64Data: string, options?: { width?: string or number or undefined; height?: string or number or undefined; name?: string or undefined; preserveAspectRatio?: boolean or undefined; inline?: boolean or undefined; }) => string` |

### :gear: calculateImageRows

| Function | Type |
| ---------- | ---------- |
| `calculateImageRows` | `(imageDimensions: ImageDimensions, targetWidthCells: number, cellDimensions?: CellDimensions) => number` |

### :gear: getPngDimensions

| Function | Type |
| ---------- | ---------- |
| `getPngDimensions` | `(base64Data: string) => ImageDimensions or null` |

### :gear: getJpegDimensions

| Function | Type |
| ---------- | ---------- |
| `getJpegDimensions` | `(base64Data: string) => ImageDimensions or null` |

### :gear: getGifDimensions

| Function | Type |
| ---------- | ---------- |
| `getGifDimensions` | `(base64Data: string) => ImageDimensions or null` |

### :gear: getWebpDimensions

| Function | Type |
| ---------- | ---------- |
| `getWebpDimensions` | `(base64Data: string) => ImageDimensions or null` |

### :gear: getImageDimensions

| Function | Type |
| ---------- | ---------- |
| `getImageDimensions` | `(base64Data: string, mimeType: string) => ImageDimensions or null` |

### :gear: renderImage

| Function | Type |
| ---------- | ---------- |
| `renderImage` | `(base64Data: string, imageDimensions: ImageDimensions, options?: ImageRenderOptions) => { sequence: string; rows: number; imageId?: number or undefined; } or null` |

### :gear: hyperlink

Wrap text in an OSC 8 hyperlink sequence.
The text is rendered as a clickable hyperlink in terminals that support OSC 8
(Ghostty, Kitty, WezTerm, iTerm2, VSCode, and others).
In terminals that do not support OSC 8, the escape sequences are ignored
and only the plain text is displayed.

| Function | Type |
| ---------- | ---------- |
| `hyperlink` | `(text: string, url: string) => string` |

Parameters:

* `text`: - The visible text to display
* `url`: - The URL to link to


### :gear: imageFallback

| Function | Type |
| ---------- | ---------- |
| `imageFallback` | `(mimeType: string, dimensions?: ImageDimensions or undefined, filename?: string or undefined) => string` |

### :gear: getSegmenter

Get the shared grapheme segmenter instance.

| Function | Type |
| ---------- | ---------- |
| `getSegmenter` | `() => Intl.Segmenter` |

### :gear: visibleWidth

Calculate the visible width of a string in terminal columns.

| Function | Type |
| ---------- | ---------- |
| `visibleWidth` | `(str: string) => number` |

### :gear: normalizeTerminalOutput

| Function | Type |
| ---------- | ---------- |
| `normalizeTerminalOutput` | `(str: string) => string` |

### :gear: extractAnsiCode

Extract ANSI escape sequences from a string at the given position.

| Function | Type |
| ---------- | ---------- |
| `extractAnsiCode` | `(str: string, pos: number) => { code: string; length: number; } or null` |

### :gear: wrapTextWithAnsi

Wrap text with ANSI codes preserved.

ONLY does word wrapping - NO padding, NO background colors.
Returns lines where each line is <= width visible chars.
Active ANSI codes are preserved across line breaks.

| Function | Type |
| ---------- | ---------- |
| `wrapTextWithAnsi` | `(text: string, width: number) => string[]` |

Parameters:

* `text`: - Text to wrap (may contain ANSI codes and newlines)
* `width`: - Maximum visible width per line


Returns:

Array of wrapped lines (NOT padded to width)

### :gear: isWhitespaceChar

Check if a character is whitespace.

| Function | Type |
| ---------- | ---------- |
| `isWhitespaceChar` | `(char: string) => boolean` |

### :gear: isPunctuationChar

Check if a character is punctuation.

| Function | Type |
| ---------- | ---------- |
| `isPunctuationChar` | `(char: string) => boolean` |

### :gear: applyBackgroundToLine

Apply background color to a line, padding to full width.

| Function | Type |
| ---------- | ---------- |
| `applyBackgroundToLine` | `(line: string, width: number, bgFn: (text: string) => string) => string` |

Parameters:

* `line`: - Line of text (may contain ANSI codes)
* `width`: - Total width to pad to
* `bgFn`: - Background color function


Returns:

Line with background applied and padded to width

### :gear: truncateToWidth

Truncate text to fit within a maximum visible width, adding ellipsis if needed.
Optionally pad with spaces to reach exactly maxWidth.
Properly handles ANSI escape codes (they don't count toward width).

| Function | Type |
| ---------- | ---------- |
| `truncateToWidth` | `(text: string, maxWidth: number, ellipsis?: string, pad?: boolean) => string` |

Parameters:

* `text`: - Text to truncate (may contain ANSI codes)
* `maxWidth`: - Maximum visible width
* `ellipsis`: - Ellipsis string to append when truncating (default: "...")
* `pad`: - If true, pad result with spaces to exactly maxWidth (default: false)


Returns:

Truncated text, optionally padded to exactly maxWidth

### :gear: sliceByColumn

Extract a range of visible columns from a line. Handles ANSI codes and wide chars.

| Function | Type |
| ---------- | ---------- |
| `sliceByColumn` | `(line: string, startCol: number, length: number, strict?: boolean) => string` |

Parameters:

* `strict`: - If true, exclude wide chars at boundary that would extend past the range


### :gear: sliceWithWidth

Like sliceByColumn but also returns the actual visible width of the result.

| Function | Type |
| ---------- | ---------- |
| `sliceWithWidth` | `(line: string, startCol: number, length: number, strict?: boolean) => { text: string; width: number; }` |

### :gear: extractSegments

Extract "before" and "after" segments from a line in a single pass.
Used for overlay compositing where we need content before and after the overlay region.
Preserves styling from before the overlay that should affect content after it.

| Function | Type |
| ---------- | ---------- |
| `extractSegments` | `(line: string, beforeEnd: number, afterStart: number, afterLen: number, strictAfter?: boolean) => { before: string; beforeWidth: number; after: string; afterWidth: number; }` |

### :gear: isFocusable

类型守卫：判断组件是否实现了 Focusable 接口（即可接收焦点）

| Function | Type |
| ---------- | ---------- |
| `isFocusable` | `(component: Component or null) => component is Component and Focusable` |

Parameters:

* `component`: - 待检查的组件


Returns:

如果组件非 null 且包含 focused 属性则返回 true

Type guard to check if a component implements Focusable

### :gear: fuzzyMatch

| Function | Type |
| ---------- | ---------- |
| `fuzzyMatch` | `(query: string, text: string) => FuzzyMatch` |

### :gear: fuzzyFilter

Filter and sort items by fuzzy match quality (best matches first).
Supports space-separated tokens: all tokens must match.

| Function | Type |
| ---------- | ---------- |
| `fuzzyFilter` | `<T>(items: T[], query: string, getText: (item: T) => string) => T[]` |

### :gear: setKeybindings

| Function | Type |
| ---------- | ---------- |
| `setKeybindings` | `(keybindings: KeybindingsManager) => void` |

### :gear: getKeybindings

| Function | Type |
| ---------- | ---------- |
| `getKeybindings` | `() => KeybindingsManager` |

### :gear: wordWrapLine

Split a line into word-wrapped chunks.
Wraps at word boundaries when possible, falling back to character-level
wrapping for words longer than the available width.

| Function | Type |
| ---------- | ---------- |
| `wordWrapLine` | `(line: string, maxWidth: number, preSegmented?: Intl.SegmentData[] or undefined) => TextChunk[]` |

Parameters:

* `line`: - The text line to wrap
* `maxWidth`: - Maximum visible width per chunk
* `preSegmented`: - Optional pre-segmented graphemes (e.g. with paste-marker awareness).
  When omitted the default Intl.Segmenter is used.


Returns:

Array of chunks with text and position information


## :wrench: Constants

- [CURSOR_MARKER](#gear-cursor_marker)
- [TUI_KEYBINDINGS](#gear-tui_keybindings)

### :gear: CURSOR_MARKER

光标位置标记 —— APC（Application Program Command）序列。
零宽度转义序列，终端会忽略其显示。
组件获得焦点时在光标位置输出此标记，TUI 查找并剔除后将硬件光标定位到该位置。

Cursor position marker - APC (Application Program Command) sequence.
This is a zero-width escape sequence that terminals ignore.
Components emit this at the cursor position when focused.
TUI finds and strips this marker, then positions the hardware cursor there.

| Constant | Type |
| ---------- | ---------- |
| `CURSOR_MARKER` | `"\u001B_pi:c\u0007"` |

### :gear: TUI_KEYBINDINGS

| Constant | Type |
| ---------- | ---------- |
| `TUI_KEYBINDINGS` | `{ readonly "tui.editor.cursorUp": { readonly defaultKeys: "up"; readonly description: "Move cursor up"; }; readonly "tui.editor.cursorDown": { readonly defaultKeys: "down"; readonly description: "Move cursor down"; }; ... 28 more ...; readonly "tui.select.cancel": { ...; }; }` |


## :factory: StdinBuffer

Buffers stdin input and emits complete sequences via the 'data' event.
Handles partial escape sequences that arrive across multiple chunks.

### Methods

- [process](#gear-process)
- [flush](#gear-flush)
- [clear](#gear-clear)
- [getBuffer](#gear-getbuffer)
- [destroy](#gear-destroy)

#### :gear: process

| Method | Type |
| ---------- | ---------- |
| `process` | `(data: string or Buffer<ArrayBufferLike>) => void` |

#### :gear: flush

| Method | Type |
| ---------- | ---------- |
| `flush` | `() => string[]` |

#### :gear: clear

| Method | Type |
| ---------- | ---------- |
| `clear` | `() => void` |

#### :gear: getBuffer

| Method | Type |
| ---------- | ---------- |
| `getBuffer` | `() => string` |

#### :gear: destroy

| Method | Type |
| ---------- | ---------- |
| `destroy` | `() => void` |

## :factory: ProcessTerminal

Real terminal using process.stdin/stdout

### Methods

- [start](#gear-start)
- [stop](#gear-stop)
- [write](#gear-write)
- [moveBy](#gear-moveby)
- [hideCursor](#gear-hidecursor)
- [showCursor](#gear-showcursor)
- [clearLine](#gear-clearline)
- [clearFromCursor](#gear-clearfromcursor)
- [clearScreen](#gear-clearscreen)
- [setTitle](#gear-settitle)
- [setProgress](#gear-setprogress)

#### :gear: start

| Method | Type |
| ---------- | ---------- |
| `start` | `(onInput: (data: string) => void, onResize: () => void) => void` |

#### :gear: stop

| Method | Type |
| ---------- | ---------- |
| `stop` | `() => void` |

#### :gear: write

| Method | Type |
| ---------- | ---------- |
| `write` | `(data: string) => void` |

#### :gear: moveBy

| Method | Type |
| ---------- | ---------- |
| `moveBy` | `(lines: number) => void` |

#### :gear: hideCursor

| Method | Type |
| ---------- | ---------- |
| `hideCursor` | `() => void` |

#### :gear: showCursor

| Method | Type |
| ---------- | ---------- |
| `showCursor` | `() => void` |

#### :gear: clearLine

| Method | Type |
| ---------- | ---------- |
| `clearLine` | `() => void` |

#### :gear: clearFromCursor

| Method | Type |
| ---------- | ---------- |
| `clearFromCursor` | `() => void` |

#### :gear: clearScreen

| Method | Type |
| ---------- | ---------- |
| `clearScreen` | `() => void` |

#### :gear: setTitle

| Method | Type |
| ---------- | ---------- |
| `setTitle` | `(title: string) => void` |

#### :gear: setProgress

| Method | Type |
| ---------- | ---------- |
| `setProgress` | `(active: boolean) => void` |

## :factory: Container

容器组件 —— 可包含子组件的组件。
采用扁平化渲染，将子组件的渲染输出按顺序拼接为一个整体。

Container - a component that contains other components

### Methods

- [addChild](#gear-addchild)
- [removeChild](#gear-removechild)
- [clear](#gear-clear)
- [invalidate](#gear-invalidate)
- [render](#gear-render)

#### :gear: addChild

添加一个子组件到容器中

| Method | Type |
| ---------- | ---------- |
| `addChild` | `(component: Component) => void` |

Parameters:

* `component`: - 要添加的组件


#### :gear: removeChild

从容器中移除指定子组件

| Method | Type |
| ---------- | ---------- |
| `removeChild` | `(component: Component) => void` |

Parameters:

* `component`: - 要移除的组件


#### :gear: clear

清空所有子组件

| Method | Type |
| ---------- | ---------- |
| `clear` | `() => void` |

#### :gear: invalidate

清空所有子组件的缓存渲染状态

| Method | Type |
| ---------- | ---------- |
| `invalidate` | `() => void` |

#### :gear: render

渲染容器：按顺序渲染所有子组件并将输出拼接为单一线条数组

| Method | Type |
| ---------- | ---------- |
| `render` | `(width: number) => string[]` |

Parameters:

* `width`: - 视口宽度


Returns:

所有子组件渲染输出的拼接结果

### Properties

- [children](#gear-children)

#### :gear: children

| Property | Type |
| ---------- | ---------- |
| `children` | `Component[]` |

## :factory: TUI

TUI 引擎 —— 基于差异渲染的终端 UI 核心类。

核心职责：
  - 组件树管理（继承自 Container，可包含子组件）
  - 浮层系统（模态对话框、选择器等叠加组件）
  - 差异渲染（仅重绘变化的行，最小化终端 I/O）
  - 输入分发（将终端原始输入路由到焦点组件或输入监听器）
  - 硬件光标定位（支持 IME 输入法的候选窗口对齐）
  - Kitty 图片协议支持（内嵌图片的渲染与清理）

TUI - Main class for managing terminal UI with differential rendering

### Methods

- [getShowHardwareCursor](#gear-getshowhardwarecursor)
- [setShowHardwareCursor](#gear-setshowhardwarecursor)
- [getClearOnShrink](#gear-getclearonshrink)
- [setClearOnShrink](#gear-setclearonshrink)
- [setFocus](#gear-setfocus)
- [showOverlay](#gear-showoverlay)
- [hideOverlay](#gear-hideoverlay)
- [hasOverlay](#gear-hasoverlay)
- [start](#gear-start)
- [addInputListener](#gear-addinputlistener)
- [removeInputListener](#gear-removeinputlistener)
- [stop](#gear-stop)
- [requestRender](#gear-requestrender)

#### :gear: getShowHardwareCursor

| Method | Type |
| ---------- | ---------- |
| `getShowHardwareCursor` | `() => boolean` |

#### :gear: setShowHardwareCursor

设置是否展示硬件光标
硬件光标用于 IME（输入法）候选窗口的位置对齐

| Method | Type |
| ---------- | ---------- |
| `setShowHardwareCursor` | `(enabled: boolean) => void` |

Parameters:

* `enabled`: - 是否启用


#### :gear: getClearOnShrink

| Method | Type |
| ---------- | ---------- |
| `getClearOnShrink` | `() => boolean` |

#### :gear: setClearOnShrink

设置内容缩减时是否触发全量重绘以清除空白行。
启用时（默认），内容减少时会全量重绘以清除残留的旧行；
禁用时，旧行会保留在终端中（减少重绘次数，适合较慢的终端）。

Set whether to trigger full re-render when content shrinks.
When true (default), empty rows are cleared when content shrinks.
When false, empty rows remain (reduces redraws on slower terminals).

| Method | Type |
| ---------- | ---------- |
| `setClearOnShrink` | `(enabled: boolean) => void` |

#### :gear: setFocus

设置键盘焦点组件
将旧焦点组件的 focused 标志设为 false，新组件的设为 true

| Method | Type |
| ---------- | ---------- |
| `setFocus` | `(component: Component or null) => void` |

Parameters:

* `component`: - 要聚焦的组件，null 表示清除焦点


#### :gear: showOverlay

显示一个浮层组件，支持可配置的定位和尺寸。
返回控制句柄用于管理浮层的可见性和焦点状态。

Show an overlay component with configurable positioning and sizing.
Returns a handle to control the overlay's visibility.

| Method | Type |
| ---------- | ---------- |
| `showOverlay` | `(component: Component, options?: OverlayOptions or undefined) => OverlayHandle` |

#### :gear: hideOverlay

隐藏最顶部的浮层并恢复上一个焦点组件

Hide the topmost overlay and restore previous focus.

| Method | Type |
| ---------- | ---------- |
| `hideOverlay` | `() => void` |

#### :gear: hasOverlay

Check if there are any visible overlays

| Method | Type |
| ---------- | ---------- |
| `hasOverlay` | `() => boolean` |

#### :gear: start

启动 TUI 引擎
流程：启动终端输入监听 -> 隐藏光标 -> 查询终端像素尺寸 -> 请求首帧渲染

| Method | Type |
| ---------- | ---------- |
| `start` | `() => void` |

#### :gear: addInputListener

添加一个输入监听器
监听器在焦点组件之前处理输入，可以消费事件或修改输入数据

| Method | Type |
| ---------- | ---------- |
| `addInputListener` | `(listener: InputListener) => () => void` |

Parameters:

* `listener`: - 输入监听函数


Returns:

用于移除监听器的函数

#### :gear: removeInputListener

移除指定输入监听器

| Method | Type |
| ---------- | ---------- |
| `removeInputListener` | `(listener: InputListener) => void` |

Parameters:

* `listener`: - 要移除的监听函数


#### :gear: stop

停止 TUI 引擎
流程：清除渲染定时器 -> 移动光标到内容末尾（防止退出时覆盖残留内容）-> 显示光标 -> 停止终端

| Method | Type |
| ---------- | ---------- |
| `stop` | `() => void` |

#### :gear: requestRender

请求渲染（异步，通过 process.nextTick 批量处理）。
多次快速连续调用会被合并为一次渲染，受 MIN_RENDER_INTERVAL_MS 帧率限制。

| Method | Type |
| ---------- | ---------- |
| `requestRender` | `(force?: boolean) => void` |

Parameters:

* `force`: - 强制定时全量重绘（清除 previousLines 等状态，触发完整渲染）


### Properties

- [terminal](#gear-terminal)
- [onDebug](#gear-ondebug)

#### :gear: terminal

| Property | Type |
| ---------- | ---------- |
| `terminal` | `Terminal` |

#### :gear: onDebug

Global callback for debug key (Shift+Ctrl+D). Called before input is forwarded to focused component.

| Property | Type |
| ---------- | ---------- |
| `onDebug` | `(() => void) or undefined` |

## :factory: CombinedAutocompleteProvider

### Methods

- [applyCompletion](#gear-applycompletion)
- [shouldTriggerFileCompletion](#gear-shouldtriggerfilecompletion)

#### :gear: applyCompletion

| Method | Type |
| ---------- | ---------- |
| `applyCompletion` | `(lines: string[], cursorLine: number, cursorCol: number, item: AutocompleteItem, prefix: string) => { lines: string[]; cursorLine: number; cursorCol: number; }` |

#### :gear: shouldTriggerFileCompletion

| Method | Type |
| ---------- | ---------- |
| `shouldTriggerFileCompletion` | `(lines: string[], cursorLine: number, cursorCol: number) => boolean` |

## :factory: KeybindingsManager

### Methods

- [matches](#gear-matches)
- [getKeys](#gear-getkeys)
- [getDefinition](#gear-getdefinition)
- [getConflicts](#gear-getconflicts)
- [setUserBindings](#gear-setuserbindings)
- [getUserBindings](#gear-getuserbindings)
- [getResolvedBindings](#gear-getresolvedbindings)

#### :gear: matches

| Method | Type |
| ---------- | ---------- |
| `matches` | `(data: string, keybinding: keyof Keybindings) => boolean` |

#### :gear: getKeys

| Method | Type |
| ---------- | ---------- |
| `getKeys` | `(keybinding: keyof Keybindings) => KeyId[]` |

#### :gear: getDefinition

| Method | Type |
| ---------- | ---------- |
| `getDefinition` | `(keybinding: keyof Keybindings) => KeybindingDefinition` |

#### :gear: getConflicts

| Method | Type |
| ---------- | ---------- |
| `getConflicts` | `() => KeybindingConflict[]` |

#### :gear: setUserBindings

| Method | Type |
| ---------- | ---------- |
| `setUserBindings` | `(userBindings: KeybindingsConfig) => void` |

#### :gear: getUserBindings

| Method | Type |
| ---------- | ---------- |
| `getUserBindings` | `() => KeybindingsConfig` |

#### :gear: getResolvedBindings

| Method | Type |
| ---------- | ---------- |
| `getResolvedBindings` | `() => KeybindingsConfig` |

## :factory: KillRing

### Methods

- [push](#gear-push)
- [peek](#gear-peek)
- [rotate](#gear-rotate)

#### :gear: push

Add text to the kill ring.

| Method | Type |
| ---------- | ---------- |
| `push` | `(text: string, opts: { prepend: boolean; accumulate?: boolean or undefined; }) => void` |

Parameters:

* `text`: - The killed text to add
* `opts`: - Push options
* `opts.prepend`: - If accumulating, prepend (backward deletion) or append (forward deletion)
* `opts.accumulate`: - Merge with the most recent entry instead of creating a new one


#### :gear: peek

Get most recent entry without modifying the ring.

| Method | Type |
| ---------- | ---------- |
| `peek` | `() => string or undefined` |

#### :gear: rotate

Move last entry to front (for yank-pop cycling).

| Method | Type |
| ---------- | ---------- |
| `rotate` | `() => void` |

## :factory: UndoStack

### Methods

- [push](#gear-push)
- [pop](#gear-pop)
- [clear](#gear-clear)

#### :gear: push

Push a deep clone of the given state onto the stack.

| Method | Type |
| ---------- | ---------- |
| `push` | `(state: S) => void` |

#### :gear: pop

Pop and return the most recent snapshot, or undefined if empty.

| Method | Type |
| ---------- | ---------- |
| `pop` | `() => S or undefined` |

#### :gear: clear

Remove all snapshots.

| Method | Type |
| ---------- | ---------- |
| `clear` | `() => void` |

## :factory: SelectList

### Methods

- [setFilter](#gear-setfilter)
- [setSelectedIndex](#gear-setselectedindex)
- [invalidate](#gear-invalidate)
- [render](#gear-render)
- [handleInput](#gear-handleinput)
- [getSelectedItem](#gear-getselecteditem)

#### :gear: setFilter

| Method | Type |
| ---------- | ---------- |
| `setFilter` | `(filter: string) => void` |

#### :gear: setSelectedIndex

| Method | Type |
| ---------- | ---------- |
| `setSelectedIndex` | `(index: number) => void` |

#### :gear: invalidate

清除缓存的渲染状态。主题变更或组件需要从头重绘时由 TUI 调用

Invalidate any cached rendering state.
Called when theme changes or when component needs to re-render from scratch.

| Method | Type |
| ---------- | ---------- |
| `invalidate` | `() => void` |

#### :gear: render

将组件渲染为指定视口宽度的文本行数组

| Method | Type |
| ---------- | ---------- |
| `render` | `(width: number) => string[]` |

#### :gear: handleInput

可选的键盘输入处理器，组件获得焦点时由 TUI 调用

Optional handler for keyboard input when component has focus

| Method | Type |
| ---------- | ---------- |
| `handleInput` | `(keyData: string) => void` |

#### :gear: getSelectedItem

| Method | Type |
| ---------- | ---------- |
| `getSelectedItem` | `() => SelectItem or null` |

### Properties

- [onSelect](#gear-onselect)
- [onCancel](#gear-oncancel)
- [onSelectionChange](#gear-onselectionchange)

#### :gear: onSelect

| Property | Type |
| ---------- | ---------- |
| `onSelect` | `((item: SelectItem) => void) or undefined` |

#### :gear: onCancel

| Property | Type |
| ---------- | ---------- |
| `onCancel` | `(() => void) or undefined` |

#### :gear: onSelectionChange

| Property | Type |
| ---------- | ---------- |
| `onSelectionChange` | `((item: SelectItem) => void) or undefined` |

## :factory: Editor

编辑器组件 — TUI 引擎的多行文本编辑器。

实现 Component 和 Focusable 接口，由交互模式的 InputArea 容器组件实例化。
内部将逻辑行（EditorState.lines[]）通过 wordWrapLine 展开为视觉行布局，
再计算滚动窗口和光标位置后输出 ANSI 转义序列。

输入处理 handleInput() 是事件分发的中央枢纽：
  按键 → 键绑定匹配 → 分支到光标移动 / 编辑操作 / 补全 / 提交。

支持的类型前缀补全通道：
  - "/" → 斜杠命令（首行有效）
  - "@" / "#" → 附件/文件符号补全（带防抖）
  - Tab → 强制触发文件路径补全

### Constructors

`public`: 构造编辑器实例。

Parameters:

* `tui`: - TUI 引擎实例，用于 requestRender 和 terminal 尺寸查询
* `theme`: - 编辑器主题（边框颜色 + SelectList 子主题）
* `options`: - 可选配置：paddingX（水平内边距）、autocompleteMaxVisible（补全列表最大可见条目数）


### Methods

- [getPaddingX](#gear-getpaddingx)
- [setPaddingX](#gear-setpaddingx)
- [getAutocompleteMaxVisible](#gear-getautocompletemaxvisible)
- [setAutocompleteMaxVisible](#gear-setautocompletemaxvisible)
- [setAutocompleteProvider](#gear-setautocompleteprovider)
- [addToHistory](#gear-addtohistory)
- [invalidate](#gear-invalidate)
- [render](#gear-render)
- [handleInput](#gear-handleinput)
- [getText](#gear-gettext)
- [getExpandedText](#gear-getexpandedtext)
- [getLines](#gear-getlines)
- [getCursor](#gear-getcursor)
- [setText](#gear-settext)
- [insertTextAtCursor](#gear-inserttextatcursor)
- [isShowingAutocomplete](#gear-isshowingautocomplete)

#### :gear: getPaddingX

| Method | Type |
| ---------- | ---------- |
| `getPaddingX` | `() => number` |

#### :gear: setPaddingX

| Method | Type |
| ---------- | ---------- |
| `setPaddingX` | `(padding: number) => void` |

#### :gear: getAutocompleteMaxVisible

| Method | Type |
| ---------- | ---------- |
| `getAutocompleteMaxVisible` | `() => number` |

#### :gear: setAutocompleteMaxVisible

| Method | Type |
| ---------- | ---------- |
| `setAutocompleteMaxVisible` | `(maxVisible: number) => void` |

#### :gear: setAutocompleteProvider

| Method | Type |
| ---------- | ---------- |
| `setAutocompleteProvider` | `(provider: AutocompleteProvider) => void` |

#### :gear: addToHistory

Add a prompt to history for up/down arrow navigation.
Called after successful submission.

| Method | Type |
| ---------- | ---------- |
| `addToHistory` | `(text: string) => void` |

#### :gear: invalidate

清除缓存的渲染状态。主题变更或组件需要从头重绘时由 TUI 调用

Invalidate any cached rendering state.
Called when theme changes or when component needs to re-render from scratch.

| Method | Type |
| ---------- | ---------- |
| `invalidate` | `() => void` |

#### :gear: render

渲染编辑器为 ANSI 转义序列字符串数组。

渲染流程：
  1. 计算内容宽度（width - 2 * paddingX），为无内边距模式预留 1 列给光标
  2. wordWrapLine 将逻辑行展开为 LayoutLine[]（layoutText）
  3. 根据光标所在视觉行调整 scrollOffset，保持光标在可见区域内
  4. 遍历可见的视觉行切片，在光标行插入反向颜色 ANSI 高亮
  5. 顶部/底部分别渲染"↑ N more" / "↓ N more"滚动指示器
  6. 如果补全列表激活，追加补全行

光标渲染策略：
  - 光标在字符上 → 用 \x1b[7m 反向高亮该 grapheme
  - 光标在行尾 → 高亮一个空格（反向颜色块）
  - 光标溢出到内边距 → 截断右侧内边距 1 字符
  - 失焦或补全显示中 → 不发射 CURSOR_MARKER

| Method | Type |
| ---------- | ---------- |
| `render` | `(width: number) => string[]` |

Parameters:

* `width`: - 终端可用宽度（列数）


Returns:

ANSI 转义字符串数组，每行一个元素

#### :gear: handleInput

处理终端原始输入 — 编辑器的输入事件中央枢纽。

输入分发流程（按优先级）：
  1. 字符跳转模式待命中 — 匹配下一个字符执行跳转
  2. 括弧粘贴模式缓冲 — 累积到结束标记后调用 handlePaste
  3. Ctrl+C — 透传给父组件
  4. 撤销 (Ctrl+/) — undo()
  5. 补全列表激活中 — 上下选择 / Tab确认 / Escape取消
  6. Tab — 触发补全 (handleTabCompletion)
  7. 删除操作 — deleteToLineEnd/deleteToLineStart/deleteWord/deleteChar
  8. Kill Ring — yank(Ctrl+Y) / yankPop(Alt+Y)
  9. 光标移动 — cursorUp/Down/Left/Right / cursorWord / cursorLine
  10. 换行 — Enter / Shift+Enter / Ctrl+Enter
  11. 提交 — Enter 匹配 submit 键绑定时
  12. 翻页 — PageUp / PageDown
  13. 字符跳转触发 — Ctrl+F / Ctrl+B
  14. Shift+Space — 插入空格
  15. 可打印字符 — insertCharacter()

| Method | Type |
| ---------- | ---------- |
| `handleInput` | `(data: string) => void` |

Parameters:

* `data`: - 终端发送的原始输入序列（ANSI 转义序列或 UTF-8 文本）


#### :gear: getText

获取当前文本（不含粘贴标记展开）。
粘贴标记保持为 [paste #N ...] 占位符，适合 UI 展示。
需要完整内容时使用 getExpandedText()。

| Method | Type |
| ---------- | ---------- |
| `getText` | `() => string` |

Returns:

逻辑行以 \n 连接的文本

#### :gear: getExpandedText

Get text with paste markers expanded to their actual content.
Use this when you need the full content (e.g., for external editor).

| Method | Type |
| ---------- | ---------- |
| `getExpandedText` | `() => string` |

#### :gear: getLines

| Method | Type |
| ---------- | ---------- |
| `getLines` | `() => string[]` |

#### :gear: getCursor

| Method | Type |
| ---------- | ---------- |
| `getCursor` | `() => { line: number; col: number; }` |

#### :gear: setText

程序化设置编辑器文本。

取消补全、退出历史浏览、规范化行尾和 Tab → 4 空格。
如果内容发生变化则 pushUndoSnapshot，使程序化变更也可撤销。

| Method | Type |
| ---------- | ---------- |
| `setText` | `(text: string) => void` |

Parameters:

* `text`: - 新文本内容


#### :gear: insertTextAtCursor

Insert text at the current cursor position.
Used for programmatic insertion (e.g., clipboard image markers).
This is atomic for undo - single undo restores entire pre-insert state.

| Method | Type |
| ---------- | ---------- |
| `insertTextAtCursor` | `(text: string) => void` |

#### :gear: isShowingAutocomplete

| Method | Type |
| ---------- | ---------- |
| `isShowingAutocomplete` | `() => boolean` |

### Properties

- [focused](#gear-focused)
- [borderColor](#gear-bordercolor)
- [onSubmit](#gear-onsubmit)
- [onChange](#gear-onchange)
- [disableSubmit](#gear-disablesubmit)

#### :gear: focused

Focusable 接口 —— TUI 引擎在焦点变更时设置此标志

| Property | Type |
| ---------- | ---------- |
| `focused` | `boolean` |

#### :gear: borderColor

| Property | Type |
| ---------- | ---------- |
| `borderColor` | `(str: string) => string` |

#### :gear: onSubmit

提交回调 —— 用户按 Enter 时调用，传入展开粘贴标记后的文本

| Property | Type |
| ---------- | ---------- |
| `onSubmit` | `((text: string) => void) or undefined` |

#### :gear: onChange

变更回调 —— 文本每次修改后调用（包括撤销/重做/历史导航），传入当前文本

| Property | Type |
| ---------- | ---------- |
| `onChange` | `((text: string) => void) or undefined` |

#### :gear: disableSubmit

禁用提交 —— 为 true 时忽略 Enter 键（如 Agent 运行中防止误提交）

| Property | Type |
| ---------- | ---------- |
| `disableSubmit` | `boolean` |

## :factory: Markdown

### Methods

- [setText](#gear-settext)
- [invalidate](#gear-invalidate)
- [render](#gear-render)

#### :gear: setText

| Method | Type |
| ---------- | ---------- |
| `setText` | `(text: string) => void` |

#### :gear: invalidate

清除缓存的渲染状态。主题变更或组件需要从头重绘时由 TUI 调用

Invalidate any cached rendering state.
Called when theme changes or when component needs to re-render from scratch.

| Method | Type |
| ---------- | ---------- |
| `invalidate` | `() => void` |

#### :gear: render

将组件渲染为指定视口宽度的文本行数组

| Method | Type |
| ---------- | ---------- |
| `render` | `(width: number) => string[]` |

## :tropical_drink: Interfaces

- [Terminal](#gear-terminal)
- [TerminalCapabilities](#gear-terminalcapabilities)
- [CellDimensions](#gear-celldimensions)
- [ImageDimensions](#gear-imagedimensions)
- [ImageRenderOptions](#gear-imagerenderoptions)
- [Component](#gear-component)
- [Focusable](#gear-focusable)
- [OverlayMargin](#gear-overlaymargin)
- [OverlayOptions](#gear-overlayoptions)
- [OverlayHandle](#gear-overlayhandle)
- [FuzzyMatch](#gear-fuzzymatch)
- [AutocompleteItem](#gear-autocompleteitem)
- [SlashCommand](#gear-slashcommand)
- [AutocompleteSuggestions](#gear-autocompletesuggestions)
- [AutocompleteProvider](#gear-autocompleteprovider)
- [Keybindings](#gear-keybindings)
- [KeybindingDefinition](#gear-keybindingdefinition)
- [KeybindingConflict](#gear-keybindingconflict)
- [SelectItem](#gear-selectitem)
- [SelectListTheme](#gear-selectlisttheme)
- [SelectListTruncatePrimaryContext](#gear-selectlisttruncateprimarycontext)
- [SelectListLayoutOptions](#gear-selectlistlayoutoptions)
- [TextChunk](#gear-textchunk)
- [EditorTheme](#gear-editortheme)
- [EditorOptions](#gear-editoroptions)
- [DefaultTextStyle](#gear-defaulttextstyle)
- [MarkdownTheme](#gear-markdowntheme)

### :gear: Terminal

Minimal terminal interface for TUI

| Property | Type | Description |
| ---------- | ---------- | ---------- |


### :gear: TerminalCapabilities



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `images` | `ImageProtocol` |  |
| `trueColor` | `boolean` |  |
| `hyperlinks` | `boolean` |  |


### :gear: CellDimensions



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `widthPx` | `number` |  |
| `heightPx` | `number` |  |


### :gear: ImageDimensions



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `widthPx` | `number` |  |
| `heightPx` | `number` |  |


### :gear: ImageRenderOptions



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `maxWidthCells` | `number or undefined` |  |
| `maxHeightCells` | `number or undefined` |  |
| `preserveAspectRatio` | `boolean or undefined` |  |
| `imageId` | `number or undefined` | Kitty image ID. If provided, reuses/replaces existing image with this ID. |
| `moveCursor` | `boolean or undefined` | Whether Kitty should apply its default cursor movement after placement. |


### :gear: Component

组件接口 —— TUI 渲染的基本单元，所有 UI 组件必须实现此接口。
提供三个核心方法：render（渲染为文本行）、handleInput（处理键盘输入）、invalidate（清除缓存）。

Component interface - all components must implement this

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `wantsKeyRelease` | `boolean or undefined` | 是否接收按键释放事件（Kitty 协议）。默认为 false，TUI 会过滤掉释放事件  If true, component receives key release events (Kitty protocol). Default is false - release events are filtered out. |


### :gear: Focusable

可获取焦点的组件接口。
组件获得焦点时，应在渲染输出中输出 CURSOR_MARKER 标记光标位置。
TUI 会找到此标记并将硬件光标定位到该位置，以便输入法候选窗口正确对齐。

Interface for components that can receive focus and display a hardware cursor.
When focused, the component should emit CURSOR_MARKER at the cursor position
in its render output. TUI will find this marker and position the hardware
cursor there for proper IME candidate window positioning.

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `focused` | `boolean` | Set by TUI when focus changes. Component should emit CURSOR_MARKER when true. |


### :gear: OverlayMargin

浮层边距配置，控制浮层与终端边缘的距离
Margin configuration for overlays

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `top` | `number or undefined` |  |
| `right` | `number or undefined` |  |
| `bottom` | `number or undefined` |  |
| `left` | `number or undefined` |  |


### :gear: OverlayOptions

浮层定位和尺寸配置选项。
尺寸和位置均支持绝对值和百分比字符串（如 "50%"）。

Options for overlay positioning and sizing.
Values can be absolute numbers or percentage strings (e.g., "50%").

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `width` | `SizeValue or undefined` | Width in columns, or percentage of terminal width (e.g., "50%") |
| `minWidth` | `number or undefined` | Minimum width in columns |
| `maxHeight` | `SizeValue or undefined` | Maximum height in rows, or percentage of terminal height (e.g., "50%") |
| `anchor` | `OverlayAnchor or undefined` | Anchor point for positioning (default: 'center') |
| `offsetX` | `number or undefined` | Horizontal offset from anchor position (positive = right) |
| `offsetY` | `number or undefined` | Vertical offset from anchor position (positive = down) |
| `row` | `SizeValue or undefined` | Row position: absolute number, or percentage (e.g., "25%" = 25% from top) |
| `col` | `SizeValue or undefined` | Column position: absolute number, or percentage (e.g., "50%" = centered horizontally) |
| `margin` | `number or OverlayMargin or undefined` | Margin from terminal edges. Number applies to all sides. |
| `visible` | `((termWidth: number, termHeight: number) => boolean) or undefined` | 根据终端尺寸控制浮层的可见性。 提供此函数时，仅当返回 true 时浮层才显示。 每次渲染周期以当前终端尺寸调用。  Control overlay visibility based on terminal dimensions. If provided, overlay is only rendered when this returns true. Called each render cycle with current terminal dimensions. |
| `nonCapturing` | `boolean or undefined` | If true, don't capture keyboard focus when shown |


### :gear: OverlayHandle

showOverlay 返回的浮层控制句柄，用于控制浮层的显示/隐藏、焦点管理

Handle returned by showOverlay for controlling the overlay

| Property | Type | Description |
| ---------- | ---------- | ---------- |


### :gear: FuzzyMatch



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `matches` | `boolean` |  |
| `score` | `number` |  |


### :gear: AutocompleteItem



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `value` | `string` |  |
| `label` | `string` |  |
| `description` | `string or undefined` |  |


### :gear: SlashCommand



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `name` | `string` |  |
| `description` | `string or undefined` |  |
| `argumentHint` | `string or undefined` |  |


### :gear: AutocompleteSuggestions



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `items` | `AutocompleteItem[]` |  |
| `prefix` | `string` |  |


### :gear: AutocompleteProvider



| Property | Type | Description |
| ---------- | ---------- | ---------- |


### :gear: Keybindings

Global keybinding registry.
Downstream packages can add keybindings via declaration merging.

| Property | Type | Description |
| ---------- | ---------- | ---------- |


### :gear: KeybindingDefinition



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `defaultKeys` | `KeyId or KeyId[]` |  |
| `description` | `string or undefined` |  |


### :gear: KeybindingConflict



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `key` | `KeyId` |  |
| `keybindings` | `string[]` |  |


### :gear: SelectItem



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `value` | `string` |  |
| `label` | `string` |  |
| `description` | `string or undefined` |  |


### :gear: SelectListTheme



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `selectedPrefix` | `(text: string) => string` |  |
| `selectedText` | `(text: string) => string` |  |
| `description` | `(text: string) => string` |  |
| `scrollInfo` | `(text: string) => string` |  |
| `noMatch` | `(text: string) => string` |  |


### :gear: SelectListTruncatePrimaryContext



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `text` | `string` |  |
| `maxWidth` | `number` |  |
| `columnWidth` | `number` |  |
| `item` | `SelectItem` |  |
| `isSelected` | `boolean` |  |


### :gear: SelectListLayoutOptions



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `minPrimaryColumnWidth` | `number or undefined` |  |
| `maxPrimaryColumnWidth` | `number or undefined` |  |
| `truncatePrimary` | `((context: SelectListTruncatePrimaryContext) => string) or undefined` |  |


### :gear: TextChunk

Represents a chunk of text for word-wrap layout.
Tracks both the text content and its position in the original line.

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `text` | `string` |  |
| `startIndex` | `number` |  |
| `endIndex` | `number` |  |


### :gear: EditorTheme



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `borderColor` | `(str: string) => string` |  |
| `selectList` | `SelectListTheme` |  |


### :gear: EditorOptions



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `paddingX` | `number or undefined` |  |
| `autocompleteMaxVisible` | `number or undefined` |  |


### :gear: DefaultTextStyle

Default text styling for markdown content.
Applied to all text unless overridden by markdown formatting.

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `color` | `((text: string) => string) or undefined` | Foreground color function |
| `bgColor` | `((text: string) => string) or undefined` | Background color function |
| `bold` | `boolean or undefined` | Bold text |
| `italic` | `boolean or undefined` | Italic text |
| `strikethrough` | `boolean or undefined` | Strikethrough text |
| `underline` | `boolean or undefined` | Underline text |


### :gear: MarkdownTheme

Theme functions for markdown elements.
Each function takes text and returns styled text with ANSI codes.

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `heading` | `(text: string) => string` |  |
| `link` | `(text: string) => string` |  |
| `linkUrl` | `(text: string) => string` |  |
| `code` | `(text: string) => string` |  |
| `codeBlock` | `(text: string) => string` |  |
| `codeBlockBorder` | `(text: string) => string` |  |
| `quote` | `(text: string) => string` |  |
| `quoteBorder` | `(text: string) => string` |  |
| `hr` | `(text: string) => string` |  |
| `listBullet` | `(text: string) => string` |  |
| `bold` | `(text: string) => string` |  |
| `italic` | `(text: string) => string` |  |
| `strikethrough` | `(text: string) => string` |  |
| `underline` | `(text: string) => string` |  |
| `highlightCode` | `((code: string, lang?: string or undefined) => string[]) or undefined` |  |
| `codeBlockIndent` | `string or undefined` | Prefix applied to each rendered code block line (default: "  ") |


## :cocktail: Types

- [KeyId](#gear-keyid)
- [KeyEventType](#gear-keyeventtype)
- [StdinBufferOptions](#gear-stdinbufferoptions)
- [StdinBufferEventMap](#gear-stdinbuffereventmap)
- [ImageProtocol](#gear-imageprotocol)
- [OverlayAnchor](#gear-overlayanchor)
- [SizeValue](#gear-sizevalue)
- [Keybinding](#gear-keybinding)
- [KeybindingDefinitions](#gear-keybindingdefinitions)
- [KeybindingsConfig](#gear-keybindingsconfig)

### :gear: KeyId

Union type of all valid key identifiers.
Provides autocomplete and catches typos at compile time.

| Type | Type |
| ---------- | ---------- |
| `KeyId` | `BaseKey or ModifiedKeyId<BaseKey>` |

### :gear: KeyEventType

Event types from Kitty keyboard protocol (flag 2)
1 = key press, 2 = key repeat, 3 = key release

| Type | Type |
| ---------- | ---------- |
| `KeyEventType` | `press" or "repeat" or "release` |

### :gear: StdinBufferOptions

| Type | Type |
| ---------- | ---------- |
| `StdinBufferOptions` | `{ /** * Maximum time to wait for sequence completion (default: 10ms) * After this time, the buffer is flushed even if incomplete */ timeout?: number; }` |

### :gear: StdinBufferEventMap

| Type | Type |
| ---------- | ---------- |
| `StdinBufferEventMap` | `{ data: [string]; paste: [string]; }` |

### :gear: ImageProtocol

| Type | Type |
| ---------- | ---------- |
| `ImageProtocol` | `kitty" or "iterm2" or null` |

### :gear: OverlayAnchor

浮层定位锚点，决定浮层组件在终端中的对齐方式
Anchor position for overlays

| Type | Type |
| ---------- | ---------- |
| `OverlayAnchor` | `| "center" or "top-left" or "top-right" or "bottom-left" or "bottom-right" or "top-center" or "bottom-center" or "left-center" or "right-center` |

### :gear: SizeValue

Value that can be absolute (number) or percentage (string like "50%")

| Type | Type |
| ---------- | ---------- |
| `SizeValue` | `number or `${number}%`` |

### :gear: Keybinding

| Type | Type |
| ---------- | ---------- |
| `Keybinding` | `keyof Keybindings` |

### :gear: KeybindingDefinitions

| Type | Type |
| ---------- | ---------- |
| `KeybindingDefinitions` | `Record<string, KeybindingDefinition>` |

### :gear: KeybindingsConfig

| Type | Type |
| ---------- | ---------- |
| `KeybindingsConfig` | `Record<string, KeyId or KeyId[] or undefined>` |

