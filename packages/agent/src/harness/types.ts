/**
 * @fileoverview Agent Harness 的类型定义 — 编码代理会话的完整类型体系。
 *
 * Agent Harness 是 coding-agent 包的核心运行时，在 Agent 类之上封装了
 * 会话管理、文件系统操作、工具执行、压缩、分支等完整功能。本文件定义了
 * Harness 所需的全部类型接口。
 *
 * 类型分为以下几类：
 *   - 资源类型：Skill（技能）、PromptTemplate（提示模板）
 *   - 执行环境：ExecutionEnv（文件系统和进程操作的抽象接口）
 *   - 会话存储：SessionTreeEntry（会话树条目）、SessionStorage（存储后端接口）
 *   - 运行时状态：AgentHarnessContext、AgentHarnessConversationState、AgentHarnessOperationState
 *   - 事件系统：AgentHarnessOwnEvent 及各事件的结果类型
 *   - 压缩与分支：CompactionSettings、CompactionPreparation、TreePreparation
 *   - 配置选项：AgentHarnessOptions
 */

import type { ImageContent, Model, TextContent } from "@earendil-works/pi-ai";
import type { AgentEvent, AgentMessage, AgentTool, ThinkingLevel } from "../index.js";
import type { Session } from "./session/session.js";

// ============================================================================
// 资源类型
// ============================================================================

/**
 * 技能（Skill）— 按需加载的能力描述
 *
 * 技能从 SKILL.md 文件加载，也可由应用程序直接提供。
 * 技能内容会在适当时机注入到系统提示中，指导模型执行特定任务。
 */
export interface Skill {
	/** 技能命令名（如 "gitnexus-cli"） */
	name: string;
	/** 简短描述，告诉模型何时使用此技能 */
	description: string;
	/** 完整的技能指令内容 */
	content: string;
	/** 技能文件的绝对路径，用于模型可见的位置提示和解析相对引用 */
	filePath: string;
	/** 设为 true 时，技能不出现在模型可见的列表中，但仍可通过显式命令调用 */
	disableModelInvocation?: boolean;
}

/**
 * 提示模板（Prompt Template）— 可展开的斜杠命令
 *
 * 用户输入 `/templatename args` 时，模板内容中的占位符被替换后作为实际提示发送。
 */
export interface PromptTemplate {
	/** 斜杠命令名（不含前导 `/`） */
	name: string;
	/** 可选的描述，用于命令列表和自动补全 */
	description?: string;
	/** 模板内容，参数占位符由 expandPromptTemplate 展开 */
	content: string;
}

/** Harness 可用的资源（提示模板和技能） */
export interface AgentHarnessResources {
	/** 提示模板列表 */
	promptTemplates?: PromptTemplate[];
	/** 技能列表 */
	skills?: Skill[];
}

// ============================================================================
// 执行环境类型
// ============================================================================

/** 文件系统对象的类型 */
export type FileKind = "file" | "directory" | "symlink";

/**
 * 文件操作错误码
 *
 * 与具体后端无关的稳定错误分类，所有文件操作异常都使用这些代码。
 */
export type FileErrorCode =
	| "not_found"
	| "permission_denied"
	| "not_directory"
	| "is_directory"
	| "invalid"
	| "not_supported"
	| "unknown";

/**
 * 文件操作错误
 *
 * 封装了后端无关的错误码、消息和关联路径，替代直接抛出底层异常。
 */
export class FileError extends Error {
	constructor(
		/** 后端无关的错误码 */
		public code: FileErrorCode,
		message: string,
		/** 关联的失败路径（绝对路径） */
		public path?: string,
		options?: ErrorOptions,
	) {
		super(message, options);
		this.name = "FileError";
	}
}

/** 文件系统对象的元数据 */
export interface FileInfo {
	/** 文件名（不含路径） */
	name: string;
	/** 绝对路径（不跟踪符号链接） */
	path: string;
	/** 对象类型 */
	kind: FileKind;
	/** 字节大小 */
	size: number;
	/** 修改时间（毫秒 Unix 时间戳） */
	mtimeMs: number;
}

/** 进程执行选项 */
export interface ExecutionEnvExecOptions {
	/** 工作目录（相对于 ExecutionEnv.cwd） */
	cwd?: string;
	/** 额外的环境变量（覆盖默认值） */
	env?: Record<string, string>;
	/** 超时（秒） */
	timeout?: number;
	/** 中止信号 */
	signal?: AbortSignal;
	/** stdout 增量回调 */
	onStdout?: (chunk: string) => void;
	/** stderr 增量回调 */
	onStderr?: (chunk: string) => void;
}

/**
 * 执行环境（ExecutionEnv）— 文件系统和进程操作的抽象接口
 *
 * 提供统一的文件读写、目录操作和命令执行能力。
 * 路径参数可以是绝对路径或相对于 cwd 的相对路径。
 * 所有文件操作失败时抛出 FileError 而非底层异常。
 *
 * 默认实现：NodeExecutionEnv（基于 Node.js 的 fs 和 child_process）
 */
export interface ExecutionEnv {
	/** 当前工作目录 */
	cwd: string;

	/** 在 cwd 下执行 shell 命令 */
	exec(
		command: string,
		options?: ExecutionEnvExecOptions,
	): Promise<{ stdout: string; stderr: string; exitCode: number }>;

	/** 读取 UTF-8 文本文件 */
	readTextFile(path: string): Promise<string>;
	/** 读取二进制文件 */
	readBinaryFile(path: string): Promise<Uint8Array>;
	/** 创建或覆写文件（自动创建父目录） */
	writeFile(path: string, content: string | Uint8Array): Promise<void>;
	/** 获取路径的元数据（不跟踪符号链接） */
	fileInfo(path: string): Promise<FileInfo>;
	/** 列出目录的直接子项 */
	listDir(path: string): Promise<FileInfo[]>;
	/** 获取规范路径（跟踪符号链接） */
	realPath(path: string): Promise<string>;
	/** 检查路径是否存在（仅对缺失返回 false，其他错误仍抛异常） */
	exists(path: string): Promise<boolean>;
	/** 创建目录 */
	createDir(path: string, options?: { recursive?: boolean }): Promise<void>;
	/** 删除文件或目录 */
	remove(path: string, options?: { recursive?: boolean; force?: boolean }): Promise<void>;
	/** 创建临时目录并返回绝对路径 */
	createTempDir(prefix?: string): Promise<string>;
	/** 创建临时文件并返回绝对路径 */
	createTempFile(options?: { prefix?: string; suffix?: string }): Promise<string>;

	/** 释放环境持有的资源 */
	cleanup(): Promise<void>;
}

// ============================================================================
// 会话树条目类型
// ============================================================================

/** 会话树条目的基础字段 */
export interface SessionTreeEntryBase {
	type: string;
	/** 唯一 ID */
	id: string;
	/** 父条目 ID（根条目为 null） */
	parentId: string | null;
	/** ISO 格式时间戳 */
	timestamp: string;
}

/** 消息条目 — 存储一条 Agent 消息 */
export interface MessageEntry extends SessionTreeEntryBase {
	type: "message";
	message: AgentMessage;
}

/** 思考级别变更条目 */
export interface ThinkingLevelChangeEntry extends SessionTreeEntryBase {
	type: "thinking_level_change";
	thinkingLevel: string;
}

/** 模型变更条目 */
export interface ModelChangeEntry extends SessionTreeEntryBase {
	type: "model_change";
	provider: string;
	modelId: string;
}

/** 压缩条目 — 记录一次上下文压缩操作 */
export interface CompactionEntry<T = unknown> extends SessionTreeEntryBase {
	type: "compaction";
	/** 压缩生成的摘要文本 */
	summary: string;
	/** 压缩后保留的第一条条目 ID */
	firstKeptEntryId: string;
	/** 压缩前的 Token 数 */
	tokensBefore: number;
	/** 扩展详情（如自定义的压缩指标） */
	details?: T;
	/** 是否由扩展钩子触发 */
	fromHook?: boolean;
}

/** 分支摘要条目 — 记录跳转到会话树其他分支时的摘要 */
export interface BranchSummaryEntry<T = unknown> extends SessionTreeEntryBase {
	type: "branch_summary";
	/** 跳转源的条目 ID */
	fromId: string;
	/** 摘要文本 */
	summary: string;
	details?: T;
	fromHook?: boolean;
}

/** 自定义条目 — 扩展可存储任意结构化数据 */
export interface CustomEntry<T = unknown> extends SessionTreeEntryBase {
	type: "custom";
	customType: string;
	data?: T;
}

/** 自定义消息条目 — 扩展可注入自定义显示消息 */
export interface CustomMessageEntry<T = unknown> extends SessionTreeEntryBase {
	type: "custom_message";
	customType: string;
	content: string | (TextContent | ImageContent)[];
	details?: T;
	/** 是否在 UI 中显示 */
	display: boolean;
}

/** 标签条目 — 为会话树中的条目添加书签标记 */
export interface LabelEntry extends SessionTreeEntryBase {
	type: "label";
	/** 目标条目 ID */
	targetId: string;
	/** 标签文本（undefined 表示删除标签） */
	label: string | undefined;
}

/** 会话信息条目 — 存储会话名称等元信息 */
export interface SessionInfoEntry extends SessionTreeEntryBase {
	type: "session_info"; // 历史名称，保持向后兼容
	name?: string;
}

/** 会话树条目的联合类型 */
export type SessionTreeEntry =
	| MessageEntry
	| ThinkingLevelChangeEntry
	| ModelChangeEntry
	| CompactionEntry
	| BranchSummaryEntry
	| CustomEntry
	| CustomMessageEntry
	| LabelEntry
	| SessionInfoEntry;

/** 从会话条目重建的上下文快照 */
export interface SessionContext {
	messages: AgentMessage[];
	thinkingLevel: string;
	model: { provider: string; modelId: string } | null;
}

// ============================================================================
// 会话存储接口
// ============================================================================

/** 会话基础元数据 */
export interface SessionMetadata {
	id: string;
	createdAt: string;
}

/** JSONL 存储后端的扩展元数据 */
export interface JsonlSessionMetadata extends SessionMetadata {
	cwd: string;
	path: string;
	/** 父会话路径（分叉时会设置） */
	parentSessionPath?: string;
}

/**
 * 会话存储后端接口
 *
 * 定义了会话数据的读写操作。条目以树结构组织（通过 parentId），
 * 支持追加、查询、路径遍历等操作。
 * 实现类：MemorySessionStorage（内存）、JsonlSessionStorage（JSONL 文件）
 */
export interface SessionStorage<TMetadata extends SessionMetadata = SessionMetadata> {
	getMetadata(): Promise<TMetadata>;
	/** 获取当前活跃分支的叶节点 ID */
	getLeafId(): Promise<string | null>;
	/** 设置活跃分支的叶节点 ID（用于树导航） */
	setLeafId(leafId: string | null): Promise<void>;
	/** 生成唯一的条目 ID */
	createEntryId(): Promise<string>;
	/** 追加条目到存储 */
	appendEntry(entry: SessionTreeEntry): Promise<void>;
	/** 按 ID 获取单个条目 */
	getEntry(id: string): Promise<SessionTreeEntry | undefined>;
	/** 按类型查找所有条目 */
	findEntries<TType extends SessionTreeEntry["type"]>(
		type: TType,
	): Promise<Array<Extract<SessionTreeEntry, { type: TType }>>>;
	/** 获取条目的标签 */
	getLabel(id: string): Promise<string | undefined>;
	/** 获取从叶节点到根的路径上所有条目 */
	getPathToRoot(leafId: string | null): Promise<SessionTreeEntry[]>;
	/** 获取所有条目 */
	getEntries(): Promise<SessionTreeEntry[]>;
}

export type { Session } from "./session/session.js";

/** 会话创建选项 */
export interface SessionCreateOptions {
	id?: string;
}

/** 会话分叉选项 */
export interface SessionForkOptions {
	/** 分叉点条目 ID */
	entryId?: string;
	/** 分叉位置：before（条目之前）或 at（条目处） */
	position?: "before" | "at";
	id?: string;
}

/**
 * 会话仓库接口 — 管理会话的创建、打开、列表和删除
 *
 * 抽象了底层存储（内存、JSONL 文件等），提供统一的会话生命周期管理。
 */
export interface SessionRepo<
	TMetadata extends SessionMetadata = SessionMetadata,
	TCreateOptions extends SessionCreateOptions = SessionCreateOptions,
	TListOptions = void,
> {
	/** 创建新会话 */
	create(options: TCreateOptions): Promise<Session<TMetadata>>;
	/** 打开已有会话 */
	open(metadata: TMetadata): Promise<Session<TMetadata>>;
	/** 列出所有会话 */
	list(options?: TListOptions): Promise<TMetadata[]>;
	/** 删除会话 */
	delete(metadata: TMetadata): Promise<void>;
	/** 从已有会话分叉出新会话 */
	fork(source: TMetadata, options: SessionForkOptions & TCreateOptions): Promise<Session<TMetadata>>;
}

/** JSONL 存储后端的创建选项 */
export interface JsonlSessionCreateOptions extends SessionCreateOptions {
	cwd: string;
	parentSessionPath?: string;
}

/** JSONL 存储后端的列表过滤选项 */
export interface JsonlSessionListOptions {
	cwd?: string;
}

/** JSONL 会话仓库的完整接口 */
export interface JsonlSessionRepoApi
	extends SessionRepo<JsonlSessionMetadata, JsonlSessionCreateOptions, JsonlSessionListOptions> {}

// ============================================================================
// 运行时状态类型
// ============================================================================

/** 待写入的变更（在 Agent 循环结束后批量应用） */
export interface AgentHarnessPendingMutations {
	/** 待追加的消息 */
	appendMessages: AgentMessage[];
	/** 模型变更 */
	model?: Model<any>;
	/** 思考级别变更 */
	thinkingLevel?: ThinkingLevel;
	/** 活跃工具名列表变更 */
	activeToolNames?: string[];
}

/** 对话状态 — 当前会话的快照 */
export interface AgentHarnessConversationState {
	/** 当前会话实例 */
	session: Session;
	/** 当前使用的模型 */
	model: Model<any>;
	/** 当前思考级别 */
	thinkingLevel: ThinkingLevel;
	/** 当前激活的工具名列表 */
	activeToolNames: string[];
	/** 下一轮待处理的消息队列 */
	nextTurnQueue: AgentMessage[];
}

/** 操作状态 — 当前运行时操作的管理状态 */
export interface AgentHarnessOperationState {
	/** 是否空闲（无正在进行的操作） */
	idle: boolean;
	/** 当前活跃操作的 ID */
	liveOperationId?: string;
	/** 是否已请求中止 */
	abortRequested: boolean;
	/** 转向消息队列 */
	steerQueue: AgentMessage[];
	/** 后续消息队列 */
	followUpQueue: AgentMessage[];
	/** 待写入的变更 */
	pendingMutations: AgentHarnessPendingMutations;
}

/** 存档点快照 — 用于保存和恢复 Agent 状态 */
export interface SavePointSnapshot {
	/** 消息历史 */
	messages: AgentMessage[];
	/** 当前模型 */
	model: Model<any> | undefined;
	/** 当前思考级别 */
	thinkingLevel: ThinkingLevel;
	/** 活跃工具名 */
	activeToolNames: string[];
	/** 系统提示 */
	systemPrompt: string;
}

/**
 * Harness 上下文 — 传递给工具和钩子的完整运行时上下文
 *
 * 包含执行环境、对话状态、操作状态和中止信号，
 * 是工具执行和扩展钩子的主要参数。
 */
export interface AgentHarnessContext {
	/** 文件系统和进程执行环境 */
	env: ExecutionEnv;
	/** 对话状态 */
	conversation: AgentHarnessConversationState;
	/** 操作状态 */
	operation: AgentHarnessOperationState;
	/** 中止信号 */
	abortSignal?: AbortSignal;
}

// ============================================================================
// Harness 事件类型
// ============================================================================

/** 队列更新事件 — 转向/后续/下一轮消息队列变化时发出 */
export interface QueueUpdateEvent {
	type: "queue_update";
	steer: AgentMessage[];
	followUp: AgentMessage[];
	nextTurn: AgentMessage[];
}

/** 存档点事件 — Agent 循环的安全保存点 */
export interface SavePointEvent {
	type: "save_point";
	liveOperationId: string;
	hadPendingMutations: boolean;
}

/** 中止事件 — 操作被中止时发出 */
export interface AbortEvent {
	type: "abort";
	/** 被清除的转向消息 */
	clearedSteer: AgentMessage[];
	/** 被清除的后续消息 */
	clearedFollowUp: AgentMessage[];
}

/** 结束事件 — 操作完成后发出 */
export interface SettledEvent {
	type: "settled";
	nextTurnCount: number;
}

/** Agent 启动前事件 — 允许扩展修改提示和系统提示 */
export interface BeforeAgentStartEvent {
	type: "before_agent_start";
	prompt: string;
	images?: ImageContent[];
	systemPrompt: string;
	resources: AgentHarnessResources;
}

/** 上下文事件 — 发送给 LLM 的消息列表（转换前） */
export interface ContextEvent {
	type: "context";
	messages: AgentMessage[];
}

/** 提供商请求前事件 — 允许修改发送给 LLM 的载荷 */
export interface BeforeProviderRequestEvent {
	type: "before_provider_request";
	payload: unknown;
}

/** 提供商响应后事件 — 提供 HTTP 状态码和响应头 */
export interface AfterProviderResponseEvent {
	type: "after_provider_response";
	status: number;
	headers: Record<string, string>;
}

/** 工具调用事件 — 工具即将执行时发出 */
export interface ToolCallEvent {
	type: "tool_call";
	toolCallId: string;
	toolName: string;
	input: Record<string, unknown>;
}

/** 工具结果事件 — 工具执行完成后发出 */
export interface ToolResultEvent {
	type: "tool_result";
	toolCallId: string;
	toolName: string;
	input: Record<string, unknown>;
	content: Array<TextContent | ImageContent>;
	details: unknown;
	isError: boolean;
}

/** 压缩前事件 — 允许扩展取消或自定义压缩 */
export interface SessionBeforeCompactEvent {
	type: "session_before_compact";
	preparation: CompactionPreparation;
	branchEntries: SessionTreeEntry[];
	customInstructions?: string;
	signal: AbortSignal;
}

/** 压缩完成事件 */
export interface SessionCompactEvent {
	type: "session_compact";
	compactionEntry: CompactionEntry;
	fromHook: boolean;
}

/** 树导航前事件 — 允许扩展取消或自定义分支摘要 */
export interface SessionBeforeTreeEvent {
	type: "session_before_tree";
	preparation: TreePreparation;
	signal: AbortSignal;
}

/** 树导航完成事件 */
export interface SessionTreeEvent {
	type: "session_tree";
	newLeafId: string | null;
	oldLeafId: string | null;
	summaryEntry?: BranchSummaryEntry;
	fromHook?: boolean;
}

/** 模型选择事件 — 模型切换时发出 */
export interface ModelSelectEvent {
	type: "model_select";
	model: Model<any>;
	previousModel: Model<any> | undefined;
	/** 来源：set（直接设置）或 restore（从存档点恢复） */
	source: "set" | "restore";
}

/** 思考级别选择事件 */
export interface ThinkingLevelSelectEvent {
	type: "thinking_level_select";
	level: ThinkingLevel;
	previousLevel: ThinkingLevel;
}

/** Harness 自身发出的事件联合类型（不包含 Agent 层的底层事件） */
export type AgentHarnessOwnEvent =
	| QueueUpdateEvent
	| SavePointEvent
	| AbortEvent
	| SettledEvent
	| BeforeAgentStartEvent
	| ContextEvent
	| BeforeProviderRequestEvent
	| AfterProviderResponseEvent
	| ToolCallEvent
	| ToolResultEvent
	| SessionBeforeCompactEvent
	| SessionCompactEvent
	| SessionBeforeTreeEvent
	| SessionTreeEvent
	| ModelSelectEvent
	| ThinkingLevelSelectEvent;

/** Harness 完整事件 = Agent 底层事件 + Harness 自身事件 */
export type AgentHarnessEvent = AgentEvent | AgentHarnessOwnEvent;

// ============================================================================
// 事件结果类型
// ============================================================================

/** before_agent_start 事件的处理结果 */
export interface BeforeAgentStartResult {
	/** 替换发送给 LLM 的消息列表 */
	messages?: AgentMessage[];
	/** 替换系统提示 */
	systemPrompt?: string;
}

/** context 事件的处理结果 */
export interface ContextResult {
	messages: AgentMessage[];
}

/** before_provider_request 事件的处理结果 */
export interface BeforeProviderRequestResult {
	payload: unknown;
}

/** tool_call 事件的处理结果（可阻止工具执行） */
export interface ToolCallResult {
	/** 设为 true 阻止工具执行 */
	block?: boolean;
	/** 阻止的原因（显示在错误结果中） */
	reason?: string;
}

/** tool_result 事件的处理结果（可覆盖工具结果的部分字段） */
export interface ToolResultPatch {
	/** 替换内容 */
	content?: Array<TextContent | ImageContent>;
	/** 替换详情 */
	details?: unknown;
	/** 替换错误标志 */
	isError?: boolean;
	/** 提示 Agent 在当前批次后停止 */
	terminate?: boolean;
}

/** session_before_compact 事件的处理结果 */
export interface SessionBeforeCompactResult {
	/** 取消压缩 */
	cancel?: boolean;
	/** 使用自定义压缩结果替代内置压缩 */
	compaction?: CompactResult;
}

/** session_before_tree 事件的处理结果 */
export interface SessionBeforeTreeResult {
	/** 取消树导航 */
	cancel?: boolean;
	/** 自定义摘要 */
	summary?: { summary: string; details?: unknown };
	/** 自定义压缩指令 */
	customInstructions?: string;
	/** 是否替换（而非追加）压缩指令 */
	replaceInstructions?: boolean;
	/** 为新分支设置标签 */
	label?: string;
}

/**
 * 事件类型到处理结果类型的映射
 *
 * 每种事件的监听器可返回对应的结果类型。undefined 表示不需要修改。
 * 事件系统按监听器注册顺序依次调用，后者的结果覆盖前者。
 */
export type AgentHarnessEventResultMap = {
	before_agent_start: BeforeAgentStartResult | undefined;
	context: ContextResult | undefined;
	before_provider_request: BeforeProviderRequestResult | undefined;
	after_provider_response: undefined;
	tool_call: ToolCallResult | undefined;
	tool_result: ToolResultPatch | undefined;
	session_before_compact: SessionBeforeCompactResult | undefined;
	session_compact: undefined;
	session_before_tree: SessionBeforeTreeResult | undefined;
	session_tree: undefined;
	model_select: undefined;
	thinking_level_select: undefined;
	queue_update: undefined;
	save_point: undefined;
	abort: undefined;
	settled: undefined;
};

// ============================================================================
// 其他配置类型
// ============================================================================

/** prompt 方法的选项 */
export interface AgentHarnessPromptOptions {
	images?: ImageContent[];
}

/** 中止操作的结果 */
export interface AbortResult {
	/** 被清除的转向消息 */
	clearedSteer: AgentMessage[];
	/** 被清除的后续消息 */
	clearedFollowUp: AgentMessage[];
}

/** 压缩操作的结果 */
export interface CompactResult {
	/** 摘要文本 */
	summary: string;
	/** 保留的第一条条目 ID */
	firstKeptEntryId: string;
	/** 压缩前的 Token 数 */
	tokensBefore: number;
	details?: unknown;
}

/** 树导航的结果 */
export interface NavigateTreeResult {
	/** 是否被取消 */
	cancelled: boolean;
	/** 编辑器中的文本（用于重新发送） */
	editorText?: string;
	/** 生成的分支摘要条目 */
	summaryEntry?: BranchSummaryEntry;
}

/** 压缩设置 */
export interface CompactionSettings {
	/** 是否启用自动压缩 */
	enabled: boolean;
	/** 保留的 Token 余量（上下文窗口减去此值后为可用空间） */
	reserveTokens: number;
	/** 始终保留的最近消息 Token 数（不参与压缩） */
	keepRecentTokens: number;
}

/** 压缩前的准备数据 */
export interface CompactionPreparation {
	/** 保留的第一条条目 ID */
	firstKeptEntryId: string;
	/** 需要摘要的消息列表 */
	messagesToSummarize: AgentMessage[];
	/** 轮次前缀消息（压缩后保留在摘要前） */
	turnPrefixMessages: AgentMessage[];
	/** 是否为拆分轮次（工具调用在压缩点之后） */
	isSplitTurn: boolean;
	/** 压缩前的 Token 数 */
	tokensBefore: number;
	/** 之前的摘要文本 */
	previousSummary?: string;
	/** 文件操作追踪 */
	fileOps: FileOperations;
	/** 压缩设置 */
	settings: CompactionSettings;
}

/** 文件操作追踪记录（用于压缩时识别文件相关的对话上下文） */
export interface FileOperations {
	read: Set<string>;
	written: Set<string>;
	edited: Set<string>;
}

/** 树导航的准备数据 */
export interface TreePreparation {
	/** 目标条目 ID */
	targetId: string;
	/** 当前叶节点 ID */
	oldLeafId: string | null;
	/** 公共祖先条目 ID */
	commonAncestorId: string | null;
	/** 需要摘要的条目列表 */
	entriesToSummarize: SessionTreeEntry[];
	/** 用户是否请求生成摘要 */
	userWantsSummary: boolean;
	customInstructions?: string;
	replaceInstructions?: boolean;
	label?: string;
}

/** 生成分支摘要的选项 */
export interface GenerateBranchSummaryOptions {
	model: Model<any>;
	apiKey: string;
	headers?: Record<string, string>;
	signal: AbortSignal;
	customInstructions?: string;
	replaceInstructions?: boolean;
	reserveTokens?: number;
}

/** 分支摘要的结果 */
export interface BranchSummaryResult {
	summary?: string;
	readFiles?: string[];
	modifiedFiles?: string[];
	aborted?: boolean;
	error?: string;
}

/**
 * Agent Harness 的配置选项
 *
 * 创建 Harness 实例时传入，包含执行环境、会话、工具、资源、
 * 系统提示、认证、模型等所有必需的配置。
 */
export interface AgentHarnessOptions {
	/** 文件系统和进程执行环境 */
	env: ExecutionEnv;
	/** 会话实例 */
	session: Session;
	/** 可用工具列表 */
	tools?: AgentTool[];
	/** 资源（提示模板和技能），支持静态值或动态函数 */
	resources?:
		| AgentHarnessResources
		| ((context: AgentHarnessContext) => AgentHarnessResources | Promise<AgentHarnessResources>);
	/** 系统提示，支持静态字符串或动态生成函数 */
	systemPrompt?:
		| string
		| ((context: {
				env: ExecutionEnv;
				session: Session;
				model: Model<any>;
				thinkingLevel: ThinkingLevel;
				activeTools: AgentTool[];
				resources: AgentHarnessResources;
		  }) => string | Promise<string>);
	/** 认证请求回调（当需要 API Key 时调用） */
	requestAuth?: (model: Model<any>) => Promise<{ apiKey: string; headers?: Record<string, string> } | undefined>;
	/** 初始模型 */
	model: Model<any>;
	/** 初始思考级别 */
	thinkingLevel?: ThinkingLevel;
	/** 初始活跃工具名列表 */
	activeToolNames?: string[];
}

export type { AgentHarness } from "./agent-harness.js";
