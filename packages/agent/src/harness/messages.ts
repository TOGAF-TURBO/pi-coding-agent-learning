/**
 * @fileoverview Messages — 自定义消息类型及其与 LLM 消息格式的转换。
 *
 * 定义了 Agent 层的扩展消息类型（bash 执行结果、自定义消息、分支摘要、
 * 压缩摘要），并通过 TypeScript 的模块增强（module augmentation）将它们
 * 注册到 AgentMessage 联合类型中。
 *
 * 核心函数 convertToLlm() 将所有自定义消息类型转换为 pi-ai 的标准 Message
 * 格式，供 LLM 提供商消费：
 *   - bashExecution → user 消息（命令 + 输出的文本表示）
 *   - custom → user 消息（字符串或多模态内容）
 *   - branchSummary → user 消息（包裹在 <summary> XML 标签中）
 *   - compactionSummary → user 消息（包裹在 <summary> XML 标签中）
 */

import type { ImageContent, Message, TextContent } from "@earendil-works/pi-ai";
import type { AgentMessage } from "../types.js";

// ============================================================================
// 压缩摘要的 XML 包装文本
// ============================================================================

/** 压缩摘要的前缀模板 */
export const COMPACTION_SUMMARY_PREFIX = `The conversation history before this point was compacted into the following summary:

<summary>
`;

/** 压缩摘要的后缀模板 */
export const COMPACTION_SUMMARY_SUFFIX = `
</summary>`;

/** 分支摘要的前缀模板 */
export const BRANCH_SUMMARY_PREFIX = `The following is a summary of a branch that this conversation came back from:

<summary>
`;

/** 分支摘要的后缀模板 */
export const BRANCH_SUMMARY_SUFFIX = `</summary>`;

// ============================================================================
// 自定义消息类型定义
// ============================================================================

/**
 * Bash 命令执行结果消息
 *
 * 记录 shell 命令的执行结果，包括命令文本、输出、退出码等。
 * 通过 excludeFromContext 可标记不纳入 LLM 上下文的消息。
 */
export interface BashExecutionMessage {
	role: "bashExecution";
	/** 执行的命令 */
	command: string;
	/** 命令输出 */
	output: string;
	/** 进程退出码（未完成时为 undefined） */
	exitCode: number | undefined;
	/** 是否被取消 */
	cancelled: boolean;
	/** 输出是否被截断 */
	truncated: boolean;
	/** 截断时完整输出的文件路径 */
	fullOutputPath?: string;
	timestamp: number;
	/** 设为 true 时从 LLM 上下文中排除 */
	excludeFromContext?: boolean;
}

/**
 * 自定义消息 — 扩展可注入任意内容到对话流中
 *
 * 支持字符串或多模态内容，display 控制是否在 UI 中显示。
 * details 可携带任意结构化数据供扩展使用。
 */
export interface CustomMessage<T = unknown> {
	role: "custom";
	/** 自定义类型标识 */
	customType: string;
	/** 消息内容（文本或多模态） */
	content: string | (TextContent | ImageContent)[];
	/** 是否在 UI 中显示 */
	display: boolean;
	/** 扩展详情 */
	details?: T;
	timestamp: number;
}

/**
 * 分支摘要消息 — 会话树导航时生成的上下文摘要
 *
 * 当对话跳转到会话树的其他分支时，被跳过分支的内容摘要
 * 作为消息注入，帮助模型理解上下文的切换。
 */
export interface BranchSummaryMessage {
	role: "branchSummary";
	/** 摘要文本 */
	summary: string;
	/** 跳转来源的条目 ID */
	fromId: string;
	timestamp: number;
}

/**
 * 压缩摘要消息 — 上下文压缩时生成的历史摘要
 *
 * 当对话历史过长触发压缩时，被压缩的内容以摘要形式保留，
 * 作为消息注入新的上下文中。
 */
export interface CompactionSummaryMessage {
	role: "compactionSummary";
	/** 摘要文本 */
	summary: string;
	/** 压缩前的 Token 数 */
	tokensBefore: number;
	timestamp: number;
}

// ============================================================================
// 模块增强 — 将自定义消息类型注册到 AgentMessage 联合类型
// ============================================================================

declare module "../types.js" {
	interface CustomAgentMessages {
		bashExecution: BashExecutionMessage;
		custom: CustomMessage;
		branchSummary: BranchSummaryMessage;
		compactionSummary: CompactionSummaryMessage;
	}
}

// ============================================================================
// 消息转换函数
// ============================================================================

/**
 * 将 Bash 执行结果转换为可读文本
 *
 * @param msg - Bash 执行消息
 * @returns 格式化的文本（包含命令、输出、状态信息）
 */
export function bashExecutionToText(msg: BashExecutionMessage): string {
	let text = `Ran \`${msg.command}\`\n`;
	if (msg.output) {
		text += `\`\`\`\n${msg.output}\n\`\`\``;
	} else {
		text += "(no output)";
	}
	if (msg.cancelled) {
		text += "\n\n(command cancelled)";
	} else if (msg.exitCode !== null && msg.exitCode !== undefined && msg.exitCode !== 0) {
		text += `\n\nCommand exited with code ${msg.exitCode}`;
	}
	if (msg.truncated && msg.fullOutputPath) {
		text += `\n\n[Output truncated. Full output: ${msg.fullOutputPath}]`;
	}
	return text;
}

/** 创建分支摘要消息 */
export function createBranchSummaryMessage(summary: string, fromId: string, timestamp: string): BranchSummaryMessage {
	return {
		role: "branchSummary",
		summary,
		fromId,
		timestamp: new Date(timestamp).getTime(),
	};
}

/** 创建压缩摘要消息 */
export function createCompactionSummaryMessage(
	summary: string,
	tokensBefore: number,
	timestamp: string,
): CompactionSummaryMessage {
	return {
		role: "compactionSummary",
		summary,
		tokensBefore,
		timestamp: new Date(timestamp).getTime(),
	};
}

/** 创建自定义消息 */
export function createCustomMessage(
	customType: string,
	content: string | (TextContent | ImageContent)[],
	display: boolean,
	details: unknown | undefined,
	timestamp: string,
): CustomMessage {
	return {
		role: "custom",
		customType,
		content,
		display,
		details,
		timestamp: new Date(timestamp).getTime(),
	};
}

/**
 * 将 Agent 消息列表转换为 LLM 可消费的标准 Message 格式
 *
 * 转换规则：
 *   - bashExecution（未排除）→ user 消息（命令输出的文本表示）
 *   - custom → user 消息（字符串内容转为文本块）
 *   - branchSummary → user 消息（<summary> XML 标签包裹）
 *   - compactionSummary → user 消息（<summary> XML 标签包裹）
 *   - user/assistant/toolResult → 原样传递
 *   - excludeFromContext=true 的 bashExecution → 过滤掉
 *
 * @param messages - Agent 消息列表
 * @returns LLM 标准消息列表（过滤掉不可见的消息）
 */
export function convertToLlm(messages: AgentMessage[]): Message[] {
	return messages
		.map((m): Message | undefined => {
			switch (m.role) {
				case "bashExecution":
					// 排除标记为不纳入上下文的 bash 消息
					if (m.excludeFromContext) {
						return undefined;
					}
					return {
						role: "user",
						content: [{ type: "text", text: bashExecutionToText(m) }],
						timestamp: m.timestamp,
					};
				case "custom": {
					// 字符串内容转为文本块，多模态内容原样传递
					const content = typeof m.content === "string" ? [{ type: "text" as const, text: m.content }] : m.content;
					return {
						role: "user",
						content,
						timestamp: m.timestamp,
					};
				}
				case "branchSummary":
					return {
						role: "user",
						content: [{ type: "text" as const, text: BRANCH_SUMMARY_PREFIX + m.summary + BRANCH_SUMMARY_SUFFIX }],
						timestamp: m.timestamp,
					};
				case "compactionSummary":
					return {
						role: "user",
						content: [
							{ type: "text" as const, text: COMPACTION_SUMMARY_PREFIX + m.summary + COMPACTION_SUMMARY_SUFFIX },
						],
						timestamp: m.timestamp,
					};
				case "user":
				case "assistant":
				case "toolResult":
					// 标准消息类型原样传递
					return m;
				default:
					return undefined;
			}
		})
		.filter((m): m is Message => m !== undefined);
}
