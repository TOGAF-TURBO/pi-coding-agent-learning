---
title: "Day 9: 上下文压缩"
---

**学习目标**：理解对话过长时的自动摘要机制

## 阅读清单

| 顺序 | 文件 | 关键行 | 要理解的概念 | 术语 |
|------|------|--------|-------------|------|
| 1 | `packages/coding-agent/src/core/compaction/compaction.ts:1` | `shouldCompact()` | 如何判断需要压缩 | 上下文压缩 |
| 2 | `packages/coding-agent/src/core/compaction/compaction.ts:50` | `findCutPoint()` | 安全裁剪点的查找算法 | 上下文压缩 |
| 3 | `packages/coding-agent/src/core/compaction/compaction.ts:100` | `compact()` | 调用 LLM 生成摘要 | 上下文压缩 |
| 4 | `packages/coding-agent/src/core/compaction/branch-summarization.ts:1` | `generateBranchSummary()` | 为会话分支生成描述文字 | 分支摘要 |

## 压缩流程

```
shouldCompact() → findCutPoint() → compact()
    ↓               ↓                ↓
判断是否超限      找安全裁剪点      LLM 生成摘要
```

**触发条件**：当上下文 token 数超过模型窗口的一定比例（通常 80%）时，`shouldCompact()` 返回 true。

**安全裁剪**：`findCutPoint()` 不能随意截断——需要在消息边界、完整的工具调用后、非关键消息等位置找到"安全"的裁剪点，避免切断关键的上下文。

**摘要生成**：调用 LLM 对裁减掉的部分生成精简摘要，保留关键信息（文件改动、重要决策、错误信息），丢弃冗余对话。

## 分支摘要

`generateBranchSummary()` 为会话树的每个分支生成简短的文字描述，用于分支切换时的列表展示。
