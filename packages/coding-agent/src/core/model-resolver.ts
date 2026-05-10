/**
 * @fileoverview 模型解析、作用域限定与初始模型选择
 *
 * 本模块负责将用户提供的模型标识（字符串模式）解析为具体的 Model 对象。
 * 它是 CLI 参数处理和模型作用域配置的核心组件。
 *
 * 主要功能：
 * - `defaultModelPerProvider`: 各已知提供商的默认模型 ID 映射
 * - `findExactModelReferenceMatch`: 精确匹配模型引用（支持 provider/model 格式）
 * - `parseModelPattern`: 解析模型模式，提取思维级别（thinking level）
 * - `resolveModelScope`: 批量解析模型模式（支持 glob 通配符）
 * - `resolveCliModel`: 从 CLI 参数解析单个模型
 * - `findInitialModel`: 按优先级链选择初始模型
 * - `restoreModelFromSession`: 从会话中恢复模型并处理回退
 *
 * 匹配策略（按优先级）：
 * 1. 精确匹配完整的 "provider/modelId" 格式
 * 2. 精确匹配裸 modelId（跨提供商唯一时才命中）
 * 3. 部分匹配 modelId 或模型名称
 * 4. 优先选择别名版本（无日期后缀），其次选择最新日期版本
 *
 * 思维级别解析：
 * - 格式为 "pattern:level"（如 "claude-sonnet-4:high"）
 * - 支持冒号作为模型 ID 一部分的情况（如 OpenRouter 的 ":exacto" 后缀）
 * - 从最后一个冒号开始解析，递归处理多层冒号
 */

import type { ThinkingLevel } from "@earendil-works/pi-agent-core";
import { type Api, type KnownProvider, type Model, modelsAreEqual } from "@earendil-works/pi-ai";
import chalk from "chalk";
import { minimatch } from "minimatch";
import { isValidThinkingLevel } from "../cli/args.js";
import { DEFAULT_THINKING_LEVEL } from "./defaults.js";
import type { ModelRegistry } from "./model-registry.js";

/**
 * 各已知提供商的默认模型 ID
 *
 * 当用户指定了提供商但未指定模型，或需要回退到默认模型时使用此映射。
 * 更新此映射时需同步更新 `packages/ai/scripts/generate-models.ts`。
 */
export const defaultModelPerProvider: Record<KnownProvider, string> = {
	"amazon-bedrock": "us.anthropic.claude-opus-4-6-v1",
	anthropic: "claude-opus-4-7",
	openai: "gpt-5.4",
	"azure-openai-responses": "gpt-5.4",
	"openai-codex": "gpt-5.5",
	deepseek: "deepseek-v4-pro",
	google: "gemini-3.1-pro-preview",
	"google-vertex": "gemini-3.1-pro-preview",
	"github-copilot": "gpt-5.4",
	openrouter: "moonshotai/kimi-k2.6",
	"vercel-ai-gateway": "zai/glm-5.1",
	xai: "grok-4.20-0309-reasoning",
	groq: "openai/gpt-oss-120b",
	cerebras: "zai-glm-4.7",
	zai: "glm-5.1",
	mistral: "devstral-medium-latest",
	minimax: "MiniMax-M2.7",
	"minimax-cn": "MiniMax-M2.7",
	moonshotai: "kimi-k2.6",
	"moonshotai-cn": "kimi-k2.6",
	huggingface: "moonshotai/Kimi-K2.6",
	fireworks: "accounts/fireworks/models/kimi-k2p6",
	together: "moonshotai/Kimi-K2.6",
	opencode: "kimi-k2.6",
	"opencode-go": "kimi-k2.6",
	"kimi-coding": "kimi-for-coding",
	"cloudflare-workers-ai": "@cf/moonshotai/kimi-k2.6",
	"cloudflare-ai-gateway": "workers-ai/@cf/moonshotai/kimi-k2.6",
	xiaomi: "mimo-v2.5-pro",
	"xiaomi-token-plan-cn": "mimo-v2.5-pro",
	"xiaomi-token-plan-ams": "mimo-v2.5-pro",
	"xiaomi-token-plan-sgp": "mimo-v2.5-pro",
};

/**
 * 作用域限定后的模型，包含可选的思维级别
 */
export interface ScopedModel {
	model: Model<Api>;
	/** 从模式中显式指定的思维级别（如 "model:high"），未指定则为 undefined */
	thinkingLevel?: ThinkingLevel;
}

/**
 * 判断模型 ID 是否为别名（无日期后缀）
 *
 * 日期后缀通常格式为 -YYYYMMDD（如 claude-sonnet-4-5-20250929）。
 * 别名（如 claude-sonnet-4-5）指向最新日期版本，更适合作为默认选择。
 *
 * @param id - 模型 ID
 * @returns 如果是别名则返回 true
 */
function isAlias(id: string): boolean {
	if (id.endsWith("-latest")) return true;

	// 无日期后缀的 ID 视为别名
	const datePattern = /-\d{8}$/;
	return !datePattern.test(id);
}

/**
 * 在可用模型列表中查找精确匹配的模型引用
 *
 * 支持以下格式：
 * - 完整引用 "provider/modelId"
 * - 裸 modelId（跨提供商唯一时才命中，模糊匹配会被拒绝）
 *
 * @param modelReference - 用户提供的模型引用字符串
 * @param availableModels - 可用模型列表
 * @returns 匹配的模型，未找到或匹配不唯一时返回 undefined
 */
export function findExactModelReferenceMatch(
	modelReference: string,
	availableModels: Model<Api>[],
): Model<Api> | undefined {
	const trimmedReference = modelReference.trim();
	if (!trimmedReference) {
		return undefined;
	}

	const normalizedReference = trimmedReference.toLowerCase();

	// 优先匹配 "provider/modelId" 格式（精确且无歧义）
	const canonicalMatches = availableModels.filter(
		(model) => `${model.provider}/${model.id}`.toLowerCase() === normalizedReference,
	);
	if (canonicalMatches.length === 1) {
		return canonicalMatches[0];
	}
	if (canonicalMatches.length > 1) {
		// 多个匹配视为歧义，拒绝返回
		return undefined;
	}

	// 尝试将斜杠分隔的引用拆分为 provider + modelId 进行匹配
	const slashIndex = trimmedReference.indexOf("/");
	if (slashIndex !== -1) {
		const provider = trimmedReference.substring(0, slashIndex).trim();
		const modelId = trimmedReference.substring(slashIndex + 1).trim();
		if (provider && modelId) {
			const providerMatches = availableModels.filter(
				(model) =>
					model.provider.toLowerCase() === provider.toLowerCase() &&
					model.id.toLowerCase() === modelId.toLowerCase(),
			);
			if (providerMatches.length === 1) {
				return providerMatches[0];
			}
			if (providerMatches.length > 1) {
				return undefined;
			}
		}
	}

	// 最后尝试仅按裸 ID 匹配，要求全局唯一
	const idMatches = availableModels.filter((model) => model.id.toLowerCase() === normalizedReference);
	return idMatches.length === 1 ? idMatches[0] : undefined;
}

/**
 * 尝试将模式匹配到可用模型
 *
 * 匹配策略：
 * 1. 先尝试精确匹配
 * 2. 精确匹配失败后，进行部分匹配（ID 或名称包含模式）
 * 3. 优先选择别名版本（无日期后缀），其次选择最新日期版本
 *
 * @param modelPattern - 模型匹配模式
 * @param availableModels - 可用模型列表
 * @returns 匹配的模型，未找到时返回 undefined
 */
function tryMatchModel(modelPattern: string, availableModels: Model<Api>[]): Model<Api> | undefined {
	const exactMatch = findExactModelReferenceMatch(modelPattern, availableModels);
	if (exactMatch) {
		return exactMatch;
	}

	// 部分匹配：模型 ID 或名称包含模式字符串
	const matches = availableModels.filter(
		(m) =>
			m.id.toLowerCase().includes(modelPattern.toLowerCase()) ||
			m.name?.toLowerCase().includes(modelPattern.toLowerCase()),
	);

	if (matches.length === 0) {
		return undefined;
	}

	// 区分别名和日期版本，别名优先（别名指向最新版本）
	const aliases = matches.filter((m) => isAlias(m.id));
	const datedVersions = matches.filter((m) => !isAlias(m.id));

	if (aliases.length > 0) {
		// 多个别名时选择字典序最大的（通常是最新版本）
		aliases.sort((a, b) => b.id.localeCompare(a.id));
		return aliases[0];
	} else {
		// 无别名时选择日期最新的版本
		datedVersions.sort((a, b) => b.id.localeCompare(a.id));
		return datedVersions[0];
	}
}

/**
 * 模型模式解析结果
 */
export interface ParsedModelResult {
	model: Model<Api> | undefined;
	/** 从模式中解析出的思维级别，未指定则为 undefined */
	thinkingLevel?: ThinkingLevel;
	warning: string | undefined;
}

/**
 * 构建回退模型对象
 *
 * 当用户指定的模型 ID 在提供商中不存在时，使用该提供商的默认模型
 * 作为模板，保留用户指定的 ID，以便后续通过 API 密钥认证时使用。
 *
 * @param provider - 提供商名称
 * @param modelId - 用户指定的模型 ID
 * @param availableModels - 可用模型列表
 * @returns 回退模型对象，提供商无可用模型时返回 undefined
 */
function buildFallbackModel(provider: string, modelId: string, availableModels: Model<Api>[]): Model<Api> | undefined {
	const providerModels = availableModels.filter((m) => m.provider === provider);
	if (providerModels.length === 0) return undefined;

	// 优先使用该提供商的默认模型作为模板
	const defaultId = defaultModelPerProvider[provider as KnownProvider];
	const baseModel = defaultId
		? (providerModels.find((m) => m.id === defaultId) ?? providerModels[0])
		: providerModels[0];

	return {
		...baseModel,
		id: modelId,
		name: modelId,
	};
}

/**
 * 解析模型模式，提取模型和思维级别
 *
 * 处理策略（支持 ID 中包含冒号的模型，如 OpenRouter 的 ":exacto" 后缀）：
 * 1. 先尝试将完整模式作为模型 ID 匹配
 * 2. 如果匹配成功，返回该模型（思维级别为 undefined）
 * 3. 如果匹配失败且包含冒号，从最后一个冒号处拆分：
 *    - 后缀是有效的思维级别 → 使用该级别，递归解析前缀
 *    - 后缀无效 → 根据模式发出警告，递归解析前缀
 *
 * @param pattern - 模型匹配模式（如 "claude-sonnet-4:high"）
 * @param availableModels - 可用模型列表
 * @param options.allowInvalidThinkingLevelFallback - 无效思维级别时是否回退（CLI 模式为 false）
 * @returns 解析结果，包含模型、思维级别和可能的警告
 *
 * @internal 导出用于测试
 */
export function parseModelPattern(
	pattern: string,
	availableModels: Model<Api>[],
	options?: { allowInvalidThinkingLevelFallback?: boolean },
): ParsedModelResult {
	// 先尝试精确匹配
	const exactMatch = tryMatchModel(pattern, availableModels);
	if (exactMatch) {
		return { model: exactMatch, thinkingLevel: undefined, warning: undefined };
	}

	// 精确匹配失败，尝试从最后一个冒号处拆分思维级别
	const lastColonIndex = pattern.lastIndexOf(":");
	if (lastColonIndex === -1) {
		return { model: undefined, thinkingLevel: undefined, warning: undefined };
	}

	const prefix = pattern.substring(0, lastColonIndex);
	const suffix = pattern.substring(lastColonIndex + 1);

	if (isValidThinkingLevel(suffix)) {
		// 后缀是有效的思维级别，递归解析前缀部分
		const result = parseModelPattern(prefix, availableModels, options);
		if (result.model) {
			// 仅在内部递归无警告时才应用此思维级别
			return {
				model: result.model,
				thinkingLevel: result.warning ? undefined : suffix,
				warning: result.warning,
			};
		}
		return result;
	} else {
		// 后缀不是有效的思维级别
		const allowFallback = options?.allowInvalidThinkingLevelFallback ?? true;
		if (!allowFallback) {
			// 严格模式（CLI --model 解析）：将冒号后缀视为模型 ID 的一部分，直接失败
			// 避免意外解析到不同的模型
			return { model: undefined, thinkingLevel: undefined, warning: undefined };
		}

		// 作用域模式：递归解析前缀并发出警告
		const result = parseModelPattern(prefix, availableModels, options);
		if (result.model) {
			return {
				model: result.model,
				thinkingLevel: undefined,
				warning: `Invalid thinking level "${suffix}" in pattern "${pattern}". Using default instead.`,
			};
		}
		return result;
	}
}

/**
 * 批量解析模型模式为实际 Model 对象，支持可选的思维级别
 *
 * 模式格式："pattern:level"（:level 可选）
 * 对于每个模式，查找所有匹配的模型并选择最佳版本：
 * 1. 优先选择别名（如 claude-sonnet-4-5）而非日期版本（claude-sonnet-4-5-20250929）
 * 2. 无别名时选择最新日期版本
 *
 * 支持模式中包含冒号的模型 ID（如 OpenRouter 的 "model:exacto"）。
 * 算法先尝试匹配完整模式，然后逐步剥离冒号后缀进行匹配。
 *
 * @param patterns - 模型匹配模式数组，支持 glob 通配符
 * @param modelRegistry - 模型注册表
 * @returns 去重后的作用域模型列表
 */
export async function resolveModelScope(patterns: string[], modelRegistry: ModelRegistry): Promise<ScopedModel[]> {
	const availableModels = await modelRegistry.getAvailable();
	const scopedModels: ScopedModel[] = [];

	for (const pattern of patterns) {
		// 检测 glob 通配符模式
		if (pattern.includes("*") || pattern.includes("?") || pattern.includes("[")) {
			// 提取可选的思维级别后缀（如 "provider/*:high"）
			const colonIdx = pattern.lastIndexOf(":");
			let globPattern = pattern;
			let thinkingLevel: ThinkingLevel | undefined;

			if (colonIdx !== -1) {
				const suffix = pattern.substring(colonIdx + 1);
				if (isValidThinkingLevel(suffix)) {
					thinkingLevel = suffix;
					globPattern = pattern.substring(0, colonIdx);
				}
			}

			// 同时匹配 "provider/modelId" 格式和裸 modelId，
			// 这样 "*sonnet*" 无需写成 "anthropic/*sonnet*"
			const matchingModels = availableModels.filter((m) => {
				const fullId = `${m.provider}/${m.id}`;
				return minimatch(fullId, globPattern, { nocase: true }) || minimatch(m.id, globPattern, { nocase: true });
			});

			if (matchingModels.length === 0) {
				console.warn(chalk.yellow(`Warning: No models match pattern "${pattern}"`));
				continue;
			}

			for (const model of matchingModels) {
				if (!scopedModels.find((sm) => modelsAreEqual(sm.model, model))) {
					scopedModels.push({ model, thinkingLevel });
				}
			}
			continue;
		}

		const { model, thinkingLevel, warning } = parseModelPattern(pattern, availableModels);

		if (warning) {
			console.warn(chalk.yellow(`Warning: ${warning}`));
		}

		if (!model) {
			console.warn(chalk.yellow(`Warning: No models match pattern "${pattern}"`));
			continue;
		}

		// 去重
		if (!scopedModels.find((sm) => modelsAreEqual(sm.model, model))) {
			scopedModels.push({ model, thinkingLevel });
		}
	}

	return scopedModels;
}

/**
 * CLI 模型解析结果
 */
export interface ResolveCliModelResult {
	model: Model<Api> | undefined;
	thinkingLevel?: ThinkingLevel;
	warning: string | undefined;
	/** 适合 CLI 显示的错误信息。设置时 model 为 undefined */
	error: string | undefined;
}

/**
 * 从 CLI 参数解析单个模型
 *
 * 支持的输入格式：
 * - `--provider <provider> --model <pattern>`
 * - `--model <provider>/<pattern>`
 * - 模糊匹配（与模型作用域相同的规则：精确 ID → 部分 ID/名称匹配）
 *
 * 注意：此函数不直接应用思维级别，但会从 "<pattern>:<thinking>" 格式中
 * 解析并返回思维级别，供调用方应用。
 *
 * @param options.cliProvider - CLI --provider 参数
 * @param options.cliModel - CLI --model 参数
 * @param options.modelRegistry - 模型注册表
 * @returns 解析结果，包含模型、思维级别、警告或错误
 */
export function resolveCliModel(options: {
	cliProvider?: string;
	cliModel?: string;
	modelRegistry: ModelRegistry;
}): ResolveCliModelResult {
	const { cliProvider, cliModel, modelRegistry } = options;

	if (!cliModel) {
		return { model: undefined, warning: undefined, error: undefined };
	}

	// 使用全部模型而非仅有认证配置的模型，允许 --api-key 用于首次设置
	const availableModels = modelRegistry.getAll();
	if (availableModels.length === 0) {
		return {
			model: undefined,
			warning: undefined,
			error: "No models available. Check your installation or add models to models.json.",
		};
	}

	// 构建大小写不敏感的提供商查找映射
	const providerMap = new Map<string, string>();
	for (const m of availableModels) {
		providerMap.set(m.provider.toLowerCase(), m.provider);
	}

	let provider = cliProvider ? providerMap.get(cliProvider.toLowerCase()) : undefined;
	if (cliProvider && !provider) {
		return {
			model: undefined,
			warning: undefined,
			error: `Unknown provider "${cliProvider}". Use --list-models to see available providers/models.`,
		};
	}

	// 当未显式指定 --provider 时，尝试从 "provider/model" 格式推断提供商。
	// 当第一个斜杠前的部分匹配已知提供商时，优先按此解释，
	// 而非匹配 ID 中包含斜杠的模型（如 "zai/glm-5" 应解析为
	// provider=zai, model=glm-5，而非 vercel-ai-gateway 下的 "zai/glm-5"）
	let pattern = cliModel;
	let inferredProvider = false;

	if (!provider) {
		const slashIndex = cliModel.indexOf("/");
		if (slashIndex !== -1) {
			const maybeProvider = cliModel.substring(0, slashIndex);
			const canonical = providerMap.get(maybeProvider.toLowerCase());
			if (canonical) {
				provider = canonical;
				pattern = cliModel.substring(slashIndex + 1);
				inferredProvider = true;
			}
		}
	}

	// 如果未从斜杠推断出提供商，先尝试跨所有模型的精确匹配。
	// 这处理 ID 中自然包含斜杠的模型（如 OpenRouter 风格的 ID）
	if (!provider) {
		const lower = cliModel.toLowerCase();
		const exact = availableModels.find(
			(m) => m.id.toLowerCase() === lower || `${m.provider}/${m.id}`.toLowerCase() === lower,
		);
		if (exact) {
			return { model: exact, warning: undefined, thinkingLevel: undefined, error: undefined };
		}
	}

	// 当同时提供了 --provider 和 --model <provider>/<pattern> 时，
	// 容忍冗余的 provider 前缀并剥离
	if (cliProvider && provider) {
		const prefix = `${provider}/`;
		if (cliModel.toLowerCase().startsWith(prefix.toLowerCase())) {
			pattern = cliModel.substring(prefix.length);
		}
	}

	// 在限定提供商范围内解析模型（严格模式：无效思维级别视为模型 ID 的一部分）
	const candidates = provider ? availableModels.filter((m) => m.provider === provider) : availableModels;
	const { model, thinkingLevel, warning } = parseModelPattern(pattern, candidates, {
		allowInvalidThinkingLevelFallback: false,
	});

	if (model) {
		return { model, thinkingLevel, warning, error: undefined };
	}

	// 如果从斜杠推断出提供商但在该提供商下未找到匹配，
	// 回退到跨所有模型将完整输入作为裸模型 ID 匹配。
	// 这处理 OpenRouter 风格的 ID 如 "openai/gpt-4o:extended"，
	// 其中 "openai" 看起来像提供商但完整字符串实际上是 openrouter 的模型 ID。
	if (inferredProvider) {
		const lower = cliModel.toLowerCase();
		const exact = availableModels.find(
			(m) => m.id.toLowerCase() === lower || `${m.provider}/${m.id}`.toLowerCase() === lower,
		);
		if (exact) {
			return { model: exact, warning: undefined, thinkingLevel: undefined, error: undefined };
		}
		const fallback = parseModelPattern(cliModel, availableModels, {
			allowInvalidThinkingLevelFallback: false,
		});
		if (fallback.model) {
			return {
				model: fallback.model,
				thinkingLevel: fallback.thinkingLevel,
				warning: fallback.warning,
				error: undefined,
			};
		}
	}

	// 尝试构建回退模型（使用提供商的默认模型作为模板）
	if (provider) {
		const fallbackModel = buildFallbackModel(provider, pattern, availableModels);
		if (fallbackModel) {
			const fallbackWarning = warning
				? `${warning} Model "${pattern}" not found for provider "${provider}". Using custom model id.`
				: `Model "${pattern}" not found for provider "${provider}". Using custom model id.`;
			return { model: fallbackModel, thinkingLevel: undefined, warning: fallbackWarning, error: undefined };
		}
	}

	const display = provider ? `${provider}/${pattern}` : cliModel;
	return {
		model: undefined,
		thinkingLevel: undefined,
		warning,
		error: `Model "${display}" not found. Use --list-models to see available models.`,
	};
}

/**
 * 初始模型选择结果
 */
export interface InitialModelResult {
	model: Model<Api> | undefined;
	thinkingLevel: ThinkingLevel;
	fallbackMessage: string | undefined;
}

/**
 * 按优先级链选择初始模型
 *
 * 优先级顺序：
 * 1. CLI 参数指定的提供商和模型
 * 2. 作用域模型中的第一个（继续/恢复会话时跳过）
 * 3. 从会话中恢复的模型（由调用方在恢复场景中处理）
 * 4. 用户设置中保存的默认模型
 * 5. 第一个具有有效 API 密钥的可用模型
 *
 * @param options - 初始模型选择选项
 * @returns 初始模型选择结果
 */
export async function findInitialModel(options: {
	cliProvider?: string;
	cliModel?: string;
	scopedModels: ScopedModel[];
	isContinuing: boolean;
	defaultProvider?: string;
	defaultModelId?: string;
	defaultThinkingLevel?: ThinkingLevel;
	modelRegistry: ModelRegistry;
}): Promise<InitialModelResult> {
	const {
		cliProvider,
		cliModel,
		scopedModels,
		isContinuing,
		defaultProvider,
		defaultModelId,
		defaultThinkingLevel,
		modelRegistry,
	} = options;

	let model: Model<Api> | undefined;
	let thinkingLevel: ThinkingLevel = DEFAULT_THINKING_LEVEL;

	// 优先级 1：CLI 参数
	if (cliProvider && cliModel) {
		const resolved = resolveCliModel({
			cliProvider,
			cliModel,
			modelRegistry,
		});
		if (resolved.error) {
			console.error(chalk.red(resolved.error));
			process.exit(1);
		}
		if (resolved.model) {
			return { model: resolved.model, thinkingLevel: DEFAULT_THINKING_LEVEL, fallbackMessage: undefined };
		}
	}

	// 优先级 2：作用域模型的第一个（继续/恢复会话时跳过，由会话恢复逻辑处理）
	if (scopedModels.length > 0 && !isContinuing) {
		return {
			model: scopedModels[0].model,
			thinkingLevel: scopedModels[0].thinkingLevel ?? defaultThinkingLevel ?? DEFAULT_THINKING_LEVEL,
			fallbackMessage: undefined,
		};
	}

	// 优先级 3：用户设置中保存的默认模型
	if (defaultProvider && defaultModelId) {
		const found = modelRegistry.find(defaultProvider, defaultModelId);
		if (found) {
			model = found;
			if (defaultThinkingLevel) {
				thinkingLevel = defaultThinkingLevel;
			}
			return { model, thinkingLevel, fallbackMessage: undefined };
		}
	}

	// 优先级 4：第一个具有有效 API 密钥的可用模型
	const availableModels = await modelRegistry.getAvailable();

	if (availableModels.length > 0) {
		// 优先从已知提供商中查找默认模型
		for (const provider of Object.keys(defaultModelPerProvider) as KnownProvider[]) {
			const defaultId = defaultModelPerProvider[provider];
			const match = availableModels.find((m) => m.provider === provider && m.id === defaultId);
			if (match) {
				return { model: match, thinkingLevel: DEFAULT_THINKING_LEVEL, fallbackMessage: undefined };
			}
		}

		// 无默认模型匹配时使用第一个可用模型
		return { model: availableModels[0], thinkingLevel: DEFAULT_THINKING_LEVEL, fallbackMessage: undefined };
	}

	// 无可用模型
	return { model: undefined, thinkingLevel: DEFAULT_THINKING_LEVEL, fallbackMessage: undefined };
}

/**
 * 从会话中恢复模型，失败时回退到可用模型
 *
 * @param savedProvider - 会话中保存的提供商
 * @param savedModelId - 会话中保存的模型 ID
 * @param currentModel - 当前已加载的模型（作为首选回退）
 * @param shouldPrintMessages - 是否打印状态消息到控制台
 * @param modelRegistry - 模型注册表
 * @returns 恢复或回退的模型，以及可能的回退消息
 */
export async function restoreModelFromSession(
	savedProvider: string,
	savedModelId: string,
	currentModel: Model<Api> | undefined,
	shouldPrintMessages: boolean,
	modelRegistry: ModelRegistry,
): Promise<{ model: Model<Api> | undefined; fallbackMessage: string | undefined }> {
	const restoredModel = modelRegistry.find(savedProvider, savedModelId);

	// 检查模型是否仍然存在且已配置认证
	const hasConfiguredAuth = restoredModel ? modelRegistry.hasConfiguredAuth(restoredModel) : false;

	if (restoredModel && hasConfiguredAuth) {
		if (shouldPrintMessages) {
			console.log(chalk.dim(`Restored model: ${savedProvider}/${savedModelId}`));
		}
		return { model: restoredModel, fallbackMessage: undefined };
	}

	// 模型不可用（不存在或未配置认证）
	const reason = !restoredModel ? "model no longer exists" : "no auth configured";

	if (shouldPrintMessages) {
		console.error(chalk.yellow(`Warning: Could not restore model ${savedProvider}/${savedModelId} (${reason}).`));
	}

	// 回退策略 1：使用当前已有的模型
	if (currentModel) {
		if (shouldPrintMessages) {
			console.log(chalk.dim(`Falling back to: ${currentModel.provider}/${currentModel.id}`));
		}
		return {
			model: currentModel,
			fallbackMessage: `Could not restore model ${savedProvider}/${savedModelId} (${reason}). Using ${currentModel.provider}/${currentModel.id}.`,
		};
	}

	// 回退策略 2：从可用模型中选择默认模型或第一个可用模型
	const availableModels = await modelRegistry.getAvailable();

	if (availableModels.length > 0) {
		let fallbackModel: Model<Api> | undefined;
		for (const provider of Object.keys(defaultModelPerProvider) as KnownProvider[]) {
			const defaultId = defaultModelPerProvider[provider];
			const match = availableModels.find((m) => m.provider === provider && m.id === defaultId);
			if (match) {
				fallbackModel = match;
				break;
			}
		}

		if (!fallbackModel) {
			fallbackModel = availableModels[0];
		}

		if (shouldPrintMessages) {
			console.log(chalk.dim(`Falling back to: ${fallbackModel.provider}/${fallbackModel.id}`));
		}

		return {
			model: fallbackModel,
			fallbackMessage: `Could not restore model ${savedProvider}/${savedModelId} (${reason}). Using ${fallbackModel.provider}/${fallbackModel.id}.`,
		};
	}

	// 无可用模型
	return { model: undefined, fallbackMessage: undefined };
}
