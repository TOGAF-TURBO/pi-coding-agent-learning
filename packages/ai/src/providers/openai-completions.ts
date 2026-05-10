/**
 * @fileoverview OpenAI Chat Completions API 提供商实现 — 项目中最核心的参考实现。
 *
 * 实现了 OpenAI Chat Completions 协议的流式调用，同时也是 30+ 家兼容提供商
 * （如 GLM、DeepSeek、Groq、OpenRouter、Together AI 等）的统一入口。
 * 通过兼容性检测（compat）机制，自动适配不同提供商的行为差异。
 *
 * 核心流程：
 *   1. 创建 OpenAI 客户端 → 2. 构建请求参数 → 3. 流式调用 → 4. 解析 SSE 响应 → 5. 发出事件
 *
 * 提供以下功能：
 *   - streamOpenAICompletions: 标准流式调用（完整选项）
 *   - streamSimpleOpenAICompletions: 简化流式调用（映射为标准选项）
 *   - convertMessages: 将内部消息格式转换为 OpenAI Chat Completions 格式
 *
 * 兼容性机制：
 *   通过 OpenAICompletionsCompat 类型的 compat 字段控制不同提供商的行为差异，
 *   包括思考格式（thinkingFormat）、Token 字段名（maxTokensField）、角色支持等。
 *   兼容性设置可通过模型定义显式指定，或由 detectCompat() 从提供商/URL 自动检测。
 */

import OpenAI from "openai";
import type {
	ChatCompletionAssistantMessageParam,
	ChatCompletionChunk,
	ChatCompletionContentPart,
	ChatCompletionContentPartImage,
	ChatCompletionContentPartText,
	ChatCompletionDeveloperMessageParam,
	ChatCompletionMessageParam,
	ChatCompletionSystemMessageParam,
	ChatCompletionToolMessageParam,
} from "openai/resources/chat/completions.js";
import { getEnvApiKey } from "../env-api-keys.js";
import { calculateCost, clampThinkingLevel } from "../models.js";
import type {
	AssistantMessage,
	CacheRetention,
	Context,
	ImageContent,
	Message,
	Model,
	OpenAICompletionsCompat,
	SimpleStreamOptions,
	StopReason,
	StreamFunction,
	StreamOptions,
	TextContent,
	ThinkingContent,
	Tool,
	ToolCall,
	ToolResultMessage,
} from "../types.js";
import { AssistantMessageEventStream } from "../utils/event-stream.js";
import { headersToRecord } from "../utils/headers.js";
import { parseStreamingJson } from "../utils/json-parse.js";
import { sanitizeSurrogates } from "../utils/sanitize-unicode.js";
import { isCloudflareProvider, resolveCloudflareBaseUrl } from "./cloudflare.js";
import { buildCopilotDynamicHeaders, hasCopilotVisionInput } from "./github-copilot-headers.js";
import { buildBaseOptions } from "./simple-options.js";
import { transformMessages } from "./transform-messages.js";

// ============================================================================
// 辅助函数
// ============================================================================

/**
 * 检查对话历史中是否包含工具调用或工具结果
 *
 * 某些提供商（如通过代理的 Anthropic）要求当消息中包含工具调用时，
 * 请求参数中必须携带 tools 字段，即使工具列表为空。
 * @param messages - 对话消息列表
 * @returns 是否包含工具相关消息
 */
function hasToolHistory(messages: Message[]): boolean {
	for (const msg of messages) {
		if (msg.role === "toolResult") {
			return true;
		}
		if (msg.role === "assistant") {
			if (msg.content.some((block) => block.type === "toolCall")) {
				return true;
			}
		}
	}
	return false;
}

// 以下为内容块类型守卫函数
function isTextContentBlock(block: { type: string }): block is TextContent {
	return block.type === "text";
}

function isThinkingContentBlock(block: { type: string }): block is ThinkingContent {
	return block.type === "thinking";
}

function isToolCallBlock(block: { type: string }): block is ToolCall {
	return block.type === "toolCall";
}

function isImageContentBlock(block: { type: string }): block is ImageContent {
	return block.type === "image";
}

// ============================================================================
// 类型定义
// ============================================================================

/** OpenAI Completions 提供商的扩展选项 */
export interface OpenAICompletionsOptions extends StreamOptions {
	/** 工具选择策略：auto=自动决定，none=禁用，required=必须调用，或指定函数名 */
	toolChoice?: "auto" | "none" | "required" | { type: "function"; function: { name: string } };
	/** 推理强度级别 */
	reasoningEffort?: "minimal" | "low" | "medium" | "high" | "xhigh";
}

/** Anthropic 风格的缓存控制参数 */
interface OpenAICompatCacheControl {
	type: "ephemeral";
	ttl?: string;
}

/**
 * 已解析的兼容性设置
 *
 * 将可选的 OpenAICompletionsCompat 解析为所有字段都有默认值的完整对象，
 * 避免运行时反复检查 undefined。
 */
type ResolvedOpenAICompletionsCompat = Omit<Required<OpenAICompletionsCompat>, "cacheControlFormat"> & {
	cacheControlFormat?: OpenAICompletionsCompat["cacheControlFormat"];
};

type ChatCompletionInstructionMessageParam = ChatCompletionDeveloperMessageParam | ChatCompletionSystemMessageParam;

type ChatCompletionTextPartWithCacheControl = ChatCompletionContentPartText & {
	cache_control?: OpenAICompatCacheControl;
};

type ChatCompletionToolWithCacheControl = OpenAI.Chat.Completions.ChatCompletionTool & {
	cache_control?: OpenAICompatCacheControl;
};

// ============================================================================
// 流式调用实现
// ============================================================================

/**
 * 解析缓存保留策略
 *
 * 优先使用显式传入的策略，其次检查环境变量 PI_CACHE_RETENTION=long，
 * 默认使用 short 策略。
 * @param cacheRetention - 显式指定的缓存策略
 * @returns 解析后的缓存策略
 */
function resolveCacheRetention(cacheRetention?: CacheRetention): CacheRetention {
	if (cacheRetention) {
		return cacheRetention;
	}
	if (typeof process !== "undefined" && process.env.PI_CACHE_RETENTION === "long") {
		return "long";
	}
	return "short";
}

/**
 * 标准流式调用 — 向 OpenAI Completions API 发起流式请求
 *
 * 完整的 SSE 流式响应处理流程：
 *   1. 创建 OpenAI 客户端，配置 API Key、Base URL 和请求头
 *   2. 构建请求参数（消息转换、工具转换、思考模式配置等）
 *   3. 发起流式请求，逐块解析 SSE 事件
 *   4. 维护内容块状态机（文本块、思考块、工具调用块）
 *   5. 发出标准化事件到 AssistantMessageEventStream
 *
 * 错误处理：捕获所有异常，将错误信息写入助手消息并发出 error 事件。
 */
export const streamOpenAICompletions: StreamFunction<"openai-completions", OpenAICompletionsOptions> = (
	model: Model<"openai-completions">,
	context: Context,
	options?: OpenAICompletionsOptions,
): AssistantMessageEventStream => {
	const stream = new AssistantMessageEventStream();

	(async () => {
		// 初始化助手消息对象，用于逐步填充内容
		const output: AssistantMessage = {
			role: "assistant",
			content: [],
			api: model.api,
			provider: model.provider,
			model: model.id,
			usage: {
				input: 0,
				output: 0,
				cacheRead: 0,
				cacheWrite: 0,
				totalTokens: 0,
				cost: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0, total: 0 },
			},
			stopReason: "stop",
			timestamp: Date.now(),
		};

		try {
			// 解析 API Key：优先使用选项传入 → 环境变量 → 空
			const apiKey = options?.apiKey || getEnvApiKey(model.provider) || "";
			const compat = getCompat(model);
			const cacheRetention = resolveCacheRetention(options?.cacheRetention);
			// 仅在启用缓存时传递会话 ID（用于会话亲和性和缓存键）
			const cacheSessionId = cacheRetention === "none" ? undefined : options?.sessionId;
			const client = createClient(model, context, apiKey, options?.headers, cacheSessionId, compat);

			// 构建请求参数，并允许 onPayload 钩子修改
			let params = buildParams(model, context, options, compat, cacheRetention);
			const nextParams = await options?.onPayload?.(params, model);
			if (nextParams !== undefined) {
				params = nextParams as OpenAI.Chat.Completions.ChatCompletionCreateParamsStreaming;
			}

			// 请求级选项：信号、超时、重试
			const requestOptions = {
				...(options?.signal ? { signal: options.signal } : {}),
				...(options?.timeoutMs !== undefined ? { timeout: options.timeoutMs } : {}),
				...(options?.maxRetries !== undefined ? { maxRetries: options.maxRetries } : {}),
			};
			const { data: openaiStream, response } = await client.chat.completions
				.create(params, requestOptions)
				.withResponse();
			await options?.onResponse?.({ status: response.status, headers: headersToRecord(response.headers) }, model);
			stream.push({ type: "start", partial: output });

			// ===== 流式内容块状态机 =====
			// 跟踪当前打开的内容块，用于增量更新

			/** 扩展的工具调用块，包含流式解析所需的临时字段 */
			interface StreamingToolCallBlock extends ToolCall {
				/** 未解析的参数 JSON 片段 */
				partialArgs?: string;
				/** SSE 流中的索引位置 */
				streamIndex?: number;
			}
			type StreamingBlock = TextContent | ThinkingContent | StreamingToolCallBlock;
			type StreamingToolCallDelta = NonNullable<ChatCompletionChunk.Choice.Delta["tool_calls"]>[number];

			// 当前活跃的内容块
			let textBlock: TextContent | null = null;
			let thinkingBlock: ThinkingContent | null = null;
			// 工具调用块按流索引和 ID 双重索引，处理增量更新
			const toolCallBlocksByIndex = new Map<number, StreamingToolCallBlock>();
			const toolCallBlocksById = new Map<string, StreamingToolCallBlock>();
			const blocks = output.content as StreamingBlock[];

			/** 获取内容块在输出数组中的索引 */
			const getContentIndex = (block: StreamingBlock) => blocks.indexOf(block);

			/** 完成一个内容块：解析最终数据、发出结束事件 */
			const finishBlock = (block: StreamingBlock) => {
				const contentIndex = getContentIndex(block);
				if (contentIndex === -1) {
					return;
				}
				if (block.type === "text") {
					stream.push({
						type: "text_end",
						contentIndex,
						content: block.text,
						partial: output,
					});
				} else if (block.type === "thinking") {
					stream.push({
						type: "thinking_end",
						contentIndex,
						content: block.thinking,
						partial: output,
					});
				} else if (block.type === "toolCall") {
					// 最终解析流式 JSON 参数
					block.arguments = parseStreamingJson(block.partialArgs);
					// 清理流式解析的临时字段，回放时不需要
					delete block.partialArgs;
					delete block.streamIndex;
					stream.push({
						type: "toolcall_end",
						contentIndex,
						toolCall: block,
						partial: output,
					});
				}
			};

			/** 确保存在一个活跃的文本块（不存在则创建） */
			const ensureTextBlock = () => {
				if (!textBlock) {
					textBlock = { type: "text", text: "" };
					blocks.push(textBlock);
					stream.push({ type: "text_start", contentIndex: getContentIndex(textBlock), partial: output });
				}
				return textBlock;
			};

			/** 确保存在一个活跃的思考块 */
			const ensureThinkingBlock = (thinkingSignature: string) => {
				if (!thinkingBlock) {
					thinkingBlock = {
						type: "thinking",
						thinking: "",
						thinkingSignature,
					};
					blocks.push(thinkingBlock);
					stream.push({ type: "thinking_start", contentIndex: getContentIndex(thinkingBlock), partial: output });
				}
				return thinkingBlock;
			};

			/** 确保存在对应索引/ID 的工具调用块（不存在则创建） */
			const ensureToolCallBlock = (toolCall: StreamingToolCallDelta) => {
				const streamIndex = typeof toolCall.index === "number" ? toolCall.index : undefined;
				let block = streamIndex !== undefined ? toolCallBlocksByIndex.get(streamIndex) : undefined;
				if (!block && toolCall.id) {
					block = toolCallBlocksById.get(toolCall.id);
				}
				if (!block) {
					block = {
						type: "toolCall",
						id: toolCall.id || "",
						name: toolCall.function?.name || "",
						arguments: {},
						partialArgs: "",
						streamIndex,
					};
					if (streamIndex !== undefined) {
						toolCallBlocksByIndex.set(streamIndex, block);
					}
					if (toolCall.id) {
						toolCallBlocksById.set(toolCall.id, block);
					}
					blocks.push(block);
					stream.push({
						type: "toolcall_start",
						contentIndex: getContentIndex(block),
						partial: output,
					});
				}
				// 更新索引映射（流式响应中 index 可能延迟到达）
				if (streamIndex !== undefined && block.streamIndex === undefined) {
					block.streamIndex = streamIndex;
					toolCallBlocksByIndex.set(streamIndex, block);
				}
				if (toolCall.id) {
					toolCallBlocksById.set(toolCall.id, block);
				}
				return block;
			};

			// ===== SSE 流处理主循环 =====
			for await (const chunk of openaiStream) {
				if (!chunk || typeof chunk !== "object") continue;

				// 记录响应 ID（同一流的所有 chunk 共享）
				output.responseId ||= chunk.id;
				// 记录服务端实际使用的模型名（可能与请求的模型 ID 不同）
				if (typeof chunk.model === "string" && chunk.model.length > 0 && chunk.model !== model.id) {
					output.responseModel ||= chunk.model;
				}
				// 解析用量信息（某些提供商在流中发送）
				if (chunk.usage) {
					output.usage = parseChunkUsage(chunk.usage, model);
				}

				const choice = Array.isArray(chunk.choices) ? chunk.choices[0] : undefined;
				if (!choice) continue;

				// 回退：某些提供商（如 Moonshot）在 choice.usage 而非 chunk.usage 中返回用量
				if (!chunk.usage && (choice as any).usage) {
					output.usage = parseChunkUsage((choice as any).usage, model);
				}

				// 处理完成原因
				if (choice.finish_reason) {
					const finishReasonResult = mapStopReason(choice.finish_reason);
					output.stopReason = finishReasonResult.stopReason;
					if (finishReasonResult.errorMessage) {
						output.errorMessage = finishReasonResult.errorMessage;
					}
				}

				// 处理增量内容
				if (choice.delta) {
					// --- 文本内容 ---
					if (
						choice.delta.content !== null &&
						choice.delta.content !== undefined &&
						choice.delta.content.length > 0
					) {
						const block = ensureTextBlock();
						block.text += choice.delta.content;
						stream.push({
							type: "text_delta",
							contentIndex: getContentIndex(block),
							delta: choice.delta.content,
							partial: output,
						});
					}

					// --- 思考/推理内容 ---
					// 不同提供商使用不同字段名：reasoning_content（llama.cpp）、reasoning（其他兼容端点）
					// 使用第一个非空字段，避免重复（如 chutes.ai 同时返回两个字段）
					const reasoningFields = ["reasoning_content", "reasoning", "reasoning_text"];
					const deltaFields = choice.delta as Record<string, unknown>;
					let foundReasoningField: string | null = null;
					for (const field of reasoningFields) {
						const value = deltaFields[field];
						if (typeof value === "string" && value.length > 0) {
							foundReasoningField = field;
							break;
						}
					}

					if (foundReasoningField) {
						const delta = deltaFields[foundReasoningField];
						if (typeof delta === "string" && delta.length > 0) {
							const block = ensureThinkingBlock(foundReasoningField);
							block.thinking += delta;
							stream.push({
								type: "thinking_delta",
								contentIndex: getContentIndex(block),
								delta,
								partial: output,
							});
						}
					}

					// --- 工具调用 ---
					if (choice?.delta?.tool_calls) {
						for (const toolCall of choice.delta.tool_calls) {
							const block = ensureToolCallBlock(toolCall);
							// 增量更新工具调用的 ID 和名称
							if (!block.id && toolCall.id) {
								block.id = toolCall.id;
								toolCallBlocksById.set(toolCall.id, block);
							}
							if (!block.name && toolCall.function?.name) {
								block.name = toolCall.function.name;
							}

							// 累积参数 JSON 片段并尝试增量解析
							let delta = "";
							if (toolCall.function?.arguments) {
								delta = toolCall.function.arguments;
								block.partialArgs = (block.partialArgs ?? "") + toolCall.function.arguments;
								block.arguments = parseStreamingJson(block.partialArgs);
							}
							stream.push({
								type: "toolcall_delta",
								contentIndex: getContentIndex(block),
								delta,
								partial: output,
							});
						}
					}

					// --- 加密推理签名（OpenAI Codex 等使用的 reasoning_details） ---
					const reasoningDetails = (choice.delta as any).reasoning_details;
					if (reasoningDetails && Array.isArray(reasoningDetails)) {
						for (const detail of reasoningDetails) {
							if (detail.type === "reasoning.encrypted" && detail.id && detail.data) {
								// 将加密推理签名关联到对应的工具调用
								const matchingToolCall = output.content.find(
									(b) => b.type === "toolCall" && b.id === detail.id,
								) as ToolCall | undefined;
								if (matchingToolCall) {
									matchingToolCall.thoughtSignature = JSON.stringify(detail);
								}
							}
						}
					}
				}
			}

			// 完成所有剩余的活跃内容块
			for (const block of blocks) {
				finishBlock(block);
			}

			// 检查中止状态
			if (options?.signal?.aborted) {
				throw new Error("Request was aborted");
			}
			if (output.stopReason === "aborted") {
				throw new Error("Request was aborted");
			}
			if (output.stopReason === "error") {
				throw new Error(output.errorMessage || "Provider returned an error stop reason");
			}

			stream.push({ type: "done", reason: output.stopReason, message: output });
			stream.end();
		} catch (error) {
			// 错误处理：清理临时字段，构造错误消息
			for (const block of output.content) {
				delete (block as { index?: number }).index;
				delete (block as { partialArgs?: string }).partialArgs;
				delete (block as { streamIndex?: number }).streamIndex;
			}
			output.stopReason = options?.signal?.aborted ? "aborted" : "error";
			output.errorMessage = error instanceof Error ? error.message : JSON.stringify(error);
			// OpenRouter 等代理可能在错误对象的 metadata.raw 中附加详细信息
			const rawMetadata = (error as any)?.error?.metadata?.raw;
			if (rawMetadata) output.errorMessage += `\n${rawMetadata}`;
			stream.push({ type: "error", reason: output.stopReason, error: output });
			stream.end();
		}
	})();

	return stream;
};

/**
 * 简化流式调用 — 将 SimpleStreamOptions 映射为 OpenAICompletionsOptions
 *
 * 处理思考级别的钳位（clamp）和工具选择参数的透传，
 * 然后委托给 streamOpenAICompletions。
 */
export const streamSimpleOpenAICompletions: StreamFunction<"openai-completions", SimpleStreamOptions> = (
	model: Model<"openai-completions">,
	context: Context,
	options?: SimpleStreamOptions,
): AssistantMessageEventStream => {
	const apiKey = options?.apiKey || getEnvApiKey(model.provider);
	if (!apiKey) {
		throw new Error(`No API key for provider: ${model.provider}`);
	}

	const base = buildBaseOptions(model, options, apiKey);
	// 将思考级别钳位到模型支持的范围内
	const clampedReasoning = options?.reasoning ? clampThinkingLevel(model, options.reasoning) : undefined;
	const reasoningEffort = clampedReasoning === "off" ? undefined : clampedReasoning;
	const toolChoice = (options as OpenAICompletionsOptions | undefined)?.toolChoice;

	return streamOpenAICompletions(model, context, {
		...base,
		reasoningEffort,
		toolChoice,
	} satisfies OpenAICompletionsOptions);
};

// ============================================================================
// 客户端创建
// ============================================================================

/**
 * 创建配置好的 OpenAI 客户端实例
 *
 * 根据模型提供商自动配置：
 *   - GitHub Copilot：注入动态请求头（模型路由、视觉支持检测）
 *   - Cloudflare AI Gateway：替换 Authorization 头为 cf-aig-authorization
 *   - 启用缓存时：添加会话亲和性头（session_id、x-session-affinity）
 *
 * @param model - 目标模型
 * @param context - 对话上下文（Copilot 用途）
 * @param apiKey - API 密钥
 * @param optionsHeaders - 外部传入的额外请求头
 * @param sessionId - 缓存会话 ID
 * @param compat - 已解析的兼容性设置
 * @returns 配置好的 OpenAI 客户端
 */
function createClient(
	model: Model<"openai-completions">,
	context: Context,
	apiKey?: string,
	optionsHeaders?: Record<string, string>,
	sessionId?: string,
	compat: ResolvedOpenAICompletionsCompat = getCompat(model),
) {
	if (!apiKey) {
		if (!process.env.OPENAI_API_KEY) {
			throw new Error(
				"OpenAI API key is required. Set OPENAI_API_KEY environment variable or pass it as an argument.",
			);
		}
		apiKey = process.env.OPENAI_API_KEY;
	}

	const headers = { ...model.headers };

	// GitHub Copilot 特殊处理：注入动态模型路由和视觉检测头
	if (model.provider === "github-copilot") {
		const hasImages = hasCopilotVisionInput(context.messages);
		const copilotHeaders = buildCopilotDynamicHeaders({
			messages: context.messages,
			hasImages,
		});
		Object.assign(headers, copilotHeaders);
	}

	// 缓存会话亲和性：帮助提供商将请求路由到同一缓存节点
	if (sessionId && compat.sendSessionAffinityHeaders) {
		headers.session_id = sessionId;
		headers["x-client-request-id"] = sessionId;
		headers["x-session-affinity"] = sessionId;
	}

	// 外部头最后合并，可覆盖默认值
	if (optionsHeaders) {
		Object.assign(headers, optionsHeaders);
	}

	// Cloudflare AI Gateway 使用不同的认证头
	const defaultHeaders =
		model.provider === "cloudflare-ai-gateway"
			? {
					...headers,
					Authorization: headers.Authorization ?? null,
					"cf-aig-authorization": `Bearer ${apiKey}`,
				}
			: headers;

	return new OpenAI({
		apiKey,
		baseURL: isCloudflareProvider(model.provider) ? resolveCloudflareBaseUrl(model) : model.baseUrl,
		dangerouslyAllowBrowser: true,
		defaultHeaders,
	});
}

// ============================================================================
// 请求参数构建
// ============================================================================

/**
 * 构建 OpenAI Chat Completions API 的完整请求参数
 *
 * 包括消息转换、工具转换、思考模式配置、缓存控制、路由偏好等。
 * 根据兼容性设置自动适配不同提供商的参数格式差异。
 */
function buildParams(
	model: Model<"openai-completions">,
	context: Context,
	options?: OpenAICompletionsOptions,
	compat: ResolvedOpenAICompletionsCompat = getCompat(model),
	cacheRetention: CacheRetention = resolveCacheRetention(options?.cacheRetention),
) {
	const messages = convertMessages(model, context, compat);
	const cacheControl = getCompatCacheControl(compat, cacheRetention);

	const params: OpenAI.Chat.Completions.ChatCompletionCreateParamsStreaming = {
		model: model.id,
		messages,
		stream: true,
		// OpenAI 提示缓存键（使用会话 ID 作为缓存键）
		prompt_cache_key:
			(model.baseUrl.includes("api.openai.com") && cacheRetention !== "none") ||
			(cacheRetention === "long" && compat.supportsLongCacheRetention)
				? options?.sessionId
				: undefined,
		prompt_cache_retention: cacheRetention === "long" && compat.supportsLongCacheRetention ? "24h" : undefined,
	};

	// 请求流式用量信息（Token 统计）
	if (compat.supportsUsageInStreaming !== false) {
		(params as any).stream_options = { include_usage: true };
	}

	// OpenAI 状态存储默认关闭
	if (compat.supportsStore) {
		params.store = false;
	}

	// maxTokens 字段名因提供商而异：max_completion_tokens（新标准）vs max_tokens（旧标准）
	if (options?.maxTokens) {
		if (compat.maxTokensField === "max_tokens") {
			(params as any).max_tokens = options.maxTokens;
		} else {
			params.max_completion_tokens = options.maxTokens;
		}
	}

	if (options?.temperature !== undefined) {
		params.temperature = options.temperature;
	}

	// 工具配置
	if (context.tools && context.tools.length > 0) {
		params.tools = convertTools(context.tools, compat);
		// ZAI/GLM 提供商的工具流式输出
		if (compat.zaiToolStream) {
			(params as any).tool_stream = true;
		}
	} else if (hasToolHistory(context.messages)) {
		// 某些提供商要求有工具历史时必须传 tools 参数
		params.tools = [];
	}

	// Anthropic 风格的缓存控制（通过 LiteLLM 代理的 Anthropic 模型）
	if (cacheControl) {
		applyAnthropicCacheControl(messages, params.tools, cacheControl);
	}

	if (options?.toolChoice) {
		params.tool_choice = options.toolChoice;
	}

	// ===== 思考/推理模式配置 =====
	// 不同提供商使用不同的参数格式，通过 thinkingFormat 区分
	if (compat.thinkingFormat === "zai" && model.reasoning) {
		// ZAI/GLM：顶层 enable_thinking 布尔值
		(params as any).enable_thinking = !!options?.reasoningEffort;
	} else if (compat.thinkingFormat === "qwen" && model.reasoning) {
		// Qwen：顶层 enable_thinking 布尔值
		(params as any).enable_thinking = !!options?.reasoningEffort;
	} else if (compat.thinkingFormat === "qwen-chat-template" && model.reasoning) {
		// Qwen（聊天模板模式）：通过 chat_template_kwargs 传递
		(params as any).chat_template_kwargs = {
			enable_thinking: !!options?.reasoningEffort,
			preserve_thinking: true,
		};
	} else if (compat.thinkingFormat === "deepseek" && model.reasoning) {
		// DeepSeek：thinking 对象 + reasoning_effort
		(params as any).thinking = { type: options?.reasoningEffort ? "enabled" : "disabled" };
		if (options?.reasoningEffort) {
			(params as any).reasoning_effort =
				model.thinkingLevelMap?.[options.reasoningEffort] ?? options.reasoningEffort;
		}
	} else if (compat.thinkingFormat === "openrouter" && model.reasoning) {
		// OpenRouter：嵌套 reasoning.effort 对象
		const openRouterParams = params as typeof params & { reasoning?: { effort?: string } };
		if (options?.reasoningEffort) {
			openRouterParams.reasoning = {
				effort: model.thinkingLevelMap?.[options.reasoningEffort] ?? options.reasoningEffort,
			};
		} else if (model.thinkingLevelMap?.off !== null) {
			openRouterParams.reasoning = { effort: model.thinkingLevelMap?.off ?? "none" };
		}
	} else if (compat.thinkingFormat === "together" && model.reasoning) {
		// Together AI：reasoning.enabled + 可选 reasoning_effort
		const togetherParams = params as Omit<typeof params, "reasoning_effort"> & {
			reasoning?: { enabled: boolean };
			reasoning_effort?: string;
		};
		togetherParams.reasoning = { enabled: !!options?.reasoningEffort };
		if (options?.reasoningEffort && compat.supportsReasoningEffort) {
			togetherParams.reasoning_effort = model.thinkingLevelMap?.[options.reasoningEffort] ?? options.reasoningEffort;
		}
	} else if (options?.reasoningEffort && model.reasoning && compat.supportsReasoningEffort) {
		// OpenAI 标准格式：reasoning_effort 字段
		(params as any).reasoning_effort = model.thinkingLevelMap?.[options.reasoningEffort] ?? options.reasoningEffort;
	} else if (!options?.reasoningEffort && model.reasoning && compat.supportsReasoningEffort) {
		// 显式关闭推理时的映射值
		const offValue = model.thinkingLevelMap?.off;
		if (typeof offValue === "string") {
			(params as any).reasoning_effort = offValue;
		}
	}

	// OpenRouter 提供商路由偏好
	if (model.baseUrl.includes("openrouter.ai") && model.compat?.openRouterRouting) {
		(params as any).provider = model.compat.openRouterRouting;
	}

	// Vercel AI Gateway 提供商路由偏好
	if (model.baseUrl.includes("ai-gateway.vercel.sh") && model.compat?.vercelGatewayRouting) {
		const routing = model.compat.vercelGatewayRouting;
		if (routing.only || routing.order) {
			const gatewayOptions: Record<string, string[]> = {};
			if (routing.only) gatewayOptions.only = routing.only;
			if (routing.order) gatewayOptions.order = routing.order;
			(params as any).providerOptions = { gateway: gatewayOptions };
		}
	}

	return params;
}

// ============================================================================
// 缓存控制
// ============================================================================

/** 根据兼容性设置和缓存策略生成缓存控制参数 */
function getCompatCacheControl(
	compat: ResolvedOpenAICompletionsCompat,
	cacheRetention: CacheRetention,
): OpenAICompatCacheControl | undefined {
	if (compat.cacheControlFormat !== "anthropic" || cacheRetention === "none") {
		return undefined;
	}

	const ttl = cacheRetention === "long" && compat.supportsLongCacheRetention ? "1h" : undefined;
	return { type: "ephemeral", ...(ttl ? { ttl } : {}) };
}

/**
 * 应用 Anthropic 风格的缓存控制到消息和工具
 *
 * 在系统提示、最后一条对话消息和最后一个工具上添加 cache_control 标记，
 * 最大化 Anthropic 提示缓存的命中率。
 */
function applyAnthropicCacheControl(
	messages: ChatCompletionMessageParam[],
	tools: OpenAI.Chat.Completions.ChatCompletionTool[] | undefined,
	cacheControl: OpenAICompatCacheControl,
): void {
	addCacheControlToSystemPrompt(messages, cacheControl);
	addCacheControlToLastTool(tools, cacheControl);
	addCacheControlToLastConversationMessage(messages, cacheControl);
}

/** 在系统/开发者消息上添加缓存控制 */
function addCacheControlToSystemPrompt(
	messages: ChatCompletionMessageParam[],
	cacheControl: OpenAICompatCacheControl,
): void {
	for (const message of messages) {
		if (message.role === "system" || message.role === "developer") {
			addCacheControlToInstructionMessage(message, cacheControl);
			return;
		}
	}
}

/** 在最后一条用户/助手消息上添加缓存控制 */
function addCacheControlToLastConversationMessage(
	messages: ChatCompletionMessageParam[],
	cacheControl: OpenAICompatCacheControl,
): void {
	for (let i = messages.length - 1; i >= 0; i--) {
		const message = messages[i];
		if (message.role === "user" || message.role === "assistant") {
			if (addCacheControlToMessage(message, cacheControl)) {
				return;
			}
		}
	}
}

/** 在最后一个工具定义上添加缓存控制 */
function addCacheControlToLastTool(
	tools: OpenAI.Chat.Completions.ChatCompletionTool[] | undefined,
	cacheControl: OpenAICompatCacheControl,
): void {
	if (!tools || tools.length === 0) {
		return;
	}

	const lastTool = tools[tools.length - 1] as ChatCompletionToolWithCacheControl;
	lastTool.cache_control = cacheControl;
}

function addCacheControlToInstructionMessage(
	message: ChatCompletionInstructionMessageParam,
	cacheControl: OpenAICompatCacheControl,
): boolean {
	return addCacheControlToTextContent(message, cacheControl);
}

function addCacheControlToMessage(
	message: ChatCompletionMessageParam,
	cacheControl: OpenAICompatCacheControl,
): boolean {
	if (message.role === "user" || message.role === "assistant") {
		return addCacheControlToTextContent(message, cacheControl);
	}
	return false;
}

/**
 * 在消息的最后一个文本部分上添加缓存控制
 *
 * 如果消息内容是字符串，先转换为内容块数组。
 * 从后向前查找第一个文本部分并添加 cache_control。
 * @returns 是否成功添加
 */
function addCacheControlToTextContent(
	message:
		| ChatCompletionInstructionMessageParam
		| ChatCompletionAssistantMessageParam
		| Extract<ChatCompletionMessageParam, { role: "user" }>,
	cacheControl: OpenAICompatCacheControl,
): boolean {
	const content = message.content;
	if (typeof content === "string") {
		if (content.length === 0) {
			return false;
		}
		message.content = [
			{
				type: "text",
				text: content,
				cache_control: cacheControl,
			},
		] as ChatCompletionTextPartWithCacheControl[];
		return true;
	}

	if (!Array.isArray(content)) {
		return false;
	}

	for (let i = content.length - 1; i >= 0; i--) {
		const part = content[i];
		if (part?.type === "text") {
			const textPart = part as ChatCompletionTextPartWithCacheControl;
			textPart.cache_control = cacheControl;
			return true;
		}
	}

	return false;
}

// ============================================================================
// 消息格式转换
// ============================================================================

/**
 * 将内部消息格式转换为 OpenAI Chat Completions API 的消息格式
 *
 * 转换规则：
 *   - 系统提示：根据 compat 决定使用 system 还是 developer 角色
 *   - 用户消息：文本直接传递，图片转为 image_url 内容块
 *   - 助手消息：文本、思考块、工具调用分别转换
 *   - 工具结果：转为 tool 角色，图片作为附加用户消息发送
 *   - 特殊处理：空助手消息跳过、工具调用 ID 规范化
 *
 * @param model - 目标模型
 * @param context - 包含消息和工具的对话上下文
 * @param compat - 已解析的兼容性设置
 * @returns OpenAI Chat Completions 格式的消息数组
 */
export function convertMessages(
	model: Model<"openai-completions">,
	context: Context,
	compat: ResolvedOpenAICompletionsCompat,
): ChatCompletionMessageParam[] {
	const params: ChatCompletionMessageParam[] = [];

	/**
	 * 规范化工具调用 ID
	 *
	 * OpenAI Responses API 生成的 ID 格式为 "{call_id}|{长ID}"，
	 * 其中长 ID 可达 400+ 字符且含特殊字符（+, /, =）。
	 * 提取管道符前的 call_id 部分，清理非法字符并截断到 40 字符。
	 */
	const normalizeToolCallId = (id: string): string => {
		if (id.includes("|")) {
			const [callId] = id.split("|");
			return callId.replace(/[^a-zA-Z0-9_-]/g, "_").slice(0, 40);
		}

		if (model.provider === "openai") return id.length > 40 ? id.slice(0, 40) : id;
		return id;
	};

	// 先通过 transformMessages 进行跨提供商兼容性处理
	const transformedMessages = transformMessages(context.messages, model, (id) => normalizeToolCallId(id));

	// 系统提示：推理模型使用 developer 角色，普通模型使用 system 角色
	if (context.systemPrompt) {
		const useDeveloperRole = model.reasoning && compat.supportsDeveloperRole;
		const role = useDeveloperRole ? "developer" : "system";
		params.push({ role: role, content: sanitizeSurrogates(context.systemPrompt) });
	}

	let lastRole: string | null = null;

	for (let i = 0; i < transformedMessages.length; i++) {
		const msg = transformedMessages[i];
		// 某些提供商不允许工具结果后直接跟用户消息，需要插入合成助手消息
		if (compat.requiresAssistantAfterToolResult && lastRole === "toolResult" && msg.role === "user") {
			params.push({
				role: "assistant",
				content: "I have processed the tool results.",
			});
		}

		// --- 用户消息 ---
		if (msg.role === "user") {
			if (typeof msg.content === "string") {
				params.push({
					role: "user",
					content: sanitizeSurrogates(msg.content),
				});
			} else {
				// 多模态内容：文本 + 图片
				const content: ChatCompletionContentPart[] = msg.content.map((item): ChatCompletionContentPart => {
					if (item.type === "text") {
						return {
							type: "text",
							text: sanitizeSurrogates(item.text),
						} satisfies ChatCompletionContentPartText;
					} else {
						// 图片转为 data URI 格式的 image_url
						return {
							type: "image_url",
							image_url: {
								url: `data:${item.mimeType};base64,${item.data}`,
							},
						} satisfies ChatCompletionContentPartImage;
					}
				});
				if (content.length === 0) continue;
				params.push({
					role: "user",
					content,
				});
			}
		} else if (msg.role === "assistant") {
			// --- 助手消息 ---
			const assistantMsg: ChatCompletionAssistantMessageParam = {
				role: "assistant",
				// 某些提供商不接受 null content
				content: compat.requiresAssistantAfterToolResult ? "" : null,
			};

			// 提取文本部分
			const assistantTextParts = msg.content
				.filter(isTextContentBlock)
				.filter((block) => block.text.trim().length > 0)
				.map(
					(block) =>
						({
							type: "text",
							text: sanitizeSurrogates(block.text),
						}) satisfies ChatCompletionContentPartText,
				);
			const assistantText = assistantTextParts.map((part) => part.text).join("");

			// 思考块处理
			const nonEmptyThinkingBlocks = msg.content
				.filter(isThinkingContentBlock)
				.filter((block) => block.thinking.trim().length > 0);
			if (nonEmptyThinkingBlocks.length > 0) {
				if (compat.requiresThinkingAsText) {
					// 将思考内容转为普通文本（不加标签，避免模型模仿）
					const thinkingText = nonEmptyThinkingBlocks
						.map((block) => sanitizeSurrogates(block.thinking))
						.join("\n\n");
					assistantMsg.content = [{ type: "text", text: thinkingText }, ...assistantTextParts];
				} else {
					// 始终使用纯字符串格式发送助手内容（OpenAI 标准格式）
					// 使用内容块数组格式会导致某些模型（如 DeepSeek V3.2 via NVIDIA NIM）
					// 在输出中镜像内容块结构，产生递归嵌套
					if (assistantText.length > 0) {
						assistantMsg.content = assistantText;
					}

					// 使用思考块的签名作为字段名（llama.cpp + gpt-oss 的机制）
					const signature = nonEmptyThinkingBlocks[0].thinkingSignature;
					if (signature && signature.length > 0) {
						(assistantMsg as any)[signature] = nonEmptyThinkingBlocks.map((block) => block.thinking).join("\n");
					}
				}
			} else if (assistantText.length > 0) {
				assistantMsg.content = assistantText;
			}

			// 工具调用块
			const toolCalls = msg.content.filter(isToolCallBlock);
			if (toolCalls.length > 0) {
				assistantMsg.tool_calls = toolCalls.map((tc) => ({
					id: tc.id,
					type: "function" as const,
					function: {
						name: tc.name,
						arguments: JSON.stringify(tc.arguments),
					},
				}));
				// 恢复加密推理签名（OpenAI Codex 的 reasoning_details）
				const reasoningDetails = toolCalls
					.filter((tc) => tc.thoughtSignature)
					.map((tc) => {
						try {
							return JSON.parse(tc.thoughtSignature!);
						} catch {
							return null;
						}
					})
					.filter(Boolean);
				if (reasoningDetails.length > 0) {
					(assistantMsg as any).reasoning_details = reasoningDetails;
				}
			}

			// DeepSeek 要求助手消息上始终有 reasoning_content 字段
			if (
				compat.requiresReasoningContentOnAssistantMessages &&
				model.reasoning &&
				(assistantMsg as { reasoning_content?: string }).reasoning_content === undefined
			) {
				(assistantMsg as { reasoning_content?: string }).reasoning_content = "";
			}

			// 跳过空助手消息（无内容也无工具调用）
			// 某些提供商要求"内容或工具调用至少有其一"
			const content = assistantMsg.content;
			const hasContent =
				content !== null &&
				content !== undefined &&
				(typeof content === "string" ? content.length > 0 : content.length > 0);
			if (!hasContent && !assistantMsg.tool_calls) {
				continue;
			}
			params.push(assistantMsg);
		} else if (msg.role === "toolResult") {
			// --- 工具结果消息 ---
			const imageBlocks: Array<{ type: "image_url"; image_url: { url: string } }> = [];
			let j = i;

			for (; j < transformedMessages.length && transformedMessages[j].role === "toolResult"; j++) {
				const toolMsg = transformedMessages[j] as ToolResultMessage;

				const textResult = toolMsg.content
					.filter(isTextContentBlock)
					.map((block) => block.text)
					.join("\n");
				const hasImages = toolMsg.content.some((c) => c.type === "image");

				const hasText = textResult.length > 0;
				const toolResultMsg: ChatCompletionToolMessageParam = {
					role: "tool",
					content: sanitizeSurrogates(hasText ? textResult : "(see attached image)"),
					tool_call_id: toolMsg.toolCallId,
				};
				// 某些提供商要求工具结果消息包含 name 字段
				if (compat.requiresToolResultName && toolMsg.toolName) {
					(toolResultMsg as any).name = toolMsg.toolName;
				}
				params.push(toolResultMsg);

				// 收集图片块（如果模型支持视觉）
				if (hasImages && model.input.includes("image")) {
					for (const block of toolMsg.content) {
						if (isImageContentBlock(block)) {
							imageBlocks.push({
								type: "image_url",
								image_url: {
									url: `data:${block.mimeType};base64,${block.data}`,
								},
							});
						}
					}
				}
			}

			// 回退外层循环索引（已处理连续的工具结果消息）
			i = j - 1;

			// 工具结果中的图片需要作为附加用户消息发送
			if (imageBlocks.length > 0) {
				if (compat.requiresAssistantAfterToolResult) {
					params.push({
						role: "assistant",
						content: "I have processed the tool results.",
					});
				}

				params.push({
					role: "user",
					content: [
						{
							type: "text",
							text: "Attached image(s) from tool result:",
						},
						...imageBlocks,
					],
				});
				lastRole = "user";
			} else {
				lastRole = "toolResult";
			}
			continue;
		}

		lastRole = msg.role;
	}

	return params;
}

// ============================================================================
// 工具格式转换
// ============================================================================

/**
 * 将内部工具定义转换为 OpenAI Chat Completions 的工具格式
 * @param tools - 内部工具定义列表
 * @param compat - 兼容性设置
 * @returns OpenAI 格式的工具列表
 */
function convertTools(
	tools: Tool[],
	compat: ResolvedOpenAICompletionsCompat,
): OpenAI.Chat.Completions.ChatCompletionTool[] {
	return tools.map((tool) => ({
		type: "function",
		function: {
			name: tool.name,
			description: tool.description,
			parameters: tool.parameters as any, // TypeBox 已生成 JSON Schema
			// 仅在提供商支持时包含 strict 字段，某些提供商会拒绝未知字段
			...(compat.supportsStrictMode !== false && { strict: false }),
		},
	}));
}

// ============================================================================
// 用量解析
// ============================================================================

/**
 * 解析 SSE 流中的 Token 用量数据
 *
 * 统一不同提供商的用量报告格式为 pi-ai 的标准 Usage 结构：
 *   - input：非缓存的输入 Token
 *   - output：输出 Token（含推理 Token）
 *   - cacheRead：从之前请求的缓存中读取的 Token
 *   - cacheWrite：本次请求写入缓存的 Token
 *
 * 某些提供商（如 OpenRouter）的 cached_tokens 包含了当前写入量，
 * 此函数从 cacheRead 中扣除 cacheWrite，避免重复计数。
 *
 * @param rawUsage - SSE 响应中的原始用量数据
 * @param model - 用于成本计算的模型定义
 * @returns 标准化的 Usage 对象
 */
function parseChunkUsage(
	rawUsage: {
		prompt_tokens?: number;
		completion_tokens?: number;
		prompt_cache_hit_tokens?: number;
		prompt_tokens_details?: { cached_tokens?: number; cache_write_tokens?: number };
	},
	model: Model<"openai-completions">,
): AssistantMessage["usage"] {
	const promptTokens = rawUsage.prompt_tokens || 0;
	const reportedCachedTokens = rawUsage.prompt_tokens_details?.cached_tokens ?? rawUsage.prompt_cache_hit_tokens ?? 0;
	const cacheWriteTokens = rawUsage.prompt_tokens_details?.cache_write_tokens || 0;

	// 规范化缓存 Token：
	// 某些提供商的 cached_tokens = 之前命中 + 本次写入，需从中扣除写入部分
	const cacheReadTokens =
		cacheWriteTokens > 0 ? Math.max(0, reportedCachedTokens - cacheWriteTokens) : reportedCachedTokens;

	// input = 总输入 - 缓存读取 - 缓存写入（仅计算非缓存的输入）
	const input = Math.max(0, promptTokens - cacheReadTokens - cacheWriteTokens);
	// output 已包含推理 Token（OpenAI 标准）
	const outputTokens = rawUsage.completion_tokens || 0;
	const usage: AssistantMessage["usage"] = {
		input,
		output: outputTokens,
		cacheRead: cacheReadTokens,
		cacheWrite: cacheWriteTokens,
		totalTokens: input + outputTokens + cacheReadTokens + cacheWriteTokens,
		cost: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0, total: 0 },
	};
	calculateCost(model, usage);
	return usage;
}

/**
 * 将 OpenAI 的 finish_reason 映射为 pi-ai 的标准 StopReason
 *
 * @param reason - OpenAI 的完成原因
 * @returns 标准化的停止原因和可选的错误消息
 */
function mapStopReason(reason: ChatCompletionChunk.Choice["finish_reason"] | string): {
	stopReason: StopReason;
	errorMessage?: string;
} {
	if (reason === null) return { stopReason: "stop" };
	switch (reason) {
		case "stop":
		case "end":
			return { stopReason: "stop" };
		case "length":
			return { stopReason: "length" };
		case "function_call":
		case "tool_calls":
			return { stopReason: "toolUse" };
		case "content_filter":
			return { stopReason: "error", errorMessage: "Provider finish_reason: content_filter" };
		case "network_error":
			return { stopReason: "error", errorMessage: "Provider finish_reason: network_error" };
		default:
			return {
				stopReason: "error",
				errorMessage: `Provider finish_reason: ${reason}`,
			};
	}
}

// ============================================================================
// 兼容性检测
// ============================================================================

/**
 * 从提供商名称和 Base URL 自动检测兼容性设置
 *
 * 检测逻辑：
 *   - 提供商标识符优先于 URL 匹配（显式配置优先）
 *   - 根据 isNonStandard 标志统一处理非标准提供商的行为差异
 *   - 思考格式根据提供商自动选择（zai/deepseek/together/openrouter/openai）
 *
 * @param model - 目标模型
 * @returns 完全解析的兼容性设置
 */
function detectCompat(model: Model<"openai-completions">): ResolvedOpenAICompletionsCompat {
	const provider = model.provider;
	const baseUrl = model.baseUrl;

	// 各提供商的识别规则
	const isZai = provider === "zai" || baseUrl.includes("api.z.ai");
	const isTogether =
		provider === "together" || baseUrl.includes("api.together.ai") || baseUrl.includes("api.together.xyz");
	const isMoonshot = provider === "moonshotai" || provider === "moonshotai-cn" || baseUrl.includes("api.moonshot.");
	const isCloudflareWorkersAI = provider === "cloudflare-workers-ai" || baseUrl.includes("api.cloudflare.com");
	const isCloudflareAiGateway = provider === "cloudflare-ai-gateway" || baseUrl.includes("gateway.ai.cloudflare.com");

	// 非标准提供商：不完全遵循 OpenAI 最新规范的提供商
	const isNonStandard =
		provider === "cerebras" ||
		baseUrl.includes("cerebras.ai") ||
		provider === "xai" ||
		baseUrl.includes("api.x.ai") ||
		isTogether ||
		baseUrl.includes("chutes.ai") ||
		baseUrl.includes("deepseek.com") ||
		isZai ||
		isMoonshot ||
		provider === "opencode" ||
		baseUrl.includes("opencode.ai") ||
		isCloudflareWorkersAI ||
		isCloudflareAiGateway;

	// 使用旧版 max_tokens 字段的提供商
	const useMaxTokens = baseUrl.includes("chutes.ai") || isMoonshot || isCloudflareAiGateway || isTogether;

	const isGrok = provider === "xai" || baseUrl.includes("api.x.ai");
	const isDeepSeek = provider === "deepseek" || baseUrl.includes("deepseek.com");
	// OpenRouter 代理的 Anthropic 模型需要 Anthropic 风格的缓存控制
	const cacheControlFormat = provider === "openrouter" && model.id.startsWith("anthropic/") ? "anthropic" : undefined;

	return {
		supportsStore: !isNonStandard,
		supportsDeveloperRole: !isNonStandard,
		supportsReasoningEffort: !isGrok && !isZai && !isMoonshot && !isTogether && !isCloudflareAiGateway,
		supportsUsageInStreaming: true,
		maxTokensField: useMaxTokens ? "max_tokens" : "max_completion_tokens",
		requiresToolResultName: false,
		requiresAssistantAfterToolResult: false,
		requiresThinkingAsText: false,
		requiresReasoningContentOnAssistantMessages: isDeepSeek,
		thinkingFormat: isDeepSeek
			? "deepseek"
			: isZai
				? "zai"
				: isTogether
					? "together"
					: provider === "openrouter" || baseUrl.includes("openrouter.ai")
						? "openrouter"
						: "openai",
		openRouterRouting: {},
		vercelGatewayRouting: {},
		zaiToolStream: false,
		supportsStrictMode: !isMoonshot && !isTogether && !isCloudflareAiGateway,
		cacheControlFormat,
		sendSessionAffinityHeaders: false,
		supportsLongCacheRetention: !(isTogether || isCloudflareWorkersAI || isCloudflareAiGateway),
	};
}

/**
 * 获取模型的兼容性设置
 *
 * 优先使用模型定义中显式指定的 compat 字段（覆盖自动检测），
 * 未指定的字段回退到自动检测的值。
 * @param model - 目标模型
 * @returns 完全解析的兼容性设置
 */
function getCompat(model: Model<"openai-completions">): ResolvedOpenAICompletionsCompat {
	const detected = detectCompat(model);
	if (!model.compat) return detected;

	// 逐字段合并：显式值优先，自动检测作为回退
	return {
		supportsStore: model.compat.supportsStore ?? detected.supportsStore,
		supportsDeveloperRole: model.compat.supportsDeveloperRole ?? detected.supportsDeveloperRole,
		supportsReasoningEffort: model.compat.supportsReasoningEffort ?? detected.supportsReasoningEffort,
		supportsUsageInStreaming: model.compat.supportsUsageInStreaming ?? detected.supportsUsageInStreaming,
		maxTokensField: model.compat.maxTokensField ?? detected.maxTokensField,
		requiresToolResultName: model.compat.requiresToolResultName ?? detected.requiresToolResultName,
		requiresAssistantAfterToolResult:
			model.compat.requiresAssistantAfterToolResult ?? detected.requiresAssistantAfterToolResult,
		requiresThinkingAsText: model.compat.requiresThinkingAsText ?? detected.requiresThinkingAsText,
		requiresReasoningContentOnAssistantMessages:
			model.compat.requiresReasoningContentOnAssistantMessages ??
			detected.requiresReasoningContentOnAssistantMessages,
		thinkingFormat: model.compat.thinkingFormat ?? detected.thinkingFormat,
		openRouterRouting: model.compat.openRouterRouting ?? {},
		vercelGatewayRouting: model.compat.vercelGatewayRouting ?? detected.vercelGatewayRouting,
		zaiToolStream: model.compat.zaiToolStream ?? detected.zaiToolStream,
		supportsStrictMode: model.compat.supportsStrictMode ?? detected.supportsStrictMode,
		cacheControlFormat: model.compat.cacheControlFormat ?? detected.cacheControlFormat,
		sendSessionAffinityHeaders: model.compat.sendSessionAffinityHeaders ?? detected.sendSessionAffinityHeaders,
		supportsLongCacheRetention: model.compat.supportsLongCacheRetention ?? detected.supportsLongCacheRetention,
	};
}
