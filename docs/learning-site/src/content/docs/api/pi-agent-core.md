---
title: pi-agent-core API 参考
---


- [agentLoop](#gear-agentloop)
- [agentLoopContinue](#gear-agentloopcontinue)
- [runAgentLoop](#gear-runagentloop)
- [runAgentLoopContinue](#gear-runagentloopcontinue)

### :gear: agentLoop

启动带新提示消息的 Agent 循环（EventStream 版本）
提示被添加到上下文并发出对应事件

| Function | Type |
| ---------- | ---------- |
| `agentLoop` | `(prompts: AgentMessage[], context: AgentContext, config: AgentLoopConfig, signal?: AbortSignal or undefined, streamFn?: StreamFn or undefined) => EventStream<...>` |

Returns:

事件流，最终产出所有新增消息

### :gear: agentLoopContinue

从当前上下文继续 Agent 循环（EventStream 版本）
不添加新消息，用于重试 —— 上下文中已有用户消息或工具结果

**重要**：上下文中最后一条消息经 convertToLlm 后必须是 user 或 toolResult 类型，
否则 LLM 提供商会拒绝请求。

| Function | Type |
| ---------- | ---------- |
| `agentLoopContinue` | `(context: AgentContext, config: AgentLoopConfig, signal?: AbortSignal or undefined, streamFn?: StreamFn or undefined) => EventStream<...>` |

### :gear: runAgentLoop

启动带初始提示的完整 Agent 循环（async 版本）

| Function | Type |
| ---------- | ---------- |
| `runAgentLoop` | `(prompts: AgentMessage[], context: AgentContext, config: AgentLoopConfig, emit: AgentEventSink, signal?: AbortSignal or undefined, streamFn?: StreamFn or undefined) => Promise<...>` |

Parameters:

* `prompts`: - 初始提示消息数组
* `context`: - Agent 上下文快照
* `config`: - 循环配置
* `emit`: - 事件接收器
* `signal`: - 可选的中止信号
* `streamFn`: - 可选的自定义流式函数


Returns:

本次循环新增的所有消息

### :gear: runAgentLoopContinue

从现有上下文继续 Agent 循环（async 版本）
不追加新消息，直接从当前上下文开始

| Function | Type |
| ---------- | ---------- |
| `runAgentLoopContinue` | `(context: AgentContext, config: AgentLoopConfig, emit: AgentEventSink, signal?: AbortSignal or undefined, streamFn?: StreamFn or undefined) => Promise<...>` |


## :factory: Agent

Agent 主类 —— 底层 Agent 循环的有状态封装

Agent 拥有当前对话记录、发出生命周期事件、执行工具，
并提供消息队列 API 用于转向和后续消息注入。

事件层级：
  agent_start -> turn_start -> message_* -> tool_execution_* -> turn_end -> agent_end

### Methods

- [subscribe](#gear-subscribe)
- [steer](#gear-steer)
- [followUp](#gear-followup)
- [clearSteeringQueue](#gear-clearsteeringqueue)
- [clearFollowUpQueue](#gear-clearfollowupqueue)
- [clearAllQueues](#gear-clearallqueues)
- [hasQueuedMessages](#gear-hasqueuedmessages)
- [abort](#gear-abort)
- [waitForIdle](#gear-waitforidle)
- [reset](#gear-reset)

#### :gear: subscribe

订阅 Agent 生命周期事件
监听器 Promise 按订阅顺序等待，并包含在当前运行的结算中。
agent_end 是最终事件，但 Agent 在所有等待中的 agent_end 监听器完成后才变为空闲。

| Method | Type |
| ---------- | ---------- |
| `subscribe` | `(listener: (event: AgentEvent, signal: AbortSignal) => void or Promise<void>) => () => void` |

Parameters:

* `listener`: - 事件监听器，接收事件和中止信号


Returns:

取消订阅函数

#### :gear: steer

将消息入队到转向队列
消息将在当前助手轮次完成后注入

| Method | Type |
| ---------- | ---------- |
| `steer` | `(message: AgentMessage) => void` |

#### :gear: followUp

将消息入队到后续队列
消息仅在 Agent 即将停止时处理

| Method | Type |
| ---------- | ---------- |
| `followUp` | `(message: AgentMessage) => void` |

#### :gear: clearSteeringQueue

清空转向队列

| Method | Type |
| ---------- | ---------- |
| `clearSteeringQueue` | `() => void` |

#### :gear: clearFollowUpQueue

清空后续队列

| Method | Type |
| ---------- | ---------- |
| `clearFollowUpQueue` | `() => void` |

#### :gear: clearAllQueues

清空所有队列

| Method | Type |
| ---------- | ---------- |
| `clearAllQueues` | `() => void` |

#### :gear: hasQueuedMessages

任一队列是否还有待处理消息

| Method | Type |
| ---------- | ---------- |
| `hasQueuedMessages` | `() => boolean` |

#### :gear: abort

中止当前运行

| Method | Type |
| ---------- | ---------- |
| `abort` | `() => void` |

#### :gear: waitForIdle

等待当前运行和所有等待中的事件监听器完成
在 agent_end 监听器结算后 resolve

| Method | Type |
| ---------- | ---------- |
| `waitForIdle` | `() => Promise<void>` |

#### :gear: reset

清空对话状态、运行时状态和消息队列

| Method | Type |
| ---------- | ---------- |
| `reset` | `() => void` |

### Properties

- [convertToLlm](#gear-converttollm)
- [transformContext](#gear-transformcontext)
- [streamFn](#gear-streamfn)
- [getApiKey](#gear-getapikey)
- [onPayload](#gear-onpayload)
- [onResponse](#gear-onresponse)
- [beforeToolCall](#gear-beforetoolcall)
- [afterToolCall](#gear-aftertoolcall)
- [sessionId](#gear-sessionid)
- [thinkingBudgets](#gear-thinkingbudgets)
- [transport](#gear-transport)
- [maxRetryDelayMs](#gear-maxretrydelayms)
- [toolExecution](#gear-toolexecution)

#### :gear: convertToLlm

| Property | Type |
| ---------- | ---------- |
| `convertToLlm` | `(messages: AgentMessage[]) => Message[] or Promise<Message[]>` |

#### :gear: transformContext

| Property | Type |
| ---------- | ---------- |
| `transformContext` | `((messages: AgentMessage[], signal?: AbortSignal or undefined) => Promise<AgentMessage[]>) or undefined` |

#### :gear: streamFn

| Property | Type |
| ---------- | ---------- |
| `streamFn` | `StreamFn` |

#### :gear: getApiKey

| Property | Type |
| ---------- | ---------- |
| `getApiKey` | `((provider: string) => string or Promise<string or undefined> or undefined) or undefined` |

#### :gear: onPayload

| Property | Type |
| ---------- | ---------- |
| `onPayload` | `((payload: unknown, model: Model<Api>) => unknown) or undefined` |

#### :gear: onResponse

| Property | Type |
| ---------- | ---------- |
| `onResponse` | `((response: ProviderResponse, model: Model<Api>) => void or Promise<void>) or undefined` |

#### :gear: beforeToolCall

| Property | Type |
| ---------- | ---------- |
| `beforeToolCall` | `((context: BeforeToolCallContext, signal?: AbortSignal or undefined) => Promise<BeforeToolCallResult or undefined>) or undefined` |

#### :gear: afterToolCall

| Property | Type |
| ---------- | ---------- |
| `afterToolCall` | `((context: AfterToolCallContext, signal?: AbortSignal or undefined) => Promise<AfterToolCallResult or undefined>) or undefined` |

#### :gear: sessionId

会话标识符，转发给提供商用于缓存感知后端

| Property | Type |
| ---------- | ---------- |
| `sessionId` | `string or undefined` |

#### :gear: thinkingBudgets

思考级别 Token 预算，转发给流式函数

| Property | Type |
| ---------- | ---------- |
| `thinkingBudgets` | `ThinkingBudgets or undefined` |

#### :gear: transport

首选传输方式

| Property | Type |
| ---------- | ---------- |
| `transport` | `Transport` |

#### :gear: maxRetryDelayMs

提供商请求的重试延迟上限

| Property | Type |
| ---------- | ---------- |
| `maxRetryDelayMs` | `number or undefined` |

#### :gear: toolExecution

多工具调用的执行策略

| Property | Type |
| ---------- | ---------- |
| `toolExecution` | `ToolExecutionMode` |

## :tropical_drink: Interfaces

- [BeforeToolCallResult](#gear-beforetoolcallresult)
- [AfterToolCallResult](#gear-aftertoolcallresult)
- [BeforeToolCallContext](#gear-beforetoolcallcontext)
- [AfterToolCallContext](#gear-aftertoolcallcontext)
- [ShouldStopAfterTurnContext](#gear-shouldstopafterturncontext)
- [AgentLoopConfig](#gear-agentloopconfig)
- [CustomAgentMessages](#gear-customagentmessages)
- [AgentState](#gear-agentstate)
- [AgentToolResult](#gear-agenttoolresult)
- [AgentTool](#gear-agenttool)
- [AgentContext](#gear-agentcontext)
- [AgentOptions](#gear-agentoptions)

### :gear: BeforeToolCallResult

beforeToolCall 钩子的返回结果
返回 { block: true } 可阻止工具执行，循环会发出一个错误工具结果替代

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `block` | `boolean or undefined` | 是否阻止执行 |
| `reason` | `string or undefined` | 阻止原因，显示在错误工具结果中。省略则使用默认阻止消息 |


### :gear: AfterToolCallResult

afterToolCall 钩子的返回结果
各字段合并语义：提供的字段替换原始值，省略的字段保持原始值
不进行深度合并

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `content` | `(TextContent or ImageContent)[] or undefined` | 替换工具结果的完整内容数组 |
| `details` | `unknown` | 替换工具结果的完整详情载荷 |
| `isError` | `boolean or undefined` | 替换工具结果的错误标志 |
| `terminate` | `boolean or undefined` | 提示 Agent 在当前工具批次完成后停止 仅当批次中所有已完成的工具结果都设为 true 时才触发提前终止 |


### :gear: BeforeToolCallContext

beforeToolCall 钩子的上下文参数

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `assistantMessage` | `AssistantMessage` | 请求工具调用的助手消息 |
| `toolCall` | `ToolCall` | 助手消息中的原始工具调用块 |
| `args` | `unknown` | 经 Schema 校验后的工具参数 |
| `context` | `AgentContext` | 工具调用准备时的 Agent 上下文快照 |


### :gear: AfterToolCallContext

afterToolCall 钩子的上下文参数

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `assistantMessage` | `AssistantMessage` | 请求工具调用的助手消息 |
| `toolCall` | `ToolCall` | 助手消息中的原始工具调用块 |
| `args` | `unknown` | 经 Schema 校验后的工具参数 |
| `result` | `AgentToolResult<any>` | 应用 afterToolCall 覆盖之前的已执行工具结果 |
| `isError` | `boolean` | 当前工具结果是否被视为错误 |
| `context` | `AgentContext` | 工具调用完成时的 Agent 上下文快照 |


### :gear: ShouldStopAfterTurnContext

shouldStopAfterTurn 钩子的上下文参数

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `message` | `AssistantMessage` | 完成本轮的助手消息 |
| `toolResults` | `ToolResultMessage<any>[]` | 传递给前一个 turn_end 事件的工具结果消息 |
| `context` | `AgentContext` | 助手消息和工具结果追加后的 Agent 上下文 |
| `newMessages` | `AgentMessage[]` | 如果循环在此时退出将返回的消息。初始提示运行包含提示消息，续接运行不包含已有上下文消息 |


### :gear: AgentLoopConfig

Agent 循环的完整配置
继承 SimpleStreamOptions（温度、Token 上限、推理级别等），增加 Agent 特有的配置项

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `model` | `Model<any>` | 使用的模型 |
| `convertToLlm` | `(messages: AgentMessage[]) => Message[] or Promise<Message[]>` | 将 AgentMessage[] 转换为 LLM 可理解的 Message[]  每条 AgentMessage 必须被转换为 UserMessage、AssistantMessage 或 ToolResultMessage。 无法转换的消息（如仅用于 UI 的通知）应被过滤掉。  契约：不得抛出异常或 reject，应返回安全的回退值 |
| `transformContext` | `((messages: AgentMessage[], signal?: AbortSignal or undefined) => Promise<AgentMessage[]>) or undefined` | 在 convertToLlm 之前对上下文进行可选的变换  适用于在 AgentMessage 层面进行的操作： - 上下文窗口管理（裁剪旧消息） - 从外部来源注入上下文  契约：不得抛出异常或 reject |
| `getApiKey` | `((provider: string) => string or Promise<string or undefined> or undefined) or undefined` | 为每次 LLM 调用动态解析 API 密钥  适用于短期 OAuth Token（如 GitHub Copilot），这些 Token 可能在 长时间工具执行阶段过期  契约：不得抛出异常或 reject，无密钥时返回 undefined |
| `shouldStopAfterTurn` | `((context: ShouldStopAfterTurnContext) => boolean or Promise<boolean>) or undefined` | 每轮完全完成后（turn_end 已发出）调用  返回 true 时，循环发出 agent_end 并退出，不再轮询转向或后续队列。 当前助手响应和工具执行正常完成。  用于在上下文过满之前请求优雅停止  契约：不得抛出异常或 reject |
| `getSteeringMessages` | `(() => Promise<AgentMessage[]>) or undefined` | 运行期间注入转向消息  在当前助手轮次完成工具调用执行后调用（除非 shouldStopAfterTurn 先退出）。 返回的消息会在下一次 LLM 调用前添加到上下文中。 当前助手消息的工具调用不会被跳过。  契约：不得抛出或 reject，无转向消息时返回 [] |
| `getFollowUpMessages` | `(() => Promise<AgentMessage[]>) or undefined` | Agent 即将停止时返回后续消息进行处理  在 Agent 没有更多工具调用和转向消息时调用。 返回消息后，消息被添加到上下文，Agent 继续下一轮。  契约：不得抛出或 reject，无后续消息时返回 [] |
| `toolExecution` | `ToolExecutionMode or undefined` | 工具执行模式 - "sequential"：逐个执行工具调用 - "parallel"：顺序预检后并发执行，按完成顺序发出事件，按原始顺序发出消息 默认值："parallel" |
| `beforeToolCall` | `((context: BeforeToolCallContext, signal?: AbortSignal or undefined) => Promise<BeforeToolCallResult or undefined>) or undefined` | 工具执行前的钩子（参数校验完成后调用） 返回 { block: true } 阻止执行，循环发出错误工具结果替代 钩子接收到 Agent 中止信号，有责任响应它 |
| `afterToolCall` | `((context: AfterToolCallContext, signal?: AbortSignal or undefined) => Promise<AfterToolCallResult or undefined>) or undefined` | 工具执行完成后的钩子（在 tool_execution_end 和工具结果消息事件发出之前） 返回 AfterToolCallResult 可覆盖已执行工具结果的部分字段 钩子接收到 Agent 中止信号，有责任响应它 |


### :gear: CustomAgentMessages

自定义应用消息的可扩展接口
应用可通过声明合并扩展：

| Property | Type | Description |
| ---------- | ---------- | ---------- |


Examples:

```typescript
declare module "@mariozechner/agent" {
  interface CustomAgentMessages {
    artifact: ArtifactMessage;
    notification: NotificationMessage;
  }
}
```


### :gear: AgentState

Agent 的公开状态接口

tools 和 messages 使用访问器属性，实现可以在存储前复制赋值的数组

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `systemPrompt` | `string` | 每次模型请求时发送的系统提示 |
| `model` | `Model<any>` | 后续轮次使用的活跃模型 |
| `thinkingLevel` | `ThinkingLevel` | 后续轮次请求的推理级别 |
| `isStreaming` | `boolean` | Agent 正在处理提示或续接时为 true 直到等待中的 agent_end 监听器完成后才变为 false |
| `streamingMessage` | `AgentMessage or undefined` | 当前流式响应的部分助手消息（如存在） |
| `pendingToolCalls` | `ReadonlySet<string>` | 当前正在执行的工具调用 ID 集合 |
| `errorMessage` | `string or undefined` | 最近一次失败或中止的助手轮次的错误消息 |


### :gear: AgentToolResult

工具执行的最终或部分结果

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `content` | `(TextContent or ImageContent)[]` | 返回给模型的文本或图像内容 |
| `details` | `T` | 用于日志或 UI 渲染的任意结构化详情 |
| `terminate` | `boolean or undefined` | 提示 Agent 在当前工具批次完成后停止 仅当批次中所有已完成的工具结果都设为 true 时才触发提前终止 |


### :gear: AgentTool

Agent 运行时使用的工具定义
在基础 Tool 接口上增加执行函数、参数预处理和执行模式

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `label` | `string` | 用于 UI 显示的人类可读标签 |
| `prepareArguments` | `((args: unknown) => StaticType<[], "Encode", {}, {}, TParameters>) or undefined` | Schema 校验前的原始工具调用参数兼容性垫片 必须返回匹配 TParameters 的对象 |
| `execute` | `(toolCallId: string, params: StaticType<[], "Encode", {}, {}, TParameters>, signal?: AbortSignal or undefined, onUpdate?: AgentToolUpdateCallback<TDetails> or undefined) => Promise<...>` | 执行工具调用 失败时应抛出异常，而非将错误编码在 content 中 |
| `executionMode` | `ToolExecutionMode or undefined` | 单个工具的执行模式覆盖 - "sequential"：此工具必须与其他工具调用逐一执行 - "parallel"：此工具可与其他工具调用并发执行 省略则使用默认执行模式 |


### :gear: AgentContext

传入底层 Agent 循环的上下文快照

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `systemPrompt` | `string` | 请求中包含的系统提示 |
| `messages` | `AgentMessage[]` | 模型可见的对话记录 |
| `tools` | `AgentTool<any, any>[] or undefined` | 本次运行可用的工具 |


### :gear: AgentOptions

Agent 构造选项

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `initialState` | `Partial<Omit<AgentState, "pendingToolCalls" or "isStreaming" or "streamingMessage" or "errorMessage">> or undefined` | 初始状态（系统提示、模型、工具列表、消息历史等） |
| `convertToLlm` | `((messages: AgentMessage[]) => Message[] or Promise<Message[]>) or undefined` | 自定义 AgentMessage -> LLM Message 转换函数 |
| `transformContext` | `((messages: AgentMessage[], signal?: AbortSignal or undefined) => Promise<AgentMessage[]>) or undefined` | 上下文变换函数（在 convertToLlm 之前应用） |
| `streamFn` | `StreamFn or undefined` | 自定义 LLM 流式调用函数，默认使用 streamSimple |
| `getApiKey` | `((provider: string) => string or Promise<string or undefined> or undefined) or undefined` | 动态 API 密钥解析器（适用于短期 OAuth Token） |
| `onPayload` | `((payload: unknown, model: Model<Api>) => unknown) or undefined` | 发送给 LLM 前的载荷拦截器 |
| `onResponse` | `((response: ProviderResponse, model: Model<Api>) => void or Promise<void>) or undefined` | LLM 响应拦截器 |
| `beforeToolCall` | `((context: BeforeToolCallContext, signal?: AbortSignal or undefined) => Promise<BeforeToolCallResult or undefined>) or undefined` | 工具执行前钩子 |
| `afterToolCall` | `((context: AfterToolCallContext, signal?: AbortSignal or undefined) => Promise<AfterToolCallResult or undefined>) or undefined` | 工具执行后钩子 |
| `steeringMode` | `QueueMode or undefined` | 转向队列排空模式，默认 "one-at-a-time" |
| `followUpMode` | `QueueMode or undefined` | 后续队列排空模式，默认 "one-at-a-time" |
| `sessionId` | `string or undefined` | 会话标识符，转发给提供商用于缓存感知后端 |
| `thinkingBudgets` | `ThinkingBudgets or undefined` | 思考级别 Token 预算，转发给流式函数 |
| `transport` | `Transport or undefined` | 首选传输方式，转发给流式函数 |
| `maxRetryDelayMs` | `number or undefined` | 提供商请求的重试延迟上限 |
| `toolExecution` | `ToolExecutionMode or undefined` | 工具执行策略（并行/顺序） |


## :cocktail: Types

- [StreamFn](#gear-streamfn)
- [ToolExecutionMode](#gear-toolexecutionmode)
- [AgentToolCall](#gear-agenttoolcall)
- [ThinkingLevel](#gear-thinkinglevel)
- [AgentMessage](#gear-agentmessage)
- [AgentToolUpdateCallback](#gear-agenttoolupdatecallback)
- [AgentEvent](#gear-agentevent)
- [AgentEventSink](#gear-agenteventsink)

### :gear: StreamFn

Agent 循环使用的 LLM 流式调用函数

契约：
- 请求/模型/运行时失败不得抛出异常或返回 rejected promise
- 必须返回 AssistantMessageEventStream
- 失败必须通过流协议事件编码，最终产出 stopReason 为 "error" 或 "aborted" 的 AssistantMessage

| Type | Type |
| ---------- | ---------- |
| `StreamFn` | `( ...args: Parameters<typeof streamSimple> ) => ReturnType<typeof streamSimple> or Promise<ReturnType<typeof streamSimple>>` |

### :gear: ToolExecutionMode

工具调用执行模式
- "sequential"：逐个准备、执行和完成每个工具调用
- "parallel"：依次准备工具调用，然后并发执行允许的工具，
  tool_execution_end 按完成顺序发出，工具结果消息按原始顺序发出

| Type | Type |
| ---------- | ---------- |
| `ToolExecutionMode` | `sequential" or "parallel` |

### :gear: AgentToolCall

从助手消息内容中提取的工具调用块类型

| Type | Type |
| ---------- | ---------- |
| `AgentToolCall` | `Extract<AssistantMessage["content"][number], { type: "toolCall" }>` |

### :gear: ThinkingLevel

推理/思考级别
注意："xhigh" 仅被选定的模型系列支持，使用模型的思考级别元数据检测具体模型是否支持

| Type | Type |
| ---------- | ---------- |
| `ThinkingLevel` | `off" or "minimal" or "low" or "medium" or "high" or "xhigh` |

### :gear: AgentMessage

Agent 消息联合类型
包含标准 LLM 消息（UserMessage、AssistantMessage、ToolResultMessage）和自定义消息
此抽象允许应用添加自定义消息类型，同时保持类型安全和与基础 LLM 消息的兼容性

| Type | Type |
| ---------- | ---------- |
| `AgentMessage` | `Message or CustomAgentMessages[keyof CustomAgentMessages]` |

### :gear: AgentToolUpdateCallback

工具用于流式传输部分执行更新的回调函数

| Type | Type |
| ---------- | ---------- |
| `AgentToolUpdateCallback` | `(partialResult: AgentToolResult<T>) => void` |

### :gear: AgentEvent

Agent 发出的事件类型，用于 UI 更新

agent_end 是一次运行的最后一个事件，但等待中的 Agent.subscribe() 监听器
仍是运行结算的一部分。Agent 仅在这些监听器完成后才变为空闲。

事件层级：
  agent_start
    turn_start
      message_start (用户消息)
      message_end
      message_start (助手消息)
        message_update (流式增量)
      message_end
      tool_execution_start -> tool_execution_update -> tool_execution_end
      message_start (工具结果)
      message_end
    turn_end
  agent_end

| Type | Type |
| ---------- | ---------- |
| `AgentEvent` | `| { type: "agent_start" } or { type: "agent_end"; messages: AgentMessage[] } // 轮次生命周期 —— 一轮包含一次助手响应 + 其工具调用/结果 or { type: "turn_start" } or { type: "turn_end"; message: AgentMessage; toolResults: ToolResultMessage[] } // 消息生命周期 —— 为用户、助手和工具结果消息发出 or { type: "message_start"; message: AgentMessage } // 仅在流式传输期间为助手消息发出 or { type: "message_update"; message: AgentMessage; assistantMessageEvent: AssistantMessageEvent } or { type: "message_end"; message: AgentMessage } // 工具执行生命周期 or { type: "tool_execution_start"; toolCallId: string; toolName: string; args: any } or { type: "tool_execution_update"; toolCallId: string; toolName: string; args: any; partialResult: any } or { type: "tool_execution_end"; toolCallId: string; toolName: string; result: any; isError: boolean }` |

### :gear: AgentEventSink

Agent 事件接收器类型

| Type | Type |
| ---------- | ---------- |
| `AgentEventSink` | `(event: AgentEvent) => Promise<void> or void` |

