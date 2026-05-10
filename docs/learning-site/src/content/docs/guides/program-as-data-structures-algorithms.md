---
title: "程序 = 数据结构 + 算法"
description: "Wirth 公式的三维扩展"
---

# 数据结构 + 算法 = 程序 —— 在 pi 项目中的应用

> 基于 Wirth 公式的三维扩展：现代程序 = 数据结构 + 算法 + 类型约束 + I/O 副作用

## 一、核心公式解读

```
Wirth 的原始公式：  程序 = 数据结构 + 算法
TypeScript 扩展：   程序 = 数据结构 + 算法 + 类型约束 + 副作用(I/O)
```

在 TypeScript/Node.js 项目中：

- **数据结构** = interface / type / enum / class 字段 —— 回答「程序知道什么」
- **算法** = function 体 / method 体 / Promise 链 —— 回答「程序做什么」
- **类型约束** = 泛型约束 / 字面量联合 / `satisfies` / `as` —— 回答「什么可以做、什么不可以做」
- **副作用** = I/O 调用 / `fs.readFile` / HTTP 请求 / `process.exit` / stdout 写入 —— 回答「程序如何与外界交互」

## 二、函数签名是四者的唯一交汇点

```typescript
// 函数签名声明了全部四个维度
export function stream<TApi extends Api>(        // ← 类型约束（泛型约束）
  model: Model<TApi>,                             // ← 数据结构（参数类型）
  context: Context,                               // ← 数据结构
  options?: StreamOptions                         // ← 数据结构 + 类型约束（? 可选）
): AssistantMessageEventStream {                  // ← 数据结构（返回类型）
  // 函数体 = 算法
  const provider = getApiProvider(model.api);     // ← 算法（查找逻辑）
  return provider.stream(model, context, opts);   // ← 副作用（委托给 HTTP 调用）
}
```

**单一函数签名同时声明了**：
- 它处理什么数据（`Model`, `Context`）
- 它如何约束调用方（泛型 `<TApi extends Api>` 编译期绑定）
- 它产出什么（`AssistantMessageEventStream`）
- 它的副作用方向（流式返回暗示异步 I/O）

## 三、pi 项目中的四维映射

### 3.1 数据结构层

| 代码位置 | 定义了「程序知道什么」 |
|----------|---------------------|
| `packages/ai/src/types.ts` | Message、Model、Context、Tool、Usage（AI 层的所有实体） |
| `packages/agent/src/types.ts` | AgentState、AgentEvent、AgentTool、AgentMessage（Agent 层的实体） |
| `packages/coding-agent/src/core/extensions/types.ts` | ExtensionAPI、ToolDefinition（扩展系统的实体） |
| `packages/coding-agent/src/core/session-manager.ts` | SessionEntry（会话持久化实体） |

### 3.2 算法层

| 代码位置 | 定义了「程序做什么」 |
|----------|---------------------|
| `packages/ai/src/stream.ts` | stream/complete：核心调用编排 |
| `packages/agent/src/agent-loop.ts` | runAgentLoop：Agent 循环的迭代逻辑 |
| `packages/ai/src/providers/transform-messages.ts` | 消息格式转换（图片降级、孤儿修复） |
| `packages/coding-agent/src/core/compaction/compaction.ts` | findCutPoint/compact：上下文压缩算法 |

### 3.3 类型约束层（扩展维度）

| 代码位置 | 作用 |
|----------|------|
| `Model<TApi extends Api>` | 编译期绑定模型到 API 协议，防止 OpenAI 模型调用 Anthropic SDK |
| `KnownProvider`（32 个字面量联合） | 限制提供商参数只能取预定义值 |
| `TypeBox` Schema（`Type.Object`, `Type.String`） | 运行时验证工具参数的形状 |
| `satisfies Model<any>`（agent.ts:78） | 确保 DEFAULT_MODEL 满足 Model 接口的完整约束 |

### 3.4 副作用层（扩展维度）

| 代码位置 | 副作用类型 |
|----------|-----------|
| `packages/ai/src/providers/openai-completions.ts` | HTTP 请求（fetch OpenAI API） |
| `packages/coding-agent/src/core/bash-executor.ts` | 子进程创建（spawn shell） |
| `packages/coding-agent/src/core/session-manager.ts` | 文件 I/O（读写 JSONL 会话文件） |
| `packages/coding-agent/src/modes/rpc/rpc-mode.ts` | stdin/stdout 管道通信 |
| `packages/tui/src/tui.ts` | 终端 ANSI 输出（写 tty） |
| `packages/coding-agent/src/core/auth-storage.ts` | 文件 I/O（读写 auth.json） |

## 四、以 pi 的核心流程为例串联四维

一次完整的 Agent 对话调用链：

```
用户输入 "帮我列出所有 .ts 文件"
        │
        ▼  [数据结构：string]
        │
createAgentSession({ model: getModel("anthropic", "claude-sonnet-4") })
        │
        ▼  [类型约束：model.api === "anthropic-messages"]
        │
agent.prompt("帮我列出所有 .ts 文件")
        │
        ▼  [算法：runAgentLoop]
        │   1. convertToLlm(messages)       → Message[]  [数据结构转换]
        │   2. stream(model, context, opts)  → EventStream [副作用：HTTP 调用]
        │   3. 检测到 tool_call("find")      → ToolCall   [数据结构]
        │   4. executeFindTool(args)          → ToolResult [算法 + 副作用：文件 I/O]
        │   5. 将结果反馈 LLM，循环回到 1
        │
        ▼  [数据结构：AgentEvent[]]
        │
subscribe(listener) 触发 message_end
        │
        ▼  [副作用：SessionManager.appendMessage → JSONL 文件写入]
```

## 五、利用此框架阅读项目的建议

**对初级程序员的阅读顺序**：

1. **先读数据结构**：打开 `types.ts`，理解 `Message`、`Model`、`Context` 长什么样——这就是程序「知道」的东西
2. **再读类型约束**：注意 `extends`、字面量联合、`?` 可选标记——这些告诉你「什么能做、什么不能做」
3. **然后读算法**：找核心函数（`stream`、`runAgentLoop`、`compact`），看它们如何把数据结构 A 变成数据结构 B
4. **最后读副作用**：找到 `fetch`、`spawn`、`readFileSync`、`process.stdout.write`——这些是程序触碰外部世界的地方

**判断一个函数重要性的启发式**：

```
函数重要程度 ≈ 参数类型的数量 × 参数类型的复杂度 × 调用链深度

例如：
  stream(model, context, options?) → 3 个参数，全部是复杂类型 → 核心函数
  getUserMessageText(message) → 1 个参数，简单返回 string → 工具函数
```

## 六、回到 Wirth 公式

Niklaus Wirth 1976 年在《Algorithms + Data Structures = Programs》中写下的公式，在 TypeScript 项目中依然成立——只是现代类型系统让「数据结构」变得比 Pascal 时期更丰富（泛型、联合、交叉类型），也让「算法」更多地表现为异步流控制和事件驱动的 `AsyncGenerator`。

pi 项目是理解这个公式的绝佳案例：它用 32 个提供商的数据结构 + Agent Loop 的算法 + 泛型约束的类型安全 + HTTP/文件/终端的副作用，构建了一个完整的编码代理。
