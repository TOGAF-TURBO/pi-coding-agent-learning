---
title: "Day 8: 会话管理"
---

**学习目标**：理解会话的持久化、分支和恢复

## 阅读清单

| 顺序 | 文件 | 关键行 | 要理解的概念 | 术语 |
|------|------|--------|-------------|------|
| 1 | `packages/coding-agent/src/core/session-manager.ts:1` | `SessionEntry` 类型 | JSONL 文件的数据结构 | JSONL 会话文件 |
| 2 | `packages/coding-agent/src/core/session-manager.ts:200` | `buildContext()` | 从 JSONL 树重建消息历史 | 会话上下文重建 |
| 3 | `packages/coding-agent/src/core/session-manager.ts:400` | `appendMessage()` | 新消息如何追加到 JSONL | JSONL 会话文件 |
| 4 | `packages/coding-agent/src/core/session-manager.ts:600` | 分支导航方法 | fork/switch/clone | 会话恢复 |
| 5 | `packages/coding-agent/src/core/sdk.ts:236` | `createAgentSession()` | 整个会话组装的入口 | 零必填初始化 |

## 核心概念

**JSONL 存储格式**：会话以 JSONL（每行一个 JSON 对象）文件持久化。每行是一个 `SessionEntry`，包含消息本身和树结构元数据（parent、children 引用）。

**上下文重建**：`buildContext()` 从 JSONL 文件的树结构中重建完整的消息历史。通过追踪 parent 链向上遍历，构建出当前分支的线性消息序列。

**会话分支**：pi 支持消息级别的分支（非文件系统级别）。每个消息可以有多个子消息（多个对话分叉），在树的不同分支间自由切换。

## 会话操作

- **fork**：从当前消息创建新分支
- **switch**：在已有分支间切换
- **clone**：复制整个会话树
