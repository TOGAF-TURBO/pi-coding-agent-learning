/**
 * @fileoverview 模型注册表与模型相关的工具函数。
 *
 * 核心功能：
 *   - getModel()：按提供商和模型 ID 获取模型定义
 *   - getProviders()：获取所有已注册的提供商列表
 *   - getModels()：获取指定提供商的所有模型
 *   - calculateCost()：根据 Token 用量和模型费率计算成本
 *   - getSupportedThinkingLevels()：获取模型支持的思考级别
 *   - clampThinkingLevel()：将请求的思考级别限制到模型支持的范围内
 *   - modelsAreEqual()：比较两个模型是否相同（基于 id + provider）
 *
 * 模型数据来源：
 *   由 scripts/generate-models.ts 自动生成到 models.generated.ts，
 *   本模块在加载时将其初始化到内存注册表中。
 */

import { MODELS } from "./models.generated.js";
import type { Api, KnownProvider, Model, ModelThinkingLevel, Usage } from "./types.js";

// 模型注册表：按提供商 ID 分组的两级 Map
// 外层 Map 键为提供商 ID，内层 Map 键为模型 ID
const modelRegistry: Map<string, Map<string, Model<Api>>> = new Map();

// 模块加载时从生成的模型数据初始化注册表
for (const [provider, models] of Object.entries(MODELS)) {
	const providerModels = new Map<string, Model<Api>>();
	for (const [id, model] of Object.entries(models)) {
		providerModels.set(id, model as Model<Api>);
	}
	modelRegistry.set(provider, providerModels);
}

/**
 * 从生成的模型数据中推断模型使用的 API 类型
 */
type ModelApi<
	TProvider extends KnownProvider,
	TModelId extends keyof (typeof MODELS)[TProvider],
> = (typeof MODELS)[TProvider][TModelId] extends { api: infer TApi } ? (TApi extends Api ? TApi : never) : never;

/**
 * 按提供商和模型 ID 获取模型定义
 * @param provider - 提供商 ID
 * @param modelId - 模型 ID
 * @returns 模型定义对象，支持完整的类型推断
 */
export function getModel<TProvider extends KnownProvider, TModelId extends keyof (typeof MODELS)[TProvider]>(
	provider: TProvider,
	modelId: TModelId,
): Model<ModelApi<TProvider, TModelId>> {
	const providerModels = modelRegistry.get(provider);
	return providerModels?.get(modelId as string) as Model<ModelApi<TProvider, TModelId>>;
}

/**
 * 获取所有已注册的提供商列表
 * @returns 提供商 ID 数组
 */
export function getProviders(): KnownProvider[] {
	return Array.from(modelRegistry.keys()) as KnownProvider[];
}

/**
 * 获取指定提供商的所有模型
 * @param provider - 提供商 ID
 * @returns 该提供商下所有模型的数组
 */
export function getModels<TProvider extends KnownProvider>(
	provider: TProvider,
): Model<ModelApi<TProvider, keyof (typeof MODELS)[TProvider]>>[] {
	const models = modelRegistry.get(provider);
	return models ? (Array.from(models.values()) as Model<ModelApi<TProvider, keyof (typeof MODELS)[TProvider]>>[]) : [];
}

/**
 * 根据 Token 用量和模型费率计算实际成本
 * 费率单位为美元/百万 Token，计算结果直接写入 usage.cost
 * @param model - 模型定义（含费率信息）
 * @param usage - Token 用量统计，计算结果会修改其 cost 字段
 * @returns 成本明细
 */
export function calculateCost<TApi extends Api>(model: Model<TApi>, usage: Usage): Usage["cost"] {
	usage.cost.input = (model.cost.input / 1000000) * usage.input;
	usage.cost.output = (model.cost.output / 1000000) * usage.output;
	usage.cost.cacheRead = (model.cost.cacheRead / 1000000) * usage.cacheRead;
	usage.cost.cacheWrite = (model.cost.cacheWrite / 1000000) * usage.cacheWrite;
	usage.cost.total = usage.cost.input + usage.cost.output + usage.cost.cacheRead + usage.cost.cacheWrite;
	return usage.cost;
}

// 所有可用的思考级别，按强度升序排列
const EXTENDED_THINKING_LEVELS: ModelThinkingLevel[] = ["off", "minimal", "low", "medium", "high", "xhigh"];

/**
 * 获取模型支持的思考级别列表
 * 不支持推理的模型仅返回 ["off"]；支持推理的模型根据 thinkingLevelMap 过滤
 * @param model - 目标模型
 * @returns 该模型支持的思考级别数组
 */
export function getSupportedThinkingLevels<TApi extends Api>(model: Model<TApi>): ModelThinkingLevel[] {
	if (!model.reasoning) return ["off"];

	return EXTENDED_THINKING_LEVELS.filter((level) => {
		const mapped = model.thinkingLevelMap?.[level];
		// thinkingLevelMap 中值为 null 表示该级别不被支持
		if (mapped === null) return false;
		// "xhigh" 需要显式映射才支持
		if (level === "xhigh") return mapped !== undefined;
		return true;
	});
}

/**
 * 将请求的思考级别限制到模型支持的范围内
 * 如果请求的级别不被支持，优先向上查找更高的级别，其次向下回退
 * @param model - 目标模型
 * @param level - 请求的思考级别
 * @returns 调整后的有效思考级别
 */
export function clampThinkingLevel<TApi extends Api>(
	model: Model<TApi>,
	level: ModelThinkingLevel,
): ModelThinkingLevel {
	const availableLevels = getSupportedThinkingLevels(model);
	if (availableLevels.includes(level)) return level;

	const requestedIndex = EXTENDED_THINKING_LEVELS.indexOf(level);
	if (requestedIndex === -1) return availableLevels[0] ?? "off";

	// 优先向上查找更高强度
	for (let i = requestedIndex; i < EXTENDED_THINKING_LEVELS.length; i++) {
		const candidate = EXTENDED_THINKING_LEVELS[i];
		if (availableLevels.includes(candidate)) return candidate;
	}
	// 向下回退到更低强度
	for (let i = requestedIndex - 1; i >= 0; i--) {
		const candidate = EXTENDED_THINKING_LEVELS[i];
		if (availableLevels.includes(candidate)) return candidate;
	}
	return availableLevels[0] ?? "off";
}

/**
 * 比较两个模型是否相同（基于 id 和 provider 双重判断）
 * 任一参数为 null/undefined 时返回 false
 * @param a - 第一个模型
 * @param b - 第二个模型
 * @returns 两个模型是否相同
 */
export function modelsAreEqual<TApi extends Api>(
	a: Model<TApi> | null | undefined,
	b: Model<TApi> | null | undefined,
): boolean {
	if (!a || !b) return false;
	return a.id === b.id && a.provider === b.provider;
}
