---
title: "阅读路线图（完整版）"
description: "14 天渐进式源码学习"
---

# pi 源代码阅读路线图

> 为初级程序员设计的渐进式学习路径。按学习梯度将术语索引条目映射到具体源码位置，形成「第 N 天读 file.ts:line」的可执行计划。
>
> 配合 `docs/terminology-index.md`（概念定义）和 `docs/capability-analysis.md`（能力清单）使用。

## 学习梯度设计原则

- **自底向上**：从最底层（AI 层类型定义）开始，逐渐上升到应用层（CLI/TUI/扩展）
- **先小后大**：先读 50-200 行的小文件，建立概念后再读 500+ 行的大文件
- **先类型后实现**：先理解数据结构（types.ts），再理解算法（agent-loop.ts）
- **穿插验证**：每个模块读完有对应的学习沙箱示例可运行验证

## 能力等级划分

| 等级 | 达成条件 | 应该能做的事 |
|------|---------|-------------|
| **L1 入门** | 完成 Day 1-3 | 理解 pi 的核心类型体系和 LLM 调用流程 |
| **L2 熟练** | 完成 Day 4-6 | 理解 Agent 循环和工具执行机制 |
| **L3 精通** | 完成 Day 7-10 | 理解扩展系统、会话管理、上下文压缩 |
| **L4 专家** | 完成 Day 11-14 | 理解 TUI 渲染、RPC 协议、Provider 适配 |

---

## L1：入门 —— 理解核心类型和 LLM 调用

### Day 1：核心类型定义（~550 行）

**目标**：理解 pi 的数据结构基础——程序"知道"什么

| 阅读顺序 | 文件 | 行数 | 关键行 | 要理解的概念 | 术语索引条目 |
|---------|------|------|--------|-------------|-------------|
| 1 | `packages/ai/src/types.ts:33` | ~10 | `KnownApi` 联合类型 | 程序内置支持哪些 LLM API 协议 | API 标识符、API 提供商 |
| 2 | `packages/ai/src/types.ts:65` | ~30 | `KnownProvider` 联合类型 | 程序内置支持哪些 LLM 服务商 | 提供商标识符 |
| 3 | `packages/ai/src/types.ts:100` | ~30 | `Model` 接口 | 一个模型实例包含什么信息（ID、能力、成本、上下文窗口） | 模型 |
| 4 | `packages/ai/src/types.ts:160` | ~40 | `Message` 联合类型 | 对话消息的三种角色：user/assistant/toolResult | 对话上下文、Agent 消息 |
| 5 | `packages/ai/src/types.ts:220` | ~30 | `Tool` 接口 | 工具的静态定义（名称、描述、JSON Schema 参数） | 工具 |
| 6 | `packages/ai/src/types.ts:250` | ~50 | `AssistantMessageEvent` | 流式输出能产生哪些增量事件 | 流式调用、助手消息事件流 |

**验证**：运行 `npx tsx packages/coding-agent/examples/learning/01-minimal-agent.ts`

### Day 2：流式调用入口（~60 行）

**目标**：理解 LLM 调用的四个入口函数

| 阅读顺序 | 文件 | 关键行 | 要理解的概念 |
|---------|------|--------|-------------|
| 1 | `packages/ai/src/stream.ts:1` | 完整文件 | `stream()`、`complete()`、`streamSimple()`、`completeSimple()` 四兄弟 |
| 2 | `packages/ai/src/stream.ts:41` | `resolveApiProvider()` | 如何根据 model.api 分发到具体 Provider |
| 3 | `packages/ai/src/api-registry.ts:30` | `registerApiProvider()` | Provider 如何注册到全局注册表 |

**关键问题**：
- `stream()` 和 `complete()` 的区别是什么？
- `streamSimple()` 比 `stream()` 多了什么处理？
- 为什么 `resolveApiProvider()` 失败时抛错，而不是返回 undefined？

### Day 3：模型查找与成本计算（~100 行）

**目标**：理解模型如何在代码中被查找和使用

| 阅读顺序 | 文件 | 关键行 | 要理解的概念 |
|---------|------|--------|-------------|
| 1 | `packages/ai/src/models.ts:1` | `getModel()`, `getModels()`, `getProviders()` | 模型查询 API |
| 2 | `packages/ai/src/models.ts:30` | `calculateCost()` | Token 用量如何换算为美元成本 |
| 3 | `packages/ai/src/models.ts:50` | `clampThinkingLevel()` | 思考级别如何按模型能力钳位 |

---

## L2：熟练 —— 理解 Agent 循环和工具执行

### Day 4：Agent 主类（~550 行）

**目标**：理解 Agent 的有状态封装

| 阅读顺序 | 文件 | 关键行 | 要理解的概念 | 术语索引条目 |
|---------|------|--------|-------------|-------------|
| 1 | `packages/agent/src/types.ts:1` | 完整文件 | AgentState、AgentEvent、AgentTool 的类型定义 | Agent 状态、Agent 事件 |
| 2 | `packages/agent/src/agent.ts:50` | `defaultConvertToLlm()` | 默认的消息转换函数 | 消息转换 |
| 3 | `packages/agent/src/agent.ts:97` | `createMutableAgentState()` | 状态对象如何保持不可变性 | Agent 状态 |
| 4 | `packages/agent/src/agent.ts:127` | `AgentOptions` 接口 | Agent 有哪些可配置维度 | 零必填初始化 |
| 5 | `packages/agent/src/agent.ts:162` | `Agent` 类构造函数 | Agent 如何绑定状态和配置 | Agent 循环 |
| 6 | `packages/agent/src/agent.ts:395` | `prompt()` 方法 | 发起对话的入口 | 转向消息、后续消息 |
| 7 | `packages/agent/src/agent.ts:468` | `runPromptMessages()` | 如何归一化输入并启动循环 | Agent 循环 |

**验证**：运行 `npx tsx packages/coding-agent/examples/learning/03-streaming-events.ts`

### Day 5：Agent 循环核心（~700 行）

**目标**：理解双层 while-true 循环的结构

| 阅读顺序 | 文件 | 关键行 | 要理解的概念 |
|---------|------|--------|-------------|
| 1 | `packages/agent/src/agent-loop.ts:128` | `runAgentLoop()` | 如何从 prompt 消息启动循环 |
| 2 | `packages/agent/src/agent-loop.ts:208` | `runLoop()` | 双层循环结构（内层工具调用 + 外层后续消息） |
| 3 | `packages/agent/src/agent-loop.ts:319` | `streamAssistantResponse()` | LLM 调用的桥梁函数 |
| 4 | `packages/agent/src/agent-loop.ts:327` | `transformContext()` | 上下文变换钩子位置 |
| 5 | `packages/agent/src/agent-loop.ts:333` | `convertToLlm()` | 消息格式转换钩子位置 |

**关键问题**：
- 转向消息（steering）和后续消息（follow-up）的差异是什么？
- 什么条件下内层循环退出？什么条件下外层循环退出？
- `shouldStopAfterTurn` 钩子在什么时机触发？

### Day 6：工具执行（~300 行）

**目标**：理解 LLM 请求的工具调用如何被准备、校验、执行

| 阅读顺序 | 文件 | 关键行 | 要理解的概念 | 术语索引条目 |
|---------|------|--------|-------------|-------------|
| 1 | `packages/agent/src/agent-loop.ts:424` | `executeToolCalls()` | 并行/顺序分叉逻辑 | 工具执行模式 |
| 2 | `packages/agent/src/agent-loop.ts:452` | `executeToolCallsSequential()` | 顺序执行：逐个准备、执行、完成 | 工具钩子 |
| 3 | `packages/agent/src/agent-loop.ts:512` | `executeToolCallsParallel()` | 三阶段并行执行 | 工具钩子 |
| 4 | `packages/agent/src/agent-loop.ts:643` | `prepareToolCall()` | 查找、预处理、校验、beforeToolCall | 工具钩子 |
| 5 | `packages/agent/src/agent-loop.ts:701` | `executePreparedToolCall()` | 实际执行工具函数 | 工具钩子 |
| 6 | `packages/agent/src/agent-loop.ts:744` | `finalizeExecutedToolCall()` | afterToolCall 钩子应用 | 工具钩子、提前终止 |

**验证**：运行 `npx tsx packages/coding-agent/examples/learning/04-agent-loop-trace.ts`

---

## L3：精通 —— 理解扩展、会话、压缩

### Day 7：扩展系统（~1600 行）

**目标**：理解插件如何注册工具、命令、事件处理器

| 阅读顺序 | 文件 | 关键行 | 要理解的概念 |
|---------|------|--------|-------------|
| 1 | `packages/coding-agent/src/core/extensions/types.ts:1` | `ExtensionAPI` 接口 | 扩展可以调用哪些 API |
| 2 | `packages/coding-agent/src/core/extensions/types.ts:100` | 事件类型列表（31 种） | 扩展可以监听哪些系统事件 |
| 3 | `packages/coding-agent/src/core/extensions/types.ts:200` | `ToolDefinition` 接口 | 自定义工具的定义结构 |
| 4 | `packages/coding-agent/src/core/extensions/types.ts:350` | `ExtensionContext` 接口 | 扩展的运行时上下文 |
| 5 | `packages/coding-agent/src/core/extensions/loader.ts:1` | `ExtensionLoader` 类 | 如何用 jiti 动态加载 .ts 扩展 |

### Day 8：会话管理（~1400 行）

**目标**：理解会话的持久化、分支和恢复

| 阅读顺序 | 文件 | 关键行 | 要理解的概念 | 术语索引条目 |
|---------|------|--------|-------------|-------------|
| 1 | `packages/coding-agent/src/core/session-manager.ts:1` | `SessionEntry` 类型 | JSONL 文件的数据结构 | JSONL 会话文件 |
| 2 | `packages/coding-agent/src/core/session-manager.ts:200` | `buildContext()` | 从 JSONL 树重建消息历史 | 会话上下文重建 |
| 3 | `packages/coding-agent/src/core/session-manager.ts:400` | `appendMessage()` | 新消息如何追加到 JSONL | JSONL 会话文件 |
| 4 | `packages/coding-agent/src/core/session-manager.ts:600` | 分支导航方法 | fork/switch/clone | 会话恢复与会话新建 |
| 5 | `packages/coding-agent/src/core/sdk.ts:236` | `createAgentSession()` | 整个会话组装的入口 | 零必填初始化 |

### Day 9：上下文压缩（~850 行）

**目标**：理解对话过长时的自动摘要机制

| 阅读顺序 | 文件 | 关键行 | 要理解的概念 | 术语索引条目 |
|---------|------|--------|-------------|-------------|
| 1 | `packages/coding-agent/src/core/compaction/compaction.ts:1` | `shouldCompact()` | 如何判断需要压缩 | 上下文压缩 |
| 2 | `packages/coding-agent/src/core/compaction/compaction.ts:50` | `findCutPoint()` | 安全裁剪点的查找算法 | 上下文压缩 |
| 3 | `packages/coding-agent/src/core/compaction/compaction.ts:100` | `compact()` | 调用 LLM 生成摘要 | 上下文压缩 |
| 4 | `packages/coding-agent/src/core/compaction/branch-summarization.ts:1` | `generateBranchSummary()` | 为会话分支生成描述文字 | 分支摘要 |

### Day 10：消息转换与 Provider 适配（~1200 行）

**目标**：理解消息在不同 Provider 间的转换逻辑

| 阅读顺序 | 文件 | 关键行 | 要理解的概念 | 术语索引条目 |
|---------|------|--------|-------------|-------------|
| 1 | `packages/ai/src/providers/transform-messages.ts:1` | 图片降级、孤儿修复、ID 规范化 | 跨 Provider 消息兼容的三道工序 | 消息转换、图片降级、孤儿工具调用 |
| 2 | `packages/ai/src/providers/openai-completions.ts:930` | `convertMessages()` | AgentMessage → OpenAI 格式 | 协议适配链 |
| 3 | `packages/ai/src/providers/openai-completions.ts:1190` | `convertTools()` | Tool → OpenAI function 格式 | 协议适配链 |
| 4 | `packages/ai/src/providers/openai-completions.ts:1307` | `detectCompat()` | 30+ Provider 的自动差异检测 | 兼容性设置 |

---

## L4：专家 —— 理解 TUI、RPC、Provider 实现

### Day 11：TUI 引擎（~1320 行）

**目标**：理解终端差异渲染引擎的工作原理

| 阅读顺序 | 文件 | 关键行 | 要理解的概念 |
|---------|------|--------|-------------|
| 1 | `packages/tui/src/tui.ts:1` | `Component`、`Focusable` 接口 | 组件抽象和焦点管理 |
| 2 | `packages/tui/src/tui.ts:100` | `Container` 类 | 组件的容器管理 |
| 3 | `packages/tui/src/tui.ts:200` | `TUI` 类构造函数 | 差异渲染引擎如何初始化 |
| 4 | `packages/tui/src/tui.ts:400` | `start()` 方法 | 渲染循环如何工作 |
| 5 | `packages/tui/src/tui.ts:600` | `showOverlay()` | 浮层系统（对话框、下拉菜单） |

**验证**：启动 `pi` 交互模式，观察终端渲染效果

### Day 12：编辑器组件（~2300 行）

**目标**：理解多行文本编辑器的内部机制

| 阅读顺序 | 文件 | 关键行 | 要理解的概念 | 术语索引条目 |
|---------|------|--------|-------------|-------------|
| 1 | `packages/tui/src/components/editor.ts:1` | 构造函数、字段定义 | 编辑器的状态模型 | 编辑器组件 |
| 2 | `packages/tui/src/components/editor.ts:200` | `handleInput()` | 输入处理的分发逻辑 | 编辑器组件 |
| 3 | `packages/tui/src/components/editor.ts:400` | 光标移动系统 | `moveToVisualLine`、`computeVerticalMoveColumn` | 编辑器组件 |
| 4 | `packages/tui/src/components/editor.ts:800` | 撤销/重做 | undo stack 的实现 | 编辑器组件 |
| 5 | `packages/tui/src/components/editor.ts:1200` | 自动补全 | autocomplete 子系统 | 编辑器组件 |
| 6 | `packages/tui/src/components/editor.ts:1600` | Kill Ring | 类 Emacs 剪切历史 | 编辑器组件 |

### Day 13：RPC 协议（~1000 行）

**目标**：理解无头模式的 stdin/stdout JSON 协议

| 阅读顺序 | 文件 | 关键行 | 要理解的概念 | 术语索引条目 |
|---------|------|--------|-------------|-------------|
| 1 | `packages/coding-agent/src/modes/rpc/rpc-types.ts:1` | `RpcCommand` 联合类型 | 27 种 RPC 命令的数据结构 | RPC 命令 |
| 2 | `packages/coding-agent/src/modes/rpc/rpc-mode.ts:48` | `runRpcMode()` | RPC 模式的完整启动流程 | RPC 模式 |
| 3 | `packages/coding-agent/src/modes/rpc/rpc-mode.ts:371` | `handleCommand()` | 命令分发到各个处理器 | RPC 命令 |
| 4 | `packages/coding-agent/src/modes/rpc/jsonl.ts:1` | `attachJsonlLineReader()` | JSON 行的流式解析 | JSON 行协议 |

### Day 14：Provider 实现全景对比

**目标**：理解不同 Provider 实现的共性和差异

| 阅读顺序 | 文件 | 行数 | 要关注的特点 |
|---------|------|------|-------------|
| 1 | `packages/ai/src/providers/openai-completions.ts` | 1148 | **最完整的参考实现**，30+ Provider 通过 compat 适配 |
| 2 | `packages/ai/src/providers/anthropic.ts` | 1201 | 原生 SDK（@anthropic-ai/sdk），思考实现为 Extended Thinking |
| 3 | `packages/ai/src/providers/google.ts` | ~500 | 原生 SDK（@google/genai），需要 GoogleGenAI 客户端 |
| 4 | `packages/ai/src/providers/amazon-bedrock.ts` | 960 | AWS SDK（@aws-sdk/client-bedrock-runtime），缓存点特色 |
| 5 | `packages/ai/src/providers/faux.ts` | ~500 | **测试用模拟 Provider**，完全不调用真实 API |
| 6 | `packages/ai/src/providers/register-builtins.ts` | 558 | 9 个 Provider 的延迟注册机制 |

**对比维度**：
- 使用原生 SDK vs 通用 OpenAI 格式
- 认证方式（API Key Bearer / AWS Credentials / OAuth / Copilot Token）
- 思考参数编码（reasoning_effort vs thinking.type vs enable_thinking）
- 流式解析方式（SSE 状态机 vs SDK 原生迭代器）

---

## 附录：快速索引

按概念快速定位源码位置：

| 我想理解... | 从这里开始读 | 预计时间 |
|-----------|------------|---------|
| 程序支持哪些 LLM | `packages/ai/src/types.ts:65` KnownProvider | 5 分钟 |
| 一次 LLM 调用发生了什么 | `packages/ai/src/stream.ts:41` | 15 分钟 |
| Agent 循环如何运转 | `packages/agent/src/agent-loop.ts:208` runLoop | 30 分钟 |
| 工具如何被执行 | `packages/agent/src/agent-loop.ts:424` | 20 分钟 |
| 扩展如何注册到系统 | `packages/coding-agent/src/core/extensions/loader.ts` | 15 分钟 |
| 对话历史如何持久化 | `packages/coding-agent/src/core/session-manager.ts:200` buildContext | 20 分钟 |
| 上下文压缩如何工作 | `packages/coding-agent/src/core/compaction/compaction.ts` | 15 分钟 |
| 终端 UI 如何渲染 | `packages/tui/src/tui.ts:400` start() | 20 分钟 |
| 编辑器如何处理输入 | `packages/tui/src/components/editor.ts:200` handleInput | 20 分钟 |
| CLI 参数如何变成 Agent 会话 | `packages/coding-agent/src/main.ts:591` main() | 15 分钟 |
| SDK 如何创建会话 | `packages/coding-agent/src/core/sdk.ts:236` createAgentSession | 10 分钟 |
| RPC 如何与外部进程通信 | `packages/coding-agent/src/modes/rpc/rpc-mode.ts:48` | 15 分钟 |
