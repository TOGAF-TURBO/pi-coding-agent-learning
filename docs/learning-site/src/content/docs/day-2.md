---
title: "Day 2: 流式调用入口"
---

**学习目标**：理解 LLM 调用的四个入口函数

## 阅读清单

| 顺序 | 文件 | 关键行 | 要理解的概念 |
|------|------|--------|-------------|
| 1 | `packages/ai/src/stream.ts:1` | 完整文件 | `stream()`、`complete()`、`streamSimple()`、`completeSimple()` 四兄弟 |
| 2 | `packages/ai/src/stream.ts:41` | `resolveApiProvider()` | 如何根据 model.api 分发到具体 Provider |
| 3 | `packages/ai/src/api-registry.ts:30` | `registerApiProvider()` | Provider 如何注册到全局注册表 |

## 关键问题

在阅读过程中思考以下问题：

1. `stream()` 和 `complete()` 的区别是什么？
2. `streamSimple()` 比 `stream()` 多了什么处理？
3. 为什么 `resolveApiProvider()` 失败时抛错，而不是返回 undefined？

## 核心概念

`stream()` 返回一个 `AsyncGenerator`，逐步产出 `AssistantMessageEvent`；而 `complete()` 消费整个流后返回完整的 `AssistantMessage`。

`streamSimple()` 在 `stream()` 基础上提供了简化的输入格式——接受字符串、单条消息或消息数组，统一归一化为消息列表。

`resolveApiProvider()` 从全局注册表中查找与 model.api 匹配的 provider 实现，失败时直接抛错以尽早暴露配置问题。
