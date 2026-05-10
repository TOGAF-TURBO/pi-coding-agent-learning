---
title: "Day 14: Provider 实现全景对比"
---

**学习目标**：理解不同 Provider 实现的共性和差异

## 阅读清单

| 顺序 | 文件 | 行数 | 要关注的特点 |
|------|------|------|-------------|
| 1 | `packages/ai/src/providers/openai-completions.ts` | 1148 | **最完整的参考实现**，30+ Provider 通过 compat 适配 |
| 2 | `packages/ai/src/providers/anthropic.ts` | 1201 | 原生 SDK（@anthropic-ai/sdk），思考实现为 Extended Thinking |
| 3 | `packages/ai/src/providers/google.ts` | ~500 | 原生 SDK（@google/genai），需要 GoogleGenAI 客户端 |
| 4 | `packages/ai/src/providers/amazon-bedrock.ts` | 960 | AWS SDK（@aws-sdk/client-bedrock-runtime），缓存点特色 |
| 5 | `packages/ai/src/providers/faux.ts` | ~500 | **测试用模拟 Provider**，完全不调用真实 API |
| 6 | `packages/ai/src/providers/register-builtins.ts` | 558 | 9 个 Provider 的延迟注册机制 |

## 对比维度

### SDK 策略

| Provider | SDK 方式 |
|----------|---------|
| OpenAI 兼容系 | 统一 HTTP 请求格式 |
| Anthropic | `@anthropic-ai/sdk` 原生 SDK |
| Google | `@google/genai` 原生 SDK |
| Bedrock | `@aws-sdk/client-bedrock-runtime` AWS SDK |
| Faux | 无 SDK，纯内存模拟 |

### 认证方式

- **API Key Bearer**：OpenAI 兼容系、Anthropic、Google — HTTP 头 `Authorization: Bearer <key>`
- **AWS Credentials**：Bedrock — 通过 AWS SDK 凭据链（环境变量、~/.aws/credentials、IAM 角色）
- **OAuth / Copilot Token**：GitHub Copilot — 设备码流或 OAuth token
- **无认证**：Faux — 完全本地，无需任何认证

### 思考参数编码

不同 Provider 对"思考"（reasoning）参数的编码方式完全不同：

- OpenAI：`reasoning_effort: "high" | "medium" | "low"`
- Anthropic：`thinking: { type: "enabled", budget_tokens: N }`
- Bedrock（Claude）：同 Anthropic 但字段名可能不同
- Google：`thinking_config: { thinking_budget: N }`

### 流式解析

- OpenAI 兼容系：手动构建 SSE 状态机，逐行解析 `data: ` 前缀和 `[DONE]` 终止符
- Anthropic / Google / Bedrock：使用原生 SDK 的 AsyncIterator，直接遍历事件
- Faux：简单的 async generator，直接产出事件对象
