/**
 * @fileoverview 软件开发工具包（SDK）入口模块
 *
 * 本文件是 pi coding-agent 的公共 API 层，对外暴露 `createAgentSession` 作为
 * 创建编码代理会话的唯一入口点。职责包括：
 *
 * 1. **会话组装**：将认证存储、模型注册表、设置管理器、会话管理器、
 *    资源加载器等核心组件组装为一个完整的 AgentSession。
 * 2. **模型解析**：按优先级依次尝试 —— 调用方显式指定 → 恢复已有会话的模型 →
 *    用户设置中的默认值 → 注册表中第一个可用模型。
 * 3. **思考等级恢复与钳位**：恢复会话时沿用之前的 thinking level，
 *    并根据模型能力进行 clamp。
 * 4. **工具过滤**：支持 `noTools` / `tools` 白名单来控制初始激活的工具集。
 * 5. **图片阻断**：当 blockImages 设置启用时，在 convertToLlm 层过滤消息中的图片内容。
 * 6. **厂商属性头**：为 OpenRouter / Cloudflare 等特定 provider 附加归因 HTTP 头。
 *
 * 同时重新导出工具工厂函数和扩展相关类型，使消费方无需深入子模块即可使用。
 */

import { join } from "node:path";
import { Agent, type AgentMessage, type ThinkingLevel } from "@earendil-works/pi-agent-core";
import { clampThinkingLevel, type Message, type Model, streamSimple } from "@earendil-works/pi-ai";
import { getAgentDir } from "../config.js";
import { AgentSession } from "./agent-session.js";
import { formatNoModelsAvailableMessage } from "./auth-guidance.js";
import { AuthStorage } from "./auth-storage.js";
import { DEFAULT_THINKING_LEVEL } from "./defaults.js";
import type { ExtensionRunner, LoadExtensionsResult, SessionStartEvent, ToolDefinition } from "./extensions/index.js";
import { convertToLlm } from "./messages.js";
import { ModelRegistry } from "./model-registry.js";
import { findInitialModel } from "./model-resolver.js";
import type { ResourceLoader } from "./resource-loader.js";
import { DefaultResourceLoader } from "./resource-loader.js";
import { getDefaultSessionDir, SessionManager } from "./session-manager.js";
import { SettingsManager } from "./settings-manager.js";
import { isInstallTelemetryEnabled } from "./telemetry.js";
import { time } from "./timings.js";
import {
	createBashTool,
	createCodingTools,
	createEditTool,
	createFindTool,
	createGrepTool,
	createLsTool,
	createReadOnlyTools,
	createReadTool,
	createWriteTool,
	type ToolName,
	withFileMutationQueue,
} from "./tools/index.js";

/** 创建代理会话的配置选项 */
export interface CreateAgentSessionOptions {
	/** 项目工作目录，用于本地发现配置文件。默认值：process.cwd() */
	cwd?: string;
	/** 全局配置目录。默认值：~/.pi/agent */
	agentDir?: string;

	/** 认证凭证存储。默认值：AuthStorage.create(agentDir/auth.json) */
	authStorage?: AuthStorage;
	/** 模型注册表。默认值：ModelRegistry.create(authStorage, agentDir/models.json) */
	modelRegistry?: ModelRegistry;

	/** 要使用的模型。默认值：依次从设置、注册表中查找 */
	model?: Model<any>;
	/** 思考深度等级。默认值：从设置获取，否则为 'medium'（按模型能力钳位） */
	thinkingLevel?: ThinkingLevel;
	/** 可用于循环切换的模型列表（交互模式下 Ctrl+P 触发） */
	scopedModels?: Array<{ model: Model<any>; thinkingLevel?: ThinkingLevel }>;

	/**
	 * 可选的默认工具抑制模式，当未提供显式白名单时生效。
	 *
	 * - "all"：不启用任何工具
	 * - "builtin"：禁用默认内置工具（read、bash、edit、write），
	 *   但保留扩展/自定义工具
	 */
	noTools?: "all" | "builtin";
	/**
	 * 可选的工具名称白名单。
	 *
	 * 省略时启用默认内置工具（read、bash、edit、write），
	 * 扩展/自定义工具也保持启用，除非 `noTools` 改变了默认行为。
	 * 提供时，仅启用列表中的工具。
	 */
	tools?: string[];
	/** 要注册的自定义工具（在内置工具之外额外添加） */
	customTools?: ToolDefinition[];

	/** 资源加载器。省略时使用 DefaultResourceLoader */
	resourceLoader?: ResourceLoader;

	/** 会话管理器。默认值：SessionManager.create(cwd) */
	sessionManager?: SessionManager;

	/** 设置管理器。默认值：SettingsManager.create(cwd, agentDir) */
	settingsManager?: SettingsManager;
	/** 会话启动事件元数据，传递给扩展运行时以完成启动初始化 */
	sessionStartEvent?: SessionStartEvent;
}

/** createAgentSession 的返回结果 */
export interface CreateAgentSessionResult {
	/** 创建的代理会话实例 */
	session: AgentSession;
	/** 扩展加载结果（用于交互模式下的 UI 上下文设置） */
	extensionsResult: LoadExtensionsResult;
	/** 当恢复的会话使用了与保存时不同的模型时产生的警告信息 */
	modelFallbackMessage?: string;
}

// 重新导出

export * from "./agent-session-runtime.js";
export type {
	ExtensionAPI,
	ExtensionCommandContext,
	ExtensionContext,
	ExtensionFactory,
	SlashCommandInfo,
	SlashCommandSource,
	ToolDefinition,
} from "./extensions/index.js";
export type { PromptTemplate } from "./prompt-templates.js";
export type { Skill } from "./skills.js";
export type { Tool } from "./tools/index.js";

export {
	withFileMutationQueue,
	// 工具工厂函数（支持自定义工作目录）
	createCodingTools,
	createReadOnlyTools,
	createReadTool,
	createBashTool,
	createEditTool,
	createWriteTool,
	createGrepTool,
	createFindTool,
	createLsTool,
};

// 辅助函数

/**
 * 获取默认的代理配置目录路径。
 * 封装 getAgentDir() 以便在本模块内统一调用。
 *
 * @returns 代理配置目录的绝对路径
 */
function getDefaultAgentDir(): string {
	return getAgentDir();
}

/**
 * 根据模型 provider 和遥测设置，生成厂商归因 HTTP 头。
 * 仅在用户启用安装遥测且模型为特定厂商时才附加。
 *
 * @param model - 当前使用的模型实例
 * @param settingsManager - 设置管理器，用于检查遥测开关
 * @returns 归因头对象，无需附加时返回 undefined
 */
function getAttributionHeaders(
	model: Model<any>,
	settingsManager: SettingsManager,
): Record<string, string> | undefined {
	if (!isInstallTelemetryEnabled(settingsManager)) {
		return undefined;
	}

	// OpenRouter 要求通过 HTTP-Referer 和自定义头标识来源应用
	if (model.provider === "openrouter" || model.baseUrl.includes("openrouter.ai")) {
		return {
			"HTTP-Referer": "https://pi.dev",
			"X-OpenRouter-Title": "pi",
			"X-OpenRouter-Categories": "cli-agent",
		};
	}

	// Cloudflare Workers AI / AI Gateway 需要通过 User-Agent 标识
	if (
		model.provider === "cloudflare-workers-ai" ||
		model.provider === "cloudflare-ai-gateway" ||
		model.baseUrl.includes("api.cloudflare.com") ||
		model.baseUrl.includes("gateway.ai.cloudflare.com")
	) {
		return {
			"User-Agent": "pi-coding-agent",
		};
	}

	return undefined;
}

/**
 * 创建一个完整的代理会话（AgentSession）。
 *
 * 本函数是 pi SDK 的核心入口，负责将认证、模型、工具、扩展、
 * 会话持久化等所有子系统组装在一起。支持从零创建新会话，
 * 也支持恢复已有会话（包括模型和思考等级的恢复）。
 *
 * @param options - 创建会话的配置选项，全部可选
 * @returns 包含会话实例、扩展加载结果和可选模型回退警告的对象
 *
 * @example
 * ```typescript
 * // 最简调用 —— 使用全部默认值
 * const { session } = await createAgentSession();
 *
 * // 指定模型
 * import { getModel } from '@earendil-works/pi-ai';
 * const { session } = await createAgentSession({
 *   model: getModel('anthropic', 'claude-opus-4-5'),
 *   thinkingLevel: 'high',
 * });
 *
 * // 继续上一次会话
 * const { session, modelFallbackMessage } = await createAgentSession({
 *   continueSession: true,
 * });
 *
 * // 完全控制 —— 自定义资源加载器和会话管理器
 * const loader = new DefaultResourceLoader({
 *   cwd: process.cwd(),
 *   agentDir: getAgentDir(),
 *   settingsManager: SettingsManager.create(),
 * });
 * await loader.reload();
 * const { session } = await createAgentSession({
 *   model: myModel,
 *   tools: [readTool, bashTool],
 *   resourceLoader: loader,
 *   sessionManager: SessionManager.inMemory(),
 * });
 * ```
 */
export async function createAgentSession(options: CreateAgentSessionOptions = {}): Promise<CreateAgentSessionResult> {
	const cwd = options.cwd ?? options.sessionManager?.getCwd() ?? process.cwd();
	const agentDir = options.agentDir ?? getDefaultAgentDir();
	let resourceLoader = options.resourceLoader;

	// 使用调用方提供的实例，或创建默认的认证存储和模型注册表
	const authPath = options.agentDir ? join(agentDir, "auth.json") : undefined;
	const modelsPath = options.agentDir ? join(agentDir, "models.json") : undefined;
	const authStorage = options.authStorage ?? AuthStorage.create(authPath);
	const modelRegistry = options.modelRegistry ?? ModelRegistry.create(authStorage, modelsPath);

	const settingsManager = options.settingsManager ?? SettingsManager.create(cwd, agentDir);
	const sessionManager = options.sessionManager ?? SessionManager.create(cwd, getDefaultSessionDir(cwd, agentDir));

	// 若未提供资源加载器，创建默认实例并加载配置
	if (!resourceLoader) {
		resourceLoader = new DefaultResourceLoader({ cwd, agentDir, settingsManager });
		await resourceLoader.reload();
		time("resourceLoader.reload");
	}

	// 检查是否有已存在的会话数据可供恢复
	const existingSession = sessionManager.buildSessionContext();
	const hasExistingSession = existingSession.messages.length > 0;
	const hasThinkingEntry = sessionManager.getBranch().some((entry) => entry.type === "thinking_level_change");

	let model = options.model;
	let modelFallbackMessage: string | undefined;

	// 如果会话有历史数据，尝试从中恢复模型
	if (!model && hasExistingSession && existingSession.model) {
		const restoredModel = modelRegistry.find(existingSession.model.provider, existingSession.model.modelId);
		if (restoredModel && modelRegistry.hasConfiguredAuth(restoredModel)) {
			model = restoredModel;
		}
		// 模型不可用（未认证或已删除），记录回退信息
		if (!model) {
			modelFallbackMessage = `Could not restore model ${existingSession.model.provider}/${existingSession.model.modelId}`;
		}
	}

	// 仍然没有模型，通过 findInitialModel 按优先级查找（设置默认值 → provider 默认值）
	if (!model) {
		const result = await findInitialModel({
			scopedModels: [],
			isContinuing: hasExistingSession,
			defaultProvider: settingsManager.getDefaultProvider(),
			defaultModelId: settingsManager.getDefaultModel(),
			defaultThinkingLevel: settingsManager.getDefaultThinkingLevel(),
			modelRegistry,
		});
		model = result.model;
		if (!model) {
			// 完全没有可用模型
			modelFallbackMessage = formatNoModelsAvailableMessage();
		} else if (modelFallbackMessage) {
			// 原模型不可用但找到了替代模型
			modelFallbackMessage += `. Using ${model.provider}/${model.id}`;
		}
	}

	let thinkingLevel = options.thinkingLevel;

	// 如果会话有历史数据，从中恢复思考等级
	if (thinkingLevel === undefined && hasExistingSession) {
		thinkingLevel = hasThinkingEntry
			? (existingSession.thinkingLevel as ThinkingLevel)
			: (settingsManager.getDefaultThinkingLevel() ?? DEFAULT_THINKING_LEVEL);
	}

	// 回退到设置中的默认值
	if (thinkingLevel === undefined) {
		thinkingLevel = settingsManager.getDefaultThinkingLevel() ?? DEFAULT_THINKING_LEVEL;
	}

	// 按模型能力钳位思考等级，防止模型不支持时出错
	if (!model) {
		thinkingLevel = "off";
	} else {
		thinkingLevel = clampThinkingLevel(model, thinkingLevel) as ThinkingLevel;
	}

	// 默认激活的内置工具名称列表
	const defaultActiveToolNames: ToolName[] = ["read", "bash", "edit", "write"];
	// 允许的工具白名单：显式提供时使用，noTools="all" 时为空，否则 undefined（不限）
	const allowedToolNames = options.tools ?? (options.noTools === "all" ? [] : undefined);
	// 初始激活的工具名列表：显式白名单 > noTools 空 > 默认内置
	const initialActiveToolNames: string[] = options.tools
		? [...options.tools]
		: options.noTools
			? []
			: defaultActiveToolNames;

	let agent: Agent;

	// 构造带图片阻断能力的 convertToLlm 包装器（纵深防御策略）
	const convertToLlmWithBlockImages = (messages: AgentMessage[]): Message[] => {
		const converted = convertToLlm(messages);
		// 动态检查设置，使会话中途的修改也能生效
		if (!settingsManager.getBlockImages()) {
			return converted;
		}
		// 过滤所有消息中的 ImageContent，替换为文本占位符
		return converted.map((msg) => {
			if (msg.role === "user" || msg.role === "toolResult") {
				const content = msg.content;
				if (Array.isArray(content)) {
					const hasImages = content.some((c) => c.type === "image");
					if (hasImages) {
						const filteredContent = content
							.map((c) =>
								c.type === "image" ? { type: "text" as const, text: "Image reading is disabled." } : c,
							)
							.filter(
								(c, i, arr) =>
									// 去重连续的 "Image reading is disabled." 文本块
									!(
										c.type === "text" &&
										c.text === "Image reading is disabled." &&
										i > 0 &&
										arr[i - 1].type === "text" &&
										(arr[i - 1] as { type: "text"; text: string }).text === "Image reading is disabled."
									),
							);
						return { ...msg, content: filteredContent };
					}
				}
			}
			return msg;
		});
	};

	// 引用容器，延迟绑定扩展运行时（Agent 构造时运行时尚未创建）
	const extensionRunnerRef: { current?: ExtensionRunner } = {};

	agent = new Agent({
		initialState: {
			systemPrompt: "",
			model,
			thinkingLevel,
			tools: [],
		},
		convertToLlm: convertToLlmWithBlockImages,
		streamFn: async (model, context, options) => {
			const auth = await modelRegistry.getApiKeyAndHeaders(model);
			if (!auth.ok) {
				throw new Error(auth.error);
			}
			const providerRetrySettings = settingsManager.getProviderRetrySettings();
			const attributionHeaders = getAttributionHeaders(model, settingsManager);
			return streamSimple(model, context, {
				...options,
				apiKey: auth.apiKey,
				// 优先使用调用方传入的超时/重试配置，回退到 provider 级别设置
				timeoutMs: options?.timeoutMs ?? providerRetrySettings.timeoutMs,
				maxRetries: options?.maxRetries ?? providerRetrySettings.maxRetries,
				maxRetryDelayMs: options?.maxRetryDelayMs ?? providerRetrySettings.maxRetryDelayMs,
				// 合并归因头、认证头和调用方自定义头
				headers:
					attributionHeaders || auth.headers || options?.headers
						? { ...attributionHeaders, ...auth.headers, ...options?.headers }
						: undefined,
			});
		},
		// 扩展钩子：在发送请求给 provider 之前，允许扩展修改 payload
		onPayload: async (payload, _model) => {
			const runner = extensionRunnerRef.current;
			if (!runner?.hasHandlers("before_provider_request")) {
				return payload;
			}
			return runner.emitBeforeProviderRequest(payload);
		},
		// 扩展钩子：在收到 provider 响应后通知扩展
		onResponse: async (response, _model) => {
			const runner = extensionRunnerRef.current;
			if (!runner?.hasHandlers("after_provider_response")) {
				return;
			}
			await runner.emit({
				type: "after_provider_response",
				status: response.status,
				headers: response.headers,
			});
		},
		sessionId: sessionManager.getSessionId(),
		// 扩展钩子：允许扩展在上下文发送给 provider 之前修改消息
		transformContext: async (messages) => {
			const runner = extensionRunnerRef.current;
			if (!runner) return messages;
			return runner.emitContext(messages);
		},
		steeringMode: settingsManager.getSteeringMode(),
		followUpMode: settingsManager.getFollowUpMode(),
		transport: settingsManager.getTransport(),
		thinkingBudgets: settingsManager.getThinkingBudgets(),
		maxRetryDelayMs: settingsManager.getProviderRetrySettings().maxRetryDelayMs,
	});

	// 恢复会话历史消息，或为新会话保存初始状态
	if (hasExistingSession) {
		agent.state.messages = existingSession.messages;
		// 旧会话若没有 thinking_level_change 记录则补一条
		if (!hasThinkingEntry) {
			sessionManager.appendThinkingLevelChange(thinkingLevel);
		}
	} else {
		// 新会话：记录初始模型和思考等级，以便后续恢复
		if (model) {
			sessionManager.appendModelChange(model.provider, model.id);
		}
		sessionManager.appendThinkingLevelChange(thinkingLevel);
	}

	const session = new AgentSession({
		agent,
		sessionManager,
		settingsManager,
		cwd,
		scopedModels: options.scopedModels,
		resourceLoader,
		customTools: options.customTools,
		modelRegistry,
		initialActiveToolNames,
		allowedToolNames,
		extensionRunnerRef,
		sessionStartEvent: options.sessionStartEvent,
	});
	const extensionsResult = resourceLoader.getExtensions();

	return {
		session,
		extensionsResult,
		modelFallbackMessage,
	};
}
