/**
 * @fileoverview Agent 主类，Agent 运行时的有状态封装。
 *
 * Agent 类是整个 Agent 运行时的高层 API，负责：
 *   - 维护对话状态（系统提示、消息历史、工具列表、模型配置）
 *   - 管理 Agent 循环的生命周期（启动、运行、停止、中止）
 *   - 提供消息队列机制（转向队列 steering + 后续队列 follow-up）
 *   - 发出生命周期事件（agent_start/turn_start/message_start/tool_execution 等）
 *   - 将底层 agent-loop 的无状态函数封装为有状态的对象 API
 *
 * 使用方式：
 *   1. 构造 Agent 实例，传入初始配置
 *   2. 订阅事件（subscribe）
 *   3. 发起提示（prompt）或继续对话（continue）
 *   4. 运行期间可通过 steer() 注入转向消息，followUp() 排队后续消息
 *   5. 可通过 abort() 中止运行，waitForIdle() 等待完成
 */

import {
	type ImageContent,
	type Message,
	type Model,
	type SimpleStreamOptions,
	streamSimple,
	type TextContent,
	type ThinkingBudgets,
	type Transport,
} from "@earendil-works/pi-ai";
import { runAgentLoop, runAgentLoopContinue } from "./agent-loop.js";
import type {
	AfterToolCallContext,
	AfterToolCallResult,
	AgentContext,
	AgentEvent,
	AgentLoopConfig,
	AgentMessage,
	AgentState,
	AgentTool,
	BeforeToolCallContext,
	BeforeToolCallResult,
	StreamFn,
	ToolExecutionMode,
} from "./types.js";

/**
 * 默认的消息转换函数：过滤出标准 LLM 消息（user、assistant、toolResult）
 * @param messages - Agent 消息数组
 * @returns LLM 可理解的消息数组
 */
function defaultConvertToLlm(messages: AgentMessage[]): Message[] {
	return messages.filter(
		(message) => message.role === "user" || message.role === "assistant" || message.role === "toolResult",
	);
}

// 空用量对象，用于构建错误消息
const EMPTY_USAGE = {
	input: 0,
	output: 0,
	cacheRead: 0,
	cacheWrite: 0,
	totalTokens: 0,
	cost: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0, total: 0 },
};

// 默认模型占位符
const DEFAULT_MODEL = {
	id: "unknown",
	name: "unknown",
	api: "unknown",
	provider: "unknown",
	baseUrl: "",
	reasoning: false,
	input: [],
	cost: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0 },
	contextWindow: 0,
	maxTokens: 0,
} satisfies Model<any>;

// 消息队列的排空模式
type QueueMode = "all" | "one-at-a-time";

// Agent 的可变内部状态（包含 AgentState 的所有可写字段）
type MutableAgentState = Omit<AgentState, "isStreaming" | "streamingMessage" | "pendingToolCalls" | "errorMessage"> & {
	isStreaming: boolean;
	streamingMessage?: AgentMessage;
	pendingToolCalls: Set<string>;
	errorMessage?: string;
};

/**
 * 创建可变的 Agent 状态对象
 * tools 和 messages 使用 getter/setter，赋值时自动复制数组
 * @param initialState - 可选的初始状态部分
 * @returns 可变状态对象
 */
function createMutableAgentState(
	initialState?: Partial<Omit<AgentState, "pendingToolCalls" | "isStreaming" | "streamingMessage" | "errorMessage">>,
): MutableAgentState {
	let tools = initialState?.tools?.slice() ?? [];
	let messages = initialState?.messages?.slice() ?? [];

	return {
		systemPrompt: initialState?.systemPrompt ?? "",
		model: initialState?.model ?? DEFAULT_MODEL,
		thinkingLevel: initialState?.thinkingLevel ?? "off",
		get tools() {
			return tools;
		},
		set tools(nextTools: AgentTool<any>[]) {
			tools = nextTools.slice();
		},
		get messages() {
			return messages;
		},
		set messages(nextMessages: AgentMessage[]) {
			messages = nextMessages.slice();
		},
		isStreaming: false,
		streamingMessage: undefined,
		pendingToolCalls: new Set<string>(),
		errorMessage: undefined,
	};
}

/** Agent 构造选项 */
export interface AgentOptions {
	/** 初始状态（系统提示、模型、工具列表、消息历史等） */
	initialState?: Partial<Omit<AgentState, "pendingToolCalls" | "isStreaming" | "streamingMessage" | "errorMessage">>;
	/** 自定义 AgentMessage -> LLM Message 转换函数 */
	convertToLlm?: (messages: AgentMessage[]) => Message[] | Promise<Message[]>;
	/** 上下文变换函数（在 convertToLlm 之前应用） */
	transformContext?: (messages: AgentMessage[], signal?: AbortSignal) => Promise<AgentMessage[]>;
	/** 自定义 LLM 流式调用函数，默认使用 streamSimple */
	streamFn?: StreamFn;
	/** 动态 API 密钥解析器（适用于短期 OAuth Token） */
	getApiKey?: (provider: string) => Promise<string | undefined> | string | undefined;
	/** 发送给 LLM 前的载荷拦截器 */
	onPayload?: SimpleStreamOptions["onPayload"];
	/** LLM 响应拦截器 */
	onResponse?: SimpleStreamOptions["onResponse"];
	/** 工具执行前钩子 */
	beforeToolCall?: (context: BeforeToolCallContext, signal?: AbortSignal) => Promise<BeforeToolCallResult | undefined>;
	/** 工具执行后钩子 */
	afterToolCall?: (context: AfterToolCallContext, signal?: AbortSignal) => Promise<AfterToolCallResult | undefined>;
	/** 转向队列排空模式，默认 "one-at-a-time" */
	steeringMode?: QueueMode;
	/** 后续队列排空模式，默认 "one-at-a-time" */
	followUpMode?: QueueMode;
	/** 会话标识符，转发给提供商用于缓存感知后端 */
	sessionId?: string;
	/** 思考级别 Token 预算，转发给流式函数 */
	thinkingBudgets?: ThinkingBudgets;
	/** 首选传输方式，转发给流式函数 */
	transport?: Transport;
	/** 提供商请求的重试延迟上限 */
	maxRetryDelayMs?: number;
	/** 工具执行策略（并行/顺序） */
	toolExecution?: ToolExecutionMode;
}

/**
 * 待处理消息队列
 * 支持两种排空模式：
 *   - "all"：一次排空全部消息
 *   - "one-at-a-time"：每次只排空第一条消息
 */
class PendingMessageQueue {
	private messages: AgentMessage[] = [];

	constructor(public mode: QueueMode) {}

	/** 入队一条消息 */
	enqueue(message: AgentMessage): void {
		this.messages.push(message);
	}

	/** 队列中是否有待处理消息 */
	hasItems(): boolean {
		return this.messages.length > 0;
	}

	/**
	 * 排空队列
	 * "all" 模式返回全部消息并清空队列
	 * "one-at-a-time" 模式只返回第一条消息
	 */
	drain(): AgentMessage[] {
		if (this.mode === "all") {
			const drained = this.messages.slice();
			this.messages = [];
			return drained;
		}

		const first = this.messages[0];
		if (!first) {
			return [];
		}
		this.messages = this.messages.slice(1);
		return [first];
	}

	/** 清空队列 */
	clear(): void {
		this.messages = [];
	}
}

/** 活跃运行的结构，包含 Promise、resolve 函数和中止控制器 */
type ActiveRun = {
	promise: Promise<void>;
	resolve: () => void;
	abortController: AbortController;
};

/**
 * Agent 主类 —— 底层 Agent 循环的有状态封装
 *
 * Agent 拥有当前对话记录、发出生命周期事件、执行工具，
 * 并提供消息队列 API 用于转向和后续消息注入。
 *
 * 事件层级：
 *   agent_start -> turn_start -> message_* -> tool_execution_* -> turn_end -> agent_end
 */
export class Agent {
	// 内部可变状态
	private _state: MutableAgentState;
	// 事件监听器集合，按订阅顺序调用
	private readonly listeners = new Set<(event: AgentEvent, signal: AbortSignal) => Promise<void> | void>();
	// 转向消息队列（助手轮次完成后注入）
	private readonly steeringQueue: PendingMessageQueue;
	// 后续消息队列（Agent 即将停止时注入）
	private readonly followUpQueue: PendingMessageQueue;

	// 以下为可公开配置的回调/属性
	public convertToLlm: (messages: AgentMessage[]) => Message[] | Promise<Message[]>;
	public transformContext?: (messages: AgentMessage[], signal?: AbortSignal) => Promise<AgentMessage[]>;
	public streamFn: StreamFn;
	public getApiKey?: (provider: string) => Promise<string | undefined> | string | undefined;
	public onPayload?: SimpleStreamOptions["onPayload"];
	public onResponse?: SimpleStreamOptions["onResponse"];
	public beforeToolCall?: (
		context: BeforeToolCallContext,
		signal?: AbortSignal,
	) => Promise<BeforeToolCallResult | undefined>;
	public afterToolCall?: (
		context: AfterToolCallContext,
		signal?: AbortSignal,
	) => Promise<AfterToolCallResult | undefined>;

	// 当前活跃运行（运行期间存在，空闲时为 undefined）
	private activeRun?: ActiveRun;
	/** 会话标识符，转发给提供商用于缓存感知后端 */
	public sessionId?: string;
	/** 思考级别 Token 预算，转发给流式函数 */
	public thinkingBudgets?: ThinkingBudgets;
	/** 首选传输方式 */
	public transport: Transport;
	/** 提供商请求的重试延迟上限 */
	public maxRetryDelayMs?: number;
	/** 多工具调用的执行策略 */
	public toolExecution: ToolExecutionMode;

	constructor(options: AgentOptions = {}) {
		this._state = createMutableAgentState(options.initialState);
		this.convertToLlm = options.convertToLlm ?? defaultConvertToLlm;
		this.transformContext = options.transformContext;
		// 默认使用 pi-ai 的 streamSimple 作为流式函数
		this.streamFn = options.streamFn ?? streamSimple;
		this.getApiKey = options.getApiKey;
		this.onPayload = options.onPayload;
		this.onResponse = options.onResponse;
		this.beforeToolCall = options.beforeToolCall;
		this.afterToolCall = options.afterToolCall;
		this.steeringQueue = new PendingMessageQueue(options.steeringMode ?? "one-at-a-time");
		this.followUpQueue = new PendingMessageQueue(options.followUpMode ?? "one-at-a-time");
		this.sessionId = options.sessionId;
		this.thinkingBudgets = options.thinkingBudgets;
		this.transport = options.transport ?? "auto";
		this.maxRetryDelayMs = options.maxRetryDelayMs;
		this.toolExecution = options.toolExecution ?? "parallel";
	}

	/**
	 * 订阅 Agent 生命周期事件
	 * 监听器 Promise 按订阅顺序等待，并包含在当前运行的结算中。
	 * agent_end 是最终事件，但 Agent 在所有等待中的 agent_end 监听器完成后才变为空闲。
	 * @param listener - 事件监听器，接收事件和中止信号
	 * @returns 取消订阅函数
	 */
	subscribe(listener: (event: AgentEvent, signal: AbortSignal) => Promise<void> | void): () => void {
		this.listeners.add(listener);
		return () => this.listeners.delete(listener);
	}

	/**
	 * 获取当前 Agent 状态
	 * 赋值 state.tools 或 state.messages 时会自动复制顶层数组
	 */
	get state(): AgentState {
		return this._state;
	}

	/** 转向队列排空模式 */
	set steeringMode(mode: QueueMode) {
		this.steeringQueue.mode = mode;
	}

	get steeringMode(): QueueMode {
		return this.steeringQueue.mode;
	}

	/** 后续队列排空模式 */
	set followUpMode(mode: QueueMode) {
		this.followUpQueue.mode = mode;
	}

	get followUpMode(): QueueMode {
		return this.followUpQueue.mode;
	}

	/**
	 * 将消息入队到转向队列
	 * 消息将在当前助手轮次完成后注入
	 */
	steer(message: AgentMessage): void {
		this.steeringQueue.enqueue(message);
	}

	/**
	 * 将消息入队到后续队列
	 * 消息仅在 Agent 即将停止时处理
	 */
	followUp(message: AgentMessage): void {
		this.followUpQueue.enqueue(message);
	}

	/** 清空转向队列 */
	clearSteeringQueue(): void {
		this.steeringQueue.clear();
	}

	/** 清空后续队列 */
	clearFollowUpQueue(): void {
		this.followUpQueue.clear();
	}

	/** 清空所有队列 */
	clearAllQueues(): void {
		this.clearSteeringQueue();
		this.clearFollowUpQueue();
	}

	/** 任一队列是否还有待处理消息 */
	hasQueuedMessages(): boolean {
		return this.steeringQueue.hasItems() || this.followUpQueue.hasItems();
	}

	/** 当前运行的中止信号 */
	get signal(): AbortSignal | undefined {
		return this.activeRun?.abortController.signal;
	}

	/** 中止当前运行 */
	abort(): void {
		this.activeRun?.abortController.abort();
	}

	/**
	 * 等待当前运行和所有等待中的事件监听器完成
	 * 在 agent_end 监听器结算后 resolve
	 */
	waitForIdle(): Promise<void> {
		return this.activeRun?.promise ?? Promise.resolve();
	}

	/** 清空对话状态、运行时状态和消息队列 */
	reset(): void {
		this._state.messages = [];
		this._state.isStreaming = false;
		this._state.streamingMessage = undefined;
		this._state.pendingToolCalls = new Set<string>();
		this._state.errorMessage = undefined;
		this.clearFollowUpQueue();
		this.clearSteeringQueue();
	}

	/**
	 * 发起新的提示
	 * 支持文本、单条消息或消息数组作为输入
	 * @throws Agent 已在处理中时抛出错误
	 */
	async prompt(message: AgentMessage | AgentMessage[]): Promise<void>;
	async prompt(input: string, images?: ImageContent[]): Promise<void>;
	async prompt(input: string | AgentMessage | AgentMessage[], images?: ImageContent[]): Promise<void> {
		if (this.activeRun) {
			throw new Error(
				"Agent is already processing a prompt. Use steer() or followUp() to queue messages, or wait for completion.",
			);
		}
		const messages = this.normalizePromptInput(input, images);
		await this.runPromptMessages(messages);
	}

	/**
	 * 从当前对话记录继续对话
	 * 最后一条消息必须是用户消息或工具结果消息
	 * @throws Agent 已在处理中或消息不满足条件时抛出错误
	 */
	async continue(): Promise<void> {
		if (this.activeRun) {
			throw new Error("Agent is already processing. Wait for completion before continuing.");
		}

		const lastMessage = this._state.messages[this._state.messages.length - 1];
		if (!lastMessage) {
			throw new Error("No messages to continue from");
		}

		// 最后一条是助手消息时，尝试使用队列中的消息继续
		if (lastMessage.role === "assistant") {
			const queuedSteering = this.steeringQueue.drain();
			if (queuedSteering.length > 0) {
				await this.runPromptMessages(queuedSteering, { skipInitialSteeringPoll: true });
				return;
			}

			const queuedFollowUps = this.followUpQueue.drain();
			if (queuedFollowUps.length > 0) {
				await this.runPromptMessages(queuedFollowUps);
				return;
			}

			throw new Error("Cannot continue from message role: assistant");
		}

		await this.runContinuation();
	}

	/**
	 * 标准化提示输入为消息数组
	 * 字符串输入被包装为用户消息，可选附带图片
	 */
	private normalizePromptInput(
		input: string | AgentMessage | AgentMessage[],
		images?: ImageContent[],
	): AgentMessage[] {
		if (Array.isArray(input)) {
			return input;
		}

		if (typeof input !== "string") {
			return [input];
		}

		const content: Array<TextContent | ImageContent> = [{ type: "text", text: input }];
		if (images && images.length > 0) {
			content.push(...images);
		}
		return [{ role: "user", content, timestamp: Date.now() }];
	}

	/**
	 * 使用新消息运行 Agent 循环
	 * @param messages - 初始消息数组
	 * @param options - 运行选项（如跳过首次转向轮询）
	 */
	private async runPromptMessages(
		messages: AgentMessage[],
		options: { skipInitialSteeringPoll?: boolean } = {},
	): Promise<void> {
		await this.runWithLifecycle(async (signal) => {
			await runAgentLoop(
				messages,
				this.createContextSnapshot(),
				this.createLoopConfig(options),
				(event) => this.processEvents(event),
				signal,
				this.streamFn,
			);
		});
	}

	/**
	 * 从现有上下文继续运行 Agent 循环
	 */
	private async runContinuation(): Promise<void> {
		await this.runWithLifecycle(async (signal) => {
			await runAgentLoopContinue(
				this.createContextSnapshot(),
				this.createLoopConfig(),
				(event) => this.processEvents(event),
				signal,
				this.streamFn,
			);
		});
	}

	/**
	 * 创建当前状态的上下文快照
	 * 消息和工具列表会被复制，确保底层循环不影响当前状态
	 */
	private createContextSnapshot(): AgentContext {
		return {
			systemPrompt: this._state.systemPrompt,
			messages: this._state.messages.slice(),
			tools: this._state.tools.slice(),
		};
	}

	/**
	 * 从当前配置构建 Agent 循环配置对象
	 * 转向和后续队列通过闭包连接到底层循环的 getSteeringMessages/getFollowUpMessages
	 */
	private createLoopConfig(options: { skipInitialSteeringPoll?: boolean } = {}): AgentLoopConfig {
		let skipInitialSteeringPoll = options.skipInitialSteeringPoll === true;
		return {
			model: this._state.model,
			reasoning: this._state.thinkingLevel === "off" ? undefined : this._state.thinkingLevel,
			sessionId: this.sessionId,
			onPayload: this.onPayload,
			onResponse: this.onResponse,
			transport: this.transport,
			thinkingBudgets: this.thinkingBudgets,
			maxRetryDelayMs: this.maxRetryDelayMs,
			toolExecution: this.toolExecution,
			beforeToolCall: this.beforeToolCall,
			afterToolCall: this.afterToolCall,
			convertToLlm: this.convertToLlm,
			transformContext: this.transformContext,
			getApiKey: this.getApiKey,
			// 转向消息：从队列中排空，首次可跳过
			getSteeringMessages: async () => {
				if (skipInitialSteeringPoll) {
					skipInitialSteeringPoll = false;
					return [];
				}
				return this.steeringQueue.drain();
			},
			getFollowUpMessages: async () => this.followUpQueue.drain(),
		};
	}

	/**
	 * 带生命周期管理的运行包装器
	 * 负责设置/清理 activeRun、设置 isStreaming 标志、处理失败
	 */
	private async runWithLifecycle(executor: (signal: AbortSignal) => Promise<void>): Promise<void> {
		if (this.activeRun) {
			throw new Error("Agent is already processing.");
		}

		const abortController = new AbortController();
		let resolvePromise = () => {};
		const promise = new Promise<void>((resolve) => {
			resolvePromise = resolve;
		});
		this.activeRun = { promise, resolve: resolvePromise, abortController };

		this._state.isStreaming = true;
		this._state.streamingMessage = undefined;
		this._state.errorMessage = undefined;

		try {
			await executor(abortController.signal);
		} catch (error) {
			await this.handleRunFailure(error, abortController.signal.aborted);
		} finally {
			this.finishRun();
		}
	}

	/**
	 * 处理运行失败
	 * 构建一条错误助手消息，发出完整的生命周期事件序列
	 */
	private async handleRunFailure(error: unknown, aborted: boolean): Promise<void> {
		const failureMessage = {
			role: "assistant",
			content: [{ type: "text", text: "" }],
			api: this._state.model.api,
			provider: this._state.model.provider,
			model: this._state.model.id,
			usage: EMPTY_USAGE,
			stopReason: aborted ? "aborted" : "error",
			errorMessage: error instanceof Error ? error.message : String(error),
			timestamp: Date.now(),
		} satisfies AgentMessage;
		await this.processEvents({ type: "message_start", message: failureMessage });
		await this.processEvents({ type: "message_end", message: failureMessage });
		await this.processEvents({ type: "turn_end", message: failureMessage, toolResults: [] });
		await this.processEvents({ type: "agent_end", messages: [failureMessage] });
	}

	/**
	 * 完成运行：清除运行时状态，resolve 活跃运行的 Promise
	 */
	private finishRun(): void {
		this._state.isStreaming = false;
		this._state.streamingMessage = undefined;
		this._state.pendingToolCalls = new Set<string>();
		this.activeRun?.resolve();
		this.activeRun = undefined;
	}

	/**
	 * 处理底层循环发出的事件
	 * 先更新内部状态，再按顺序通知所有监听器
	 *
	 * agent_end 仅表示不再有更多循环事件，运行在 finishRun() 清除状态后才算空闲
	 */
	private async processEvents(event: AgentEvent): Promise<void> {
		switch (event.type) {
			case "message_start":
				this._state.streamingMessage = event.message;
				break;

			case "message_update":
				this._state.streamingMessage = event.message;
				break;

			case "message_end":
				this._state.streamingMessage = undefined;
				this._state.messages.push(event.message);
				break;

			case "tool_execution_start": {
				const pendingToolCalls = new Set(this._state.pendingToolCalls);
				pendingToolCalls.add(event.toolCallId);
				this._state.pendingToolCalls = pendingToolCalls;
				break;
			}

			case "tool_execution_end": {
				const pendingToolCalls = new Set(this._state.pendingToolCalls);
				pendingToolCalls.delete(event.toolCallId);
				this._state.pendingToolCalls = pendingToolCalls;
				break;
			}

			case "turn_end":
				if (event.message.role === "assistant" && event.message.errorMessage) {
					this._state.errorMessage = event.message.errorMessage;
				}
				break;

			case "agent_end":
				this._state.streamingMessage = undefined;
				break;
		}

		const signal = this.activeRun?.abortController.signal;
		if (!signal) {
			throw new Error("Agent listener invoked outside active run");
		}
		for (const listener of this.listeners) {
			await listener(event, signal);
		}
	}
}
