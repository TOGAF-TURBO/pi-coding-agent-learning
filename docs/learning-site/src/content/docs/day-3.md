---
title: "Day 3: 模型查找与成本计算"
---

**学习目标**：理解模型如何在代码中被查找和使用

## 阅读清单

| 顺序 | 文件 | 关键行 | 要理解的概念 |
|------|------|--------|-------------|
| 1 | `packages/ai/src/models.ts:1` | `getModel()`, `getModels()`, `getProviders()` | 模型查询 API |
| 2 | `packages/ai/src/models.ts:30` | `calculateCost()` | Token 用量如何换算为美元成本 |
| 3 | `packages/ai/src/models.ts:50` | `clampThinkingLevel()` | 思考级别如何按模型能力钳位 |

## 核心概念

`getModel(modelId)` 从 `models.generated.ts` 生成的模型表中精确查找。模型表包含每个模型的 ID、能力标签（vision、thinking 等）、输入/输出 token 定价和上下文窗口大小。

`calculateCost()` 将 token 用量乘以模型单价得出美元成本——输入 token 和输出 token 通常采用不同费率。

`clampThinkingLevel()` 根据模型的思考能力（`thinkingLevel`）自动调整请求的思考等级，避免对不支持的模型发送无效参数。
