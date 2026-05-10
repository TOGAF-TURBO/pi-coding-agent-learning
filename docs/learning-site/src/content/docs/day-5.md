---
title: "Day 5: Agent 循环核心"
---

**学习目标**：理解双层 while-true 循环的结构

## 阅读清单

| 顺序 | 文件 | 关键行 | 要理解的概念 |
|------|------|--------|-------------|
| 1 | `packages/agent/src/agent-loop.ts:128` | `runAgentLoop()` | 如何从 prompt 消息启动循环 |
| 2 | `packages/agent/src/agent-loop.ts:208` | `runLoop()` | 双层循环结构（内层工具调用 + 外层后续消息） |
| 3 | `packages/agent/src/agent-loop.ts:319` | `streamAssistantResponse()` | LLM 调用的桥梁函数 |
| 4 | `packages/agent/src/agent-loop.ts:327` | `transformContext()` | 上下文变换钩子位置 |
| 5 | `packages/agent/src/agent-loop.ts:333` | `convertToLlm()` | 消息格式转换钩子位置 |

## 双层循环结构

**外层循环**（follow-up 循环）：处理后续消息列表，每次迭代取一条消息作为本轮转向。

**内层循环**（tool-call 循环）：在一次对话回合中反复调用 LLM，直到 LLM 不再请求工具调用为止。

```
while (有后续消息) {          ← 外层
    while (LLM 请求工具调用) {  ← 内层
        执行工具 → 把结果发给 LLM
    }
}
```

## 关键问题

1. 转向消息（steering）和后续消息（follow-up）的差异是什么？
2. 什么条件下内层循环退出？什么条件下外层循环退出？
3. `shouldStopAfterTurn` 钩子在什么时机触发？
