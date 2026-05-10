/**
 * @fileoverview AI 包的核心类型定义，定义了统一 LLM 接口层使用的所有公共类型。
 *
 * 核心概念：
 *   - Api / KnownApi：LLM 服务 API 的标识符（如 "openai-completions"、"anthropic-messages"）
 *   - Provider / KnownProvider：提供商标识符（如 "openai"、"anthropic"、"google"）
 *   - Model：模型定义，包含 ID、能力、成本、上下文窗口等元信息
 *   - Message：消息体系（UserMessage、AssistantMessage、ToolResultMessage）
 *   - Context：对话上下文，包含系统提示、消息历史和工具列表
 *   - StreamFunction / StreamOptions：流式调用函数签名和选项
 *   - AssistantMessageEvent：事件流协议，定义了 start/delta/done/error 等事件类型
 *
 * 设计原则：
 *   所有 LLM 提供商共享统一的类型接口，提供商实现只需将各自的 API 格式
 *   转换为这些标准类型，从而实现多提供商的无缝切换。
 */

import type { AssistantMessageDiagnostic } from "./utils/diagnostics.js";
import type { AssistantMessageEventStream } from "./utils/event-stream.js";

export type { AssistantMessageEventStream } from "./utils/event-stream.js";

// ─── API 标识符 ───────────────────────────────────────────────
// 每个 API 标识符对应一种具体的 LLM 服务协议（如 OpenAI 的 Chat Completions、
// Anthropic 的 Messages API）。提供商通过实现对应 API 的流式函数来接入系统。

/**
 * 内置已知的 API 协议标识符
 * 每种标识符对应一套不同的请求/响应格式
 */
export type KnownApi =
	| "openai-completions" // OpenAI Chat Completions API（使用最广泛，众多兼容提供商）
	| "mistral-conversations" // Mistral Conversations API
	| "openai-responses" // OpenAI Responses API（新一代 API）
	| "azure-openai-responses" // Azure 托管的 OpenAI Responses API
	| "openai-codex-responses" // OpenAI Codex Responses API
	| "anthropic-messages" // Anthropic Messages API（Claude 系列模型）
	| "bedrock-converse-stream" // AWS Bedrock Converse Stream API
	| "google-generative-ai" // Google Generative AI API（Gemini 系列模型）
	| "google-vertex"; // Google Vertex AI API

/**
 * API 标识符，支持已知标识符和自定义字符串
 * `KnownApi` 提供类型提示和校验，`(string & {})` 允许扩展自定义 API
 */
export type Api = KnownApi | (string & {});

/**
 * 内置已知的图像生成 API 标识符
 */
export type KnownImagesApi = "openrouter-images";

/**
 * 图像生成 API 标识符
 */
export type ImagesApi = KnownImagesApi | (string & {});

// ─── 提供商标识符 ─────────────────────────────────────────────
// 提供商代表一个 LLM 服务供应商（如 OpenAI、Anthropic），
// 一个提供商可能使用多种 API 协议。

/**
 * 内置已知的 LLM 提供商
 */
export type KnownProvider =
	| "amazon-bedrock"
	| "anthropic"
	| "google"
	| "google-vertex"
	| "openai"
	| "azure-openai-responses"
	| "openai-codex"
	| "deepseek"
	| "github-copilot"
	| "xai"
	| "groq"
	| "cerebras"
	| "openrouter"
	| "vercel-ai-gateway"
	| "zai"
	| "mistral"
	| "minimax"
	| "minimax-cn"
	| "moonshotai"
	| "moonshotai-cn"
	| "huggingface"
	| "fireworks"
	| "together"
	| "opencode"
	| "opencode-go"
	| "kimi-coding"
	| "cloudflare-workers-ai"
	| "cloudflare-ai-gateway"
	| "xiaomi"
	| "xiaomi-token-plan-cn"
	| "xiaomi-token-plan-ams"
	| "xiaomi-token-plan-sgp";

/**
 * 提供商标识符，支持已知和自定义提供商
 */
export type Provider = KnownProvider | string;

/**
 * 内置已知的图像生成提供商
 */
export type KnownImagesProvider = "openrouter";

/**
 * 图像生成提供商标识符
 */
export type ImagesProvider = KnownImagesProvider | string;

// ─── 推理/思考级别 ─────────────────────────────────────────────
// 用于控制模型的"思考深度"，不同提供商映射到各自的具体参数。

/**
 * 思考深度级别（不含关闭选项）
 */
export type ThinkingLevel = "minimal" | "low" | "medium" | "high" | "xhigh";

/**
 * 模型思考级别，在 ThinkingLevel 基础上增加 "off"（关闭思考）
 */
export type ModelThinkingLevel = "off" | ThinkingLevel;

/**
 * 思考级别到提供商特定值的映射表
 * 缺失的键使用提供商默认值，null 表示该级别不被支持
 */
export type ThinkingLevelMap = Partial<Record<ModelThinkingLevel, string | null>>;

/**
 * 各思考级别的 Token 预算（仅适用于基于 Token 控制思考的提供商）
 */
export interface ThinkingBudgets {
	minimal?: number;
	low?: number;
	medium?: number;
	high?: number;
}

// ─── 通用选项类型 ─────────────────────────────────────────────

/**
 * 提示缓存保留策略
 * - "none"：不缓存
 * - "short"：短期缓存（默认）
 * - "long"：长期缓存（如 Anthropic 的 1 小时 TTL）
 */
export type CacheRetention = "none" | "short" | "long";

/**
 * HTTP 传输协议类型
 * - "sse"：Server-Sent Events
 * - "websocket"：WebSocket
 * - "websocket-cached"：带缓存的 WebSocket
 * - "auto"：自动选择
 */
export type Transport = "sse" | "websocket" | "websocket-cached" | "auto";

/**
 * 提供商 HTTP 响应的封装
 */
export interface ProviderResponse {
	status: number;
	headers: Record<string, string>;
}

/**
 * 流式调用的完整选项
 * 所有提供商共享的基础选项，各提供商可从中提取自己支持的字段
 */
export interface StreamOptions {
	/** 生成温度，控制输出的随机性，0.0 为确定性输出 */
	temperature?: number;
	/** 最大输出 Token 数 */
	maxTokens?: number;
	/** 中止信号，用于取消正在进行的流式请求 */
	signal?: AbortSignal;
	/** API 密钥，覆盖默认凭据 */
	apiKey?: string;
	/**
	 * 首选传输协议，不支持此选项的提供商会忽略
	 */
	transport?: Transport;
	/**
	 * 提示缓存保留偏好，提供商标映射到各自支持的值
	 * 默认值："short"
	 */
	cacheRetention?: CacheRetention;
	/**
	 * 可选的会话标识符，用于支持基于会话的缓存
	 * 提供商可用于启用提示缓存、请求路由等会话感知功能
	 */
	sessionId?: string;
	/**
	 * 发送前的请求体拦截/替换回调
	 * 返回 undefined 保持请求体不变，返回新值则替换原始请求体
	 */
	onPayload?: (payload: unknown, model: Model<Api>) => unknown | undefined | Promise<unknown | undefined>;
	/**
	 * HTTP 响应接收后的回调（在响应体流被消费之前触发）
	 */
	onResponse?: (response: ProviderResponse, model: Model<Api>) => void | Promise<void>;
	/**
	 * 自定义 HTTP 请求头，与提供商默认请求头合并，可覆盖默认值
	 * 部分提供商不支持（如 AWS Bedrock 使用 SDK 认证）
	 */
	headers?: Record<string, string>;
	/** HTTP 请求超时时间（毫秒），如 OpenAI 和 Anthropic SDK 默认 10 分钟 */
	timeoutMs?: number;
	/** 最大重试次数，如 OpenAI 和 Anthropic SDK 默认 2 次 */
	maxRetries?: number;
	/**
	 * 服务端请求长时间等待时的最大延迟上限（毫秒）
	 * 超过此值则立即失败并返回包含请求延迟的错误，允许上层重试逻辑处理
	 * 默认值：60000（60 秒），设为 0 禁用上限
	 */
	maxRetryDelayMs?: number;
	/**
	 * 附加到 API 请求的元数据
	 * 提供商提取自己理解的字段，忽略其余（如 Anthropic 使用 user_id 进行滥用追踪）
	 */
	metadata?: Record<string, unknown>;
}

/**
 * 提供商特定的流式调用选项，在 StreamOptions 基础上允许任意额外字段
 */
export type ProviderStreamOptions = StreamOptions & Record<string, unknown>;

/**
 * 图像生成调用的选项
 * 结构与 StreamOptions 类似但用于图像生成场景
 */
export interface ImagesOptions {
	signal?: AbortSignal;
	apiKey?: string;
	/**
	 * 发送前的请求体拦截/替换回调
	 */
	onPayload?: (payload: unknown, model: ImagesModel<ImagesApi>) => unknown | undefined | Promise<unknown | undefined>;
	/**
	 * HTTP 响应接收后的回调
	 */
	onResponse?: (response: ProviderResponse, model: ImagesModel<ImagesApi>) => void | Promise<void>;
	/**
	 * 自定义 HTTP 请求头
	 */
	headers?: Record<string, string>;
	/** HTTP 请求超时时间（毫秒） */
	timeoutMs?: number;
	/** 最大重试次数 */
	maxRetries?: number;
	/**
	 * 服务端请求长时间等待时的最大延迟上限（毫秒）
	 * 默认值：60000（60 秒），设为 0 禁用上限
	 */
	maxRetryDelayMs?: number;
	/**
	 * 附加到 API 请求的元数据
	 */
	metadata?: Record<string, unknown>;
}

/**
 * 提供商特定的图像生成选项
 */
export type ProviderImagesOptions = ImagesOptions & Record<string, unknown>;

/**
 * 简化流式调用选项，在 StreamOptions 基础上增加推理控制参数
 * 用于 streamSimple() 和 completeSimple() 等简化调用入口
 */
export interface SimpleStreamOptions extends StreamOptions {
	/** 推理/思考级别，统一映射到各提供商的具体参数 */
	reasoning?: ThinkingLevel;
	/** 自定义各思考级别的 Token 预算（仅适用于基于 Token 控制的提供商） */
	thinkingBudgets?: ThinkingBudgets;
}

/**
 * 泛型流式调用函数类型
 *
 * 契约：
 * - 必须返回 AssistantMessageEventStream
 * - 请求/模型/运行时失败应编码在返回的流中，而非抛出异常
 * - 错误终止必须产生一个 stopReason 为 "error" 或 "aborted" 的 AssistantMessage，
 *   通过流协议发出
 */
export type StreamFunction<TApi extends Api = Api, TOptions extends StreamOptions = StreamOptions> = (
	model: Model<TApi>,
	context: Context,
	options?: TOptions,
) => AssistantMessageEventStream;

/**
 * 泛型图像生成函数类型
 * 与 StreamFunction 不同，图像生成是异步非流式的
 */
export type ImagesFunction<TApi extends ImagesApi = ImagesApi, TOptions extends ImagesOptions = ImagesOptions> = (
	model: ImagesModel<TApi>,
	context: ImagesContext,
	options?: TOptions,
) => Promise<AssistantImages>;

// ─── 消息内容类型 ─────────────────────────────────────────────

/**
 * 文本签名（V1 版本），用于标识文本内容的阶段
 */
export interface TextSignatureV1 {
	v: 1;
	id: string;
	phase?: "commentary" | "final_answer";
}

/**
 * 文本内容块
 */
export interface TextContent {
	type: "text";
	text: string;
	/** 文本签名，如 OpenAI Responses 的 message metadata */
	textSignature?: string;
}

/**
 * 推理/思考内容块
 * 包含模型的推理过程文本，部分提供商会返回签名用于多轮对话连续性
 */
export interface ThinkingContent {
	type: "thinking";
	thinking: string;
	/** 推理签名，如 OpenAI Responses 的 reasoning item ID，用于多轮对话传递 */
	thinkingSignature?: string;
	/**
	 * 是否被安全过滤器编辑（脱敏）
	 * 编辑后的加密载荷存储在 thinkingSignature 中，可传回 API 保持多轮连续性
	 */
	redacted?: boolean;
}

/**
 * 图像内容块
 */
export interface ImageContent {
	type: "image";
	/** Base64 编码的图像数据 */
	data: string;
	/** MIME 类型（如 "image/jpeg"、"image/png"） */
	mimeType: string;
}

/**
 * 工具调用内容块
 * 表示 LLM 请求调用一个工具
 */
export interface ToolCall {
	type: "toolCall";
	/** 工具调用的唯一标识符，用于将工具结果与调用配对 */
	id: string;
	/** 要调用的工具名称 */
	name: string;
	/** 传递给工具的参数 */
	arguments: Record<string, any>;
	/** Google 特有：用于复用思考上下文的不透明签名 */
	thoughtSignature?: string;
}

// ─── Token 用量 ───────────────────────────────────────────────

/**
 * Token 用量和成本统计
 */
export interface Usage {
	input: number;
	output: number;
	cacheRead: number;
	cacheWrite: number;
	/** 总 Token 数（input + output，不含缓存） */
	totalTokens: number;
	/** 各项成本，单位为美元 */
	cost: {
		input: number;
		output: number;
		cacheRead: number;
		cacheWrite: number;
		total: number;
	};
}

/**
 * 流式响应的停止原因
 * - "stop"：正常结束
 * - "length"：达到最大 Token 数
 * - "toolUse"：LLM 请求调用工具（需要继续循环）
 * - "error"：发生错误
 * - "aborted"：请求被中止
 */
export type StopReason = "stop" | "length" | "toolUse" | "error" | "aborted";

// ─── 消息类型 ─────────────────────────────────────────────────
// 三种消息类型构成完整的对话循环：
// UserMessage -> AssistantMessage -> ToolResultMessage -> AssistantMessage -> ...

/**
 * 用户消息
 */
export interface UserMessage {
	role: "user";
	/** 文本内容，或文本/图像混合内容数组 */
	content: string | (TextContent | ImageContent)[];
	/** Unix 时间戳（毫秒） */
	timestamp: number;
}

/**
 * 助手消息
 * 包含 LLM 的完整响应，可能同时包含文本、推理内容和工具调用
 */
export interface AssistantMessage {
	role: "assistant";
	/** 内容数组，可包含文本、推理和工具调用 */
	content: (TextContent | ThinkingContent | ToolCall)[];
	/** 使用的 API 标识符 */
	api: Api;
	/** 使用的提供商标识符 */
	provider: Provider;
	/** 请求的模型 ID */
	model: string;
	/** 实际响应的模型 ID（当与请求不同时，如 OpenRouter auto 路由） */
	responseModel?: string;
	/** 提供商特定的响应/消息标识符 */
	responseId?: string;
	/** 脱敏后的提供商/运行时诊断信息（用于失败和恢复的分析） */
	diagnostics?: AssistantMessageDiagnostic[];
	/** Token 用量统计 */
	usage: Usage;
	/** 停止原因 */
	stopReason: StopReason;
	/** 错误消息（stopReason 为 error/aborted 时存在） */
	errorMessage?: string;
	/** Unix 时间戳（毫秒） */
	timestamp: number;
}

/**
 * 工具结果消息
 * 将工具执行的结果返回给 LLM，完成一轮工具调用循环
 */
export interface ToolResultMessage<TDetails = any> {
	role: "toolResult";
	/** 对应的 ToolCall ID */
	toolCallId: string;
	/** 工具名称 */
	toolName: string;
	/** 工具输出内容，支持文本和图像 */
	content: (TextContent | ImageContent)[];
	/** 工具特定的附加详情 */
	details?: TDetails;
	/** 是否为错误结果 */
	isError: boolean;
	/** Unix 时间戳（毫秒） */
	timestamp: number;
}

/**
 * 消息联合类型，包含用户消息、助手消息和工具结果消息
 */
export type Message = UserMessage | AssistantMessage | ToolResultMessage;

// ─── 图像生成类型 ─────────────────────────────────────────────

export type ImagesInputContent = TextContent | ImageContent;
export type ImagesOutputContent = TextContent | ImageContent;

/**
 * 图像生成的输入上下文
 */
export interface ImagesContext {
	input: ImagesInputContent[];
}

/**
 * 图像生成的停止原因
 */
export type ImagesStopReason = "stop" | "error" | "aborted";

/**
 * 图像生成的完整响应结果
 */
export interface AssistantImages {
	api: ImagesApi;
	provider: ImagesProvider;
	model: string;
	output: ImagesOutputContent[];
	responseId?: string;
	usage?: Usage;
	stopReason: ImagesStopReason;
	errorMessage?: string;
	/** Unix 时间戳（毫秒） */
	timestamp: number;
}

// ─── 工具定义 ─────────────────────────────────────────────────

import type { TSchema } from "typebox";

/**
 * 工具定义，描述一个可供 LLM 调用的工具
 * 使用 TypeBox 的 TSchema 定义参数的 JSON Schema
 */
export interface Tool<TParameters extends TSchema = TSchema> {
	/** 工具名称 */
	name: string;
	/** 工具描述，帮助 LLM 决定何时调用此工具 */
	description: string;
	/** 参数的 JSON Schema 定义 */
	parameters: TParameters;
}

// ─── 对话上下文 ───────────────────────────────────────────────

/**
 * 对话上下文，包含 LLM 调用所需的全部输入信息
 */
export interface Context {
	/** 系统提示词 */
	systemPrompt?: string;
	/** 消息历史（用户、助手、工具结果的交替序列） */
	messages: Message[];
	/** 可用工具列表，不提供则 LLM 不会调用工具 */
	tools?: Tool[];
}

// ─── 事件流协议 ───────────────────────────────────────────────

/**
 * 助手消息事件协议，定义 AssistantMessageEventStream 中的事件类型
 *
 * 流的生命周期：
 * 1. `start`：流开始，携带初始 partial AssistantMessage
 * 2. 增量更新阶段（text/thinking/toolcall 的 start -> delta -> end）
 * 3. 终止事件（二选一）：
 *    - `done`：成功完成，携带最终 AssistantMessage
 *    - `error`：失败或中止，携带 stopReason 为 "error"/"aborted" 的 AssistantMessage
 */
export type AssistantMessageEvent =
	| { type: "start"; partial: AssistantMessage }
	| { type: "text_start"; contentIndex: number; partial: AssistantMessage }
	| { type: "text_delta"; contentIndex: number; delta: string; partial: AssistantMessage }
	| { type: "text_end"; contentIndex: number; content: string; partial: AssistantMessage }
	| { type: "thinking_start"; contentIndex: number; partial: AssistantMessage }
	| { type: "thinking_delta"; contentIndex: number; delta: string; partial: AssistantMessage }
	| { type: "thinking_end"; contentIndex: number; content: string; partial: AssistantMessage }
	| { type: "toolcall_start"; contentIndex: number; partial: AssistantMessage }
	| { type: "toolcall_delta"; contentIndex: number; delta: string; partial: AssistantMessage }
	| { type: "toolcall_end"; contentIndex: number; toolCall: ToolCall; partial: AssistantMessage }
	| { type: "done"; reason: Extract<StopReason, "stop" | "length" | "toolUse">; message: AssistantMessage }
	| { type: "error"; reason: Extract<StopReason, "aborted" | "error">; error: AssistantMessage };

// ─── 兼容性配置 ───────────────────────────────────────────────
// 这些接口用于自定义提供商的行为，覆盖基于 URL 的自动检测。

/**
 * OpenAI Chat Completions 兼容 API 的兼容性设置
 * 用于覆盖基于 URL 的自动检测，适配不同的 OpenAI 兼容提供商
 */
export interface OpenAICompletionsCompat {
	/** 是否支持 store 字段。默认从 URL 自动检测 */
	supportsStore?: boolean;
	/** 是否支持 developer 角色（而非 system）。默认从 URL 自动检测 */
	supportsDeveloperRole?: boolean;
	/** 是否支持 reasoning_effort 参数。默认从 URL 自动检测 */
	supportsReasoningEffort?: boolean;
	/** 是否支持 stream_options: { include_usage: true } 获取流式 Token 用量。默认 true */
	supportsUsageInStreaming?: boolean;
	/** 使用哪个字段限制最大 Token 数。默认从 URL 自动检测 */
	maxTokensField?: "max_completion_tokens" | "max_tokens";
	/** 工具结果是否需要 name 字段。默认从 URL 自动检测 */
	requiresToolResultName?: boolean;
	/** 工具结果后是否需要中间插入一条助手消息。默认从 URL 自动检测 */
	requiresAssistantAfterToolResult?: boolean;
	/** 是否需要将思考块转换为带 <thinking> 标签的文本块。默认从 URL 自动检测 */
	requiresThinkingAsText?: boolean;
	/** 启用推理时，历史助手消息是否必须包含空的 reasoning_content 字段。默认从 URL 自动检测 */
	requiresReasoningContentOnAssistantMessages?: boolean;
	/**
	 * 推理/思考参数的格式
	 * - "openai"：使用 reasoning_effort
	 * - "openrouter"：使用 reasoning: { effort }
	 * - "deepseek"：使用 thinking: { type } + reasoning_effort
	 * - "together"：使用 reasoning: { enabled } + reasoning_effort
	 * - "zai"：使用顶层 enable_thinking: boolean
	 * - "qwen"：使用顶层 enable_thinking: boolean
	 * - "qwen-chat-template"：使用 chat_template_kwargs.enable_thinking
	 * 默认 "openai"
	 */
	thinkingFormat?: "openai" | "openrouter" | "deepseek" | "together" | "zai" | "qwen" | "qwen-chat-template";
	/** OpenRouter 特有的路由偏好，仅在 baseUrl 指向 OpenRouter 时使用 */
	openRouterRouting?: OpenRouterRouting;
	/** Vercel AI Gateway 路由偏好，仅在 baseUrl 指向 Vercel AI Gateway 时使用 */
	vercelGatewayRouting?: VercelGatewayRouting;
	/** z.ai 是否支持顶层 tool_stream: true 流式工具调用增量。默认 false */
	zaiToolStream?: boolean;
	/** 是否支持工具定义中的 strict 字段。默认 true */
	supportsStrictMode?: boolean;
	/**
	 * 提示缓存的缓存控制格式
	 * "anthropic" 表示在系统提示、最后一个工具定义和最后一条用户/助手文本内容上
	 * 应用 Anthropic 风格的 cache_control 标记
	 */
	cacheControlFormat?: "anthropic";
	/** 缓存启用时是否发送会话亲和性请求头（session_id、x-client-request-id、x-session-affinity）。默认 false */
	sendSessionAffinityHeaders?: boolean;
	/** 是否支持长期缓存保留（如 prompt_cache_retention: "24h" 或 Anthropic 的 cache_control.ttl: "1h"）。默认 true */
	supportsLongCacheRetention?: boolean;
}

/**
 * OpenAI Responses API 的兼容性设置
 */
export interface OpenAIResponsesCompat {
	/** 缓存启用时是否发送 OpenAI session_id 缓存亲和性请求头。默认 true */
	sendSessionIdHeader?: boolean;
	/** 是否支持 prompt_cache_retention: "24h"。默认 true */
	supportsLongCacheRetention?: boolean;
}

/**
 * Anthropic Messages 兼容 API 的兼容性设置
 */
export interface AnthropicMessagesCompat {
	/**
	 * 是否支持按工具设置 eager_input_streaming
	 * 为 false 时，Anthropic 提供商会省略 tools[].eager_input_streaming
	 * 并发送旧版 fine-grained-tool-streaming-2025-05-14 beta 请求头
	 * 默认 true
	 */
	supportsEagerToolInputStreaming?: boolean;
	/** 是否支持 Anthropic 长期缓存保留（cache_control.ttl: "1h"）。默认 true */
	supportsLongCacheRetention?: boolean;
}

/**
 * OpenRouter 提供商路由偏好
 * 控制请求路由到哪些上游提供商
 * @see https://openrouter.ai/docs/guides/routing/provider-selection
 */
export interface OpenRouterRouting {
	/** 是否允许备用提供商处理请求。默认 true */
	allow_fallbacks?: boolean;
	/** 是否仅路由到支持请求中所有参数的提供商。默认 false */
	require_parameters?: boolean;
	/** 数据收集策略。"allow"（默认）允许可能存储/训练数据的提供商；"deny" 仅使用不收集用户数据的提供商 */
	data_collection?: "deny" | "allow";
	/** 是否限制路由到仅 ZDR（零数据保留）端点 */
	zdr?: boolean;
	/** 是否限制路由到仅允许文本蒸馏的模型 */
	enforce_distillable_text?: boolean;
	/** 按顺序尝试的提供商名称/别名列表，不可用时依次回退 */
	order?: string[];
	/** 仅允许的提供商名称/别名白名单 */
	only?: string[];
	/** 要跳过的提供商名称/别名黑名单 */
	ignore?: string[];
	/** 按量化级别筛选提供商（如 ["fp16", "bf16", "fp8", "fp6", "int8", "int4", "fp4", "fp32"]） */
	quantizations?: string[];
	/**
	 * 排序策略
	 * 可以是字符串（如 "price"、"throughput"、"latency"）或包含 by 和 partition 的对象
	 */
	sort?:
		| string
		| {
				/** 排序指标："price"、"throughput"、"latency" */
				by?: string;
				/** 分区策略："model"（默认）或 "none" */
				partition?: string | null;
		  };
	/** 每百万 Token 的最大价格上限（美元） */
	max_price?: {
		/** 每百万提示 Token 价格 */
		prompt?: number | string;
		/** 每百万补全 Token 价格 */
		completion?: number | string;
		/** 每张图像价格 */
		image?: number | string;
		/** 每单位音频价格 */
		audio?: number | string;
		/** 每次请求价格 */
		request?: number | string;
	};
	/** 首选最小吞吐量（Token/秒），可以是数字（应用于 p50）或包含分位数的对象 */
	preferred_min_throughput?:
		| number
		| {
				p50?: number;
				p75?: number;
				p90?: number;
				p99?: number;
		  };
	/** 首选最大延迟（秒），可以是数字（应用于 p50）或包含分位数的对象 */
	preferred_max_latency?:
		| number
		| {
				p50?: number;
				p75?: number;
				p90?: number;
				p99?: number;
		  };
}

/**
 * Vercel AI Gateway 路由偏好
 * 控制网关将请求路由到哪些上游提供商
 * @see https://vercel.com/docs/ai-gateway/models-and-providers/provider-options
 */
export interface VercelGatewayRouting {
	/** 仅使用的提供商别名白名单（如 ["bedrock", "anthropic"]） */
	only?: string[];
	/** 按顺序尝试的提供商别名列表（如 ["anthropic", "openai"]） */
	order?: string[];
}

// ─── 模型定义 ─────────────────────────────────────────────────

/**
 * 统一模型定义接口
 * 包含模型的完整元信息，用于路由、计费和能力判断
 */
export interface Model<TApi extends Api> {
	/** 模型 ID（如 "gpt-4o"、"claude-3-5-sonnet-20241022"） */
	id: string;
	/** 人类可读的模型名称 */
	name: string;
	/** 模型使用的 API 协议 */
	api: TApi;
	/** 模型所属提供商 */
	provider: Provider;
	/** API 基础 URL */
	baseUrl: string;
	/** 模型是否支持推理/思考功能 */
	reasoning: boolean;
	/**
	 * 将统一思考级别映射到提供商/模型特定的值
	 * 缺失的键使用提供商默认值，null 表示该级别不被支持
	 */
	thinkingLevelMap?: ThinkingLevelMap;
	/** 模型支持的输入类型 */
	input: ("text" | "image")[];
	/** 模型使用成本（美元/百万 Token） */
	cost: {
		input: number;
		output: number;
		cacheRead: number;
		cacheWrite: number;
	};
	/** 上下文窗口大小（Token 数） */
	contextWindow: number;
	/** 最大输出 Token 数 */
	maxTokens: number;
	/** 附加到请求的自定义 HTTP 头 */
	headers?: Record<string, string>;
	/**
	 * 兼容性覆盖配置
	 * 根据不同的 API 协议自动选择对应的兼容性接口
	 * 不设置时，从 baseUrl 自动检测
	 */
	compat?: TApi extends "openai-completions"
		? OpenAICompletionsCompat
		: TApi extends "openai-responses"
			? OpenAIResponsesCompat
			: TApi extends "anthropic-messages"
				? AnthropicMessagesCompat
				: never;
}

/**
 * 图像生成模型定义
 * 继承 Model 的大部分字段，但针对图像生成场景调整了部分类型
 */
export interface ImagesModel<TApi extends ImagesApi>
	extends Omit<Model<Api>, "api" | "provider" | "reasoning" | "contextWindow" | "maxTokens" | "compat"> {
	api: TApi;
	provider: ImagesProvider;
	/** 模型支持的输出类型 */
	output: ("text" | "image")[];
}
