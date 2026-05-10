/**
 * @fileoverview API 提供商注册表，管理 LLM 流式调用提供商的注册、查询和注销。
 *
 * 核心功能：
 *   - registerApiProvider: 注册一个新的 API 提供商（含 stream 和 streamSimple）
 *   - getApiProvider: 按_API 标识符查询已注册的提供商
 *   - getApiProviders: 获取全部已注册的提供商列表
 *   - unregisterApiProviders: 按 sourceId 注销同一来源的所有提供商
 *   - clearApiProviders: 清空全部已注册的提供商
 *
 * 内部通过 wrapStream/wrapStreamSimple 对原始流函数进行类型擦除和 API 一致性校验，
 * 统一存储为 ApiProviderInternal，实现泛型类型安全与运行时注册表的解耦。
 */

import type {
	Api,
	AssistantMessageEventStream,
	Context,
	Model,
	SimpleStreamOptions,
	StreamFunction,
	StreamOptions,
} from "./types.js";

/**
 * 类型擦除后的标准流式调用函数签名
 * @param model - 使用的模型实例
 * @param context - 对话上下文
 * @param options - 流式调用选项
 * @returns 助手消息事件流
 */
export type ApiStreamFunction = (
	model: Model<Api>,
	context: Context,
	options?: StreamOptions,
) => AssistantMessageEventStream;

/**
 * 类型擦除后的简化流式调用函数签名（使用 SimpleStreamOptions）
 * @param model - 使用的模型实例
 * @param context - 对话上下文
 * @param options - 简化流式调用选项
 * @returns 助手消息事件流
 */
export type ApiStreamSimpleFunction = (
	model: Model<Api>,
	context: Context,
	options?: SimpleStreamOptions,
) => AssistantMessageEventStream;

/**
 * 外部可用的 API 提供商接口，保留完整的泛型类型信息
 */
export interface ApiProvider<TApi extends Api = Api, TOptions extends StreamOptions = StreamOptions> {
	api: TApi;
	stream: StreamFunction<TApi, TOptions>;
	streamSimple: StreamFunction<TApi, SimpleStreamOptions>;
}

/**
 * 内部存储的 API 提供商结构，泛型已擦除为统一类型
 */
interface ApiProviderInternal {
	api: Api;
	stream: ApiStreamFunction;
	streamSimple: ApiStreamSimpleFunction;
}

/**
 * 注册表中的条目，包含提供商及其来源标识
 */
type RegisteredApiProvider = {
	provider: ApiProviderInternal;
	sourceId?: string;
};

// API 提供商注册表，以 API 标识符为键
const apiProviderRegistry = new Map<string, RegisteredApiProvider>();

/**
 * 包装泛型流式函数为类型擦除的标准签名，同时校验模型与 API 的匹配关系
 * @param api - 目标 API 标识符
 * @param stream - 原始泛型流式函数
 * @returns 类型擦除后的流式函数，调用时会校验 model.api 与 api 是否一致
 */
function wrapStream<TApi extends Api, TOptions extends StreamOptions>(
	api: TApi,
	stream: StreamFunction<TApi, TOptions>,
): ApiStreamFunction {
	return (model, context, options) => {
		if (model.api !== api) {
			throw new Error(`Mismatched api: ${model.api} expected ${api}`);
		}
		return stream(model as Model<TApi>, context, options as TOptions);
	};
}

/**
 * 包装泛型简化流式函数为类型擦除的标准签名，同时校验模型与 API 的匹配关系
 * @param api - 目标 API 标识符
 * @param streamSimple - 原始泛型简化流式函数
 * @returns 类型擦除后的简化流式函数，调用时会校验 model.api 与 api 是否一致
 */
function wrapStreamSimple<TApi extends Api>(
	api: TApi,
	streamSimple: StreamFunction<TApi, SimpleStreamOptions>,
): ApiStreamSimpleFunction {
	return (model, context, options) => {
		if (model.api !== api) {
			throw new Error(`Mismatched api: ${model.api} expected ${api}`);
		}
		return streamSimple(model as Model<TApi>, context, options);
	};
}

/**
 * 注册一个 API 提供商到全局注册表
 * @param provider - 提供商实例，包含 API 标识符和流式函数
 * @param sourceId - 可选的来源标识，用于后续按来源批量注销
 */
export function registerApiProvider<TApi extends Api, TOptions extends StreamOptions>(
	provider: ApiProvider<TApi, TOptions>,
	sourceId?: string,
): void {
	apiProviderRegistry.set(provider.api, {
		provider: {
			api: provider.api,
			stream: wrapStream(provider.api, provider.stream),
			streamSimple: wrapStreamSimple(provider.api, provider.streamSimple),
		},
		sourceId,
	});
}

/**
 * 按 API 标识符查询已注册的提供商
 * @param api - API 标识符
 * @returns 对应的提供商内部结构，未注册时返回 undefined
 */
export function getApiProvider(api: Api): ApiProviderInternal | undefined {
	return apiProviderRegistry.get(api)?.provider;
}

/**
 * 获取全部已注册的提供商列表
 * @returns 提供商内部结构数组
 */
export function getApiProviders(): ApiProviderInternal[] {
	return Array.from(apiProviderRegistry.values(), (entry) => entry.provider);
}

/**
 * 按来源标识注销所有匹配的 API 提供商
 * @param sourceId - 来源标识，注册时传入的 sourceId 相同的条目将被移除
 */
export function unregisterApiProviders(sourceId: string): void {
	for (const [api, entry] of apiProviderRegistry.entries()) {
		if (entry.sourceId === sourceId) {
			apiProviderRegistry.delete(api);
		}
	}
}

/**
 * 清空全部已注册的 API 提供商
 */
export function clearApiProviders(): void {
	apiProviderRegistry.clear();
}
