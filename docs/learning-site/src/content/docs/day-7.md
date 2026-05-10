---
title: "Day 7: 扩展系统"
---

**学习目标**：理解插件如何注册工具、命令、事件处理器

## 阅读清单

| 顺序 | 文件 | 关键行 | 要理解的概念 |
|------|------|--------|-------------|
| 1 | `packages/coding-agent/src/core/extensions/types.ts:1` | `ExtensionAPI` 接口 | 扩展可以调用哪些 API |
| 2 | `packages/coding-agent/src/core/extensions/types.ts:100` | 事件类型列表（31 种） | 扩展可以监听哪些系统事件 |
| 3 | `packages/coding-agent/src/core/extensions/types.ts:200` | `ToolDefinition` 接口 | 自定义工具的定义结构 |
| 4 | `packages/coding-agent/src/core/extensions/types.ts:350` | `ExtensionContext` 接口 | 扩展的运行时上下文 |
| 5 | `packages/coding-agent/src/core/extensions/loader.ts:1` | `ExtensionLoader` 类 | 如何用 jiti 动态加载 .ts 扩展 |

## 核心概念

`ExtensionAPI` 是扩展与系统交互的唯一接口，提供方法如 `registerTool()`、`registerCommand()`、`on()`（事件监听）。

扩展通过 `ExtensionContext` 获取当前会话的上下文信息——包括当前消息历史、模型、工作目录等。

事件类型列表（31 种）覆盖了 Agent 生命周期中的关键节点：`beforeToolCall`、`afterToolCall`、`beforeCompaction`、`afterCompaction` 等等。

`ExtensionLoader` 使用 jiti（运行时 TypeScript 编译器）动态加载 `.ts` 扩展文件，无需预编译。

## 实际应用

扩展系统是 pi 最强大的定制化能力。通过编写扩展，你可以：
- 添加自定义工具（如数据库查询、API 调用）
- 添加 `/` 指令（自定义命令）
- 在所有钩子点注入逻辑（日志、监控、安全检查）
