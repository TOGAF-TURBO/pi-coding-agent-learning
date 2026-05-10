/**
 * @fileoverview @earendil-works/pi-agent-core 包的公共 API 导出入口。
 *
 * 导出内容分类：
 *   - 核心 Agent：Agent 类和 Agent Loop 函数
 *   - Harness：Agent Harness（完整的 Agent 运行时封装）
 *   - 上下文压缩：摘要生成、Token 估算、裁剪点查找
 *   - 会话管理：JSONL/内存存储、会话仓库
 *   - 工具函数：消息处理、Shell 输出处理、文本截断
 *   - 提示模板：系统提示和工具提示的模板
 *   - 技能系统：技能文件加载和解析
 *   - 代理工具：HTTP 代理配置
 *   - 类型定义：所有公共类型
 */

// 核心 Agent 类
export * from "./agent.js";
// Agent Loop 函数
export * from "./agent-loop.js";
// Agent Harness 主类
export * from "./harness/agent-harness.js";
// 分支摘要生成
export {
	collectEntriesForBranchSummary,
	generateBranchSummary,
	prepareBranchEntries,
} from "./harness/compaction/branch-summarization.js";
// 上下文压缩
export {
	calculateContextTokens,
	compact,
	DEFAULT_COMPACTION_SETTINGS,
	estimateContextTokens,
	estimateTokens,
	findCutPoint,
	findTurnStartIndex,
	generateSummary,
	getLastAssistantUsage,
	prepareCompaction,
	serializeConversation,
	shouldCompact,
} from "./harness/compaction/compaction.js";
// 执行环境
export * from "./harness/execution-env.js";
// Harness 工厂
export * from "./harness/factory.js";
// 消息处理工具
export * from "./harness/messages.js";
// 提示模板
export * from "./harness/prompt-templates.js";
// 会话仓库（JSONL 持久化）
export * from "./harness/session/repo/jsonl.js";
// 会话仓库（内存存储）
export * from "./harness/session/repo/memory.js";
// 会话仓库（共享工具）
export * from "./harness/session/repo/shared.js";
// 会话管理
export * from "./harness/session/session.js";
// 技能加载
export * from "./harness/skills.js";
// 系统提示组装
export * from "./harness/system-prompt.js";
// Harness 类型定义
export * from "./harness/types.js";
// Shell 输出处理
export * from "./harness/utils/shell-output.js";
// 文本截断工具
export * from "./harness/utils/truncate.js";
// HTTP 代理工具
export * from "./proxy.js";
// 类型定义
export * from "./types.js";
