/**
 * @fileoverview Agent 运行时的核心类型定义。
 *
 * Agent 包是 pi 架构的中间层，位于 AI 层（LLM 调用）和 Coding Agent 层（具体应用）之间。
 * 本文件定义了 Agent 运行时所需的全部类型：
 *
 * 核心概念：
 *   - StreamFn：Agent 使用的 LLM 流式调用函数签名
 *   - AgentLoopConfig：Agent 循环的完整配置（模型、工具、钩子、执行模式等）
 *   - AgentMessage：扩展的消息类型，支持自定义消息（通过声明合并）
 *   - AgentTool：工具定义，包含执行函数、参数校验和执行模式
 *   - AgentState：Agent 的公开状态（系统提示、模型、消息历史、工具列表等）
 *   - AgentEvent：Agent 事件协议（生命周期、消息、工具执行事件）
 *
 * 设计原则：
 *   Agent 层不依赖具体的 LLM 提供商，通过 StreamFn 抽象解耦。
 *   自定义消息通过 TypeScript 声明合并实现扩展，保持类型安全。
 */

import type {
	AssistantMessage,
	AssistantMessageEvent,
	ImageContent,
	Message,
	Model,
	SimpleStreamOptions,
	streamSimple,
	TextContent,
	Tool,
	ToolResultMessage,
} from "@earendil-works/pi-ai";
import type { Static, TSchema } from "typebox";

/**
 * Agent 循环使用的 LLM 流式调用函数
 *
 * 契约：
 * - 请求/模型/运行时失败不得抛出异常或返回 rejected promise
 * - 必须返回 AssistantMessageEventStream
 * - 失败必须通过流协议事件编码，最终产出 stopReason 为 "error" 或 "aborted" 的 AssistantMessage
 */
export type StreamFn = (
	...args: Parameters<typeof streamSimple>
) => ReturnType<typeof streamSimple> | Promise<ReturnType<typeof streamSimple>>;

/**
 * 工具调用执行模式
 * - "sequential"：逐个准备、执行和完成每个工具调用
 * - "parallel"：依次准备工具调用，然后并发执行允许的工具，
 *   tool_execution_end 按完成顺序发出，工具结果消息按原始顺序发出
 */
export type ToolExecutionMode = "sequential" | "parallel";

/**
 * 从助手消息内容中提取的工具调用块类型
 */
export type AgentToolCall = Extract<AssistantMessage["content"][number], { type: "toolCall" }>;

/**
 * beforeToolCall 钩子的返回结果
 * 返回 { block: true } 可阻止工具执行，循环会发出一个错误工具结果替代
 */
export interface BeforeToolCallResult {
	/** 是否阻止执行 */
	block?: boolean;
	/** 阻止原因，显示在错误工具结果中。省略则使用默认阻止消息 */
	reason?: string;
}

/**
 * afterToolCall 钩子的返回结果
 * 各字段合并语义：提供的字段替换原始值，省略的字段保持原始值
 * 不进行深度合并
 */
export interface AfterToolCallResult {
	/** 替换工具结果的完整内容数组 */
	content?: (TextContent | ImageContent)[];
	/** 替换工具结果的完整详情载荷 */
	details?: unknown;
	/** 替换工具结果的错误标志 */
	isError?: boolean;
	/**
	 * 提示 Agent 在当前工具批次完成后停止
	 * 仅当批次中所有已完成的工具结果都设为 true 时才触发提前终止
	 */
	terminate?: boolean;
}

/**
 * beforeToolCall 钩子的上下文参数
 */
export interface BeforeToolCallContext {
	/** 请求工具调用的助手消息 */
	assistantMessage: AssistantMessage;
	/** 助手消息中的原始工具调用块 */
	toolCall: AgentToolCall;
	/** 经 Schema 校验后的工具参数 */
	args: unknown;
	/** 工具调用准备时的 Agent 上下文快照 */
	context: AgentContext;
}

/**
 * afterToolCall 钩子的上下文参数
 */
export interface AfterToolCallContext {
	/** 请求工具调用的助手消息 */
	assistantMessage: AssistantMessage;
	/** 助手消息中的原始工具调用块 */
	toolCall: AgentToolCall;
	/** 经 Schema 校验后的工具参数 */
	args: unknown;
	/** 应用 afterToolCall 覆盖之前的已执行工具结果 */
	result: AgentToolResult<any>;
	/** 当前工具结果是否被视为错误 */
	isError: boolean;
	/** 工具调用完成时的 Agent 上下文快照 */
	context: AgentContext;
}

/**
 * shouldStopAfterTurn 钩子的上下文参数
 */
export interface ShouldStopAfterTurnContext {
	/** 完成本轮的助手消息 */
	message: AssistantMessage;
	/** 传递给前一个 turn_end 事件的工具结果消息 */
	toolResults: ToolResultMessage[];
	/** 助手消息和工具结果追加后的 Agent 上下文 */
	context: AgentContext;
	/** 如果循环在此时退出将返回的消息。初始提示运行包含提示消息，续接运行不包含已有上下文消息 */
	newMessages: AgentMessage[];
}

/**
 * Agent 循环的完整配置
 * 继承 SimpleStreamOptions（温度、Token 上限、推理级别等），增加 Agent 特有的配置项
 */
export interface AgentLoopConfig extends SimpleStreamOptions {
	/** 使用的模型 */
	model: Model<any>;

	/**
	 * 将 AgentMessage[] 转换为 LLM 可理解的 Message[]
	 *
	 * 每条 AgentMessage 必须被转换为 UserMessage、AssistantMessage 或 ToolResultMessage。
	 * 无法转换的消息（如仅用于 UI 的通知）应被过滤掉。
	 *
	 * 契约：不得抛出异常或 reject，应返回安全的回退值
	 */
	convertToLlm: (messages: AgentMessage[]) => Message[] | Promise<Message[]>;

	/**
	 * 在 convertToLlm 之前对上下文进行可选的变换
	 *
	 * 适用于在 AgentMessage 层面进行的操作：
	 * - 上下文窗口管理（裁剪旧消息）
	 * - 从外部来源注入上下文
	 *
	 * 契约：不得抛出异常或 reject
	 */
	transformContext?: (messages: AgentMessage[], signal?: AbortSignal) => Promise<AgentMessage[]>;

	/**
	 * 为每次 LLM 调用动态解析 API 密钥
	 *
	 * 适用于短期 OAuth Token（如 GitHub Copilot），这些 Token 可能在
	 * 长时间工具执行阶段过期
	 *
	 * 契约：不得抛出异常或 reject，无密钥时返回 undefined
	 */
	getApiKey?: (provider: string) => Promise<string | undefined> | string | undefined;

	/**
	 * 每轮完全完成后（turn_end 已发出）调用
	 *
	 * 返回 true 时，循环发出 agent_end 并退出，不再轮询转向或后续队列。
	 * 当前助手响应和工具执行正常完成。
	 *
	 * 用于在上下文过满之前请求优雅停止
	 *
	 * 契约：不得抛出异常或 reject
	 */
	shouldStopAfterTurn?: (context: ShouldStopAfterTurnContext) => boolean | Promise<boolean>;

	/**
	 * 运行期间注入转向消息
	 *
	 * 在当前助手轮次完成工具调用执行后调用（除非 shouldStopAfterTurn 先退出）。
	 * 返回的消息会在下一次 LLM 调用前添加到上下文中。
	 * 当前助手消息的工具调用不会被跳过。
	 *
	 * 契约：不得抛出或 reject，无转向消息时返回 []
	 */
	getSteeringMessages?: () => Promise<AgentMessage[]>;

	/**
	 * Agent 即将停止时返回后续消息进行处理
	 *
	 * 在 Agent 没有更多工具调用和转向消息时调用。
	 * 返回消息后，消息被添加到上下文，Agent 继续下一轮。
	 *
	 * 契约：不得抛出或 reject，无后续消息时返回 []
	 */
	getFollowUpMessages?: () => Promise<AgentMessage[]>;

	/**
	 * 工具执行模式
	 * - "sequential"：逐个执行工具调用
	 * - "parallel"：顺序预检后并发执行，按完成顺序发出事件，按原始顺序发出消息
	 * 默认值："parallel"
	 */
	toolExecution?: ToolExecutionMode;

	/**
	 * 工具执行前的钩子（参数校验完成后调用）
	 * 返回 { block: true } 阻止执行，循环发出错误工具结果替代
	 * 钩子接收到 Agent 中止信号，有责任响应它
	 */
	beforeToolCall?: (context: BeforeToolCallContext, signal?: AbortSignal) => Promise<BeforeToolCallResult | undefined>;

	/**
	 * 工具执行完成后的钩子（在 tool_execution_end 和工具结果消息事件发出之前）
	 * 返回 AfterToolCallResult 可覆盖已执行工具结果的部分字段
	 * 钩子接收到 Agent 中止信号，有责任响应它
	 */
	afterToolCall?: (context: AfterToolCallContext, signal?: AbortSignal) => Promise<AfterToolCallResult | undefined>;
}

/**
 * 推理/思考级别
 * 注意："xhigh" 仅被选定的模型系列支持，使用模型的思考级别元数据检测具体模型是否支持
 */
export type ThinkingLevel = "off" | "minimal" | "low" | "medium" | "high" | "xhigh";

/**
 * 自定义应用消息的可扩展接口
 * 应用可通过声明合并扩展：
 *
 * @example
 * ```typescript
 * declare module "@mariozechner/agent" {
 *   interface CustomAgentMessages {
 *     artifact: ArtifactMessage;
 *     notification: NotificationMessage;
 *   }
 * }
 * ```
 */
export interface CustomAgentMessages {
	// 默认为空，应用通过声明合并扩展
}

/**
 * Agent 消息联合类型
 * 包含标准 LLM 消息（UserMessage、AssistantMessage、ToolResultMessage）和自定义消息
 * 此抽象允许应用添加自定义消息类型，同时保持类型安全和与基础 LLM 消息的兼容性
 */
export type AgentMessage = Message | CustomAgentMessages[keyof CustomAgentMessages];

/**
 * Agent 的公开状态接口
 *
 * tools 和 messages 使用访问器属性，实现可以在存储前复制赋值的数组
 */
export interface AgentState {
	/** 每次模型请求时发送的系统提示 */
	systemPrompt: string;
	/** 后续轮次使用的活跃模型 */
	model: Model<any>;
	/** 后续轮次请求的推理级别 */
	thinkingLevel: ThinkingLevel;
	/** 可用工具列表，赋值新数组时会复制顶层数组 */
	set tools(tools: AgentTool<any>[]);
	get tools(): AgentTool<any>[];
	/** 对话记录，赋值新数组时会复制顶层数组 */
	set messages(messages: AgentMessage[]);
	get messages(): AgentMessage[];
	/**
	 * Agent 正在处理提示或续接时为 true
	 * 直到等待中的 agent_end 监听器完成后才变为 false
	 */
	readonly isStreaming: boolean;
	/** 当前流式响应的部分助手消息（如存在） */
	readonly streamingMessage?: AgentMessage;
	/** 当前正在执行的工具调用 ID 集合 */
	readonly pendingToolCalls: ReadonlySet<string>;
	/** 最近一次失败或中止的助手轮次的错误消息 */
	readonly errorMessage?: string;
}

/**
 * 工具执行的最终或部分结果
 */
export interface AgentToolResult<T> {
	/** 返回给模型的文本或图像内容 */
	content: (TextContent | ImageContent)[];
	/** 用于日志或 UI 渲染的任意结构化详情 */
	details: T;
	/**
	 * 提示 Agent 在当前工具批次完成后停止
	 * 仅当批次中所有已完成的工具结果都设为 true 时才触发提前终止
	 */
	terminate?: boolean;
}

/**
 * 工具用于流式传输部分执行更新的回调函数
 */
export type AgentToolUpdateCallback<T = any> = (partialResult: AgentToolResult<T>) => void;

/**
 * Agent 运行时使用的工具定义
 * 在基础 Tool 接口上增加执行函数、参数预处理和执行模式
 */
export interface AgentTool<TParameters extends TSchema = TSchema, TDetails = any> extends Tool<TParameters> {
	/** 用于 UI 显示的人类可读标签 */
	label: string;
	/**
	 * Schema 校验前的原始工具调用参数兼容性垫片
	 * 必须返回匹配 TParameters 的对象
	 */
	prepareArguments?: (args: unknown) => Static<TParameters>;
	/**
	 * 执行工具调用
	 * 失败时应抛出异常，而非将错误编码在 content 中
	 */
	execute: (
		toolCallId: string,
		params: Static<TParameters>,
		signal?: AbortSignal,
		onUpdate?: AgentToolUpdateCallback<TDetails>,
	) => Promise<AgentToolResult<TDetails>>;
	/**
	 * 单个工具的执行模式覆盖
	 * - "sequential"：此工具必须与其他工具调用逐一执行
	 * - "parallel"：此工具可与其他工具调用并发执行
	 * 省略则使用默认执行模式
	 */
	executionMode?: ToolExecutionMode;
}

/**
 * 传入底层 Agent 循环的上下文快照
 */
export interface AgentContext {
	/** 请求中包含的系统提示 */
	systemPrompt: string;
	/** 模型可见的对话记录 */
	messages: AgentMessage[];
	/** 本次运行可用的工具 */
	tools?: AgentTool<any>[];
}

/**
 * Agent 发出的事件类型，用于 UI 更新
 *
 * agent_end 是一次运行的最后一个事件，但等待中的 Agent.subscribe() 监听器
 * 仍是运行结算的一部分。Agent 仅在这些监听器完成后才变为空闲。
 *
 * 事件层级：
 *   agent_start
 *     turn_start
 *       message_start (用户消息)
 *       message_end
 *       message_start (助手消息)
 *         message_update (流式增量)
 *       message_end
 *       tool_execution_start -> tool_execution_update -> tool_execution_end
 *       message_start (工具结果)
 *       message_end
 *     turn_end
 *   agent_end
 */
export type AgentEvent =
	// Agent 生命周期
	| { type: "agent_start" }
	| { type: "agent_end"; messages: AgentMessage[] }
	// 轮次生命周期 —— 一轮包含一次助手响应 + 其工具调用/结果
	| { type: "turn_start" }
	| { type: "turn_end"; message: AgentMessage; toolResults: ToolResultMessage[] }
	// 消息生命周期 —— 为用户、助手和工具结果消息发出
	| { type: "message_start"; message: AgentMessage }
	// 仅在流式传输期间为助手消息发出
	| { type: "message_update"; message: AgentMessage; assistantMessageEvent: AssistantMessageEvent }
	| { type: "message_end"; message: AgentMessage }
	// 工具执行生命周期
	| { type: "tool_execution_start"; toolCallId: string; toolName: string; args: any }
	| { type: "tool_execution_update"; toolCallId: string; toolName: string; args: any; partialResult: any }
	| { type: "tool_execution_end"; toolCallId: string; toolName: string; result: any; isError: boolean };
