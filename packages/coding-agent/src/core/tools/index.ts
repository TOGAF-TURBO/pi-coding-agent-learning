/**
 * @fileoverview 编码工具（Tool）统一入口
 *
 * 本文件是 coding-agent 所有内置工具的聚合模块，负责：
 * 1. 从各子模块（bash/edit/find/grep/ls/read/write）统一导出类型和工厂函数
 * 2. 提供 `createToolDefinition` / `createTool` 两个按名称创建单个工具的工厂方法
 * 3. 提供批量创建工具定义/实例的便捷函数（编码工具集、只读工具集、全部工具集）
 *
 * 工具分两类：
 * - **编码工具**（Coding Tools）：read/bash/edit/write —— 可读写文件和执行命令
 * - **只读工具**（Read-only Tools）：read/grep/find/ls —— 仅查询，不修改文件系统
 *
 * 此外还导出 truncate（截断）相关工具和 withFileMutationQueue（变更队列）。
 */

export {
	type BashOperations,
	type BashSpawnContext,
	type BashSpawnHook,
	type BashToolDetails,
	type BashToolInput,
	type BashToolOptions,
	createBashTool,
	createBashToolDefinition,
	createLocalBashOperations,
} from "./bash.js";
export {
	createEditTool,
	createEditToolDefinition,
	type EditOperations,
	type EditToolDetails,
	type EditToolInput,
	type EditToolOptions,
} from "./edit.js";
export { withFileMutationQueue } from "./file-mutation-queue.js";
export {
	createFindTool,
	createFindToolDefinition,
	type FindOperations,
	type FindToolDetails,
	type FindToolInput,
	type FindToolOptions,
} from "./find.js";
export {
	createGrepTool,
	createGrepToolDefinition,
	type GrepOperations,
	type GrepToolDetails,
	type GrepToolInput,
	type GrepToolOptions,
} from "./grep.js";
export {
	createLsTool,
	createLsToolDefinition,
	type LsOperations,
	type LsToolDetails,
	type LsToolInput,
	type LsToolOptions,
} from "./ls.js";
export {
	createReadTool,
	createReadToolDefinition,
	type ReadOperations,
	type ReadToolDetails,
	type ReadToolInput,
	type ReadToolOptions,
} from "./read.js";
export {
	DEFAULT_MAX_BYTES,
	DEFAULT_MAX_LINES,
	formatSize,
	type TruncationOptions,
	type TruncationResult,
	truncateHead,
	truncateLine,
	truncateTail,
} from "./truncate.js";
export {
	createWriteTool,
	createWriteToolDefinition,
	type WriteOperations,
	type WriteToolInput,
	type WriteToolOptions,
} from "./write.js";

import type { AgentTool } from "@earendil-works/pi-agent-core";
import type { ToolDefinition } from "../extensions/types.js";
import { type BashToolOptions, createBashTool, createBashToolDefinition } from "./bash.js";
import { createEditTool, createEditToolDefinition, type EditToolOptions } from "./edit.js";
import { createFindTool, createFindToolDefinition, type FindToolOptions } from "./find.js";
import { createGrepTool, createGrepToolDefinition, type GrepToolOptions } from "./grep.js";
import { createLsTool, createLsToolDefinition, type LsToolOptions } from "./ls.js";
import { createReadTool, createReadToolDefinition, type ReadToolOptions } from "./read.js";
import { createWriteTool, createWriteToolDefinition, type WriteToolOptions } from "./write.js";

/** 运行时工具实例 —— 对 AgentTool 的泛型封装 */
export type Tool = AgentTool<any>;

/** 工具的静态定义（描述、参数 schema 等），用于注册到扩展系统 */
export type ToolDef = ToolDefinition<any, any>;

/** 所有内置工具名称的字面量联合类型 */
export type ToolName = "read" | "bash" | "edit" | "write" | "grep" | "find" | "ls";

/** 所有内置工具名称的集合，用于快速判断某个名称是否合法 */
export const allToolNames: Set<ToolName> = new Set(["read", "bash", "edit", "write", "grep", "find", "ls"]);

/**
 * 各工具的配置选项聚合接口。
 * 所有字段均为可选，未提供时使用各工具的默认配置。
 */
export interface ToolsOptions {
	read?: ReadToolOptions;
	bash?: BashToolOptions;
	write?: WriteToolOptions;
	edit?: EditToolOptions;
	grep?: GrepToolOptions;
	find?: FindToolOptions;
	ls?: LsToolOptions;
}

/**
 * 按工具名称创建对应的静态定义。
 *
 * @param toolName - 目标工具名称
 * @param cwd - 工作目录，用于解析相对路径
 * @param options - 各工具的可选配置
 * @returns 该工具的定义对象
 * @throws 工具名称不在 `ToolName` 联合类型中时抛出异常
 */
export function createToolDefinition(toolName: ToolName, cwd: string, options?: ToolsOptions): ToolDef {
	switch (toolName) {
		case "read":
			return createReadToolDefinition(cwd, options?.read);
		case "bash":
			return createBashToolDefinition(cwd, options?.bash);
		case "edit":
			return createEditToolDefinition(cwd, options?.edit);
		case "write":
			return createWriteToolDefinition(cwd, options?.write);
		case "grep":
			return createGrepToolDefinition(cwd, options?.grep);
		case "find":
			return createFindToolDefinition(cwd, options?.find);
		case "ls":
			return createLsToolDefinition(cwd, options?.ls);
		default:
			throw new Error(`Unknown tool name: ${toolName}`);
	}
}

/**
 * 按工具名称创建对应的运行时实例。
 *
 * @param toolName - 目标工具名称
 * @param cwd - 工作目录
 * @param options - 各工具的可选配置
 * @returns 该工具的运行时实例
 * @throws 工具名称不在 `ToolName` 联合类型中时抛出异常
 */
export function createTool(toolName: ToolName, cwd: string, options?: ToolsOptions): Tool {
	switch (toolName) {
		case "read":
			return createReadTool(cwd, options?.read);
		case "bash":
			return createBashTool(cwd, options?.bash);
		case "edit":
			return createEditTool(cwd, options?.edit);
		case "write":
			return createWriteTool(cwd, options?.write);
		case "grep":
			return createGrepTool(cwd, options?.grep);
		case "find":
			return createFindTool(cwd, options?.find);
		case "ls":
			return createLsTool(cwd, options?.ls);
		default:
			throw new Error(`Unknown tool name: ${toolName}`);
	}
}

/**
 * 创建编码工具集的静态定义（read/bash/edit/write）。
 * 这组工具具有读写文件和执行命令的能力。
 */
export function createCodingToolDefinitions(cwd: string, options?: ToolsOptions): ToolDef[] {
	return [
		createReadToolDefinition(cwd, options?.read),
		createBashToolDefinition(cwd, options?.bash),
		createEditToolDefinition(cwd, options?.edit),
		createWriteToolDefinition(cwd, options?.write),
	];
}

/**
 * 创建只读工具集的静态定义（read/grep/find/ls）。
 * 这组工具仅查询文件系统，不会产生任何修改。
 */
export function createReadOnlyToolDefinitions(cwd: string, options?: ToolsOptions): ToolDef[] {
	return [
		createReadToolDefinition(cwd, options?.read),
		createGrepToolDefinition(cwd, options?.grep),
		createFindToolDefinition(cwd, options?.find),
		createLsToolDefinition(cwd, options?.ls),
	];
}

/**
 * 创建全部内置工具的静态定义，以 `Record<ToolName, ToolDef>` 形式返回，
 * 方便按名称索引。
 */
export function createAllToolDefinitions(cwd: string, options?: ToolsOptions): Record<ToolName, ToolDef> {
	return {
		read: createReadToolDefinition(cwd, options?.read),
		bash: createBashToolDefinition(cwd, options?.bash),
		edit: createEditToolDefinition(cwd, options?.edit),
		write: createWriteToolDefinition(cwd, options?.write),
		grep: createGrepToolDefinition(cwd, options?.grep),
		find: createFindToolDefinition(cwd, options?.find),
		ls: createLsToolDefinition(cwd, options?.ls),
	};
}

/**
 * 创建编码工具集的运行时实例（read/bash/edit/write）。
 * @see {@link createCodingToolDefinitions}
 */
export function createCodingTools(cwd: string, options?: ToolsOptions): Tool[] {
	return [
		createReadTool(cwd, options?.read),
		createBashTool(cwd, options?.bash),
		createEditTool(cwd, options?.edit),
		createWriteTool(cwd, options?.write),
	];
}

/**
 * 创建只读工具集的运行时实例（read/grep/find/ls）。
 * @see {@link createReadOnlyToolDefinitions}
 */
export function createReadOnlyTools(cwd: string, options?: ToolsOptions): Tool[] {
	return [
		createReadTool(cwd, options?.read),
		createGrepTool(cwd, options?.grep),
		createFindTool(cwd, options?.find),
		createLsTool(cwd, options?.ls),
	];
}

/**
 * 创建全部内置工具的运行时实例，以 `Record<ToolName, Tool>` 形式返回，
 * 方便按名称索引。
 */
export function createAllTools(cwd: string, options?: ToolsOptions): Record<ToolName, Tool> {
	return {
		read: createReadTool(cwd, options?.read),
		bash: createBashTool(cwd, options?.bash),
		edit: createEditTool(cwd, options?.edit),
		write: createWriteTool(cwd, options?.write),
		grep: createGrepTool(cwd, options?.grep),
		find: createFindTool(cwd, options?.find),
		ls: createLsTool(cwd, options?.ls),
	};
}
