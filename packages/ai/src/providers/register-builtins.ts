/**
 * @fileoverview API 提供商的延迟加载注册模块。
 *
 * 负责将所有内置 LLM 提供商注册到全局 API 注册表中。采用延迟加载策略：
 * 每个提供商的实现模块仅在首次实际调用时才被导入，避免启动时加载所有提供商代码。
 *
 * 提供以下功能：
 *   - registerBuiltInApiProviders: 注册全部内置提供商
 *   - resetApiProviders: 清空注册表并重新注册内置提供商
 *   - setBedrockProviderModule: 替换 Bedrock 提供商实现（用于外部注入）
 *   - 各提供商的延迟流式函数（stream / streamSimple）
 *
 * 延迟加载机制：
 *   createLazyStream / createLazySimpleStream 创建包装函数，
 *   首次调用时通过动态 import() 加载提供商模块，
 *   加载失败时返回包含错误信息的助手消息。
 */

import { clearApiProviders, registerApiProvider } from "../api-registry.js";
import type {
	Api,
	AssistantMessage,
	AssistantMessageEvent,
	Context,
	Model,
	SimpleStreamOptions,
	StreamFunction,
	StreamOptions,
} from "../types.js";
import { AssistantMessageEventStream } from "../utils/event-stream.js";
import type { BedrockOptions } from "./amazon-bedrock.js";
import type { AnthropicOptions } from "./anthropic.js";
import type { AzureOpenAIResponsesOptions } from "./azure-openai-responses.js";
import type { GoogleOptions } from "./google.js";
import type { GoogleVertexOptions } from "./google-vertex.js";
import type { MistralOptions } from "./mistral.js";
import type { OpenAICodexResponsesOptions } from "./openai-codex-responses.js";
import type { OpenAICompletionsOptions } from "./openai-completions.js";
import type { OpenAIResponsesOptions } from "./openai-responses.js";

// ============================================================================
// 类型定义：各提供商模块的接口
// ============================================================================

/**
 * 延迟加载的通用提供商模块接口
 *
 * 所有提供商模块统一暴露 stream 和 streamSimple 两个函数，
 * 延迟加载后通过此接口访问，抹平各提供商的导出命名差异。
 */
interface LazyProviderModule<
	TApi extends Api,
	TOptions extends StreamOptions,
	TSimpleOptions extends SimpleStreamOptions,
> {
	stream: (model: Model<TApi>, context: Context, options?: TOptions) => AsyncIterable<AssistantMessageEvent>;
	streamSimple: (
		model: Model<TApi>,
		context: Context,
		options?: TSimpleOptions,
	) => AsyncIterable<AssistantMessageEvent>;
}

// 各提供商特有的模块接口，对应其源文件中的导出函数名
interface AnthropicProviderModule {
	streamAnthropic: StreamFunction<"anthropic-messages", AnthropicOptions>;
	streamSimpleAnthropic: StreamFunction<"anthropic-messages", SimpleStreamOptions>;
}

interface AzureOpenAIResponsesProviderModule {
	streamAzureOpenAIResponses: StreamFunction<"azure-openai-responses", AzureOpenAIResponsesOptions>;
	streamSimpleAzureOpenAIResponses: StreamFunction<"azure-openai-responses", SimpleStreamOptions>;
}

interface GoogleProviderModule {
	streamGoogle: StreamFunction<"google-generative-ai", GoogleOptions>;
	streamSimpleGoogle: StreamFunction<"google-generative-ai", SimpleStreamOptions>;
}

interface GoogleVertexProviderModule {
	streamGoogleVertex: StreamFunction<"google-vertex", GoogleVertexOptions>;
	streamSimpleGoogleVertex: StreamFunction<"google-vertex", SimpleStreamOptions>;
}

interface MistralProviderModule {
	streamMistral: StreamFunction<"mistral-conversations", MistralOptions>;
	streamSimpleMistral: StreamFunction<"mistral-conversations", SimpleStreamOptions>;
}

interface OpenAICodexResponsesProviderModule {
	streamOpenAICodexResponses: StreamFunction<"openai-codex-responses", OpenAICodexResponsesOptions>;
	streamSimpleOpenAICodexResponses: StreamFunction<"openai-codex-responses", SimpleStreamOptions>;
}

interface OpenAICompletionsProviderModule {
	streamOpenAICompletions: StreamFunction<"openai-completions", OpenAICompletionsOptions>;
	streamSimpleOpenAICompletions: StreamFunction<"openai-completions", SimpleStreamOptions>;
}

interface OpenAIResponsesProviderModule {
	streamOpenAIResponses: StreamFunction<"openai-responses", OpenAIResponsesOptions>;
	streamSimpleOpenAIResponses: StreamFunction<"openai-responses", SimpleStreamOptions>;
}

interface BedrockProviderModule {
	streamBedrock: (
		model: Model<"bedrock-converse-stream">,
		context: Context,
		options?: BedrockOptions,
	) => AsyncIterable<AssistantMessageEvent>;
	streamSimpleBedrock: (
		model: Model<"bedrock-converse-stream">,
		context: Context,
		options?: SimpleStreamOptions,
	) => AsyncIterable<AssistantMessageEvent>;
}

// ============================================================================
// 模块级 Promise 缓存
// ============================================================================

// 各提供商的模块加载 Promise，使用 ||= 确保只加载一次
let anthropicProviderModulePromise:
	| Promise<LazyProviderModule<"anthropic-messages", AnthropicOptions, SimpleStreamOptions>>
	| undefined;
let azureOpenAIResponsesProviderModulePromise:
	| Promise<LazyProviderModule<"azure-openai-responses", AzureOpenAIResponsesOptions, SimpleStreamOptions>>
	| undefined;
let googleProviderModulePromise:
	| Promise<LazyProviderModule<"google-generative-ai", GoogleOptions, SimpleStreamOptions>>
	| undefined;
let googleVertexProviderModulePromise:
	| Promise<LazyProviderModule<"google-vertex", GoogleVertexOptions, SimpleStreamOptions>>
	| undefined;
let mistralProviderModulePromise:
	| Promise<LazyProviderModule<"mistral-conversations", MistralOptions, SimpleStreamOptions>>
	| undefined;
let openAICodexResponsesProviderModulePromise:
	| Promise<LazyProviderModule<"openai-codex-responses", OpenAICodexResponsesOptions, SimpleStreamOptions>>
	| undefined;
let openAICompletionsProviderModulePromise:
	| Promise<LazyProviderModule<"openai-completions", OpenAICompletionsOptions, SimpleStreamOptions>>
	| undefined;
let openAIResponsesProviderModulePromise:
	| Promise<LazyProviderModule<"openai-responses", OpenAIResponsesOptions, SimpleStreamOptions>>
	| undefined;

// Bedrock 特殊处理：支持外部注入模块（用于浏览器环境等无法动态 import 的场景）
let bedrockProviderModuleOverride:
	| LazyProviderModule<"bedrock-converse-stream", BedrockOptions, SimpleStreamOptions>
	| undefined;
let bedrockProviderModulePromise:
	| Promise<LazyProviderModule<"bedrock-converse-stream", BedrockOptions, SimpleStreamOptions>>
	| undefined;

/**
 * 替换 Bedrock 提供商的模块实现
 *
 * 用于浏览器等无法使用 Node.js 动态 import 的环境，
 * 允许外部直接注入已加载的模块。
 * @param module - Bedrock 提供商模块
 */
export function setBedrockProviderModule(module: BedrockProviderModule): void {
	bedrockProviderModuleOverride = {
		stream: module.streamBedrock,
		streamSimple: module.streamSimpleBedrock,
	};
}

// ============================================================================
// 延迟加载工具函数
// ============================================================================

/**
 * 仅在 Node.js 环境中使用的动态导入包装
 *
 * 部分提供商（如 Bedrock）依赖 Node.js 专属模块，
 * 通过此函数集中标记，方便浏览器环境识别和处理。
 * @param specifier - 模块路径
 * @returns 模块的 Promise
 */
const importNodeOnlyProvider = (specifier: string): Promise<unknown> => import(specifier);

/**
 * 将内部事件流转发到外部事件流
 *
 * 异步迭代内部流的每个事件，逐个推送到外部流，
 * 内部流结束后关闭外部流。
 * @param target - 外部事件流（接收方）
 * @param source - 内部事件流（发送方）
 */
function forwardStream(target: AssistantMessageEventStream, source: AsyncIterable<AssistantMessageEvent>): void {
	(async () => {
		for await (const event of source) {
			target.push(event);
		}
		target.end();
	})();
}

/**
 * 创建加载失败时的错误助手消息
 *
 * 当提供商模块动态导入失败时，返回一个 stopReason 为 "error" 的助手消息，
 * 确保调用方能正确收到错误信息而非无声失败。
 * @param model - 触发加载的模型
 * @param error - 加载过程中的错误对象
 * @returns 包含错误信息的助手消息
 */
function createLazyLoadErrorMessage<TApi extends Api>(model: Model<TApi>, error: unknown): AssistantMessage {
	return {
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
		stopReason: "error",
		errorMessage: error instanceof Error ? error.message : String(error),
		timestamp: Date.now(),
	};
}

/**
 * 创建延迟加载的标准流式函数
 *
 * 返回一个符合 StreamFunction 签名的函数，该函数在被调用时才触发模块加载。
 * 加载成功后将调用委托给模块的 stream 函数，加载失败时返回错误消息流。
 * @param loadModule - 模块加载器函数，返回提供商模块的 Promise
 * @returns 延迟加载的标准流式函数
 */
function createLazyStream<TApi extends Api, TOptions extends StreamOptions, TSimpleOptions extends SimpleStreamOptions>(
	loadModule: () => Promise<LazyProviderModule<TApi, TOptions, TSimpleOptions>>,
): StreamFunction<TApi, TOptions> {
	return (model, context, options) => {
		// 创建外部事件流立即返回，调用方无需等待模块加载
		const outer = new AssistantMessageEventStream();

		loadModule()
			.then((module) => {
				// 模块加载成功，委托给实际提供商的 stream 函数
				const inner = module.stream(model, context, options);
				forwardStream(outer, inner);
			})
			.catch((error) => {
				// 模块加载失败，推送错误消息并关闭流
				const message = createLazyLoadErrorMessage(model, error);
				outer.push({ type: "error", reason: "error", error: message });
				outer.end(message);
			});

		return outer;
	};
}

/**
 * 创建延迟加载的简化流式函数
 *
 * 与 createLazyStream 类似，但委托给模块的 streamSimple 函数。
 * @param loadModule - 模块加载器函数
 * @returns 延迟加载的简化流式函数
 */
function createLazySimpleStream<
	TApi extends Api,
	TOptions extends StreamOptions,
	TSimpleOptions extends SimpleStreamOptions,
>(loadModule: () => Promise<LazyProviderModule<TApi, TOptions, TSimpleOptions>>): StreamFunction<TApi, TSimpleOptions> {
	return (model, context, options) => {
		const outer = new AssistantMessageEventStream();

		loadModule()
			.then((module) => {
				const inner = module.streamSimple(model, context, options);
				forwardStream(outer, inner);
			})
			.catch((error) => {
				const message = createLazyLoadErrorMessage(model, error);
				outer.push({ type: "error", reason: "error", error: message });
				outer.end(message);
			});

		return outer;
	};
}

// ============================================================================
// 各提供商的模块加载器
// ============================================================================

// 每个加载器使用 ||= 确保模块只被导入一次（单例模式），
// 并将提供商特有的导出函数名映射为统一的 stream/streamSimple 接口。

/** 加载 Anthropic 提供商模块 */
function loadAnthropicProviderModule(): Promise<
	LazyProviderModule<"anthropic-messages", AnthropicOptions, SimpleStreamOptions>
> {
	anthropicProviderModulePromise ||= import("./anthropic.js").then((module) => {
		const provider = module as AnthropicProviderModule;
		return {
			stream: provider.streamAnthropic,
			streamSimple: provider.streamSimpleAnthropic,
		};
	});
	return anthropicProviderModulePromise;
}

/** 加载 Azure OpenAI Responses 提供商模块 */
function loadAzureOpenAIResponsesProviderModule(): Promise<
	LazyProviderModule<"azure-openai-responses", AzureOpenAIResponsesOptions, SimpleStreamOptions>
> {
	azureOpenAIResponsesProviderModulePromise ||= import("./azure-openai-responses.js").then((module) => {
		const provider = module as AzureOpenAIResponsesProviderModule;
		return {
			stream: provider.streamAzureOpenAIResponses,
			streamSimple: provider.streamSimpleAzureOpenAIResponses,
		};
	});
	return azureOpenAIResponsesProviderModulePromise;
}

/** 加载 Google Generative AI 提供商模块 */
function loadGoogleProviderModule(): Promise<
	LazyProviderModule<"google-generative-ai", GoogleOptions, SimpleStreamOptions>
> {
	googleProviderModulePromise ||= import("./google.js").then((module) => {
		const provider = module as GoogleProviderModule;
		return {
			stream: provider.streamGoogle,
			streamSimple: provider.streamSimpleGoogle,
		};
	});
	return googleProviderModulePromise;
}

/** 加载 Google Vertex AI 提供商模块 */
function loadGoogleVertexProviderModule(): Promise<
	LazyProviderModule<"google-vertex", GoogleVertexOptions, SimpleStreamOptions>
> {
	googleVertexProviderModulePromise ||= import("./google-vertex.js").then((module) => {
		const provider = module as GoogleVertexProviderModule;
		return {
			stream: provider.streamGoogleVertex,
			streamSimple: provider.streamSimpleGoogleVertex,
		};
	});
	return googleVertexProviderModulePromise;
}

/** 加载 Mistral 提供商模块 */
function loadMistralProviderModule(): Promise<
	LazyProviderModule<"mistral-conversations", MistralOptions, SimpleStreamOptions>
> {
	mistralProviderModulePromise ||= import("./mistral.js").then((module) => {
		const provider = module as MistralProviderModule;
		return {
			stream: provider.streamMistral,
			streamSimple: provider.streamSimpleMistral,
		};
	});
	return mistralProviderModulePromise;
}

/** 加载 OpenAI Codex Responses 提供商模块 */
function loadOpenAICodexResponsesProviderModule(): Promise<
	LazyProviderModule<"openai-codex-responses", OpenAICodexResponsesOptions, SimpleStreamOptions>
> {
	openAICodexResponsesProviderModulePromise ||= import("./openai-codex-responses.js").then((module) => {
		const provider = module as OpenAICodexResponsesProviderModule;
		return {
			stream: provider.streamOpenAICodexResponses,
			streamSimple: provider.streamSimpleOpenAICodexResponses,
		};
	});
	return openAICodexResponsesProviderModulePromise;
}

/** 加载 OpenAI Completions 提供商模块（最常用的 API，被 GLM、DeepSeek 等多家提供商兼容使用） */
function loadOpenAICompletionsProviderModule(): Promise<
	LazyProviderModule<"openai-completions", OpenAICompletionsOptions, SimpleStreamOptions>
> {
	openAICompletionsProviderModulePromise ||= import("./openai-completions.js").then((module) => {
		const provider = module as OpenAICompletionsProviderModule;
		return {
			stream: provider.streamOpenAICompletions,
			streamSimple: provider.streamSimpleOpenAICompletions,
		};
	});
	return openAICompletionsProviderModulePromise;
}

/** 加载 OpenAI Responses 提供商模块 */
function loadOpenAIResponsesProviderModule(): Promise<
	LazyProviderModule<"openai-responses", OpenAIResponsesOptions, SimpleStreamOptions>
> {
	openAIResponsesProviderModulePromise ||= import("./openai-responses.js").then((module) => {
		const provider = module as OpenAIResponsesProviderModule;
		return {
			stream: provider.streamOpenAIResponses,
			streamSimple: provider.streamSimpleOpenAIResponses,
		};
	});
	return openAIResponsesProviderModulePromise;
}

/**
 * 加载 Bedrock 提供商模块
 *
 * 特殊处理：优先使用外部注入的模块（通过 setBedrockProviderModule 设置），
 * 否则使用动态 import 加载。Bedrock 依赖 AWS SDK 等 Node.js 专属模块，
 * 需要通过 importNodeOnlyProvider 标记为 Node.js 环境专用。
 */
function loadBedrockProviderModule(): Promise<
	LazyProviderModule<"bedrock-converse-stream", BedrockOptions, SimpleStreamOptions>
> {
	// 优先使用外部注入的模块
	if (bedrockProviderModuleOverride) {
		return Promise.resolve(bedrockProviderModuleOverride);
	}
	// 通过动态 import 延迟加载（Node.js 环境）
	bedrockProviderModulePromise ||= importNodeOnlyProvider("./amazon-bedrock.js").then((module) => {
		const provider = module as BedrockProviderModule;
		return {
			stream: provider.streamBedrock,
			streamSimple: provider.streamSimpleBedrock,
		};
	});
	return bedrockProviderModulePromise;
}

// ============================================================================
// 延迟流式函数导出
// ============================================================================

// 每个提供商导出两个函数：stream（标准）和 streamSimple（简化），
// 它们通过 createLazyStream / createLazySimpleStream 包装，实现延迟加载。

// Anthropic（Claude 系列）
export const streamAnthropic = createLazyStream(loadAnthropicProviderModule);
export const streamSimpleAnthropic = createLazySimpleStream(loadAnthropicProviderModule);

// Azure OpenAI Responses
export const streamAzureOpenAIResponses = createLazyStream(loadAzureOpenAIResponsesProviderModule);
export const streamSimpleAzureOpenAIResponses = createLazySimpleStream(loadAzureOpenAIResponsesProviderModule);

// Google Generative AI（Gemini 系列）
export const streamGoogle = createLazyStream(loadGoogleProviderModule);
export const streamSimpleGoogle = createLazySimpleStream(loadGoogleProviderModule);

// Google Vertex AI
export const streamGoogleVertex = createLazyStream(loadGoogleVertexProviderModule);
export const streamSimpleGoogleVertex = createLazySimpleStream(loadGoogleVertexProviderModule);

// Mistral
export const streamMistral = createLazyStream(loadMistralProviderModule);
export const streamSimpleMistral = createLazySimpleStream(loadMistralProviderModule);

// OpenAI Codex Responses（ChatGPT 订阅模式）
export const streamOpenAICodexResponses = createLazyStream(loadOpenAICodexResponsesProviderModule);
export const streamSimpleOpenAICodexResponses = createLazySimpleStream(loadOpenAICodexResponsesProviderModule);

// OpenAI Completions（API Key 模式，也兼容 GLM、DeepSeek 等第三方）
export const streamOpenAICompletions = createLazyStream(loadOpenAICompletionsProviderModule);
export const streamSimpleOpenAICompletions = createLazySimpleStream(loadOpenAICompletionsProviderModule);

// OpenAI Responses
export const streamOpenAIResponses = createLazyStream(loadOpenAIResponsesProviderModule);
export const streamSimpleOpenAIResponses = createLazySimpleStream(loadOpenAIResponsesProviderModule);

// Amazon Bedrock（内部使用，不导出）
const streamBedrockLazy = createLazyStream(loadBedrockProviderModule);
const streamSimpleBedrockLazy = createLazySimpleStream(loadBedrockProviderModule);

// ============================================================================
// 注册函数
// ============================================================================

/**
 * 注册所有内置 API 提供商到全局注册表
 *
 * 将 9 个内置提供商按其 API 标识符注册，每个提供商提供
 * 标准流式函数和简化流式函数。此函数在模块加载时自动调用一次，
 * 也可通过 resetApiProviders 重新调用。
 */
export function registerBuiltInApiProviders(): void {
	registerApiProvider({
		api: "anthropic-messages",
		stream: streamAnthropic,
		streamSimple: streamSimpleAnthropic,
	});

	registerApiProvider({
		api: "openai-completions",
		stream: streamOpenAICompletions,
		streamSimple: streamSimpleOpenAICompletions,
	});

	registerApiProvider({
		api: "mistral-conversations",
		stream: streamMistral,
		streamSimple: streamSimpleMistral,
	});

	registerApiProvider({
		api: "openai-responses",
		stream: streamOpenAIResponses,
		streamSimple: streamSimpleOpenAIResponses,
	});

	registerApiProvider({
		api: "azure-openai-responses",
		stream: streamAzureOpenAIResponses,
		streamSimple: streamSimpleAzureOpenAIResponses,
	});

	registerApiProvider({
		api: "openai-codex-responses",
		stream: streamOpenAICodexResponses,
		streamSimple: streamSimpleOpenAICodexResponses,
	});

	registerApiProvider({
		api: "google-generative-ai",
		stream: streamGoogle,
		streamSimple: streamSimpleGoogle,
	});

	registerApiProvider({
		api: "google-vertex",
		stream: streamGoogleVertex,
		streamSimple: streamSimpleGoogleVertex,
	});

	registerApiProvider({
		api: "bedrock-converse-stream",
		stream: streamBedrockLazy,
		streamSimple: streamSimpleBedrockLazy,
	});
}

/**
 * 重置 API 注册表：清空所有已注册提供商，然后重新注册内置提供商
 *
 * 用于测试环境或需要恢复默认状态时调用。
 */
export function resetApiProviders(): void {
	clearApiProviders();
	registerBuiltInApiProviders();
}

// 模块加载时自动注册所有内置提供商
registerBuiltInApiProviders();
