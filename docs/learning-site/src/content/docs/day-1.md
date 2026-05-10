---
title: "Day 1: 核心类型定义"
---
**学习目标**：理解 pi 的数据结构基础——程序"知道"什么

## 阅读清单

| 顺序 | 文件 | 行数 | 要理解的概念 | 术语 |
|------|------|------|-------------|------|
| 1 | `packages/ai/src/types.ts:33` | ~10 | 程序内置支持哪些 LLM API 协议 | API 标识符 |
| 2 | `packages/ai/src/types.ts:65` | ~30 | 程序内置支持哪些 LLM 服务商 | 提供商标识符 |
| 3 | `packages/ai/src/types.ts:100` | ~30 | 一个模型实例包含什么信息（ID、能力、成本、上下文窗口） | 模型 |
| 4 | `packages/ai/src/types.ts:160` | ~40 | 对话消息的三种角色：user/assistant/toolResult | 对话上下文 |
| 5 | `packages/ai/src/types.ts:220` | ~30 | 工具的静态定义（名称、描述、JSON Schema 参数） | 工具 |
| 6 | `packages/ai/src/types.ts:250` | ~50 | 流式输出能产生哪些增量事件 | 流式调用 |

  ### 核心文件
    `packages/ai/src/types.ts`
  </Card
  ### 阅读行数
    约 ~200 行（6 个关键位置）
  </Card
</CardGrid

## 阅读步骤
1. 打开 `packages/ai/src/types.ts:33`，找到 `KnownApi` 联合类型

2. 继续到第 65 行，查看 `KnownProvider` 联合类型

3. 跳到第 100 行，阅读 `Model` 接口定义

4. 在第 160 行，理解 `Message` 联合类型的三种角色

5. 在第 220 行，学习 `Tool` 接口的静态定义

6. 在第 250 行，了解 `AssistantMessageEvent` 流式事件体系
## 验证

运行以下命令验证你对 LLM 调用流程的理解：

```bash
npx tsx packages/coding-agent/examples/learning/01-minimal-agent.ts
```
