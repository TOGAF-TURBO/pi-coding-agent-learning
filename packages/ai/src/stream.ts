/**
 * @fileoverview 流式调用的统一入口，提供简化的 LLM 调用 API。
 *
 * 核心导出函数：
 *   - stream()：流式调用，返回事件流（实时获取文本/工具调用增量）
 *   - complete()：非流式调用，等待完整响应后返回 AssistantMessage
 *   - streamSimple()：简化流式调用，使用 SimpleStreamOptions（含推理级别）
 *   - completeSimple()：简化非流式调用，使用 SimpleStreamOptions
 *   - getEnvApiKey()：从环境变量获取 API 密钥
 *
 * 调用流程：
 *   stream(model, context, options)
 *     -> resolveApiProvider(model.api)  // 按 API 标识符查找注册的提供商
 *     -> provider.stream(model, context, options)  // 委托给具体提供商
 *     -> AssistantMessageEventStream  // 返回统一的事件流
 */

// 触发内置提供商的注册（副作用导入，确保调用 stream 前提供商已就绪）
import "./providers/register-builtins.js";

import { getApiProvider } from "./api-registry.js";
import type {
	Api,
	AssistantMessage,
	AssistantMessageEventStream,
	Context,
	Model,
	ProviderStreamOptions,
	SimpleStreamOptions,
	StreamOptions,
} from "./types.js";

export { getEnvApiKey } from "./env-api-keys.js";

/**
 * 按 API 标识符查找已注册的提供商，未找到则抛出错误
 * @param api - API 标识符（如 "openai-completions"、"anthropic-messages"）
 * @returns 对应的 API 提供商内部结构
 * @throws 未注册对应 API 提供商时抛出错误
 */
function resolveApiProvider(api: Api) {
	const provider = getApiProvider(api);
	if (!provider) {
		throw new Error(`No API provider registered for api: ${api}`);
	}
	return provider;
}

/**
 * 流式调用 LLM，实时返回事件流
 * @param model - 目标模型实例
 * @param context - 对话上下文（系统提示 + 消息历史 + 工具列表）
 * @param options - 流式调用选项（温度、Token 上限、中止信号等）
 * @returns 助手消息事件流，可逐事件消费文本/工具调用增量
 */
export function stream<TApi extends Api>(
	model: Model<TApi>,
	context: Context,
	options?: ProviderStreamOptions,
): AssistantMessageEventStream {
	const provider = resolveApiProvider(model.api);
	return provider.stream(model, context, options as StreamOptions);
}

/**
 * 非流式调用 LLM，等待完整响应后返回
 * 内部调用 stream() 并等待其 result()
 * @param model - 目标模型实例
 * @param context - 对话上下文
 * @param options - 流式调用选项
 * @returns 完整的助手消息
 */
export async function complete<TApi extends Api>(
	model: Model<TApi>,
	context: Context,
	options?: ProviderStreamOptions,
): Promise<AssistantMessage> {
	const s = stream(model, context, options);
	return s.result();
}

/**
 * 简化流式调用，使用 SimpleStreamOptions（包含推理级别控制）
 * @param model - 目标模型实例
 * @param context - 对话上下文
 * @param options - 简化选项（含 reasoning 思考级别和 thinkingBudgets Token 预算）
 * @returns 助手消息事件流
 */
export function streamSimple<TApi extends Api>(
	model: Model<TApi>,
	context: Context,
	options?: SimpleStreamOptions,
): AssistantMessageEventStream {
	const provider = resolveApiProvider(model.api);
	return provider.streamSimple(model, context, options);
}

/**
 * 简化非流式调用，使用 SimpleStreamOptions
 * @param model - 目标模型实例
 * @param context - 对话上下文
 * @param options - 简化选项
 * @returns 完整的助手消息
 */
export async function completeSimple<TApi extends Api>(
	model: Model<TApi>,
	context: Context,
	options?: SimpleStreamOptions,
): Promise<AssistantMessage> {
	const s = streamSimple(model, context, options);
	return s.result();
}
