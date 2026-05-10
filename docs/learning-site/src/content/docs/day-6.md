---
title: "Day 6: 工具执行"
---

**学习目标**：理解 LLM 请求的工具调用如何被准备、校验、执行

## 阅读清单

| 顺序 | 文件 | 关键行 | 要理解的概念 | 术语 |
|------|------|--------|-------------|------|
| 1 | `packages/agent/src/agent-loop.ts:424` | `executeToolCalls()` | 并行/顺序分叉逻辑 | 工具执行模式 |
| 2 | `packages/agent/src/agent-loop.ts:452` | `executeToolCallsSequential()` | 顺序执行：逐个准备、执行、完成 | 工具钩子 |
| 3 | `packages/agent/src/agent-loop.ts:512` | `executeToolCallsParallel()` | 三阶段并行执行 | 工具钩子 |
| 4 | `packages/agent/src/agent-loop.ts:643` | `prepareToolCall()` | 查找、预处理、校验、beforeToolCall | 工具钩子 |
| 5 | `packages/agent/src/agent-loop.ts:701` | `executePreparedToolCall()` | 实际执行工具函数 | 工具钩子 |
| 6 | `packages/agent/src/agent-loop.ts:744` | `finalizeExecutedToolCall()` | afterToolCall 钩子应用 | 提前终止 |

## 执行模式

**顺序模式**：逐个调用 `prepare → execute → finalize`，适用于工具间有依赖关系的场景。后续工具的输入可以依赖前一个工具的结果。

**并行模式**：三阶段分为：
1. 准备阶段：为所有工具调用并行执行 `prepare`
2. 执行阶段：所有准备好的工具并行执行
3. 完成阶段：为每个结果依次调用 `finalize`

## 工具钩子

每个工具执行经历 4 个钩子：
- `beforeToolCall` — 预处理参数、添加额外上下文
- `executePreparedToolCall` — 实际调用工具函数
- `afterToolCall` — 后处理结果、可能的提前终止

## 验证

```bash
npx tsx packages/coding-agent/examples/learning/04-agent-loop-trace.ts
```
