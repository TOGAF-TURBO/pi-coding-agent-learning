# Terminology Index

本项目使用的核心术语及其含义。

## 架构与核心概念

### API 提供商（API Provider）

封装了特定 LLM 服务（如 OpenAI、Anthropic、Google Gemini）的流式调用能力的模块。每个提供商实现统一的 `ApiProvider` 接口，包含 API 标识符、标准流式函数和简化流式函数。

来源：`packages/ai/src/api-registry.ts`

- 外部接口 `ApiProvider<TApi, TOptions>`：保留完整泛型类型信息
- 内部存储 `ApiProviderInternal`：泛型已擦除为统一类型，用于注册表存储

### API 标识符（API Identifier）

字符串类型的唯一键，用于区分不同的 LLM 服务 API（如 `"openai-completions"`、`"anthropic"`）。在注册表中作为 `Map` 的键，也在模型与提供商的匹配校验中使用。

来源：`packages/ai/src/api-registry.ts`

### 注册表（Registry）

全局的 `Map<string, RegisteredApiProvider>` 数据结构，以 API 标识符为键存储所有已注册的 API 提供商。提供注册、查询、按来源注销和清空操作。

来源：`packages/ai/src/api-registry.ts`

### 来源标识（Source ID）

可选的字符串标识符，在注册 API 提供商时传入，用于按来源批量注销（如插件卸载时清理其注册的所有提供商）。

来源：`packages/ai/src/api-registry.ts`

### 模型（Model）

LLM 模型的完整定义，包含 ID、名称、API 标识符、提供商、能力（推理、视觉等）、成本费率、上下文窗口大小等元信息。模型数据由 `scripts/generate-models.ts` 自动生成到 `models.generated.ts`。

来源：`packages/ai/src/types.ts`、`packages/ai/src/models.ts`

### 提供商标识符（Provider Identifier）

LLM 服务提供商的标识符（如 `"openai"`、`"anthropic"`、`"google"`），与 API 标识符不同，一个提供商可对应多个 API（如 OpenAI 有 completions 和 responses 两种 API）。

来源：`packages/ai/src/types.ts`

### 思考级别（Thinking Level）

模型推理/思考的强度级别，取值为 `"off"` | `"minimal"` | `"low"` | `"medium"` | `"high"` | `"xhigh"`。不同模型支持的级别不同，`thinkingLevelMap` 定义了每个级别到提供商特定参数的映射。

来源：`packages/ai/src/types.ts`、`packages/ai/src/models.ts`

### 工具（Tool）

定义了 LLM 可调用的函数接口，包含名称、描述、JSON Schema 参数定义。在 Agent 层扩展为 `AgentTool`，增加执行函数和执行模式。

来源：`packages/ai/src/types.ts`

### 工具调用（Tool Call）

助手消息中的内容块，表示 LLM 请求执行某个工具。包含工具名称、调用 ID 和参数。Agent 循环检测到工具调用后会执行对应工具并将结果反馈给 LLM。

来源：`packages/ai/src/types.ts`、`packages/agent/src/types.ts`

## 流式调用

### 流式函数（Stream Function）

向 LLM 发起请求并返回 `AssistantMessageEventStream` 的函数，接收模型实例、对话上下文和选项参数。分为两种：

- **标准流式函数**（Stream）：接收完整的 `StreamOptions`，支持所有配置项
- **简化流式函数**（StreamSimple）：接收 `SimpleStreamOptions`，映射为完整选项后调用标准流式函数

来源：`packages/ai/src/api-registry.ts`

### 流式调用入口（Stream Entry）

`packages/ai/src/stream.ts` 提供的四个顶层函数：

- `stream()`：流式调用，实时返回事件流
- `complete()`：非流式调用，等待完整响应后返回
- `streamSimple()`：简化流式调用，使用 SimpleStreamOptions
- `completeSimple()`：简化非流式调用

这些函数内部通过 `resolveApiProvider()` 查找注册的提供商，然后委托给具体提供商的流式函数。

来源：`packages/ai/src/stream.ts`

### 助手消息事件流（Assistant Message Event Stream）

LLM 流式响应的抽象表示，以事件流（text、tool_call、thinking、usage、stop 等）的形式逐步产出助手消息内容。

来源：`packages/ai/src/api-registry.ts`

### 对话上下文（Context）

包含当前对话历史、系统提示等信息的结构，作为流式函数的必要参数传入，为 LLM 提供完整的对话背景。

来源：`packages/ai/src/api-registry.ts`

### 传输方式（Transport）

LLM 调用的传输协议选项，如 `"auto"`、`"sse"`、`"websocket"` 等。不同提供商和 API 支持的传输方式不同。

来源：`packages/ai/src/types.ts`

### 缓存保留（Cache Retention）

LLM API 的提示缓存策略（如 Anthropic 的 `"ephemeral"`）。控制对话上下文中的消息和工具在 API 侧的缓存行为。

来源：`packages/ai/src/types.ts`

## 类型系统

### 类型擦除（Type Erasure）

将泛型参数具体化为最宽泛的基础类型，使不同具体类型的实例可以统一存储和操作。

在本项目中的典型应用：`packages/ai/src/api-registry.ts`

- **擦除前**：每个 API 提供商带有自己的泛型参数（如 `StreamFunction<"openai", OpenAIOptions>`、`StreamFunction<"anthropic", AnthropicOptions>`），类型不同，无法放入同一个 `Map`
- **擦除后**：通过 `wrapStream` / `wrapStreamSimple` 将所有提供商的流式函数统一包装为 `ApiStreamFunction`（签名为 `Model<Api>` + `StreamOptions`），存入 `Map<string, RegisteredApiProvider>`
- **类型安全**：`wrapStream` 在运行时校验 `model.api !== api` 时不匹配则抛错，确保擦除不丢失类型约束

类比 Java 泛型擦除：`List<String>` 在运行时变为 `List`，此处同理将具体泛型参数降级为基础类型以实现统一存储。

## Agent 层

### Agent 消息（AgentMessage）

标准 LLM 消息（UserMessage、AssistantMessage、ToolResultMessage）和自定义消息的联合类型。通过 TypeScript 声明合并支持应用层扩展自定义消息类型。

来源：`packages/agent/src/types.ts`

### Agent 循环（Agent Loop）

Agent 的核心迭代过程：
1. 调用 LLM 获取助手响应
2. 如果响应包含工具调用，执行工具并将结果反馈给 LLM
3. 重复直到 LLM 不再请求工具调用或触发停止条件

双层循环结构：内层处理工具调用+转向消息，外层处理后续消息。

来源：`packages/agent/src/agent-loop.ts`

### Agent 状态（AgentState）

Agent 的公开可变状态，包括系统提示、模型、思考级别、工具列表、消息历史、流式传输状态等。`tools` 和 `messages` 的赋值会自动复制数组。

来源：`packages/agent/src/types.ts`

### Agent 事件（AgentEvent）

Agent 发出的生命周期事件，按层级分为：
- Agent 级别：`agent_start`、`agent_end`
- 轮次级别：`turn_start`、`turn_end`
- 消息级别：`message_start`、`message_update`、`message_end`
- 工具执行：`tool_execution_start`、`tool_execution_update`、`tool_execution_end`

来源：`packages/agent/src/types.ts`

### 工具执行模式（Tool Execution Mode）

控制助手消息中多个工具调用的执行策略：
- `"sequential"`：逐个准备、执行和完成每个工具调用
- `"parallel"`：依次准备后并发执行，按完成顺序发出事件，按原始顺序发出消息

来源：`packages/agent/src/types.ts`

### 转向消息（Steering Message）

在助手轮次完成后注入对话的消息，用于在 Agent 工作期间引导其方向。通过 `Agent.steer()` 入队，由 Agent 循环在内层循环末尾轮询。

来源：`packages/agent/src/agent.ts`

### 后续消息（Follow-up Message）

在 Agent 即将停止时才处理的消息，通过 `Agent.followUp()` 入队。如果没有工具调用且没有转向消息，循环会检查后续队列，有则继续。

来源：`packages/agent/src/agent.ts`

### 上下文变换（Transform Context）

在 `convertToLlm` 之前对 `AgentMessage[]` 进行的可选变换，适用于上下文窗口管理（裁剪旧消息）或从外部来源注入上下文。

来源：`packages/agent/src/types.ts`

### 消息转换（Convert to LLM）

将 `AgentMessage[]` 转换为 LLM 可理解的 `Message[]`。无法转换的消息（如仅用于 UI 的通知）应被过滤掉。这是 Agent 层与 AI 层的边界。

来源：`packages/agent/src/types.ts`

### 工具钩子（Tool Hooks）

工具执行前后的拦截器：
- `beforeToolCall`：参数校验后、执行前调用，可阻止执行
- `afterToolCall`：执行完成后、发出事件前调用，可覆盖结果的部分字段

来源：`packages/agent/src/types.ts`

### 提前终止（Early Termination）

工具结果的 `terminate` 标志。当批次中所有工具结果都设为 `true` 时，Agent 循环在当前批次完成后停止。

来源：`packages/agent/src/types.ts`

## 认证与授权

### OAuth 提供商（OAuth Provider）

实现了 OAuth 认证流程的模块，负责引导用户完成浏览器授权并获取访问凭证。每个 OAuth 提供商有唯一的 `OAuthProviderId` 标识。

来源：`packages/ai/src/cli.ts`

### OAuth 凭证（OAuth Credentials）

通过 OAuth 流程获取的认证凭证，与提供商 ID 一起保存到本地 `auth.json` 文件中，格式为 `{ type: "oauth", ...credentials }`。

来源：`packages/ai/src/cli.ts`

### 认证文件（Auth File）

本地 JSON 文件（`auth.json`），按提供商 ID 为键存储各服务的 OAuth 凭证。CLI 工具的 `login` 命令写入，运行时读取。

来源：`packages/ai/src/cli.ts`

## 成本与计量

### Token 用量（Usage）

LLM 调用的 Token 消耗统计，包含 input、output、cacheRead、cacheWrite 四个维度的 Token 数和对应成本。成本以美元/百万 Token 为费率单位。

来源：`packages/ai/src/types.ts`、`packages/ai/src/models.ts`

### 延迟加载（Lazy Loading）

API 提供商的加载策略。通过 `createLazyStream` / `createLazySimpleStream` 创建包装函数，提供商的实现模块仅在首次实际调用 `stream()` 或 `streamSimple()` 时才被动态导入（`import()`），而非启动时加载所有模块。每个模块使用 `||=` 运算符缓存 Promise，确保只加载一次。

来源：`packages/ai/src/providers/register-builtins.ts`

### 消息转换（Transform Messages）

在发送对话历史到 LLM 前的规范化处理流程，包括：图片降级（不支持视觉的模型替换图片为占位文本）、思考块处理（跨模型时移除加密签名或降级为普通文本）、工具调用 ID 规范化（不同提供商对 ID 格式有不同要求）、孤儿工具调用修复（为缺少结果的工具调用插入合成错误结果）。

来源：`packages/ai/src/providers/transform-messages.ts`

### 图片降级（Image Downgrade）

当目标模型不支持视觉能力（`model.input` 不包含 `"image"`）时，将消息中的图片内容块替换为文本占位符的过程。连续多张图片合并为一个占位符，避免消息中出现大量重复提示。

来源：`packages/ai/src/providers/transform-messages.ts`

### 孤儿工具调用（Orphaned Tool Call）

助手消息中包含工具调用，但对话历史中缺少对应的工具结果消息。LLM API 要求每个工具调用都必须有对应结果，否则会报错。消息转换模块会为孤儿工具调用自动插入合成的错误结果（`"No result provided"`）。

来源：`packages/ai/src/providers/transform-messages.ts`

### 思考预算（Thinking Budget）

启用思考/推理模式时分配给模型内部推理过程的 Token 数量。根据思考级别分配（minimal=1024、low=2048、medium=8192、high=16384），总预算 = 基础输出 + 思考预算，不超过模型上限。当预算不足时优先保证输出空间，缩减思考预算。

来源：`packages/ai/src/providers/simple-options.ts`

### 兼容性设置（Compat）

OpenAI Completions 提供商的兼容性配置对象（`OpenAICompletionsCompat`），用于适配 30+ 家不同提供商的行为差异。包括思考格式（thinkingFormat）、Token 字段名（maxTokensField）、角色支持（supportsDeveloperRole）、工具结果格式等。可通过模型定义显式指定，或由 `detectCompat()` 从提供商名称和 Base URL 自动检测。

来源：`packages/ai/src/providers/openai-completions.ts`

### 内容块状态机（Streaming Block State Machine）

流式响应解析过程中的状态管理机制。维护三类活跃内容块（文本块、思考块、工具调用块），每收到一个 SSE chunk 时更新对应块的增量内容。文本和思考块为单实例（同一时间只有一个活跃块），工具调用块按流索引和 ID 双重索引支持多个并发块。块完成时发出结束事件并清理临时字段。

来源：`packages/ai/src/providers/openai-completions.ts`

### 完成原因映射（Stop Reason Mapping）

将 OpenAI 的 `finish_reason`（如 `"stop"`、`"tool_calls"`、`"length"`、`"content_filter"`）转换为 pi-ai 的标准 `StopReason`（`"stop"`、`"toolUse"`、`"length"`、`"error"`）的映射过程。

来源：`packages/ai/src/providers/openai-completions.ts`

### 执行环境（ExecutionEnv）

文件系统和进程操作的抽象接口，由 AgentHarness 使用。提供文件读写、目录操作、命令执行和临时文件创建等能力。所有路径相对于 `cwd` 解析，文件操作失败抛出 `FileError`。默认实现为 `NodeExecutionEnv`（基于 Node.js 的 `fs` 和 `child_process`）。

来源：`packages/agent/src/harness/types.ts`

### 变更缓冲（Pending Mutations）

AgentHarness 的线程安全机制。在 Agent 操作进行中，模型切换、思考级别变更、消息追加等操作暂存为 `pendingMutations`，在轮次结束时（`turn_end` 事件）批量写入会话存储，避免操作中途产生不一致的会话状态。

来源：`packages/agent/src/harness/agent-harness.ts`

### 事件钩子（Event Hook）

AgentHarness 的扩展机制。通过 `harness.on(eventType, handler)` 注册，与 `subscribe` 不同，钩子可以返回结果来修改行为（如阻止工具执行、覆盖工具结果、取消压缩等）。同一类型的多个钩子按注册顺序调用，后者的返回值覆盖前者。

来源：`packages/agent/src/harness/agent-harness.ts`

### 存档点（Save Point）

Agent 循环在每个轮次结束时发出的安全保存点事件。表示此时会话状态一致，所有暂存变更已应用。用于扩展在安全时机触发检查点操作（如 Git 提交）。

来源：`packages/agent/src/harness/agent-harness.ts`

### 会话上下文重建（Session Context Rebuild）

`buildSessionContext()` 函数从会话树的叶到根遍历条目，重建完整的消息历史、思考级别和模型信息。处理压缩条目时，用摘要消息替代被压缩的旧消息，从 `firstKeptEntryId` 开始保留后续消息。是 `session.buildContext()` 的核心逻辑。

来源：`packages/agent/src/harness/session/session.ts`

### 技能发现（Skill Discovery）

从目录递归加载 SKILL.md 文件的机制。每个目录优先查找 SKILL.md，找到则加载并停止递归；否则继续搜索子目录。遵循 `.gitignore`/`.ignore`/`.fdignore` 规则。技能名称必须与父目录名匹配，格式为小写字母、数字和连字符。

来源：`packages/agent/src/harness/skills.ts`

### 消息类型转换（Message Type Conversion）

`convertToLlm()` 将 Agent 层的扩展消息类型（bashExecution、custom、branchSummary、compactionSummary）转换为 pi-ai 的标准 `Message` 格式。所有自定义类型最终映射为 `user` 角色消息，摘要类消息包裹在 `<summary>` XML 标签中，`excludeFromContext` 标记的消息被过滤掉。

来源：`packages/agent/src/harness/messages.ts`

### 扩展系统（Extension System）

pi 的插件架构，允许 TypeScript 模块通过 `ExtensionAPI` 注册工具、命令、快捷键、事件处理器和自定义提供商。扩展通过工厂函数（`export default function(pi: ExtensionAPI)`) 注册能力，支持 30+ 种事件类型的订阅。扩展发现规则：本地 `.pi/extensions/`、全局 `~/.pi/agent/extensions/`、配置路径，子目录支持 `package.json#pi.extensions` 清单。

来源：`packages/coding-agent/src/core/extensions/types.ts`

### 扩展加载器（Extension Loader）

使用 jiti（TypeScript JIT 编译器）动态加载扩展模块。Bun 二进制模式通过 `virtualModules` 提供已打包的包引用，Node.js 模式通过 `alias` 解析到 `node_modules`。加载器创建初始为抛出异常的运行时桩，`bindCore()` 后替换为真正实现，确保加载期间扩展只能注册不能操作。

来源：`packages/coding-agent/src/core/extensions/loader.ts`

### AgentSession（代理会话）

所有运行模式（交互式、打印、RPC）共享的核心会话类。在 Agent 之上封装了模型管理、上下文压缩、Bash 执行、会话导航与分支、扩展集成、自动重试等完整功能。通过 `subscribe()` 监听会话级事件（队列更新、压缩进度、自动重试），通过 `bindExtensions()` 激活扩展系统。

来源：`packages/coding-agent/src/core/agent-session.ts`

### 内置工具集（Built-in Tools）

pi 提供 7 种内置工具，分为编码工具（read/bash/edit/write，可读写文件和执行命令）和只读工具（read/grep/find/ls，仅查询不修改）。每种工具都有 `ToolDefinition`（静态定义，包含参数 schema、描述、渲染器）和 `AgentTool`（运行时实例）两种形态。工具通过 `BashOperations`/`ReadOperations` 等接口实现可插拔后端，支持替换为远程执行。

来源：`packages/coding-agent/src/core/tools/index.ts`

### 上下文压缩（Context Compaction）

当对话历史接近模型上下文窗口限制时，将旧消息摘要为压缩摘要消息，保留最近消息继续对话。`prepareCompaction()` 确定压缩点和保留范围，`compact()` 调用 LLM 生成摘要，`shouldCompact()` 根据配置和 Token 使用率判断是否需要压缩。支持手动和自动触发。

来源：`packages/coding-agent/src/core/compaction/compaction.ts`

### 会话管理器（Session Manager）

基于 JSONL 文件的会话存储后端。每行存储一个树条目（消息、压缩、标签等），通过 parentId 形成分支树。提供消息追加、压缩、分支导航、标签、会话列表/删除、分叉、HTML 导出等操作。`buildContext()` 从当前分支重建消息历史。

来源：`packages/coding-agent/src/core/session-manager.ts`

### 交互模式（Interactive Mode）

pi 的主要运行模式，提供完整的终端用户界面：输入编辑器、消息流式渲染、会话导航、模型选择、内置命令、键盘快捷键、扩展 UI 对话框。业务逻辑委托给 AgentSession。

来源：`packages/coding-agent/src/modes/interactive/interactive-mode.ts`

### TUI 引擎（Terminal UI Engine）

最小化终端 UI 引擎，基于差异渲染。核心概念：Component（渲染单元）、差异计算（仅重绘变化行）、输入处理（终端原始输入→按键事件）、Kitty 图片协议（终端内嵌图片）、Overlay（浮层组件）。

来源：`packages/tui/src/tui.ts`

### 编辑器组件（Editor Component）

多行文本编辑器，支持光标移动、选择/剪贴板、Kill Ring、撤销/重做、自动补全、粘贴折叠、Vim 模式。使用 Intl.Segmenter 正确处理 Unicode grapheme cluster。

来源：`packages/tui/src/components/editor.ts`

### 成本计算（Cost Calculation）

`calculateCost()` 函数根据模型的费率（input/output/cacheRead/cacheWrite）和实际 Token 用量计算成本。费率单位为美元/百万 Token。

来源：`packages/ai/src/models.ts`

## 运行模式

### 模式分发（Mode Dispatch）

根据 CLI 标志（默认/`-p`/`--mode rpc`）将程序路由到交互式、Print 或 RPC 三种运行模式之一的分发机制。三种模式共享同一个 AgentSession 内核，差异仅在外层入口。

来源：`packages/coding-agent/src/main.ts`

### Print 模式（Print Mode）

非交互式运行模式（通过 `-p`/`--print` 触发），适用于 CI/CD 和脚本调用。处理完用户消息后直接退出，输出纯文本而不启动 TUI。

来源：`packages/coding-agent/src/modes/print-mode.ts`

### RPC 模式（RPC Mode）

无头运行模式（通过 `--mode rpc` 触发），通过 stdin/stdout 的 JSON 行协议与其他进程通信。支持 27 种远程操作命令，适用于将 pi 嵌入其他应用程序。

来源：`packages/coding-agent/src/modes/rpc/rpc-mode.ts`

### RPC 命令（RPC Command）

RPC 模式下通过 stdin 发送的 JSON 命令联合类型，包含 27 种操作：对话控制（prompt/steer/abort）、模型切换（set_model/cycle_model）、会话管理（switch_session/fork）、压缩（compact）等。

来源：`packages/coding-agent/src/modes/rpc/rpc-types.ts`

### stdout 接管（stdout Takeover）

替换 `process.stdout.write` 为自定义实现，确保非预期的第三方写入不会污染输出流。Print 和 RPC 模式在启动时接管 stdout，仅允许通过 `writeRawStdout()` 输出原始字节。

来源：`packages/coding-agent/src/core/output-guard.ts`

### JSON 行协议（JSONL Line Protocol）

RPC 模式的通信协议：stdin 逐行读取 JSON 命令，stdout 逐行输出 JSON 响应和事件。`attachJsonlLineReader()` 负责从 Readable 流中按行解析 JSON。

来源：`packages/coding-agent/src/modes/rpc/jsonl.ts`

### JSONL 会话文件（JSONL Session File）

会话持久化的文件格式，每行一个 JSON 对象表示一条会话条目（消息、压缩、标签等）。通过 `parentId` 字段形成分支树结构，支持任意深度的分支导航和回退。

来源：`packages/coding-agent/src/core/session-manager.ts`

### 会话恢复与会话新建（Session Resume vs. New）

通过 `--continue`/`--resume` 标志触发的会话恢复流程：`SessionManager.buildSessionContext()` 从 JSONL 文件的叶到根遍历树条目，恢复完整消息历史、模型信息和思考级别。若原模型不可用则生成 `modelFallbackMessage` 并用 `findInitialModel()` 替代。

来源：`packages/coding-agent/src/core/session-manager.ts`、`packages/coding-agent/src/core/sdk.ts`

### 命令树（Command Tree）

pi CLI 的命令层级结构：主入口 `pi [options] [messages...]` 下挂载 6 个子命令（`install`/`remove`/`uninstall`/`update`/`list`/`config`），外加 `--list-models` 标志。

来源：`packages/coding-agent/src/main.ts`、`packages/coding-agent/src/cli/args.ts`

## 架构模式

### 类型约束传播链（Type Constraint Propagation Chain）

泛型约束 `<TApi extends Api>` 在调用链中逐层传递的机制：从 `Model<TApi>` 的类型定义开始，经过 `stream<TApi>` 的泛型分发、`resolveApiProvider` 的注册表查找，到 Provider 实现的运行时校验，形成编译期+运行时的双重类型安全保障。

来源：`packages/ai/src/stream.ts`

### 泛型分发（Generic Dispatch）

根据 `model.api` 字面量类型自动选择对应 Provider 实现的编译期机制。`stream<TApi>(model, ...)` 从 model 参数自动推断 TApi，然后通过 `resolveApiProvider(model.api)` 查找注册表中的匹配实现。

来源：`packages/ai/src/stream.ts`、`packages/ai/src/api-registry.ts`

### 模型绑定（Model Binding）

通过 `Model<TApi>` 泛型将模型实例与 API 协议在编译期绑定的设计。例如 `Model<"openai-completions">` 实例在编译期就只能传入 OpenAI Completions 调用链，防止协议错配。

来源：`packages/ai/src/types.ts`

### 协议适配链（Protocol Adaptation Chain）

消息在发送到 LLM 的 HTTP 请求前经过的完整处理链：`transformMessages`（图片降级/孤儿修复/ID 规范化）→ `convertMessages`（AgentMessage 到 LLM 格式）→ `convertTools`（Tool Schema 转换）→ `buildParams`（编码思考参数和 Compat 适配）。

来源：`packages/ai/src/providers/openai-completions.ts`

### 配置驱动的行为分叉（Configuration-driven Behavior Fork）

一个配置字段导致调用链走向不同分支的设计模式。例如 `thinkingLevel` 决定了思考 Token 预算分配（1024~16384），而 `compat.thinkingFormat` 又将同一语义编码为不同 HTTP 参数字段名。

来源：`packages/ai/src/providers/simple-options.ts`、`packages/ai/src/providers/openai-completions.ts`

### 扩展注入点（Extension Injection Points）

Agent 循环调用链中暴露给扩展系统的 8 个关键钩子位置。按执行顺序包括：`transformContext`、`convertToLlm`、`shouldStopAfterTurn`、`getSteeringMessages`、`getFollowUpMessages`、`onPayload`、`beforeToolCall`、`afterToolCall`。

来源：`packages/agent/src/agent-loop.ts`、`packages/coding-agent/src/core/sdk.ts`

### 钩子链与责任链（Hook Chain / Chain of Responsibility）

多层拦截器按顺序形成处理链的架构模式。核心三条链：工具执行链（`beforeToolCall` → `execute` → `afterToolCall`）、消息处理链（`transformContext` → `convertToLlm`）、HTTP 拦截链（`onPayload` → `stream` → `onResponse`）。

来源：`packages/agent/src/agent-loop.ts`、`packages/coding-agent/src/core/sdk.ts`

### 声明合并（Declaration Merging）

TypeScript 的语言特性，允许扩展通过重新声明同名 interface 向 `AgentMessageTypes` 添加自定义消息类型，无需修改框架代码。

来源：`packages/agent/src/types.ts`

### 扩展槽（Extension Slot）

联合类型中预留的 `(string & {})` 成员，允许用户使用任意字符串值而 TypeScript 类型检查不报错。例如 `Api = KnownApi | (string & {})` 使得内置 9 种 API 协议之外还能动态扩展。

来源：`packages/ai/src/types.ts`

### 零必填初始化（Zero-Required Initialization）

架构设计原则：`AgentOptions` 的 16 个字段和 `CreateAgentSessionOptions` 的 14 个字段全部可选。Agent 和 AgentSession 可用零参数启动并具备最小默认行为。

来源：`packages/agent/src/types.ts`、`packages/coding-agent/src/core/sdk.ts`

### Schema 运行时校验（Schema Runtime Validation）

使用 TypeBox 在运行时验证 LLM 工具调用参数的形状。在 `prepareToolCallArguments` 阶段通过 TypeBox Schema 检验 LLM 生成的参数是否符合工具定义，防止无效参数进入执行阶段。

来源：`packages/agent/src/agent-loop.ts`

### 阻塞图片（Block Images）

通过 `blockImages` 配置全局禁用图片处理的机制。启用时将所有消息中的 `ImageContent` 替换为文本占位符，避免图片 base64 数据消耗 Token。该设置运行时动态可变。

来源：`packages/coding-agent/src/core/sdk.ts`

### 副作用全景图（Side Effect Panorama）

以程序中所有 `import from "node:*"` 和 `fetch()` 为线索穷举的全部 I/O 边界。涵盖 6 大类副作用：HTTP 请求（LLM API 调用）、文件 I/O（认证/设置/会话文件）、子进程（Bash 执行）、终端 I/O（TUI 渲染/RPC 管道）、OAuth HTTP、浏览器存储（IndexedDB）。

来源：分散在 `packages/ai/src/providers/`、`packages/coding-agent/src/core/`、`packages/tui/src/` 中。

### Provider 矩阵（Provider Matrix）

测试中按提供商维度组织的参数化矩阵，每个提供商至少有一个代表性模型通过流式测试。目前覆盖 26 个提供商（81% 覆盖率），跨 10+ 种测试文件类型验证核心能力。

来源：`packages/ai/test/stream.test.ts`
