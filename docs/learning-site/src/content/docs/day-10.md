---
title: "Day 10: 消息转换与 Provider 适配"
---

**学习目标**：理解消息在不同 Provider 间的转换逻辑

## 阅读清单

| 顺序 | 文件 | 关键行 | 要理解的概念 | 术语 |
|------|------|--------|-------------|------|
| 1 | `packages/ai/src/providers/transform-messages.ts:1` | 图片降级、孤儿修复、ID 规范化 | 跨 Provider 消息兼容的三道工序 | 消息转换 |
| 2 | `packages/ai/src/providers/openai-completions.ts:930` | `convertMessages()` | AgentMessage → OpenAI 格式 | 协议适配链 |
| 3 | `packages/ai/src/providers/openai-completions.ts:1190` | `convertTools()` | Tool → OpenAI function 格式 | 协议适配链 |
| 4 | `packages/ai/src/providers/openai-completions.ts:1307` | `detectCompat()` | 30+ Provider 的自动差异检测 | 兼容性设置 |

## 三道兼容工序

在消息到达具体 Provider 之前，`transform-messages.ts` 会经历三道工序：

1. **图片降级**：将不支持的图片格式（如 image/jpeg 原始字节）转换为目标 Provider 接受的格式（如 base64 data URI）
2. **孤儿修复**：对接没有对应 tool_use 的 tool_result（或反之），补全缺失的消息
3. **ID 规范化**：统一 tool_call_id / tool_use_id 的格式，确保跨 Provider 可追踪

## 协议适配链

```
AgentMessage → transform-messages → convertMessages() → Provider 原生格式
```

`detectCompat()` 函数是精髓所在——它通过一系列检测（如是否支持 `parallel_tool_calls`、是否支持 `stream_options`、是否支持 `response_format`）自动推断 Provider 的兼容能力，避免手动配置 30+ Provider 的差异。
