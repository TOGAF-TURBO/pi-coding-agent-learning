/**
 * @fileoverview 消息格式转换模块 — 在发送到 LLM 前对对话历史进行规范化处理。
 *
 * 不同 LLM 提供商对输入格式有不同要求（如图片支持、思考块处理、工具调用 ID 格式等）。
 * 此模块在流式调用前对消息列表进行统一转换，确保消息格式符合目标提供商的 API 要求。
 *
 * 提供以下功能：
 *   - transformMessages: 消息转换主入口，依次执行图片降级、思考块处理、工具调用 ID 规范化
 *
 * 转换流程：
 *   1. 图片降级：如果模型不支持视觉，将图片内容块替换为文本占位符
 *   2. 内容块变换：处理思考块（跨模型时去除加密签名）、规范化工具调用 ID
 *   3. 孤儿工具调用修复：为缺少结果的工具调用插入合成错误结果
 */

import type {
	Api,
	AssistantMessage,
	ImageContent,
	Message,
	Model,
	TextContent,
	ToolCall,
	ToolResultMessage,
} from "../types.js";

// 不支持视觉的模型收到图片时的替代文本
const NON_VISION_USER_IMAGE_PLACEHOLDER = "(image omitted: model does not support images)";
const NON_VISION_TOOL_IMAGE_PLACEHOLDER = "(tool image omitted: model does not support images)";

/**
 * 将连续的图片内容块合并替换为单个占位符文本
 *
 * 连续多张图片只输出一个占位符，避免消息中出现大量重复提示。
 * @param content - 原始内容块数组（文本和图片混合）
 * @param placeholder - 替换图片的文本占位符
 * @returns 仅包含文本内容块的数组
 */
function replaceImagesWithPlaceholder(content: (TextContent | ImageContent)[], placeholder: string): TextContent[] {
	const result: TextContent[] = [];
	let previousWasPlaceholder = false;

	for (const block of content) {
		if (block.type === "image") {
			// 连续图片只保留一个占位符
			if (!previousWasPlaceholder) {
				result.push({ type: "text", text: placeholder });
			}
			previousWasPlaceholder = true;
			continue;
		}

		result.push(block);
		previousWasPlaceholder = block.text === placeholder;
	}

	return result;
}

/**
 * 如果模型不支持视觉（input 不包含 "image"），将消息中的图片替换为文本占位符
 * @param messages - 原始消息列表
 * @param model - 目标模型
 * @returns 图片已降级的消息列表
 */
function downgradeUnsupportedImages<TApi extends Api>(messages: Message[], model: Model<TApi>): Message[] {
	if (model.input.includes("image")) {
		return messages;
	}

	return messages.map((msg) => {
		// 用户消息中的图片
		if (msg.role === "user" && Array.isArray(msg.content)) {
			return {
				...msg,
				content: replaceImagesWithPlaceholder(msg.content, NON_VISION_USER_IMAGE_PLACEHOLDER),
			};
		}

		// 工具结果中的图片
		if (msg.role === "toolResult") {
			return {
				...msg,
				content: replaceImagesWithPlaceholder(msg.content, NON_VISION_TOOL_IMAGE_PLACEHOLDER),
			};
		}

		return msg;
	});
}

/**
 * 消息转换主入口 — 对对话历史进行跨提供商兼容性处理
 *
 * 处理步骤：
 *   1. 图片降级：不支持视觉的模型，图片替换为占位文本
 *   2. 思考块处理：跨模型时，加密思考内容被移除；明文思考转为普通文本
 *   3. 工具调用 ID 规范化：跨提供商时，通过 normalizeToolCallId 回调统一 ID 格式
 *   4. 孤儿工具调用修复：为缺少对应结果的工具调用插入合成错误结果
 *   5. 错误/中止消息过滤：跳过 stopReason 为 error 或 aborted 的助手消息
 *
 * @param messages - 原始消息列表
 * @param model - 目标模型（决定转换策略）
 * @param normalizeToolCallId - 可选的工具调用 ID 规范化函数
 *   不同提供商对工具调用 ID 的格式有不同要求：
 *   - OpenAI Responses API 生成的 ID 可达 450+ 字符且含特殊字符
 *   - Anthropic 要求 ID 匹配 ^[a-zA-Z0-9_-]+$ 且最长 64 字符
 *   通过此回调函数进行格式转换，确保跨提供商兼容
 * @returns 转换后的消息列表，可直接传递给 LLM API
 */
export function transformMessages<TApi extends Api>(
	messages: Message[],
	model: Model<TApi>,
	normalizeToolCallId?: (id: string, model: Model<TApi>, source: AssistantMessage) => string,
): Message[] {
	// 原始工具调用 ID → 规范化 ID 的映射表，用于同步修改对应的工具结果消息
	const toolCallIdMap = new Map<string, string>();
	const imageAwareMessages = downgradeUnsupportedImages(messages, model);

	// ===== 第一遍：内容块变换（图片降级、思考块处理、工具调用 ID 规范化） =====
	const transformed = imageAwareMessages.map((msg) => {
		// 用户消息直接透传（图片已在 downgradeUnsupportedImages 中处理）
		if (msg.role === "user") {
			return msg;
		}

		// 工具结果消息：应用工具调用 ID 的规范化映射
		if (msg.role === "toolResult") {
			const normalizedId = toolCallIdMap.get(msg.toolCallId);
			if (normalizedId && normalizedId !== msg.toolCallId) {
				return { ...msg, toolCallId: normalizedId };
			}
			return msg;
		}

		// 助手消息：需要处理思考块和工具调用
		if (msg.role === "assistant") {
			const assistantMsg = msg as AssistantMessage;
			// 判断此消息是否来自同一模型（同一提供商 + 同一 API + 同一模型 ID）
			const isSameModel =
				assistantMsg.provider === model.provider &&
				assistantMsg.api === model.api &&
				assistantMsg.model === model.id;

			const transformedContent = assistantMsg.content.flatMap((block) => {
				// --- 思考块处理 ---
				if (block.type === "thinking") {
					// 加密思考内容（redacted）仅在同一模型下有效，跨模型时丢弃以避免 API 错误
					if (block.redacted) {
						return isSameModel ? block : [];
					}
					// 同一模型且有签名：保留（回放时需要）
					if (isSameModel && block.thinkingSignature) return block;
					// 空思考块：跳过
					if (!block.thinking || block.thinking.trim() === "") return [];
					// 同一模型：原样保留
					if (isSameModel) return block;
					// 跨模型：将思考内容降级为普通文本，保留推理信息但不作为思考块
					return {
						type: "text" as const,
						text: block.thinking,
					};
				}

				// --- 文本块处理 ---
				if (block.type === "text") {
					if (isSameModel) return block;
					// 跨模型时重新构造，去除可能的模型特有字段
					return {
						type: "text" as const,
						text: block.text,
					};
				}

				// --- 工具调用块处理 ---
				if (block.type === "toolCall") {
					const toolCall = block as ToolCall;
					let normalizedToolCall: ToolCall = toolCall;

					// 跨模型时移除思考签名（仅原始模型能验证）
					if (!isSameModel && toolCall.thoughtSignature) {
						normalizedToolCall = { ...toolCall };
						delete (normalizedToolCall as { thoughtSignature?: string }).thoughtSignature;
					}

					// 跨模型时规范化工具调用 ID
					if (!isSameModel && normalizeToolCallId) {
						const normalizedId = normalizeToolCallId(toolCall.id, model, assistantMsg);
						if (normalizedId !== toolCall.id) {
							toolCallIdMap.set(toolCall.id, normalizedId);
							normalizedToolCall = { ...normalizedToolCall, id: normalizedId };
						}
					}

					return normalizedToolCall;
				}

				return block;
			});

			return {
				...assistantMsg,
				content: transformedContent,
			};
		}
		return msg;
	});

	// ===== 第二遍：孤儿工具调用修复 =====
	// 某些情况下（如上下文压缩、错误恢复），工具调用可能缺少对应的工具结果。
	// LLM API 要求每个工具调用都必须有对应的工具结果，否则会报错。
	// 此遍扫描消息列表，为孤儿工具调用插入合成的错误结果。
	const result: Message[] = [];
	let pendingToolCalls: ToolCall[] = [];
	let existingToolResultIds = new Set<string>();

	/** 为当前待处理的孤儿工具调用插入合成错误结果 */
	const insertSyntheticToolResults = () => {
		if (pendingToolCalls.length > 0) {
			for (const tc of pendingToolCalls) {
				if (!existingToolResultIds.has(tc.id)) {
					result.push({
						role: "toolResult",
						toolCallId: tc.id,
						toolName: tc.name,
						content: [{ type: "text", text: "No result provided" }],
						isError: true,
						timestamp: Date.now(),
					} as ToolResultMessage);
				}
			}
			pendingToolCalls = [];
			existingToolResultIds = new Set();
		}
	};

	for (let i = 0; i < transformed.length; i++) {
		const msg = transformed[i];

		if (msg.role === "assistant") {
			// 遇到新的助手消息时，先处理上一轮的孤儿工具调用
			insertSyntheticToolResults();

			// 跳过错误/中止的助手消息
			// 这些是不完整的轮次，重放会导致 API 错误：
			// - 可能包含部分内容（如无消息体的推理、不完整的工具调用）
			// - 重放会导致提供商报错（如 OpenAI 的 "reasoning without following item"）
			// - 应让模型从最后一个有效状态重新开始
			const assistantMsg = msg as AssistantMessage;
			if (assistantMsg.stopReason === "error" || assistantMsg.stopReason === "aborted") {
				continue;
			}

			// 跟踪此助手消息中的工具调用，等待后续的工具结果
			const toolCalls = assistantMsg.content.filter((b) => b.type === "toolCall") as ToolCall[];
			if (toolCalls.length > 0) {
				pendingToolCalls = toolCalls;
				existingToolResultIds = new Set();
			}

			result.push(msg);
		} else if (msg.role === "toolResult") {
			// 记录已有的工具结果 ID，用于孤儿检测
			existingToolResultIds.add(msg.toolCallId);
			result.push(msg);
		} else if (msg.role === "user") {
			// 用户消息中断了工具调用流程，为未匹配的工具调用插入合成结果
			insertSyntheticToolResults();
			result.push(msg);
		} else {
			result.push(msg);
		}
	}

	// 对话末尾可能还有未匹配的工具调用
	insertSyntheticToolResults();

	return result;
}
