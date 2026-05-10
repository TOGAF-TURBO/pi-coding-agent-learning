---
title: "Day 4: Agent 主类"
---

**学习目标**：理解 Agent 的有状态封装

## 阅读清单

| 顺序 | 文件 | 关键行 | 要理解的概念 | 术语 |
|------|------|--------|-------------|------|
| 1 | `packages/agent/src/types.ts:1` | 完整文件 | AgentState、AgentEvent、AgentTool 类型定义 | Agent 状态 |
| 2 | `packages/agent/src/agent.ts:50` | `defaultConvertToLlm()` | 默认的消息转换函数 | 消息转换 |
| 3 | `packages/agent/src/agent.ts:97` | `createMutableAgentState()` | 状态对象如何保持不可变性 | Agent 状态 |
| 4 | `packages/agent/src/agent.ts:127` | `AgentOptions` 接口 | Agent 有哪些可配置维度 | 零必填初始化 |
| 5 | `packages/agent/src/agent.ts:162` | `Agent` 类构造函数 | Agent 如何绑定状态和配置 | Agent 循环 |
| 6 | `packages/agent/src/agent.ts:395` | `prompt()` 方法 | 发起对话的入口 | 转向消息 |
| 7 | `packages/agent/src/agent.ts:468` | `runPromptMessages()` | 如何归一化输入并启动循环 | Agent 循环 |

## 核心概念

`AgentOptions` 采用零必填初始化——所有字段都有合理默认值，用户可以只传需要覆盖的选项。

`AgentState` 包含对话历史（messages）、当前模型（model）、会话元数据等，通过 `createMutableAgentState()` 维护不可变性——每次状态变更都创建新对象副本。

`prompt()` 是用户与 Agent 交互的唯一入口，接收转向消息（steering）和可选的后续消息，然后委托给 `runPromptMessages()` 启动底层循环。

## 验证

```bash
npx tsx packages/coding-agent/examples/learning/03-streaming-events.ts
```
