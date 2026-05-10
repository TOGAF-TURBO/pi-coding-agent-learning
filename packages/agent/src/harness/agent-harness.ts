/**
 * @fileoverview Agent Harness — 编码代理的核心运行时，在 Agent 类之上封装完整的应用功能。
 *
 * AgentHarness 是 coding-agent 的核心类，它将底层的 Agent 循环与文件系统、
 * 会话存储、技能系统、压缩、分支导航等应用级功能整合在一起。
 *
 * 核心职责：
 *   - 管理 Agent 生命周期：prompt、abort、waitForIdle
 *   - 会话持久化：自动将消息、模型变更、压缩等写入会话存储
 *   - 事件系统：发出 Agent 底层事件和 Harness 自身事件，支持钩子拦截
 *   - 变更缓冲：操作期间的变更暂存为 pendingMutations，轮次结束后批量应用
 *   - 资源管理：动态解析技能、提示模板和系统提示
 *
 * 典型用法：
 *   const harness = new AgentHarness({ env, session, model, tools });
 *   const response = await harness.prompt("帮我重构这段代码");
 *   await harness.subscribe((event) => { ... });
 */

import { randomUUID } from "node:crypto";
import type { AssistantMessage, ImageContent, Model } from "@earendil-works/pi-ai";
import { Agent } from "../agent.js";
import type { AgentEvent, AgentMessage, AgentTool, ThinkingLevel } from "../types.js";
import { collectEntriesForBranchSummary, generateBranchSummary } from "./compaction/branch-summarization.js";
import { compact, DEFAULT_COMPACTION_SETTINGS, prepareCompaction } from "./compaction/compaction.js";
import { expandPromptTemplate } from "./prompt-templates.js";
import { expandSkillCommand } from "./skills.js";
import type {
	AbortResult,
	AgentHarnessContext,
	AgentHarnessConversationState,
	AgentHarnessEvent,
	AgentHarnessEventResultMap,
	AgentHarnessOperationState,
	AgentHarnessOptions,
	AgentHarnessOwnEvent,
	AgentHarnessResources,
	ExecutionEnv,
	NavigateTreeResult,
	Session,
} from "./types.js";

/**
 * 创建用户消息的工厂函数
 * @param text - 消息文本
 * @param images - 可选的图片附件
 * @returns 格式化的用户消息
 */
function createUserMessage(text: string, images?: ImageContent[]): AgentMessage {
	const content: Array<{ type: "text"; text: string } | ImageContent> = [{ type: "text", text }];
	if (images) content.push(...images);
	return { role: "user", content, timestamp: Date.now() };
}

/**
 * Agent Harness — 编码代理的核心运行时
 *
 * 在 Agent 类（纯 LLM 对话循环）之上封装了应用级功能：
 *   - 会话持久化到存储后端
 *   - 变更缓冲（操作期间暂存，结束后批量写入）
 *   - 事件分发和钩子系统
 *   - 压缩和分支导航
 *   - 技能和提示模板展开
 *
 * 线程安全模型：
 *   - 空闲时（operation.idle=true）：变更直接写入会话
 *   - 操作中（operation.idle=false）：变更暂存为 pendingMutations，轮次结束后批量应用
 */
export class AgentHarness {
	/** 底层 Agent 实例 */
	readonly agent: Agent;
	/** 文件系统和进程执行环境 */
	readonly env: ExecutionEnv;
	/** 对话状态（会话、模型、思考级别、活跃工具、队列） */
	readonly conversation: AgentHarnessConversationState;
	/** 操作状态（空闲标志、操作 ID、中止标志、队列、待写入变更） */
	readonly operation: AgentHarnessOperationState;

	private session: Session;
	private resourcesInput?: AgentHarnessOptions["resources"];
	private systemPrompt: AgentHarnessOptions["systemPrompt"];
	private requestAuth?: AgentHarnessOptions["requestAuth"];
	// 工具名 → 工具实例的注册表
	private toolRegistry = new Map<string, AgentTool>();
	// 事件监听器（接收所有事件）
	private listeners = new Set<(event: AgentHarnessEvent, signal?: AbortSignal) => Promise<void> | void>();
	// 事件钩子（按类型注册，可返回结果修改行为）
	private hooks = new Map<
		keyof AgentHarnessEventResultMap,
		Set<(event: any, ctx: AgentHarnessContext) => Promise<any> | any>
	>();

	constructor(options: AgentHarnessOptions) {
		// 初始化底层 Agent
		this.agent = new Agent({
			initialState: {
				model: options.model,
				thinkingLevel: options.thinkingLevel,
				tools: options.tools ?? [],
			},
		});
		this.env = options.env;
		this.session = options.session;
		this.resourcesInput = options.resources;
		this.systemPrompt = options.systemPrompt;
		this.requestAuth = options.requestAuth;

		// 构建工具注册表
		for (const tool of this.agent.state.tools) {
			this.toolRegistry.set(tool.name, tool);
		}

		// 初始化对话状态
		this.conversation = {
			session: options.session,
			model: options.model,
			thinkingLevel: options.thinkingLevel ?? this.agent.state.thinkingLevel,
			activeToolNames: options.activeToolNames ?? this.agent.state.tools.map((tool) => tool.name),
			nextTurnQueue: [],
		};
		this.agent.state.model = this.conversation.model;
		this.agent.state.thinkingLevel = this.conversation.thinkingLevel;

		// 配置 Agent 的认证回调
		this.agent.getApiKey = async (provider) => {
			const model = this.conversation.model;
			if (!this.requestAuth || model.provider !== provider) return undefined;
			return (await this.requestAuth(model))?.apiKey;
		};

		// 初始化操作状态
		this.operation = {
			idle: true,
			abortRequested: false,
			steerQueue: [],
			followUpQueue: [],
			pendingMutations: {
				appendMessages: [],
			},
		};

		// ===== 将 Agent 的钩子点桥接到 Harness 的事件系统 =====

		// 上下文变换钩子：允许扩展在发送给 LLM 前修改消息列表
		this.agent.transformContext = async (messages, signal) => {
			const result = await this.emitHook("context", { type: "context", messages: [...messages] }, signal);
			return result?.messages ?? messages;
		};

		// 工具调用前钩子：允许阻止工具执行
		this.agent.beforeToolCall = async ({ toolCall, args }, signal) => {
			const result = await this.emitHook(
				"tool_call",
				{
					type: "tool_call",
					toolCallId: toolCall.id,
					toolName: toolCall.name,
					input: args as Record<string, unknown>,
				},
				signal,
			);
			return result ? { block: result.block, reason: result.reason } : undefined;
		};

		// 工具调用后钩子：允许覆盖工具结果
		this.agent.afterToolCall = async ({ toolCall, args, result, isError }, signal) => {
			const patch = await this.emitHook(
				"tool_result",
				{
					type: "tool_result",
					toolCallId: toolCall.id,
					toolName: toolCall.name,
					input: args as Record<string, unknown>,
					content: result.content,
					details: result.details,
					isError,
				},
				signal,
			);
			return patch
				? { content: patch.content, details: patch.details, isError: patch.isError, terminate: patch.terminate }
				: undefined;
		};

		// 提供商请求载荷钩子
		this.agent.onPayload = async (payload) => {
			const result = await this.emitHook("before_provider_request", { type: "before_provider_request", payload });
			return result?.payload ?? payload;
		};

		// 提供商响应钩子
		this.agent.onResponse = async (response) => {
			const headers = { ...(response.headers as Record<string, string>) };
			await this.emitOwn({ type: "after_provider_response", status: response.status, headers }, this.agent.signal);
		};

		// 订阅 Agent 的底层事件，转发到 Harness 的事件系统
		this.agent.subscribe(async (event, signal) => {
			await this.handleAgentEvent(event, signal);
		});

		// 从会话树同步初始状态
		void this.syncFromTree();
	}

	// ============================================================================
	// 内部工具方法
	// ============================================================================

	/** 创建当前上下文快照 */
	private createContext(signal?: AbortSignal): AgentHarnessContext {
		return {
			env: this.env,
			conversation: this.conversation,
			operation: this.operation,
			abortSignal: signal,
		};
	}

	/** 解析资源（支持静态值或动态函数） */
	private async resolveResources(signal?: AbortSignal): Promise<AgentHarnessResources> {
		return typeof this.resourcesInput === "function"
			? await this.resourcesInput(this.createContext(signal))
			: (this.resourcesInput ?? {});
	}

	/** 获取当前活跃的工具列表 */
	private getActiveTools(): AgentTool[] {
		return this.conversation.activeToolNames
			.map((name) => this.toolRegistry.get(name))
			.filter((tool): tool is AgentTool => tool !== undefined);
	}

	/**
	 * 解析系统提示
	 *
	 * 支持静态字符串或动态生成函数。动态函数接收完整的上下文信息，
	 * 可根据模型、思考级别、工具列表等动态构建系统提示。
	 */
	private async resolveSystemPrompt(resources: AgentHarnessResources): Promise<string> {
		if (!this.systemPrompt) return "You are a helpful assistant.";
		if (typeof this.systemPrompt === "string") return this.systemPrompt;
		return await this.systemPrompt({
			env: this.env,
			session: this.session,
			model: this.conversation.model,
			thinkingLevel: this.conversation.thinkingLevel,
			activeTools: this.getActiveTools(),
			resources,
		});
	}

	// ============================================================================
	// 事件系统
	// ============================================================================

	/** 发出 Harness 自身的事件（不包含 Agent 底层事件） */
	private async emitOwn(event: AgentHarnessOwnEvent, signal?: AbortSignal): Promise<void> {
		for (const listener of this.listeners) {
			await listener(event, signal);
		}
	}

	/** 发出任意事件（Agent 底层事件 + Harness 自身事件） */
	private async emitAny(event: AgentHarnessEvent, signal?: AbortSignal): Promise<void> {
		for (const listener of this.listeners) {
			await listener(event, signal);
		}
	}

	/**
	 * 发出钩子事件并收集结果
	 *
	 * 同一类型的所有钩子按注册顺序依次调用。后者的结果覆盖前者，
	 * 最终返回最后一个非 undefined 的结果。
	 */
	private async emitHook<TType extends keyof AgentHarnessEventResultMap>(
		type: TType,
		event: Extract<AgentHarnessOwnEvent, { type: TType }>,
		signal?: AbortSignal,
	): Promise<AgentHarnessEventResultMap[TType] | undefined> {
		const handlers = this.hooks.get(type);
		if (!handlers || handlers.size === 0) return undefined;
		let lastResult: AgentHarnessEventResultMap[TType] | undefined;
		for (const handler of handlers) {
			const result = await handler(event, this.createContext(signal));
			if (result !== undefined) {
				lastResult = result;
			}
		}
		return lastResult;
	}

	/** 发出队列更新事件 */
	private async emitQueueUpdate(): Promise<void> {
		await this.emitOwn({
			type: "queue_update",
			steer: [...this.operation.steerQueue],
			followUp: [...this.operation.followUpQueue],
			nextTurn: [...this.conversation.nextTurnQueue],
		});
	}

	// ============================================================================
	// 状态同步
	// ============================================================================

	/**
	 * 从会话树重建 Agent 状态
	 *
	 * 重新构建消息历史并更新系统提示，确保 Agent 状态与会话存储一致。
	 * 在初始化、压缩后、树导航后等场景调用。
	 */
	private async syncFromTree(): Promise<void> {
		const context = await this.session.buildContext();
		this.agent.state.messages = context.messages;
		if (context.model && this.conversation.model) {
			// 保持 Harness 层的模型不变，Harness 是模型的权威来源
		}
		this.agent.state.systemPrompt = await this.resolveSystemPrompt(await this.resolveResources());
	}

	/**
	 * 批量应用暂存的变更
	 *
	 * 操作期间的变更（消息追加、模型切换、思考级别切换、工具变更）
	 * 暂存为 pendingMutations，在轮次结束时一次性写入会话存储，
	 * 避免在 Agent 循环中间产生不一致的会话状态。
	 */
	private async applyPendingMutations(): Promise<void> {
		// 追加暂存的消息
		for (const message of this.operation.pendingMutations.appendMessages) {
			await this.session.appendMessage(message);
		}
		this.operation.pendingMutations.appendMessages = [];

		// 应用模型变更
		if (this.operation.pendingMutations.model) {
			const model = this.operation.pendingMutations.model;
			const previousModel = this.conversation.model;
			this.conversation.model = model;
			this.agent.state.model = model;
			await this.session.appendModelChange(model.provider, model.id);
			await this.emitOwn({ type: "model_select", model, previousModel, source: "set" });
			this.operation.pendingMutations.model = undefined;
		}

		// 应用思考级别变更
		if (this.operation.pendingMutations.thinkingLevel !== undefined) {
			const level = this.operation.pendingMutations.thinkingLevel;
			const previousLevel = this.conversation.thinkingLevel;
			this.conversation.thinkingLevel = level;
			this.agent.state.thinkingLevel = level;
			await this.session.appendThinkingLevelChange(level);
			await this.emitOwn({ type: "thinking_level_select", level, previousLevel });
			this.operation.pendingMutations.thinkingLevel = undefined;
		}

		// 应用活跃工具变更
		if (this.operation.pendingMutations.activeToolNames) {
			this.conversation.activeToolNames = [...this.operation.pendingMutations.activeToolNames];
			this.agent.state.tools = this.conversation.activeToolNames
				.map((name) => this.toolRegistry.get(name))
				.filter((tool): tool is (typeof this.agent.state.tools)[number] => tool !== undefined);
			this.operation.pendingMutations.activeToolNames = undefined;
		}

		await this.syncFromTree();
	}

	/**
	 * 处理 Agent 底层事件
	 *
	 * 将 Agent 发出的事件转发到 Harness 的事件系统，
	 * 并在关键节点执行会话持久化和状态管理：
	 *   - message_end：追加消息到会话
	 *   - turn_end：发出存档点、应用暂存变更
	 *   - agent_end：重置操作状态、发出结束事件
	 */
	private async handleAgentEvent(event: AgentEvent, signal?: AbortSignal): Promise<void> {
		// 转发所有 Agent 事件到 Harness 监听器
		await this.emitAny(event, signal);

		// 处理消息开始：从转向/后续队列中移除已识别的消息
		if (event.type === "message_start") {
			const steerIndex = this.operation.steerQueue.indexOf(event.message);
			if (steerIndex !== -1) {
				this.operation.steerQueue.splice(steerIndex, 1);
				await this.emitQueueUpdate();
			} else {
				const followUpIndex = this.operation.followUpQueue.indexOf(event.message);
				if (followUpIndex !== -1) {
					this.operation.followUpQueue.splice(followUpIndex, 1);
					await this.emitQueueUpdate();
				}
			}
		}

		// 消息完成：持久化到会话
		if (event.type === "message_end") {
			await this.session.appendMessage(event.message);
		}

		// 轮次结束：发出存档点并应用暂存变更
		if (event.type === "turn_end") {
			const hadPendingMutations =
				this.operation.pendingMutations.appendMessages.length > 0 ||
				this.operation.pendingMutations.model !== undefined ||
				this.operation.pendingMutations.thinkingLevel !== undefined ||
				this.operation.pendingMutations.activeToolNames !== undefined;
			await this.emitOwn(
				{ type: "save_point", liveOperationId: this.operation.liveOperationId ?? "unknown", hadPendingMutations },
				signal,
			);
			if (hadPendingMutations) {
				await this.applyPendingMutations();
			}
		}

		// Agent 循环结束：重置操作状态
		if (event.type === "agent_end") {
			this.operation.idle = true;
			this.operation.liveOperationId = undefined;
			this.operation.abortRequested = false;
			await this.syncFromTree();
			await this.emitOwn({ type: "settled", nextTurnCount: this.conversation.nextTurnQueue.length }, signal);
		}
	}

	// ============================================================================
	// 公共 API
	// ============================================================================

	/**
	 * 发送提示并等待最终响应
	 *
	 * 完整流程：
	 *   1. 解析资源和系统提示
	 *   2. 展开 nextTurnQueue 中的排队消息
	 *   3. 发出 before_agent_start 钩子（允许扩展修改消息和系统提示）
	 *   4. 调用底层 Agent.prompt()
	 *   5. 从新消息中提取最后一个助手消息作为响应
	 *
	 * @param text - 用户输入文本
	 * @param options - 可选的图片附件
	 * @returns 最终的助手响应消息
	 * @throws 如果 Harness 正忙（已有操作在进行）
	 */
	async prompt(text: string, options?: { images?: ImageContent[] }): Promise<AssistantMessage> {
		if (!this.operation.idle) throw new Error("AgentHarness is busy");
		const beforeLength = this.agent.state.messages.length;
		this.operation.idle = false;
		this.operation.liveOperationId = randomUUID();

		// 解析资源并构建消息列表
		const resources = await this.resolveResources(this.agent.signal);
		let messages: AgentMessage[] = [createUserMessage(text, options?.images)];

		// 合并下一轮队列中的消息
		if (this.conversation.nextTurnQueue.length > 0) {
			messages = [messages[0]!, ...this.conversation.nextTurnQueue];
			this.conversation.nextTurnQueue = [];
			await this.emitQueueUpdate();
		}

		// 解析系统提示
		this.agent.state.systemPrompt = await this.resolveSystemPrompt(resources);

		// 发出启动前钩子
		const beforeResult = await this.emitHook(
			"before_agent_start",
			{
				type: "before_agent_start",
				prompt: text,
				images: options?.images,
				systemPrompt: this.agent.state.systemPrompt,
				resources,
			},
			this.agent.signal,
		);
		if (beforeResult?.messages) messages = [...beforeResult.messages, ...messages];
		if (beforeResult?.systemPrompt) this.agent.state.systemPrompt = beforeResult.systemPrompt;

		// 执行 Agent 循环
		await this.agent.prompt(messages);

		// 提取最后一个助手消息作为响应
		let response: AssistantMessage | undefined;
		const newMessages = this.agent.state.messages.slice(beforeLength);
		for (let i = newMessages.length - 1; i >= 0; i--) {
			const message = newMessages[i]!;
			if (message.role === "assistant") {
				response = message;
				break;
			}
		}
		if (!response) throw new Error("AgentHarness prompt completed without an assistant message");
		return response;
	}

	/**
	 * 通过技能名调用技能
	 *
	 * 查找匹配的技能，展开技能指令后调用 prompt。
	 * @param name - 技能名
	 * @param additionalInstructions - 附加指令
	 * @returns 助手响应
	 */
	async skill(name: string, additionalInstructions?: string): Promise<AssistantMessage> {
		const resources = await this.resolveResources();
		const skill = (resources.skills ?? []).find((candidate) => candidate.name === name);
		if (!skill) throw new Error(`Unknown skill: ${name}`);
		return await this.prompt(expandSkillCommand(skill, additionalInstructions));
	}

	/**
	 * 通过提示模板名调用
	 *
	 * 查找匹配的模板，展开占位符后调用 prompt。
	 * @param name - 模板名
	 * @param args - 模板参数
	 * @returns 助手响应
	 */
	async promptFromTemplate(name: string, args: string[] = []): Promise<AssistantMessage> {
		const resources = await this.resolveResources();
		const template = (resources.promptTemplates ?? []).find((candidate) => candidate.name === name);
		if (!template) throw new Error(`Unknown prompt template: ${name}`);
		return await this.prompt(expandPromptTemplate(template, args));
	}

	/**
	 * 注入转向消息 — 在当前助手轮次结束后插入对话
	 *
	 * 同时添加到 Harness 队列和底层 Agent 队列。
	 * 仅在 Agent 操作中可用。
	 */
	steer(message: AgentMessage): void {
		if (this.operation.idle) throw new Error("Cannot steer while idle");
		this.operation.steerQueue.push(message);
		this.agent.steer(message);
		void this.emitQueueUpdate();
	}

	/**
	 * 注入后续消息 — 在 Agent 即将停止时才处理
	 *
	 * 同时添加到 Harness 队列和底层 Agent 队列。
	 * 仅在 Agent 操作中可用。
	 */
	followUp(message: AgentMessage): void {
		if (this.operation.idle) throw new Error("Cannot follow up while idle");
		this.operation.followUpQueue.push(message);
		this.agent.followUp(message);
		void this.emitQueueUpdate();
	}

	/**
	 * 将消息排入下一轮队列 — 在下次 prompt 时自动发送
	 *
	 * 与 steer/followUp 不同，nextTurn 可以在空闲时使用。
	 */
	nextTurn(message: AgentMessage): void {
		this.conversation.nextTurnQueue.push(message);
		void this.emitQueueUpdate();
	}

	/**
	 * 追加消息到会话
	 *
	 * 空闲时直接写入会话，操作中暂存为 pendingMutation。
	 */
	async appendMessage(message: AgentMessage): Promise<void> {
		if (this.operation.idle) {
			await this.session.appendMessage(message);
			await this.syncFromTree();
		} else {
			this.operation.pendingMutations.appendMessages.push(message);
		}
	}

	/**
	 * 执行 shell 命令
	 *
	 * 委托给 ExecutionEnv.exec，提供统一的命令执行接口。
	 */
	async shell(
		command: string,
		options?: {
			cwd?: string;
			env?: Record<string, string>;
			timeout?: number;
			signal?: AbortSignal;
			onStdout?: (chunk: string) => void;
			onStderr?: (chunk: string) => void;
		},
	): Promise<{ stdout: string; stderr: string; exitCode: number }> {
		return await this.env.exec(command, options);
	}

	/**
	 * 手动触发上下文压缩
	 *
	 * 流程：
	 *   1. 准备压缩数据（确定保留点、待摘要的消息）
	 *   2. 发出 session_before_compact 钩子（允许取消或自定义）
	 *   3. 执行压缩（生成摘要文本）
	 *   4. 将压缩结果写入会话
	 *   5. 从会话树重建状态
	 *
	 * @param customInstructions - 可选的自定义压缩指令
	 * @returns 压缩结果（摘要、保留点、压缩前 Token 数）
	 * @throws 如果 Harness 正忙、无模型、无认证、无内容可压缩或被钩子取消
	 */
	async compact(
		customInstructions?: string,
	): Promise<{ summary: string; firstKeptEntryId: string; tokensBefore: number; details?: unknown }> {
		if (!this.operation.idle) throw new Error("compact() requires idle harness");
		const model = this.conversation.model;
		if (!model) throw new Error("No model set for compaction");
		const auth = await this.requestAuth?.(model);
		if (!auth) throw new Error("No auth available for compaction");

		// 准备压缩数据
		const branchEntries = await this.session.getBranch();
		const preparation = prepareCompaction(branchEntries, DEFAULT_COMPACTION_SETTINGS);
		if (!preparation) throw new Error("Nothing to compact");

		// 发出压缩前钩子
		const hookResult = await this.emitHook("session_before_compact", {
			type: "session_before_compact",
			preparation,
			branchEntries,
			customInstructions,
			signal: new AbortController().signal,
		});
		if (hookResult?.cancel) throw new Error("Compaction cancelled");

		// 使用钩子提供的压缩结果，或执行内置压缩
		const provided = hookResult?.compaction;
		const result =
			provided ??
			(await compact(
				preparation,
				model,
				auth.apiKey,
				auth.headers,
				customInstructions,
				undefined,
				this.conversation.thinkingLevel,
			));

		// 写入压缩条目并同步状态
		const entryId = await this.session.appendCompaction(
			result.summary,
			result.firstKeptEntryId,
			result.tokensBefore,
			result.details,
			provided !== undefined,
		);
		const entry = await this.session.getEntry(entryId);
		await this.syncFromTree();
		if (entry?.type === "compaction") {
			await this.emitOwn({ type: "session_compact", compactionEntry: entry, fromHook: provided !== undefined });
		}
		return result;
	}

	/**
	 * 在会话树中导航到指定条目
	 *
	 * 支持跳转到会话历史中的任意点并继续对话。可选生成分支摘要。
	 *
	 * 流程：
	 *   1. 收集需要摘要的条目
	 *   2. 发出 session_before_tree 钩子
	 *   3. 可选：生成分支摘要
	 *   4. 移动会话叶节点
	 *   5. 同步状态
	 *
	 * @param targetId - 目标条目 ID
	 * @param options - 导航选项（是否生成摘要、自定义指令等）
	 * @returns 导航结果（是否取消、编辑器文本、摘要条目）
	 */
	async navigateTree(
		targetId: string,
		options?: { summarize?: boolean; customInstructions?: string; replaceInstructions?: boolean; label?: string },
	): Promise<NavigateTreeResult> {
		if (!this.operation.idle) throw new Error("navigateTree() requires idle harness");
		const oldLeafId = await this.session.getLeafId();
		// 已经在目标位置，无需导航
		if (oldLeafId === targetId) return { cancelled: false };
		const targetEntry = await this.session.getEntry(targetId);
		if (!targetEntry) throw new Error(`Entry ${targetId} not found`);

		// 收集需要摘要的条目
		const { entries, commonAncestorId } = await collectEntriesForBranchSummary(this.session, oldLeafId, targetId);
		const preparation = {
			targetId,
			oldLeafId,
			commonAncestorId,
			entriesToSummarize: entries,
			userWantsSummary: options?.summarize ?? false,
			customInstructions: options?.customInstructions,
			replaceInstructions: options?.replaceInstructions,
			label: options?.label,
		};

		// 发出树导航前钩子
		const signal = new AbortController().signal;
		const hookResult = await this.emitHook("session_before_tree", {
			type: "session_before_tree",
			preparation,
			signal,
		});
		if (hookResult?.cancel) return { cancelled: true };

		// 生成摘要（如果请求）
		let summaryEntry: any | undefined;
		let summaryText: string | undefined = hookResult?.summary?.summary;
		let summaryDetails: unknown = hookResult?.summary?.details;
		if (!summaryText && options?.summarize && entries.length > 0) {
			const model = this.conversation.model;
			if (!model) throw new Error("No model set for branch summary");
			const auth = await this.requestAuth?.(model);
			if (!auth) throw new Error("No auth available for branch summary");
			const branchSummary = await generateBranchSummary(entries, {
				model,
				apiKey: auth.apiKey,
				headers: auth.headers,
				signal: new AbortController().signal,
				customInstructions: hookResult?.customInstructions ?? options?.customInstructions,
				replaceInstructions: hookResult?.replaceInstructions ?? options?.replaceInstructions,
			});
			if (branchSummary.aborted) return { cancelled: true };
			if (branchSummary.error) throw new Error(branchSummary.error);
			summaryText = branchSummary.summary;
			summaryDetails = {
				readFiles: branchSummary.readFiles ?? [],
				modifiedFiles: branchSummary.modifiedFiles ?? [],
			};
		}

		// 确定新的叶节点位置
		let editorText: string | undefined;
		let newLeafId: string | null;
		// 如果目标是用户消息，跳到其父节点（即消息之前），并将消息文本放入编辑器
		if (targetEntry.type === "message" && targetEntry.message.role === "user") {
			newLeafId = targetEntry.parentId;
			const content = targetEntry.message.content;
			editorText =
				typeof content === "string"
					? content
					: content
							.filter((c): c is { readonly type: "text"; readonly text: string } => c.type === "text")
							.map((c) => c.text)
							.join("");
		} else if (targetEntry.type === "custom_message") {
			newLeafId = targetEntry.parentId;
			editorText =
				typeof targetEntry.content === "string"
					? targetEntry.content
					: targetEntry.content
							.filter((c): c is { readonly type: "text"; readonly text: string } => c.type === "text")
							.map((c) => c.text)
							.join("");
		} else {
			// 其他类型条目（助手消息、压缩条目等）直接跳到该条目
			newLeafId = targetId;
		}

		// 执行会话树的移动操作
		const summaryId = await this.session.moveTo(
			newLeafId,
			summaryText
				? {
						summary: summaryText,
						details: summaryDetails,
						fromHook: hookResult?.summary !== undefined,
					}
				: undefined,
		);
		if (summaryId) {
			summaryEntry = await this.session.getEntry(summaryId);
		}

		// 同步状态并发出事件
		await this.syncFromTree();
		await this.emitOwn({
			type: "session_tree",
			newLeafId: await this.session.getLeafId(),
			oldLeafId,
			summaryEntry,
			fromHook: hookResult?.summary !== undefined,
		});
		return { cancelled: false, editorText, summaryEntry };
	}

	/**
	 * 切换模型
	 *
	 * 空闲时立即生效，操作中暂存为 pendingMutation。
	 * @param model - 新模型
	 */
	async setModel(model: Model<any>): Promise<void> {
		if (this.operation.idle) {
			const previousModel = this.conversation.model;
			this.conversation.model = model;
			this.agent.state.model = model;
			await this.session.appendModelChange(model.provider, model.id);
			await this.emitOwn({ type: "model_select", model, previousModel, source: "set" });
		} else {
			this.operation.pendingMutations.model = model;
		}
	}

	/**
	 * 切换思考级别
	 *
	 * 空闲时立即生效，操作中暂存为 pendingMutation。
	 * @param level - 新的思考级别
	 */
	async setThinkingLevel(level: ThinkingLevel): Promise<void> {
		if (this.operation.idle) {
			const previousLevel = this.conversation.thinkingLevel;
			this.conversation.thinkingLevel = level;
			this.agent.state.thinkingLevel = level;
			await this.session.appendThinkingLevelChange(level);
			await this.emitOwn({ type: "thinking_level_select", level, previousLevel });
		} else {
			this.operation.pendingMutations.thinkingLevel = level;
		}
	}

	/**
	 * 设置活跃工具列表
	 *
	 * 空闲时立即生效，操作中暂存为 pendingMutation。
	 * @param toolNames - 工具名列表
	 */
	async setActiveTools(toolNames: string[]): Promise<void> {
		if (this.operation.idle) {
			this.conversation.activeToolNames = [...toolNames];
			this.agent.state.tools = this.getActiveTools();
		} else {
			this.operation.pendingMutations.activeToolNames = [...toolNames];
		}
	}

	/**
	 * 中止当前操作
	 *
	 * 清除所有队列中的转向和后续消息，中止 Agent 循环，等待 Agent 空闲。
	 * @returns 被清除的消息列表
	 */
	async abort(): Promise<AbortResult> {
		this.operation.abortRequested = true;
		const clearedSteer = [...this.operation.steerQueue];
		const clearedFollowUp = [...this.operation.followUpQueue];
		this.operation.steerQueue = [];
		this.operation.followUpQueue = [];
		this.agent.clearAllQueues();
		await this.emitQueueUpdate();
		this.agent.abort();
		await this.agent.waitForIdle();
		await this.emitOwn({ type: "abort", clearedSteer, clearedFollowUp });
		return { clearedSteer, clearedFollowUp };
	}

	/** 等待 Agent 变为空闲状态 */
	async waitForIdle(): Promise<void> {
		await this.agent.waitForIdle();
	}

	/**
	 * 订阅所有 Harness 事件（包括 Agent 底层事件和 Harness 自身事件）
	 * @param listener - 事件监听器
	 * @returns 取消订阅的函数
	 */
	subscribe(listener: (event: AgentHarnessEvent, signal?: AbortSignal) => Promise<void> | void): () => void {
		this.listeners.add(listener);
		return () => this.listeners.delete(listener);
	}

	/**
	 * 注册事件钩子 — 可拦截并修改特定类型的事件
	 *
	 * 与 subscribe 不同，钩子可以返回结果来修改行为（如阻止工具执行、
	 * 覆盖工具结果、取消压缩等）。同一类型的多个钩子按注册顺序调用，
	 * 后者的结果覆盖前者。
	 *
	 * @param type - 事件类型
	 * @param handler - 钩子处理函数
	 * @returns 取消注册的函数
	 */
	on<TType extends keyof AgentHarnessEventResultMap>(
		type: TType,
		handler: (
			event: Extract<import("./types.js").AgentHarnessOwnEvent, { type: TType }>,
			ctx: AgentHarnessContext,
		) => Promise<AgentHarnessEventResultMap[TType]> | AgentHarnessEventResultMap[TType],
	): () => void {
		let handlers = this.hooks.get(type);
		if (!handlers) {
			handlers = new Set();
			this.hooks.set(type, handlers);
		}
		handlers.add(handler as any);
		return () => handlers!.delete(handler as any);
	}
}
