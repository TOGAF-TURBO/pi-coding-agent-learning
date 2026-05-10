/**
 * @fileoverview Session — 会话实例，封装会话存储后端的高层操作接口。
 *
 * Session 类是对 SessionStorage 的面向对象封装，提供消息追加、模型变更、
 * 压缩、分支导航、标签等会话操作。每个 Session 实例对应一个具体的会话，
 * 通过底层的 SessionStorage 实现（内存、JSONL 文件等）持久化数据。
 *
 * 会话数据以树结构组织：每条条目通过 parentId 指向父条目，形成分支树。
 * 活跃分支由 leafId 标识，支持在树中自由导航。
 *
 * buildSessionContext() 是核心函数：从叶节点到根遍历条目，重建完整的消息
 * 历史、思考级别和模型信息，供 Agent 使用。
 */

import type { ImageContent, TextContent } from "@earendil-works/pi-ai";
import type { AgentMessage } from "../../types.js";
import { createBranchSummaryMessage, createCompactionSummaryMessage, createCustomMessage } from "../messages.js";
import type {
	BranchSummaryEntry,
	CompactionEntry,
	CustomEntry,
	CustomMessageEntry,
	LabelEntry,
	MessageEntry,
	ModelChangeEntry,
	SessionContext,
	SessionInfoEntry,
	SessionMetadata,
	SessionStorage,
	SessionTreeEntry,
	ThinkingLevelChangeEntry,
} from "../types.js";

/**
 * 从会话树条目路径重建上下文
 *
 * 遍历从叶到根的条目路径，提取最终的思考级别、模型和消息历史。
 * 处理压缩条目时，用摘要消息替代被压缩的旧消息。
 *
 * 处理逻辑：
 *   1. 单遍扫描确定最终的 thinkingLevel、model、compaction
 *   2. 如果存在压缩条目：
 *      a. 在压缩条目位置插入摘要消息
 *      b. 从 firstKeptEntryId 开始收集保留的消息
 *      c. 跳过被压缩的消息
 *   3. 无压缩时直接收集所有消息
 *
 * @param pathEntries - 从叶到根的条目路径
 * @returns 重建的上下文（消息列表、思考级别、模型信息）
 */
export function buildSessionContext(pathEntries: SessionTreeEntry[]): SessionContext {
	let thinkingLevel = "off";
	let model: { provider: string; modelId: string } | null = null;
	let compaction: CompactionEntry | null = null;

	// 第一遍：确定最终状态（后面的条目覆盖前面的）
	for (const entry of pathEntries) {
		if (entry.type === "thinking_level_change") {
			thinkingLevel = entry.thinkingLevel;
		} else if (entry.type === "model_change") {
			model = { provider: entry.provider, modelId: entry.modelId };
		} else if (entry.type === "message" && entry.message.role === "assistant") {
			// 助手消息中包含模型信息，也作为模型来源
			model = { provider: entry.message.provider, modelId: entry.message.model };
		} else if (entry.type === "compaction") {
			compaction = entry;
		}
	}

	// 第二遍：收集消息
	const messages: AgentMessage[] = [];
	const appendMessage = (entry: SessionTreeEntry) => {
		if (entry.type === "message") {
			messages.push(entry.message as AgentMessage);
		} else if (entry.type === "custom_message") {
			messages.push(
				createCustomMessage(
					entry.customType,
					entry.content as string | (TextContent | ImageContent)[],
					entry.display,
					entry.details,
					entry.timestamp,
				),
			);
		} else if (entry.type === "branch_summary" && entry.summary) {
			messages.push(createBranchSummaryMessage(entry.summary, entry.fromId, entry.timestamp));
		}
	};

	if (compaction) {
		// 有压缩条目时：摘要替代旧消息，从 firstKeptEntryId 开始保留
		messages.push(createCompactionSummaryMessage(compaction.summary, compaction.tokensBefore, compaction.timestamp));
		const compactionIdx = pathEntries.findIndex((e) => e.type === "compaction" && e.id === compaction.id);
		let foundFirstKept = false;
		// 压缩条目之前的条目中，从 firstKeptEntryId 开始保留
		for (let i = 0; i < compactionIdx; i++) {
			const entry = pathEntries[i]!;
			if (entry.id === compaction.firstKeptEntryId) foundFirstKept = true;
			if (foundFirstKept) appendMessage(entry);
		}
		// 压缩条目之后的条目全部保留
		for (let i = compactionIdx + 1; i < pathEntries.length; i++) {
			appendMessage(pathEntries[i]!);
		}
	} else {
		// 无压缩：直接收集所有消息类条目
		for (const entry of pathEntries) {
			appendMessage(entry);
		}
	}

	return { messages, thinkingLevel, model };
}

/**
 * Session — 会话实例
 *
 * 封装 SessionStorage 后端，提供面向对象的会话操作 API。
 * 所有追加操作自动设置 parentId 为当前叶节点，形成线性或分支的树结构。
 */
export class Session<TMetadata extends SessionMetadata = SessionMetadata> {
	private storage: SessionStorage<TMetadata>;

	constructor(storage: SessionStorage<TMetadata>) {
		this.storage = storage;
	}

	/** 获取会话元数据 */
	getMetadata(): Promise<TMetadata> {
		return this.storage.getMetadata();
	}

	/** 获取底层存储后端（用于需要直接访问的场景） */
	getStorage(): SessionStorage<TMetadata> {
		return this.storage;
	}

	/** 获取当前活跃分支的叶节点 ID */
	getLeafId(): Promise<string | null> {
		return this.storage.getLeafId();
	}

	/** 按 ID 获取单个条目 */
	getEntry(id: string): Promise<SessionTreeEntry | undefined> {
		return this.storage.getEntry(id);
	}

	/** 获取所有条目 */
	getEntries(): Promise<SessionTreeEntry[]> {
		return this.storage.getEntries();
	}

	/**
	 * 获取从叶到根的分支路径
	 * @param fromId - 起始条目 ID，默认为当前叶节点
	 */
	async getBranch(fromId?: string): Promise<SessionTreeEntry[]> {
		const leafId = fromId ?? (await this.storage.getLeafId());
		return this.storage.getPathToRoot(leafId);
	}

	/**
	 * 从当前分支重建完整的上下文（消息历史 + 状态）
	 *
	 * 调用 buildSessionContext 将树结构转换为扁平的消息列表。
	 */
	async buildContext(): Promise<SessionContext> {
		return buildSessionContext(await this.getBranch());
	}

	/** 获取指定条目的标签 */
	getLabel(id: string): Promise<string | undefined> {
		return this.storage.getLabel(id);
	}

	/**
	 * 获取会话名称
	 *
	 * 从 session_info 类型的条目中提取最后一个有效的名称。
	 */
	async getSessionName(): Promise<string | undefined> {
		const entries = await this.storage.findEntries("session_info");
		return entries[entries.length - 1]?.name?.trim() || undefined;
	}

	/**
	 * 追加类型化条目的内部方法
	 *
	 * 统一处理条目追加和 ID 返回。
	 */
	private async appendTypedEntry<TEntry extends SessionTreeEntry>(entry: TEntry): Promise<string> {
		await this.storage.appendEntry(entry);
		return entry.id;
	}

	/** 追加消息条目 */
	async appendMessage(message: AgentMessage): Promise<string> {
		return this.appendTypedEntry({
			type: "message",
			id: await this.storage.createEntryId(),
			parentId: await this.storage.getLeafId(),
			timestamp: new Date().toISOString(),
			message,
		} satisfies MessageEntry);
	}

	/** 追加思考级别变更条目 */
	async appendThinkingLevelChange(thinkingLevel: string): Promise<string> {
		return this.appendTypedEntry({
			type: "thinking_level_change",
			id: await this.storage.createEntryId(),
			parentId: await this.storage.getLeafId(),
			timestamp: new Date().toISOString(),
			thinkingLevel,
		} satisfies ThinkingLevelChangeEntry);
	}

	/** 追加模型变更条目 */
	async appendModelChange(provider: string, modelId: string): Promise<string> {
		return this.appendTypedEntry({
			type: "model_change",
			id: await this.storage.createEntryId(),
			parentId: await this.storage.getLeafId(),
			timestamp: new Date().toISOString(),
			provider,
			modelId,
		} satisfies ModelChangeEntry);
	}

	/**
	 * 追加压缩条目
	 *
	 * @param summary - 压缩生成的摘要文本
	 * @param firstKeptEntryId - 压缩后保留的第一条条目 ID
	 * @param tokensBefore - 压缩前的 Token 数
	 * @param details - 扩展详情
	 * @param fromHook - 是否由扩展钩子触发
	 */
	async appendCompaction<T = unknown>(
		summary: string,
		firstKeptEntryId: string,
		tokensBefore: number,
		details?: T,
		fromHook?: boolean,
	): Promise<string> {
		return this.appendTypedEntry({
			type: "compaction",
			id: await this.storage.createEntryId(),
			parentId: await this.storage.getLeafId(),
			timestamp: new Date().toISOString(),
			summary,
			firstKeptEntryId,
			tokensBefore,
			details,
			fromHook,
		} satisfies CompactionEntry<T>);
	}

	/** 追加自定义条目（扩展可存储任意结构化数据） */
	async appendCustomEntry(customType: string, data?: unknown): Promise<string> {
		return this.appendTypedEntry({
			type: "custom",
			id: await this.storage.createEntryId(),
			parentId: await this.storage.getLeafId(),
			timestamp: new Date().toISOString(),
			customType,
			data,
		} satisfies CustomEntry);
	}

	/** 追加自定义消息条目（扩展可注入自定义显示消息） */
	async appendCustomMessageEntry<T = unknown>(
		customType: string,
		content: string | (TextContent | ImageContent)[],
		display: boolean,
		details?: T,
	): Promise<string> {
		return this.appendTypedEntry({
			type: "custom_message",
			id: await this.storage.createEntryId(),
			parentId: await this.storage.getLeafId(),
			timestamp: new Date().toISOString(),
			customType,
			content,
			display,
			details,
		} satisfies CustomMessageEntry<T>);
	}

	/**
	 * 追加标签条目
	 * @param targetId - 目标条目 ID
	 * @param label - 标签文本（undefined 表示删除标签）
	 */
	async appendLabel(targetId: string, label: string | undefined): Promise<string> {
		if (!(await this.storage.getEntry(targetId))) {
			throw new Error(`Entry ${targetId} not found`);
		}
		return this.appendTypedEntry({
			type: "label",
			id: await this.storage.createEntryId(),
			parentId: await this.storage.getLeafId(),
			timestamp: new Date().toISOString(),
			targetId,
			label,
		} satisfies LabelEntry);
	}

	/** 追加会话名称条目 */
	async appendSessionName(name: string): Promise<string> {
		return this.appendTypedEntry({
			type: "session_info",
			id: await this.storage.createEntryId(),
			parentId: await this.storage.getLeafId(),
			timestamp: new Date().toISOString(),
			name: name.trim(),
		} satisfies SessionInfoEntry);
	}

	/**
	 * 移动活跃分支的叶节点
	 *
	 * 用于会话树的导航操作。将叶节点移动到指定条目，可选附加分支摘要。
	 * 这是树导航的核心操作——通过移动叶节点，buildContext 会重建不同的消息历史。
	 *
	 * @param entryId - 新的叶节点 ID（null 表示根）
	 * @param summary - 可选的分支摘要
	 * @returns 摘要条目 ID（如果提供了摘要），否则 undefined
	 */
	async moveTo(
		entryId: string | null,
		summary?: { summary: string; details?: unknown; fromHook?: boolean },
	): Promise<string | undefined> {
		if (entryId !== null && !(await this.storage.getEntry(entryId))) {
			throw new Error(`Entry ${entryId} not found`);
		}
		await this.storage.setLeafId(entryId);
		if (!summary) return undefined;
		return this.appendTypedEntry({
			type: "branch_summary",
			id: await this.storage.createEntryId(),
			parentId: entryId,
			timestamp: new Date().toISOString(),
			fromId: entryId ?? "root",
			summary: summary.summary,
			details: summary.details,
			fromHook: summary.fromHook,
		} satisfies BranchSummaryEntry);
	}
}
