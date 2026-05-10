---
title: "能力路径分析"
description: "端到端调用链追踪"
---

# pi 能力路径分析

> 回答「程序通过什么路径、在什么约束下、产生什么副作用、在哪些点可被外部修改地做到什么」
>
> 以 `pi -p "列出所有 .ts 文件"` 为线索，端到端追踪一次用户请求的完整转换路径。
>
> 分析日期：2026-05-10 | 分析版本：0.74.0

---

## 阅读指引

本文档中的调用链追踪标记 `file.ts:line` 指向源码中的精确位置，可作为阅读地图使用。

## 一、三个入口点概览

pi 支持三种接入方式，它们汇聚于同一个内核（`createAgentSession`），分叉点在外层模式分发。

```
CLI:  pi -p "prompt"            → main() → 模式分发 → runPrintMode()
SDK:  createAgentSession()      → SDK API → AgentSession.prompt()
RPC:  echo '{"type":"prompt"}'  → stdin → runRpcMode() → 命令分发
                                    │
                                    ▼
                          createAgentSession()
                              │
                     new Agent() + new AgentSession()
                                    │
                          agent.prompt() → Agent Loop → LLM HTTP
```

---

## 二、CLI 入口：端到端追踪

以 `pi -p "列出所有 .ts 文件"` 为例，追踪从命令行到 HTTP 响应的完整路径。

### 第 0 层：进程启动

```
cli.ts:32  main(process.argv.slice(2))    ← 入口
  ├── cli.ts:17  process.title = APP_NAME        [副作用：进程标题]
  ├── cli.ts:20  process.env.PI_CODING_AGENT     [副作用：环境标记]
  ├── cli.ts:22  process.emitWarning → noop     [副作用：抑制弃用警告]
  └── cli.ts:29  setGlobalDispatcher(...)        [副作用：HTTP 无限超时]
```

### 第 1 层：main() 启动阶段

```
main.ts:591  main(["-p", "列出所有 .ts 文件"])
  ├── main.ts:592  resetTimings()                          [初始化性能计时]
  ├── main.ts:600  handlePackageCommand() → false           [非包管理命令]
  └── main.ts:609  parseArgs(["-p", "列出所有 .ts 文件"])
       └── args.ts:63  → { print: true, messages: ["列出所有 .ts 文件"], ... }
```

### 第 2 层：服务创建

```
main.ts:620  resolveAppMode(parsed, isTTY) → "print"       [模式判断：-p 标志]
main.ts:624  takeOverStdout()                               [副作用：接管 stdout]
  └── output-guard.ts  → 替换 process.stdout.write

main.ts:673  createSessionManager(parsed, cwd, ...)
  └── SessionManager.create(cwd, sessionDir) → SessionManager
       [数据结构] session-manager.ts:1277
       [副作用] 在磁盘创建 JSONL 会话文件

main.ts:793  createAgentSessionRuntime(createRuntime, {...})
  ├── agent-session-services.ts:133  createAgentSessionServices()
  │     ├── AuthStorage.create(authPath)            [副作用：读取 ~/.pi/agent/auth.json]
  │     │     [数据结构] auth.json → Map<provider, OAuthCredentials>
  │     ├── SettingsManager.create(cwd, agentDir)   [副作用：读取项目级+全局 settings.json]
  │     │     [类型约束] 通过 getDefaultProvider/getDefaultModel 返回字符串字面量
  │     ├── ModelRegistry.create(authStorage, modelsPath)
  │     │     [副作用：读取 ~/.pi/agent/models.json]
  │     │     [数据结构] 将用户自定义模型与内置模型合并
  │     └── DefaultResourceLoader(...).reload()
  │           [副作用：遍历 .pi/extensions/、.pi/skills/、.pi/themes/、
  │                     ~/.pi/agent/、cwd 树中的 AGENTS.md/CLAUDE.md]
  │           [扩展点] 发现的扩展工厂存为 pendingProviderRegistrations
  │
  └── core/sdk.ts:236  createAgentSession({ model, tools, ... })
        ├── sdk.ts:237-248  [配置分支] 注入或创建 SessionManager/ModelRegistry/...
        ├── sdk.ts:258  sessionManager.buildSessionContext()
        │     [算法] 从 JSONL 树重建完整消息历史和模型/思考信息
        ├── sdk.ts:279  findInitialModel({ defaultProvider, ... })
        │     [类型约束] modelRegistry.find(provider, id) → 找到的模型已绑定 TApi
        ├── sdk.ts:315  clampThinkingLevel(model, thinkingLevel)
        │     [类型约束] 将用户选择钳位到模型支持范围内
        ├── sdk.ts:371  new Agent({ streamFn, onPayload, onResponse, ... })
        │     [算法] 创建有状态 Agent 实例，绑定流式函数和拦截器
        │     [数据结构] AgentState = { systemPrompt, model, tools, messages }
        │     [类型约束] streamFn 的 model 参数被 <TApi> 约束
        └── sdk.ts:449  new AgentSession({ agent, sessionManager, tools, ... })
              [数据结构] 封装 Agent + 会话持久化 + 扩展 + 压缩 + 重试
```

### 第 3 层：模式分发 → Print 模式

```
main.ts:908  runPrintMode(runtime, { initialMessage, images })
  └── print-mode.ts:32
        ├── print-mode.ts:73  session.bindExtensions({...})
        │     [扩展点] 绑定 UI 上下文、命令操作、关闭处理和错误监听器
        ├── print-mode.ts:103  session.subscribe(handler)
        │     [可观测] handler 接收所有 AgentSessionEvent
        │     [算法] JSON 模式逐行输出事件到 stdout
        └── print-mode.ts:121  session.prompt("列出所有 .ts 文件")
              │
              ▼ 进入 Agent 循环
```

### 第 4 层：Agent 循环核心

```
agent.ts:395  agent.prompt("列出所有 .ts 文件")
  └── agent.ts:468  runPromptMessages()
        └── agent.ts:548  runWithLifecycle()
              [副作用] 创建 AbortController
              [数据结构] 设置 isStreaming=true, streamingMessage, errorMessage
              │
              └── agent-loop.ts:128  runAgentLoop(prompts, context, config, emit)
                    ├── 数据结构转换：string → AgentMessage → 追加到 context.messages
                    ├── [可观测] 发出 agent_start → turn_start → message_start/end
                    │
                    └── agent-loop.ts:208  runLoop()
                          │
                          ┌──── 内层循环（工具调用 + 转向消息）────┐
                          │                                         │
                          │ ① agent-loop.ts:244                    │
                          │    streamAssistantResponse(context)     │
                          │    │                                    │
                          │    ├── agent-loop.ts:329                │
                          │    │   config.transformContext(messages) [扩展点 1]
                          │    │   上下文裁剪/注入
                          │    │
                          │    ├── agent-loop.ts:333                │
                          │    │   config.convertToLlm(messages)    [扩展点 2]
                          │    │   AgentMessage[] → Message[]
                          │    │   自定义消息类型映射为标准 LLM 消息
                          │    │
                          │    └── agent-loop.ts:349                │
                          │        streamFunction(model, context)   → HTTP 调用
                          │        │
                          │        └── 见第 5 层
                          │
                          │ ② agent-loop.ts:261                    │
                          │    executeToolCalls(context, message)   → 工具执行
                          │    │
                          │    └── 见第 6 层
                          │
                          │ ③ agent-loop.ts:275                    │
                          │    config.shouldStopAfterTurn?()        [扩展点 3]
                          │    停止判断钩子
                          │
                          │ ④ agent-loop.ts:288                    │
                          │    config.getSteeringMessages?.()       [扩展点 4]
                          │    轮询转向队列 → 新待处理消息
                          │
                          └────────────────────────────────────────┘
                          │
                          └── agent-loop.ts:292
                              config.getFollowUpMessages?.()         [扩展点 5]
                              轮询后续队列 → 新待处理消息（触发外层循环）
```

### 第 5 层：LLM HTTP 调用

发送给 LLM 的 HTTP 请求经过了完整的消息准备和协议适配链。

```
stream.ts:89  streamSimple(model, llmContext, options)
  ├── stream.ts:41   resolveApiProvider(model.api) → 按 api 分发的注册表查找
  │     └── api-registry.ts:140  Map.get("openai-completions") → ApiProviderInternal
  │           [类型约束] 注册表以 Api 字面量为键，编译期类型擦除但运行时不匹配抛错
  │
  └── 委托给 → openai-completions.ts:525  streamSimpleOpenAICompletions()
        ├── simple-options.ts:27  buildBaseOptions(model, options, apiKey)
        │     [算法] 映射 SimpleStreamOptions → OpenAICompletionsOptions
        │     [配置分支] 根据 thinkingLevel 设置 reasoningEffort
        │     [配置分支] maxTokens 默认 Math.min(model.maxTokens, 32000)
        │
        └── openai-completions.ts:178  streamOpenAICompletions()
              │
              ├── openai-completions.ts:208  getCompat(model)
              │     [算法] 检测 provider 特性和 baseUrl → compat 对象
              │     [配置分支] 30+ provider 有各自的 thinkingFormat / maxTokensField
              │
              ├── openai-completions.ts:212  createClient()
              │     [副作用] 创建 OpenAI 客户端实例
              │     [配置分支] Cloudflare 用 AI Gateway 认证
              │     [配置分支] GitHub Copilot 用动态头部生成
              │
              ├── openai-completions.ts:215  buildParams()
              │     ├── openai-completions.ts:644  convertMessages(model, context)
              │     │     [算法] Message[] → OpenAI ChatCompletionMessageParam[]
              │     │     └── openai-completions.ts:955  transformMessages(messages, model)
              │     │           [算法] 图片降级 / 孤儿修复 / ID 规范化
              │     │           [类型约束] 不支持图片的模型 → 文本占位符替换
              │     │
              │     └── openai-completions.ts:685  convertTools(context.tools)
              │           [算法] Tool[] → OpenAI ChatCompletionTool[]
              │           [数据结构] 将 JSON Schema 参数转为 OpenAI function 格式
              │
              ├── openai-completions.ts:218  options.onPayload?.(params, model)
              │     [扩展点 6] 发送给 LLM 前可修改请求体
              │
              ├── openai-completions.ts:228  client.chat.completions.create(params)
              │     [副作用] HTTP POST 到 LLM 提供商
              │     [副作用] SSE 流式解析（357-479 行的状态机循环）
              │     [数据结构] Chunk → TextContent / ThinkingContent / ToolCallContent
              │     [可观测] 发出 text_delta / thinking_delta / toolcall_delta
              │
              └── openai-completions.ts:481  → done/error → AssistantMessageEventStream 结束
                    [数据结构] 最终 AssistantMessage = { content[], stopReason, usage, cost }
```

### 第 6 层：工具执行

LLM 返回 `tool_call(find, pattern="**/*.ts")` 时进入工具执行路径。

```
agent-loop.ts:424  executeToolCalls(context, message, config, signal, emit)
  └── 判断 toolExecution 模式
        ├── agent-loop.ts:452  executeToolCallsSequential()
        │     │
        │     └── agent-loop.ts:512  executeToolCallsParallel()
        │           │
        │           ├── [阶段 1] prepareToolCall() × N (顺序)
        │           │     ├── agent-loop.ts:651  查找 tool 定义
        │           │     ├── agent-loop.ts:661  参数预处理 (prepareToolCallArguments)
        │           │     ├── agent-loop.ts:662  Schema 校验 (TypeBox)
        │           │     │     [类型约束] 运行时验证参数形状
        │           │     └── agent-loop.ts:664  config.beforeToolCall?()
        │           │           [扩展点 7] 可阻止执行、替换参数
        │           │
        │           ├── [阶段 2] Promise.all() 并发执行
        │           │     └── agent-loop.ts:709  tool.execute(toolCallId, args, signal, onUpdate)
        │           │           [副作用] find 工具 → 文件系统 glob 查找
        │           │           [副作用] bash 工具 → spawn 子进程
        │           │           [副作用] read 工具 → readFileSync
        │           │           [可观测] onUpdate → tool_execution_update 事件
        │           │
        │           └── [阶段 3] finalizeExecutedToolCall() + emit (按原始顺序)
        │                 └── agent-loop.ts:755  config.afterToolCall?()
        │                       [扩展点 8] 可覆盖结果内容、终止标志
        │
        └── 返回 ToolResultMessage[] → 追加到 context.messages → 循环回到 ①
              [数据结构] ToolResultMessage = { role: "toolResult", content[], toolCallId }
```

---

## 三、SDK 入口：对比分叉

SDK 绕过 CLI 参数解析和模式分发，直接进入 `createAgentSession`。

```
SDK 调用: await createAgentSession({ model, tools, ... })
  │
  ├── 与 CLI 相同：createAgentSession() 内部 → AuthStorage / ModelRegistry /
  │   SettingsManager / SessionManager / DefaultResourceLoader.reload()
  │
  └── 分叉点：无模式分发。返回值中包含 session 对象

SDK 调用: await session.prompt("列出所有 .ts 文件")
  │
  └── 与 CLI 相同：agent.prompt() → runAgentLoop() → LLM HTTP

差异：
  - CLI: main() → parseArgs() → 模式分发 → session.prompt()
  - SDK: createAgentSession() → session.prompt()
  - SDK 无 stdout 接管、无管道 stdin 读取、无版本检查、无主题初始化
```

## 四、RPC 入口：对比分叉

RPC 模式用 JSON-RPC 协议替换命令行和 TUI 交互。

```
RPC: echo '{"type":"prompt","message":"列出所有 .ts 文件"}' | pi --mode rpc
  │
  ├── main.ts:864  runRpcMode(runtime)
  │     └── rpc-mode.ts:48
  │           ├── takeOverStdout()                    [副作用：接管 stdout]
  │           ├── rebindSession()                     [扩展点：创建 RPC ExtensionUIContext]
  │           │     └── 所有对话框变为 JSON-RPC 请求
  │           ├── session.subscribe(handler)
  │           │     [算法] 所有 AgentSessionEvent → JSON 行 → stdout
  │           └── attachJsonlLineReader(stdin, handler)
  │                 [副作用] 从 stdin 逐行读取 JSON 命令
  │                 [副作用] 每收到命令 → 写入 stdout JSON 响应
  │
  │   命令分发（rpc-mode.ts:371） → handleCommand(command)
  │     ├── "prompt"     → session.prompt(message, { images })
  │     │     [数据结构] ImageContent[] 支持 base64 图片
  │     ├── "steer"      → session.agent.steer(message)
  │     │     [数据结构] 注入转向消息
  │     ├── "abort"      → session.agent.abort()
  │     │     [副作用] 触发 AbortController，中断当前 LLM 调用
  │     ├── "set_model"  → session.agent.state.model = ...
  │     │     [类型约束] 新模型必须能通过 modelRegistry.find() 找到
  │     ├── "compact"    → session.compact()
  │     │     [算法] 触发上下文压缩
  │     ├── "bash"       → session.executeBash(command)
  │     │     [副作用] spawn 子进程
  │     ├── "switch_session" → session.navigateToEntry(targetId)
  │     │     [副作用] 读取 JSONL 会话文件新分支
  │     └── "export_html"    → session.exportHtml(outputPath)
  │           [副作用] 写入 HTML 文件
  │
  └── 与 CLI 相同：所有命令最终委托给 AgentSession → Agent → Agent Loop → LLM HTTP
```

---

## 五、副作用全景图

以调用链中所有 `import from "node:*"` 和 `fetch()` 为线索，穷举程序的 I/O 边界。

| 副作用类型 | 主要出现位置 | 程序触碰的外部世界 |
|-----------|-------------|------------------|
| **HTTP 请求** | `packages/ai/src/providers/openai-completions.ts:228`（及 25 个其他 Provider） | 外部 LLM API 端点 |
| **HTTP 请求** | `packages/ai/src/providers/anthropic.ts` → Anthropic SDK | Anthropic API |
| **HTTP 请求** | `packages/ai/src/providers/google.ts` → @google/genai SDK | Google Gemini API |
| **HTTP 请求** | `packages/ai/src/providers/mistral.ts` → @mistralai/mistralai SDK | Mistral API |
| **OAuth HTTP** | `packages/ai/src/utils/oauth/` | OAuth 令牌端点 |
| **文件读取** | `AuthStorage.create()`（30+ 位置） | `~/.pi/agent/auth.json` |
| **文件读取** | `SettingsManager.create()` | 项目级+全局 `settings.json` |
| **文件读取** | `ModelRegistry.create()` | `~/.pi/agent/models.json` |
| **文件读取** | `DefaultResourceLoader.reload()` | 扩展目录、技能目录、主题目录、AGENTS.md/CLAUDE.md |
| **文件读取** | Read 工具 → `readFileSync` | 用户指定的任意文件 |
| **文件写入** | Write 工具 → `writeFileSync` | 用户指定的任意文件 |
| **文件写入** | Edit 工具 → `readFileSync` + `writeFileSync` | 编辑器改写目标文件 |
| **文件写入** | SessionManager → JSONL 追加 | `~/.pi/agent/sessions/.../session.jsonl` |
| **文件写入** | 导出 → HTML 文件 | 用户指定的输出路径 |
| **文件写入** | 临时文件 → 输出累积器 | `os.tmpdir()` |
| **子进程** | Bash 工具 → `spawn(shell, ...)` | 用户指定的任意 Shell 命令 |
| **子进程** | 工具管理 → `spawnSync("which", ...)` | 检测系统工具（fd, rg） |
| **子进程** | 版本检查 → `spawnSync` | npm registry 查询 |
| **终端输出** | TUI → `process.stdout.write` | 终端屏幕渲染 |
| **终端输出** | RPC 模式 → `writeRawStdout` → `process.stdout.write` | JSON 行协议 |
| **终端输出** | Print 模式 → `console.log` / `process.stdout.write` | 文本输出 |
| **终端输入** | TUI → `process.stdin`（原始模式） | 键盘按键事件 |
| **终端输入** | RPC 模式 → `stdin` JSONL 读取 | 外部进程的控制命令 |
| **终端输入** | CLI 管道 → `process.stdin` | 管道传入的初始消息 |
| **浏览器存储** | web-ui → IndexedDB | 会话、设置、密钥的浏览器端持久化 |

---

## 六、扩展注入点

从调用链中提取的 8 个扩展钩子，按执行顺序排列。

| 序号 | 钩子位置 | 截获的数据类型 | 可修改什么 | 阻止能力 |
|------|---------|--------------|-----------|---------|
| 1 | `agent-loop.ts:329` `transformContext(messages, signal)` | `AgentMessage[]`（完整消息历史） | 裁剪、追加、重排消息 | 否（只能变换） |
| 2 | `agent-loop.ts:333` `convertToLlm(messages)` | `AgentMessage[]` → 需返回 `Message[]` | 自定义消息类型映射为 LLM 格式 | 否（必须返回） |
| 3 | `agent-loop.ts:275` `shouldStopAfterTurn(context)` | 当前轮次的 `{ message, toolResults }` | — | **是**：返回 true 停止循环 |
| 4 | `agent-loop.ts:288` `getSteeringMessages()` | 无输入 → 需返回 `AgentMessage[]` | 注入转向消息 | 否 |
| 5 | `agent-loop.ts:292` `getFollowUpMessages()` | 无输入 → 需返回 `AgentMessage[]` | 排队后续消息 | 否 |
| 6 | `openai-completions.ts:218` `onPayload(payload, model)` | `ChatCompletionCreateParams` | 修改 HTTP 请求体 | 否 |
| 7 | `agent-loop.ts:664` `beforeToolCall(context, signal)` | `{ assistantMessage, toolCall, args, context }` | 修改参数 | **是**：返回 `{ block: true }` 阻止执行 |
| 8 | `agent-loop.ts:755` `afterToolCall(context, signal)` | `{ assistantMessage, toolCall, args, result, isError }` | 覆盖内容/错误/终止标志 | 否（只能修改结果） |

**额外扩展点**（不在核心调用链中但在扩展 API 中可用）：

| 钩子 | 触发时机 | 截获数据 |
|------|---------|---------|
| `sdk.ts:401` `onPayload` | 发送 HTTP 前 | `ChatCompletionCreateParams` |
| `sdk.ts:409` `onResponse` | 收到 HTTP 响应后 | `{ status, headers }` |
| `sdk.ts:422` `transformContext` | Agent 层的上下文变换 | `AgentMessage[]` |
| `onKey(key)` | 键盘按下 | `Key` 对象 |
| `onSessionStart/Shutdown` | 会话生命周期 | `SessionStartEvent/ShutdownEvent` |
| Extension 31 种事件类型 | 全生命周期 | 根据事件类型分发不同数据 |

---

## 七、类型约束传播链

追踪泛型约束 `<TApi extends Api>` 在调用链中的传播。

```
第 0 层：类型定义
  types.ts  export type Api = KnownApi | (string & {})
  types.ts  export type KnownProvider = "openai" | "anthropic" | ...  (32 个)

第 1 层：类型常量
  models.generated.ts  Model<"openai-completions"> 实例
  → model.api 被编译器确定为字面量 "openai-completions"

第 2 层：泛型分发
  stream.ts:56  function stream<TApi extends Api>(model: Model<TApi>)
  → TApi 从 model 参数自动推断
  → 约束：context.tools 的 Schema 类型由 toolSchemaFor<TApi> 确定

第 3 层：注册表查找
  stream.ts:41  resolveApiProvider(model.api)
  → api-registry.ts:140  Map.get(api) → RegisteredApiProvider
  → 约束：必须已注册对应 api 的 provider，否则运行时抛错

第 4 层：Options 泛型映射
  types.ts  ApiOptionsMap[TApi] → 确定 options 的具体类型
  → 约束："openai-completions" 的 options 必须是 OpenAICompletionsOptions

第 5 层：Provider 实现
  openai-completions.ts  function streamOpenAICompletions(model, context, options)
  → 约束：运行时校验 model.api === "openai-completions"，不匹配抛错
  → 约束：compat 对象按 provider 名称和 baseUrl 自动检测特有行为

第 6 层：记录类型强制
  coding-agent/src/core/model-resolver.ts
  → Record<KnownProvider, string> 确保 defaultModelPerProvider 覆盖全部 32 个提供商
  → 添加新提供商时 TypeScript 编译错误 → 强制补全
```

**类型安全覆盖边界**：
- **编译期保证**：provider/api/model 的三元绑定
- **运行时校验**：注册表查找 + `model.api === api` 双重检验
- **配置驱动**：compat 对象的自动检测按 provider name + baseUrl 匹配

---

## 八、配置驱动的行为分叉

追踪关键配置字段如何影响调用链的不同分支。

### 8.1 thinkingLevel → 思考 Token 预算

```
thinkingLevel: "high"
  ↓
simple-options.ts:27  buildBaseOptions(model, options)
  ├── thinkingLevel === "off"  → reasoningEffort = undefined [不启用思考]
  ├── thinkingLevel === "minimal" → budget = 1024, effort = "minimal"
  ├── thinkingLevel === "low"    → budget = 2048, effort = "low"
  ├── thinkingLevel === "medium" → budget = 8192, effort = "medium"
  ├── thinkingLevel === "high"   → budget = 16384, effort = "high"
  └── thinkingLevel === "xhigh" → 先 clamp: 不支持 xhigh 则降级为 high
  ↓
openai-completions.ts:637  buildParams()
  └── compat.thinkingFormat 决定如何编码：
        ├── "openai"  → { reasoning_effort }
        ├── "openrouter" → { reasoning: { effort } }
        ├── "deepseek"   → { thinking: { type } } + reasoning_effort
        ├── "together"   → { reasoning: { enabled } }
        ├── "zai"        → { enable_thinking: boolean }
        └── "qwen"       → { enable_thinking: boolean }
  ↓
[副作用] HTTP payload 中的思考参数字段名不同
[可观测] LLM 响应中出现 thinking_delta 事件（compared to 无思考时）
```

### 8.2 tools → 可用工具范围

```
tools: ["read"]         ← CLI --tools read
  ↓
sdk.ts:322  initialActiveToolNames = ["read"]
  ↓
AgentSession._buildRuntime({ activeToolNames: ["read"] })
  ↓
只有 read 工具被注入 Agent.state.tools
  ↓
agent-loop.ts 构建 llmContext.tools 时只有 read 的 schema
  ↓
LLM 提示中只有 read 工具的的描述 → LLM 只能请求 read
  [类型约束 + 提示约束] 双重约束，编译期 + 运行时 LLM 理解
```

### 8.3 noTools → 工具完全禁用

```
noTools: "all"          ← CLI --no-tools 或 -nt
  ↓
sdk.ts:322  initialActiveToolNames = []
  ↓
Agent.state.tools = []  → LLM 无法调用任何内置工具
  ↓
[副作用] 系统提示简化（无工具描述块）
[类型约束] 空的 tools 数组导致 llmContext.tools = []
```

### 8.4 blockImages → 图片阻断

```
blockImages: true
  ↓
sdk.ts:332  convertToLlmWithBlockImages(messages)
  ↓
settingsManager.getBlockImages() 动态检查（运行时可变）
  ↓
所有消息中的 ImageContent → "Image reading is disabled." 文本占位
  ↓
[类型约束] image 内容块被类型擦除为 text 内容块
[副作用] 无图片 base64 数据发送给 LLM（减少 Token 消耗）
```

### 8.5 会话恢复 vs 新建

```
--continue / --resume
  ↓
main.ts:384  selectSession(...) [副作用：TUI 会话选择器]
  ↓
sessionManager.buildSessionContext()
  ├── [算法] 从 JSONL 叶到根遍历树条目
  ├── [算法] 压缩条目 → 用摘要消息替换被压缩的旧消息
  ├── [数据结构] 恢复 model.provider + model.modelId + thinkingLevel
  └── [数据结构] 恢复完整 messages[]
  ↓
sdk.ts:266  modelRegistry.find(oldProvider, oldModelId)
  ├── 找到且已认证 → 恢复原模型
  └── 未找到或未认证 → 生成 modelFallbackMessage，用 findInitialModel() 替代
```

---

## 九、数据流串联图

以下是一次完整的 `pi -p "列出所有 .ts 文件"` 的数据转换链：

```
CLI 参数 ["-p", "列出所有 .ts 文件"]
  │  [算法] args.ts:63  parseArgs()
  ▼
Args { print: true, messages: ["列出所有 .ts 文件"] }
  │  [算法] main.ts 服务创建
  ▼
AgentSession { agent: Agent { state: AgentState } }
  │  [算法] agent-loop.ts:128  runAgentLoop()
  │          字符串 → AgentMessage → 追加  →
  ▼
AgentMessage[]: [{ role: "user", content: "列出所有 .ts 文件" }]
  │  [算法] agent-loop.ts:329  transformContext()
  │          [算法] agent-loop.ts:333  convertToLlm()
  ▼
Message[]: [{ role: "user", content: "列出所有 .ts 文件" }]
  │  [算法] openai-completions.ts:930  convertMessages()
  │          [算法] openai-completions.ts:1190  convertTools()
  ▼
ChatCompletionParams { model: "...", messages: [...], tools: [...] }
  │  [副作用] HTTP POST 到 LLM
  │  [算法] SSE 流式解析 (357-479 行状态机)
  ▼
AssistantMessageEventStream { text_delta, text_delta, ..., done }
  │  [算法] for await 消费事件
  ▼
AssistantMessage {
  content: [{ type: "tool_call", name: "find", args: { pattern: "**/*.ts" } }],
  stopReason: "toolUse",
  usage: Usage,
  cost: CostTotal
}
  │  [算法] agent-loop.ts:424  executeToolCalls()
  │          [副作用] find 工具 → glob 文件系统扫描
  ▼
ToolResultMessage[]: [{ role: "toolResult", content: "src/main.ts\nsrc/cli.ts\n...", toolCallId: "..." }]
  │  [算法] 追加到 context.messages → 循环回到 LLM 调用
  ▼
(LLM 处理工具结果后)
AssistantMessage {
  content: [{ type: "text", text: "找到以下 .ts 文件：\nsrc/main.ts\nsrc/cli.ts\n..." }],
  stopReason: "stop"
}
  │  [副作用] JSON/文本输出到 stdout
  │  [副作用] session-manager.ts JSONL 追加持久化
  ▼
[终端输出] 　　(list of .ts files displayed)
```
