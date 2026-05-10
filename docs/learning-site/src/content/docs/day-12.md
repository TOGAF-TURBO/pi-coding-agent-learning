---
title: "Day 12: 编辑器组件"
---

**学习目标**：理解多行文本编辑器的内部机制

## 阅读清单

| 顺序 | 文件 | 关键行 | 要理解的概念 | 术语 |
|------|------|--------|-------------|------|
| 1 | `packages/tui/src/components/editor.ts:1` | 构造函数、字段定义 | 编辑器的状态模型 | 编辑器组件 |
| 2 | `packages/tui/src/components/editor.ts:200` | `handleInput()` | 输入处理的分发逻辑 | 编辑器组件 |
| 3 | `packages/tui/src/components/editor.ts:400` | 光标移动系统 | `moveToVisualLine`、`computeVerticalMoveColumn` | 编辑器组件 |
| 4 | `packages/tui/src/components/editor.ts:800` | 撤销/重做 | undo stack 的实现 | 编辑器组件 |
| 5 | `packages/tui/src/components/editor.ts:1200` | 自动补全 | autocomplete 子系统 | 编辑器组件 |
| 6 | `packages/tui/src/components/editor.ts:1600` | Kill Ring | 类 Emacs 剪切历史 | 编辑器组件 |

## 状态模型

编辑器维护一个精心设计的状态模型：

- **buffer**：当前文本内容，按行存储
- **cursor**：光标位置（行 + 列），支持视觉行和物理行的区分
- **selection**：选中范围（起止位置）
- **undo stack**：操作历史栈，记录每步的可逆操作
- **kill ring**：剪切历史环，类似 Emacs 的 kill-ring 机制

## 核心子系统

**光标系统**：区分物理行（buffer 中的行）和视觉行（屏幕上折行后的行），`moveToVisualLine` 处理折行下的光标移动。

**撤销/重做**：基于操作栈的 undo 实现。每个操作都有正向和逆向 delta，撤销就是应用逆向 delta。

**自动补全**：监听输入事件，触发补全查询，在下拉框中展示候选项。

**Kill Ring**：类 Emacs 的剪切历史机制，最近剪切的文本插入到环头部，支持 `Ctrl+Y`（yank）和 `Alt+Y`（yank-pop）操作。
