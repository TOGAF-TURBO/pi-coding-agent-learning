/**
 * @fileoverview Coding Agent CLI 主入口
 *
 * 本文件是 coding agent 命令行工具的主入口，负责将 CLI 参数解析结果转化为
 * 会话（Session）运行时配置，并启动对应的运行模式。
 *
 * 核心流程：
 * 1. 解析 CLI 参数（parseArgs）并确定应用模式（interactive / print / json / rpc）
 * 2. 解析或创建会话管理器（SessionManager），处理 --session / --continue / --fork / --resume 等参数
 * 3. 创建 Agent 会话运行时（createAgentSessionRuntime），包括模型注册、资源加载、认证存储等
 * 4. 根据应用模式分发到对应的运行器（InteractiveMode / runPrintMode / runRpcMode）
 *
 * 关键设计：
 * - 会话的 cwd 可能在会话选择后才确定（如 --resume 从其他项目选择会话），
 *   因此项目级设置（settings）、模型注册、资源加载等延迟到运行时工厂中执行
 * - 支持通过管道（pipe）传入 stdin 内容作为初始消息
 * - 支持离线模式（--offline），跳过版本检查等网络操作
 */

import { resolve } from "node:path";
import { createInterface } from "node:readline";
import { type ImageContent, modelsAreEqual } from "@earendil-works/pi-ai";
import { ProcessTerminal, setKeybindings, TUI } from "@earendil-works/pi-tui";
import chalk from "chalk";
import { type Args, type Mode, parseArgs, printHelp } from "./cli/args.js";
import { processFileArguments } from "./cli/file-processor.js";
import { buildInitialMessage } from "./cli/initial-message.js";
import { listModels } from "./cli/list-models.js";
import { selectSession } from "./cli/session-picker.js";
import { ENV_SESSION_DIR, expandTildePath, getAgentDir, VERSION } from "./config.js";
import { type CreateAgentSessionRuntimeFactory, createAgentSessionRuntime } from "./core/agent-session-runtime.js";
import {
	type AgentSessionRuntimeDiagnostic,
	createAgentSessionFromServices,
	createAgentSessionServices,
} from "./core/agent-session-services.js";
import { formatNoModelsAvailableMessage } from "./core/auth-guidance.js";
import { AuthStorage } from "./core/auth-storage.js";
import { exportFromFile } from "./core/export-html/index.js";
import type { ExtensionFactory } from "./core/extensions/types.js";
import { KeybindingsManager } from "./core/keybindings.js";
import type { ModelRegistry } from "./core/model-registry.js";
import { resolveCliModel, resolveModelScope, type ScopedModel } from "./core/model-resolver.js";
import { restoreStdout, takeOverStdout } from "./core/output-guard.js";
import type { CreateAgentSessionOptions } from "./core/sdk.js";
import {
	formatMissingSessionCwdPrompt,
	getMissingSessionCwdIssue,
	MissingSessionCwdError,
	type SessionCwdIssue,
} from "./core/session-cwd.js";
import { SessionManager } from "./core/session-manager.js";
import { SettingsManager } from "./core/settings-manager.js";
import { printTimings, resetTimings, time } from "./core/timings.js";
import { runMigrations, showDeprecationWarnings } from "./migrations.js";
import { InteractiveMode, runPrintMode, runRpcMode } from "./modes/index.js";
import { ExtensionSelectorComponent } from "./modes/interactive/components/extension-selector.js";
import { initTheme, stopThemeWatcher } from "./modes/interactive/theme/theme.js";
import { handleConfigCommand, handlePackageCommand } from "./package-manager-cli.js";
import { isLocalPath } from "./utils/paths.js";

/**
 * 从管道 stdin 读取全部内容。
 *
 * 当用户通过管道传入数据时（如 `echo "prompt" | pi`），将 stdin 内容作为初始消息使用。
 * 如果 stdin 是 TTY（即交互式终端），返回 undefined。
 *
 * @returns 管道输入的文本内容，若无管道输入则返回 undefined
 */
async function readPipedStdin(): Promise<string | undefined> {
	// 交互式终端不读取 stdin，避免阻塞用户输入
	if (process.stdin.isTTY) {
		return undefined;
	}

	return new Promise((resolve) => {
		let data = "";
		process.stdin.setEncoding("utf8");
		process.stdin.on("data", (chunk) => {
			data += chunk;
		});
		process.stdin.on("end", () => {
			resolve(data.trim() || undefined);
		});
		process.stdin.resume();
	});
}

/**
 * 收集设置管理器中的错误信息，转换为诊断信息格式。
 * 用于在启动和运行时创建阶段报告配置问题。
 *
 * @param settingsManager - 设置管理器实例
 * @param context - 上下文描述（如 "startup session lookup"），用于诊断信息定位
 * @returns 诊断信息数组
 */
function collectSettingsDiagnostics(
	settingsManager: SettingsManager,
	context: string,
): AgentSessionRuntimeDiagnostic[] {
	return settingsManager.drainErrors().map(({ scope, error }) => ({
		type: "warning",
		message: `(${context}, ${scope} settings) ${error.message}`,
	}));
}

/**
 * 将诊断信息输出到 stderr，按类型（error / warning / info）着色显示。
 *
 * @param diagnostics - 诊断信息数组
 */
function reportDiagnostics(diagnostics: readonly AgentSessionRuntimeDiagnostic[]): void {
	for (const diagnostic of diagnostics) {
		const color = diagnostic.type === "error" ? chalk.red : diagnostic.type === "warning" ? chalk.yellow : chalk.dim;
		const prefix = diagnostic.type === "error" ? "Error: " : diagnostic.type === "warning" ? "Warning: " : "";
		console.error(color(`${prefix}${diagnostic.message}`));
	}
}

/**
 * 判断环境变量值是否为"真值"（"1"、"true"、"yes"，不区分大小写）。
 * 用于解析 PI_OFFLINE 等布尔型环境变量标志。
 *
 * @param value - 环境变量值
 * @returns 是否为真值
 */
function isTruthyEnvFlag(value: string | undefined): boolean {
	if (!value) return false;
	return value === "1" || value.toLowerCase() === "true" || value.toLowerCase() === "yes";
}

/** 应用运行模式：交互式 TUI、纯文本打印、JSON 输出、JSON-RPC 服务 */
type AppMode = "interactive" | "print" | "json" | "rpc";

/**
 * 根据 CLI 参数和 stdin 类型确定应用的运行模式。
 *
 * 优先级：rpc > json > print（由 --print 标志或 stdin 管道触发）> interactive
 *
 * @param parsed - 解析后的 CLI 参数
 * @param stdinIsTTY - stdin 是否为 TTY
 * @returns 应用运行模式
 */
function resolveAppMode(parsed: Args, stdinIsTTY: boolean): AppMode {
	if (parsed.mode === "rpc") {
		return "rpc";
	}
	if (parsed.mode === "json") {
		return "json";
	}
	if (parsed.print || !stdinIsTTY) {
		return "print";
	}
	return "interactive";
}

/**
 * 将应用运行模式映射为打印输出模式（排除 rpc）。
 *
 * @param appMode - 应用运行模式
 * @returns 输出模式：json 或 text
 */
function toPrintOutputMode(appMode: AppMode): Exclude<Mode, "rpc"> {
	return appMode === "json" ? "json" : "text";
}

/**
 * 准备发送给 Agent 的初始消息。
 *
 * 根据是否提供了文件参数（@file）分两条路径：
 * - 无文件参数：直接从 CLI 消息和 stdin 内容构建
 * - 有文件参数：先处理文件内容（提取文本和图像），再合并 CLI 消息
 *
 * @param parsed - 解析后的 CLI 参数
 * @param autoResizeImages - 是否自动缩放图像（由设置决定）
 * @param stdinContent - 可选的管道 stdin 内容
 * @returns 初始消息文本和图像内容
 */
async function prepareInitialMessage(
	parsed: Args,
	autoResizeImages: boolean,
	stdinContent?: string,
): Promise<{
	initialMessage?: string;
	initialImages?: ImageContent[];
}> {
	if (parsed.fileArgs.length === 0) {
		return buildInitialMessage({ parsed, stdinContent });
	}

	const { text, images } = await processFileArguments(parsed.fileArgs, { autoResizeImages });
	return buildInitialMessage({
		parsed,
		fileText: text,
		fileImages: images,
		stdinContent,
	});
}

/**
 * 会话参数解析结果。
 *
 * - path: 用户直接提供了文件路径
 * - local: 在当前项目中找到匹配的会话
 * - global: 在其他项目中找到匹配的会话（需确认是否 fork）
 * - not_found: 未在任何位置找到匹配的会话
 */
type ResolvedSession =
	| { type: "path"; path: string } // 直接文件路径
	| { type: "local"; path: string } // 在当前项目中找到
	| { type: "global"; path: string; cwd: string } // 在其他项目中找到
	| { type: "not_found"; arg: string }; // 未找到

/**
 * 将会话参数解析为文件路径。
 *
 * 解析策略：
 * 1. 如果参数看起来像文件路径（含 / 或 \ 或以 .jsonl 结尾），直接使用
 * 2. 否则视为会话 ID 前缀，在当前项目中搜索匹配项
 * 3. 若当前项目无匹配，跨所有项目搜索
 *
 * @param sessionArg - 用户传入的会话参数（路径或 ID 前缀）
 * @param cwd - 当前工作目录
 * @param sessionDir - 可选的会话存储目录
 * @returns 解析结果
 */
async function resolveSessionPath(sessionArg: string, cwd: string, sessionDir?: string): Promise<ResolvedSession> {
	// 参数包含路径分隔符或 .jsonl 后缀，视为直接路径
	if (sessionArg.includes("/") || sessionArg.includes("\\") || sessionArg.endsWith(".jsonl")) {
		return { type: "path", path: sessionArg };
	}

	// 优先在当前项目目录中搜索会话 ID 前缀
	const localSessions = await SessionManager.list(cwd, sessionDir);
	const localMatches = localSessions.filter((s) => s.id.startsWith(sessionArg));

	if (localMatches.length >= 1) {
		return { type: "local", path: localMatches[0].path };
	}

	// 当前项目无匹配，跨所有项目搜索
	const allSessions = await SessionManager.listAll();
	const globalMatches = allSessions.filter((s) => s.id.startsWith(sessionArg));

	if (globalMatches.length >= 1) {
		const match = globalMatches[0];
		return { type: "global", path: match.path, cwd: match.cwd };
	}

	return { type: "not_found", arg: sessionArg };
}

/**
 * 在终端中提示用户进行 y/n 确认。
 * 用于会话从其他项目 fork 等需要用户确认的场景。
 *
 * @param message - 提示消息
 * @returns 用户是否确认（y/yes）
 */
async function promptConfirm(message: string): Promise<boolean> {
	return new Promise((resolve) => {
		const rl = createInterface({
			input: process.stdin,
			output: process.stdout,
		});
		rl.question(`${message} [y/N] `, (answer) => {
			rl.close();
			resolve(answer.toLowerCase() === "y" || answer.toLowerCase() === "yes");
		});
	});
}

/**
 * 校验 --fork 参数与其他会话参数的互斥性。
 * --fork 不能与 --session、--continue、--resume、--no-session 同时使用。
 *
 * @param parsed - 解析后的 CLI 参数
 */
function validateForkFlags(parsed: Args): void {
	if (!parsed.fork) return;

	const conflictingFlags = [
		parsed.session ? "--session" : undefined,
		parsed.continue ? "--continue" : undefined,
		parsed.resume ? "--resume" : undefined,
		parsed.noSession ? "--no-session" : undefined,
	].filter((flag): flag is string => flag !== undefined);

	if (conflictingFlags.length > 0) {
		console.error(chalk.red(`Error: --fork cannot be combined with ${conflictingFlags.join(", ")}`));
		process.exit(1);
	}
}

/**
 * 从源会话文件 fork 一个新会话。失败时打印错误信息并退出进程。
 *
 * @param sourcePath - 源会话文件路径
 * @param cwd - 当前工作目录
 * @param sessionDir - 可选的会话存储目录
 * @returns 新创建的 SessionManager 实例
 */
function forkSessionOrExit(sourcePath: string, cwd: string, sessionDir?: string): SessionManager {
	try {
		return SessionManager.forkFrom(sourcePath, cwd, sessionDir);
	} catch (error: unknown) {
		const message = error instanceof Error ? error.message : String(error);
		console.error(chalk.red(`Error: ${message}`));
		process.exit(1);
	}
}

/**
 * 根据 CLI 参数创建或恢复会话管理器。
 *
 * 会话来源优先级：--no-session > --fork > --session > --resume > --continue > 新建
 * - --no-session：创建纯内存会话，不持久化
 * - --fork：从指定会话派生新会话
 * - --session：打开指定会话（若来自其他项目则提示 fork）
 * - --resume：启动交互式会话选择器
 * - --continue：恢复最近一次会话
 * - 默认：创建全新会话
 *
 * @param parsed - 解析后的 CLI 参数
 * @param cwd - 当前工作目录
 * @param sessionDir - 可选的会话存储目录
 * @param settingsManager - 设置管理器，用于 --resume 模式的主题初始化
 * @returns 会话管理器实例
 */
async function createSessionManager(
	parsed: Args,
	cwd: string,
	sessionDir: string | undefined,
	settingsManager: SettingsManager,
): Promise<SessionManager> {
	if (parsed.noSession) {
		return SessionManager.inMemory();
	}

	if (parsed.fork) {
		const resolved = await resolveSessionPath(parsed.fork, cwd, sessionDir);

		switch (resolved.type) {
			case "path":
			case "local":
			case "global":
				return forkSessionOrExit(resolved.path, cwd, sessionDir);

			case "not_found":
				console.error(chalk.red(`No session found matching '${resolved.arg}'`));
				process.exit(1);
		}
	}

	if (parsed.session) {
		const resolved = await resolveSessionPath(parsed.session, cwd, sessionDir);

		switch (resolved.type) {
			case "path":
			case "local":
				return SessionManager.open(resolved.path, sessionDir);

			case "global": {
				// 会话属于其他项目，提示用户是否 fork 到当前目录
				console.log(chalk.yellow(`Session found in different project: ${resolved.cwd}`));
				const shouldFork = await promptConfirm("Fork this session into current directory?");
				if (!shouldFork) {
					console.log(chalk.dim("Aborted."));
					process.exit(0);
				}
				return forkSessionOrExit(resolved.path, cwd, sessionDir);
			}

			case "not_found":
				console.error(chalk.red(`No session found matching '${resolved.arg}'`));
				process.exit(1);
		}
	}

	if (parsed.resume) {
		// --resume 模式需要 TUI 来显示会话选择器
		initTheme(settingsManager.getTheme(), true);
		try {
			const selectedPath = await selectSession(
				(onProgress) => SessionManager.list(cwd, sessionDir, onProgress),
				SessionManager.listAll,
			);
			if (!selectedPath) {
				console.log(chalk.dim("No session selected"));
				process.exit(0);
			}
			return SessionManager.open(selectedPath, sessionDir);
		} finally {
			stopThemeWatcher();
		}
	}

	if (parsed.continue) {
		return SessionManager.continueRecent(cwd, sessionDir);
	}

	return SessionManager.create(cwd, sessionDir);
}

/**
 * 根据 CLI 参数构建 Agent 会话选项。
 *
 * 处理以下配置：
 * - 模型选择（--model、--provider、--models）
 * - 思考级别（--thinking，或从模型模式中提取）
 * - 作用域模型列表（用于 Ctrl+P 模型切换）
 * - 工具控制（--no-tools、--no-builtin-tools、--tools）
 *
 * 模型解析优先级：CLI 指定 > 保存的默认值 > 作用域模型中的第一个
 * 思考级别优先级：--thinking 参数 > 模型模式中的 thinking 后缀 > 作用域模型配置
 *
 * @param parsed - 解析后的 CLI 参数
 * @param scopedModels - 已解析的作用域模型列表
 * @param hasExistingSession - 是否在恢复已有会话（恢复时跳过默认模型选择）
 * @param modelRegistry - 模型注册表
 * @param settingsManager - 设置管理器
 * @returns 会话选项、是否从模型模式中推断思考级别、诊断信息
 */
function buildSessionOptions(
	parsed: Args,
	scopedModels: ScopedModel[],
	hasExistingSession: boolean,
	modelRegistry: ModelRegistry,
	settingsManager: SettingsManager,
): {
	options: CreateAgentSessionOptions;
	cliThinkingFromModel: boolean;
	diagnostics: AgentSessionRuntimeDiagnostic[];
} {
	const options: CreateAgentSessionOptions = {};
	const diagnostics: AgentSessionRuntimeDiagnostic[] = [];
	let cliThinkingFromModel = false;

	// 从 CLI 参数解析模型
	// 支持 --provider <name> --model <pattern> 和 --model <provider>/<pattern> 两种格式
	if (parsed.model) {
		const resolved = resolveCliModel({
			cliProvider: parsed.provider,
			cliModel: parsed.model,
			modelRegistry,
		});
		if (resolved.warning) {
			diagnostics.push({ type: "warning", message: resolved.warning });
		}
		if (resolved.error) {
			diagnostics.push({ type: "error", message: resolved.error });
		}
		if (resolved.model) {
			options.model = resolved.model;
			// 支持 --model <pattern>:<thinking> 简写形式。
			// 显式的 --thinking 参数优先级更高（在后面覆盖）。
			if (!parsed.thinking && resolved.thinkingLevel) {
				options.thinkingLevel = resolved.thinkingLevel;
				cliThinkingFromModel = true;
			}
		}
	}

	if (!options.model && scopedModels.length > 0 && !hasExistingSession) {
		// 未通过 CLI 指定模型时，优先使用保存的默认模型（若在作用域内），
		// 否则使用作用域模型列表中的第一个
		const savedProvider = settingsManager.getDefaultProvider();
		const savedModelId = settingsManager.getDefaultModel();
		const savedModel = savedProvider && savedModelId ? modelRegistry.find(savedProvider, savedModelId) : undefined;
		const savedInScope = savedModel ? scopedModels.find((sm) => modelsAreEqual(sm.model, savedModel)) : undefined;

		if (savedInScope) {
			options.model = savedInScope.model;
			if (!parsed.thinking && savedInScope.thinkingLevel) {
				options.thinkingLevel = savedInScope.thinkingLevel;
			}
		} else {
			options.model = scopedModels[0].model;
			if (!parsed.thinking && scopedModels[0].thinkingLevel) {
				options.thinkingLevel = scopedModels[0].thinkingLevel;
			}
		}
	}

	// --thinking 参数的优先级最高，覆盖上面从模型配置推断的值
	if (parsed.thinking) {
		options.thinkingLevel = parsed.thinking;
	}

	// 设置 Ctrl+P 模型切换的作用域列表。
	// thinkingLevel 为 undefined 时表示"继承当前会话的思考级别"。
	if (scopedModels.length > 0) {
		options.scopedModels = scopedModels.map((sm) => ({
			model: sm.model,
			thinkingLevel: sm.thinkingLevel,
		}));
	}

	// --api-key 参数在调用方处理（需要在 authStorage 中设置）

	// 工具控制
	if (parsed.noTools) {
		options.noTools = "all";
	} else if (parsed.noBuiltinTools) {
		options.noTools = "builtin";
	}
	if (parsed.tools) {
		options.tools = [...parsed.tools];
	}

	return { options, cliThinkingFromModel, diagnostics };
}

/**
 * 将相对路径列表转换为绝对路径。
 * 仅对被识别为本地路径的值进行转换，其他值（如 URL）保持不变。
 *
 * @param cwd - 当前工作目录，用于拼接相对路径
 * @param paths - 路径列表
 * @returns 绝对路径列表，若输入为空则返回 undefined
 */
function resolveCliPaths(cwd: string, paths: string[] | undefined): string[] | undefined {
	return paths?.map((value) => (isLocalPath(value) ? resolve(cwd, value) : value));
}

/**
 * 当会话的工作目录不存在时，在交互模式下提示用户选择处理方式。
 * 使用 TUI 渲染选择器组件，提供"继续"和"取消"两个选项。
 *
 * @param issue - 会话工作目录问题信息
 * @param settingsManager - 设置管理器，用于获取主题和硬件光标配置
 * @returns 用户选择的回退工作目录，取消时返回 undefined
 */
async function promptForMissingSessionCwd(
	issue: SessionCwdIssue,
	settingsManager: SettingsManager,
): Promise<string | undefined> {
	initTheme(settingsManager.getTheme());
	setKeybindings(KeybindingsManager.create());

	return new Promise((resolve) => {
		const ui = new TUI(new ProcessTerminal(), settingsManager.getShowHardwareCursor());
		ui.setClearOnShrink(settingsManager.getClearOnShrink());

		// 防止重复调用 finish（组件的 onSelect 和 onEscape 都可能触发）
		let settled = false;
		const finish = (result: string | undefined) => {
			if (settled) {
				return;
			}
			settled = true;
			ui.stop();
			resolve(result);
		};

		const selector = new ExtensionSelectorComponent(
			formatMissingSessionCwdPrompt(issue),
			["Continue", "Cancel"],
			(option) => finish(option === "Continue" ? issue.fallbackCwd : undefined),
			() => finish(undefined),
			{ tui: ui },
		);
		ui.addChild(selector);
		ui.setFocus(selector);
		ui.start();
	});
}

/**
 * 主函数的可选配置。
 * 允许外部调用者注入自定义的扩展工厂。
 */
export interface MainOptions {
	extensionFactories?: ExtensionFactory[];
}

/**
 * CLI 主入口函数。
 *
 * 完整的启动流程：
 * 1. 处理包管理器命令（如安装扩展）和配置命令
 * 2. 解析 CLI 参数并确定运行模式
 * 3. 运行数据迁移和废弃警告
 * 4. 创建或恢复会话管理器
 * 5. 创建 Agent 会话运行时（延迟初始化，确保 cwd 正确）
 * 6. 分发到对应的运行模式（交互式 / 打印 / RPC）
 *
 * @param args - 命令行参数数组（不含 node 和脚本路径）
 * @param options - 可选配置，如自定义扩展工厂
 */
export async function main(args: string[], options?: MainOptions) {
	resetTimings();
	const offlineMode = args.includes("--offline") || isTruthyEnvFlag(process.env.PI_OFFLINE);
	if (offlineMode) {
		process.env.PI_OFFLINE = "1";
		process.env.PI_SKIP_VERSION_CHECK = "1";
	}

	// 优先处理包管理器子命令（如 pi install <package>）
	if (await handlePackageCommand(args)) {
		return;
	}

	// 处理配置子命令（如 pi config set <key> <value>）
	if (await handleConfigCommand(args)) {
		return;
	}

	const parsed = parseArgs(args);
	if (parsed.diagnostics.length > 0) {
		for (const d of parsed.diagnostics) {
			const color = d.type === "error" ? chalk.red : chalk.yellow;
			console.error(color(`${d.type === "error" ? "Error" : "Warning"}: ${d.message}`));
		}
		if (parsed.diagnostics.some((d) => d.type === "error")) {
			process.exit(1);
		}
	}
	time("parseArgs");
	let appMode = resolveAppMode(parsed, process.stdin.isTTY);
	// 非交互模式下接管 stdout，防止 TUI 输出干扰管道
	const shouldTakeOverStdout = appMode !== "interactive";
	if (shouldTakeOverStdout) {
		takeOverStdout();
	}

	if (parsed.version) {
		console.log(VERSION);
		process.exit(0);
	}

	// --export 模式：将会话导出为 HTML 等格式
	if (parsed.export) {
		let result: string;
		try {
			const outputPath = parsed.messages.length > 0 ? parsed.messages[0] : undefined;
			result = await exportFromFile(parsed.export, outputPath);
		} catch (error: unknown) {
			const message = error instanceof Error ? error.message : "Failed to export session";
			console.error(chalk.red(`Error: ${message}`));
			process.exit(1);
		}
		console.log(`Exported to: ${result}`);
		process.exit(0);
	}

	if (parsed.mode === "rpc" && parsed.fileArgs.length > 0) {
		console.error(chalk.red("Error: @file arguments are not supported in RPC mode"));
		process.exit(1);
	}

	validateForkFlags(parsed);

	// 执行数据迁移（传入 cwd 用于项目级迁移）
	const { migratedAuthProviders: migratedProviders, deprecationWarnings } = runMigrations(process.cwd());
	time("runMigrations");

	// 创建启动阶段的设置管理器，仅用于会话目录查找和会话选择器。
	// 项目级设置在运行时工厂中根据最终 cwd 重新创建。
	const cwd = process.cwd();
	const agentDir = getAgentDir();
	const startupSettingsManager = SettingsManager.create(cwd, agentDir);
	reportDiagnostics(collectSettingsDiagnostics(startupSettingsManager, "startup session lookup"));

	// 在创建 cwd 绑定的运行时服务之前，确定最终的工作目录。
	// --session 和 --resume 可能选择来自其他项目的会话，因此项目级设置、
	// 资源加载和模型注册必须在目标会话的 cwd 确定后才执行。
	const envSessionDir = process.env[ENV_SESSION_DIR];
	const sessionDir =
		parsed.sessionDir ??
		(envSessionDir ? expandTildePath(envSessionDir) : undefined) ??
		startupSettingsManager.getSessionDir();
	let sessionManager = await createSessionManager(parsed, cwd, sessionDir, startupSettingsManager);

	// 检查会话的工作目录是否仍存在（可能已被删除或移动）
	const missingSessionCwdIssue = getMissingSessionCwdIssue(sessionManager, cwd);
	if (missingSessionCwdIssue) {
		if (appMode === "interactive") {
			const selectedCwd = await promptForMissingSessionCwd(missingSessionCwdIssue, startupSettingsManager);
			if (!selectedCwd) {
				process.exit(0);
			}
			sessionManager = SessionManager.open(missingSessionCwdIssue.sessionFile!, sessionDir, selectedCwd);
		} else {
			// 非交互模式无法提示用户，直接报错退出
			console.error(chalk.red(new MissingSessionCwdError(missingSessionCwdIssue).message));
			process.exit(1);
		}
	}
	time("createSessionManager");

	// 解析 CLI 中的路径参数为绝对路径
	const resolvedExtensionPaths = resolveCliPaths(cwd, parsed.extensions);
	const resolvedSkillPaths = resolveCliPaths(cwd, parsed.skills);
	const resolvedPromptTemplatePaths = resolveCliPaths(cwd, parsed.promptTemplates);
	const resolvedThemePaths = resolveCliPaths(cwd, parsed.themes);
	const authStorage = AuthStorage.create();

	// 运行时工厂：延迟创建会话运行时。
	// 这确保了所有 cwd 绑定的服务（设置、模型注册、资源加载）在最终 cwd 确定后才创建。
	const createRuntime: CreateAgentSessionRuntimeFactory = async ({
		cwd,
		agentDir,
		sessionManager,
		sessionStartEvent,
	}) => {
		const services = await createAgentSessionServices({
			cwd,
			agentDir,
			authStorage,
			extensionFlagValues: parsed.unknownFlags,
			resourceLoaderOptions: {
				additionalExtensionPaths: resolvedExtensionPaths,
				additionalSkillPaths: resolvedSkillPaths,
				additionalPromptTemplatePaths: resolvedPromptTemplatePaths,
				additionalThemePaths: resolvedThemePaths,
				noExtensions: parsed.noExtensions,
				noSkills: parsed.noSkills,
				noPromptTemplates: parsed.noPromptTemplates,
				noThemes: parsed.noThemes,
				noContextFiles: parsed.noContextFiles,
				systemPrompt: parsed.systemPrompt,
				appendSystemPrompt: parsed.appendSystemPrompt,
				extensionFactories: options?.extensionFactories,
			},
		});
		const { settingsManager, modelRegistry, resourceLoader } = services;
		const diagnostics: AgentSessionRuntimeDiagnostic[] = [
			...services.diagnostics,
			...collectSettingsDiagnostics(settingsManager, "runtime creation"),
			...resourceLoader.getExtensions().errors.map(({ path, error }) => ({
				type: "error" as const,
				message: `Failed to load extension "${path}": ${error}`,
			})),
		];

		// 解析作用域模型列表（用于 Ctrl+P 切换和默认模型选择）
		const modelPatterns = parsed.models ?? settingsManager.getEnabledModels();
		const scopedModels =
			modelPatterns && modelPatterns.length > 0 ? await resolveModelScope(modelPatterns, modelRegistry) : [];
		const {
			options: sessionOptions,
			cliThinkingFromModel,
			diagnostics: sessionOptionDiagnostics,
		} = buildSessionOptions(
			parsed,
			scopedModels,
			sessionManager.buildSessionContext().messages.length > 0,
			modelRegistry,
			settingsManager,
		);
		diagnostics.push(...sessionOptionDiagnostics);

		// 处理 --api-key 参数：必须同时指定模型
		if (parsed.apiKey) {
			if (!sessionOptions.model) {
				diagnostics.push({
					type: "error",
					message: "--api-key requires a model to be specified via --model, --provider/--model, or --models",
				});
			} else {
				authStorage.setRuntimeApiKey(sessionOptions.model.provider, parsed.apiKey);
			}
		}

		const created = await createAgentSessionFromServices({
			services,
			sessionManager,
			sessionStartEvent,
			model: sessionOptions.model,
			thinkingLevel: sessionOptions.thinkingLevel,
			scopedModels: sessionOptions.scopedModels,
			tools: sessionOptions.tools,
			noTools: sessionOptions.noTools,
			customTools: sessionOptions.customTools,
		});

		// CLI 显式指定了思考级别时，需要将会话的思考级别同步到模型配置
		const cliThinkingOverride = parsed.thinking !== undefined || cliThinkingFromModel;
		if (created.session.model && cliThinkingOverride) {
			created.session.setThinkingLevel(created.session.thinkingLevel);
		}

		return {
			...created,
			services,
			diagnostics,
		};
	};
	time("createRuntime");

	// 创建会话运行时（内部调用上面的 createRuntime 工厂）
	const runtime = await createAgentSessionRuntime(createRuntime, {
		cwd: sessionManager.getCwd(),
		agentDir,
		sessionManager,
	});
	const { services, session, modelFallbackMessage } = runtime;
	const { settingsManager, modelRegistry, resourceLoader } = services;

	// 帮助信息需要在资源加载完成后打印（包含扩展自定义标志）
	if (parsed.help) {
		const extensionFlags = resourceLoader
			.getExtensions()
			.extensions.flatMap((extension) => Array.from(extension.flags.values()));
		printHelp(extensionFlags);
		process.exit(0);
	}

	if (parsed.listModels !== undefined) {
		const searchPattern = typeof parsed.listModels === "string" ? parsed.listModels : undefined;
		await listModels(modelRegistry, searchPattern);
		process.exit(0);
	}

	// 读取管道 stdin 内容（RPC 模式跳过，因为 stdin 用于 JSON-RPC 通信）
	let stdinContent: string | undefined;
	if (appMode !== "rpc") {
		stdinContent = await readPipedStdin();
		// 交互模式下如果存在管道输入，降级为打印模式
		if (stdinContent !== undefined && appMode === "interactive") {
			appMode = "print";
		}
	}
	time("readPipedStdin");

	const { initialMessage, initialImages } = await prepareInitialMessage(
		parsed,
		settingsManager.getImageAutoResize(),
		stdinContent,
	);
	time("prepareInitialMessage");
	initTheme(settingsManager.getTheme(), appMode === "interactive");
	time("initTheme");

	// 交互模式下显示废弃警告
	if (appMode === "interactive" && deprecationWarnings.length > 0) {
		await showDeprecationWarnings(deprecationWarnings);
	}

	const scopedModels = [...session.scopedModels];
	time("resolveModelScope");
	reportDiagnostics(runtime.diagnostics);
	if (runtime.diagnostics.some((diagnostic) => diagnostic.type === "error")) {
		process.exit(1);
	}
	time("createAgentSession");

	// 非交互模式下必须有可用模型
	if (appMode !== "interactive" && !session.model) {
		console.error(chalk.red(formatNoModelsAvailableMessage()));
		process.exit(1);
	}

	const startupBenchmark = isTruthyEnvFlag(process.env.PI_STARTUP_BENCHMARK);
	if (startupBenchmark && appMode !== "interactive") {
		console.error(chalk.red("Error: PI_STARTUP_BENCHMARK only supports interactive mode"));
		process.exit(1);
	}

	// 根据运行模式分发
	if (appMode === "rpc") {
		printTimings();
		await runRpcMode(runtime);
	} else if (appMode === "interactive") {
		// 显示作用域模型信息（可通过 Ctrl+P 切换）
		if (scopedModels.length > 0 && (parsed.verbose || !settingsManager.getQuietStartup())) {
			const modelList = scopedModels
				.map((sm) => {
					const thinkingStr = sm.thinkingLevel ? `:${sm.thinkingLevel}` : "";
					return `${sm.model.id}${thinkingStr}`;
				})
				.join(", ");
			console.log(chalk.dim(`Model scope: ${modelList} ${chalk.gray("(Ctrl+P to cycle)")}`));
		}

		const interactiveMode = new InteractiveMode(runtime, {
			migratedProviders,
			modelFallbackMessage,
			initialMessage,
			initialImages,
			initialMessages: parsed.messages,
			verbose: parsed.verbose,
		});

		// 启动性能基准测试模式：初始化 TUI 后立即退出，用于测量启动时间
		if (startupBenchmark) {
			await interactiveMode.init();
			time("interactiveMode.init");
			printTimings();
			interactiveMode.stop();
			stopThemeWatcher();
			// 等待输出缓冲区刷新完毕再退出
			if (process.stdout.writableLength > 0) {
				await new Promise<void>((resolve) => process.stdout.once("drain", resolve));
			}
			if (process.stderr.writableLength > 0) {
				await new Promise<void>((resolve) => process.stderr.once("drain", resolve));
			}
			return;
		}

		printTimings();
		await interactiveMode.run();
	} else {
		// 打印模式（text 或 json）
		printTimings();
		const exitCode = await runPrintMode(runtime, {
			mode: toPrintOutputMode(appMode),
			messages: parsed.messages,
			initialMessage,
			initialImages,
		});
		stopThemeWatcher();
		restoreStdout();
		if (exitCode !== 0) {
			process.exitCode = exitCode;
		}
		return;
	}
}
