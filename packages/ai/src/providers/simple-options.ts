/**
 * @fileoverview 简化流式选项到标准流式选项的转换工具。
 *
 * 将 SimpleStreamOptions 映射为完整的 StreamOptions，并提供思考模式下
 * maxTokens 分配的调整逻辑。所有使用简化接口的提供商都依赖此模块。
 *
 * 提供以下功能：
 *   - buildBaseOptions: 将 SimpleStreamOptions 转换为 StreamOptions
 *   - clampReasoning: 将 xhigh 思考级别降级为 high
 *   - adjustMaxTokensForThinking: 在启用思考模式时调整输出 Token 预算
 */

import type { Api, Model, SimpleStreamOptions, StreamOptions, ThinkingBudgets, ThinkingLevel } from "../types.js";

/**
 * 将简化流式选项转换为标准流式选项
 *
 * 映射规则：
 *   - maxTokens 默认取模型上限与 32000 的较小值，防止过大的输出请求
 *   - apiKey 优先使用显式传入的参数，其次使用 options 中的值
 *   - 其余字段直接透传
 * @param model - 当前使用的模型（用于获取 maxTokens 上限）
 * @param options - 简化流式选项（可选）
 * @param apiKey - 外部提供的 API 密钥（优先级高于 options.apiKey）
 * @returns 完整的标准流式选项对象
 */
export function buildBaseOptions(model: Model<Api>, options?: SimpleStreamOptions, apiKey?: string): StreamOptions {
	return {
		temperature: options?.temperature,
		maxTokens: options?.maxTokens ?? (model.maxTokens > 0 ? Math.min(model.maxTokens, 32000) : undefined),
		signal: options?.signal,
		apiKey: apiKey || options?.apiKey,
		transport: options?.transport,
		cacheRetention: options?.cacheRetention,
		sessionId: options?.sessionId,
		headers: options?.headers,
		onPayload: options?.onPayload,
		onResponse: options?.onResponse,
		timeoutMs: options?.timeoutMs,
		maxRetries: options?.maxRetries,
		maxRetryDelayMs: options?.maxRetryDelayMs,
		metadata: options?.metadata,
	};
}

/**
 * 将 xhigh 思考级别降级为 high
 *
 * 仅部分模型家族支持 xhigh 级别，此函数用于不支持的提供商中将
 * xhigh 安全降级为 high，避免请求被拒绝。
 * @param effort - 原始思考级别
 * @returns 降级后的思考级别（xhigh → high，其余不变）
 */
export function clampReasoning(effort: ThinkingLevel | undefined): Exclude<ThinkingLevel, "xhigh"> | undefined {
	return effort === "xhigh" ? "high" : effort;
}

/**
 * 在启用思考模式时调整输出 Token 预算
 *
 * 思考模式的 Token 消耗分为两部分：思考过程（内部推理）和最终输出。
 * 此函数根据思考级别分配预算，并在总预算超出模型上限时缩减思考预算，
 * 确保至少保留 minOutputTokens 给最终输出。
 *
 * 计算方式：
 *   1. 根据思考级别确定默认思考预算（如 high = 16384 tokens）
 *   2. 总预算 = 基础输出 + 思考预算，但不超过模型上限
 *   3. 如果总预算不够，优先保证输出空间，缩减思考预算
 *
 * @param baseMaxTokens - 基础输出 Token 上限
 * @param modelMaxTokens - 模型允许的最大输出 Token 数
 * @param reasoningLevel - 当前思考级别
 * @param customBudgets - 自定义思考预算（覆盖默认值）
 * @returns 调整后的 maxTokens 和 thinkingBudget
 */
export function adjustMaxTokensForThinking(
	baseMaxTokens: number,
	modelMaxTokens: number,
	reasoningLevel: ThinkingLevel,
	customBudgets?: ThinkingBudgets,
): { maxTokens: number; thinkingBudget: number } {
	// 各级别的默认思考 Token 预算
	const defaultBudgets: ThinkingBudgets = {
		minimal: 1024,
		low: 2048,
		medium: 8192,
		high: 16384,
	};
	const budgets = { ...defaultBudgets, ...customBudgets };

	// 确保最终输出至少保留这么多 Token
	const minOutputTokens = 1024;
	const level = clampReasoning(reasoningLevel)!;

	// 根据级别确定思考预算
	let thinkingBudget = budgets[level]!;

	// 总预算 = 基础输出 + 思考预算，但不超过模型上限
	const maxTokens = Math.min(baseMaxTokens + thinkingBudget, modelMaxTokens);

	// 如果总预算不足以分配思考预算，优先保证输出空间
	if (maxTokens <= thinkingBudget) {
		thinkingBudget = Math.max(0, maxTokens - minOutputTokens);
	}

	return { maxTokens, thinkingBudget };
}
