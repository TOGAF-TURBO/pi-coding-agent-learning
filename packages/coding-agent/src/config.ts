/**
 * @fileoverview 编码代理的配置与路径解析模块
 *
 * 本文件负责三项核心职责：
 * 1. **运行环境检测** — 判断当前是 Bun 编译二进制、npm/pnpm/yarn 全局安装还是其他方式，
 *    为自动更新功能提供安装方法信息。
 * 2. **包资源路径解析** — 根据运行环境（Bun 二进制 vs Node.js dist/ vs tsx src/）定位
 *    主题、模板、资源文件等打包资产的路径。
 * 3. **用户配置路径** — 解析代理配置目录（如 ~/.pi/agent/）下的认证、模型、会话、设置等
 *    文件路径。
 *
 * 关键导出：
 * - `detectInstallMethod()` — 检测安装方式
 * - `getSelfUpdateCommand()` — 获取自动更新命令
 * - `getPackageDir()` — 获取包根目录
 * - `getAgentDir()` — 获取用户配置目录
 * - `VERSION` / `APP_NAME` / `PACKAGE_NAME` — 应用元信息常量
 */

import { spawnSync } from "child_process";
import { accessSync, constants, existsSync, readFileSync, realpathSync } from "fs";
import { homedir } from "os";
import { basename, dirname, join, resolve, sep, win32 } from "path";
import { fileURLToPath } from "url";
import { shouldUseWindowsShell } from "./utils/child-process.js";

// =============================================================================
// 运行环境检测
// =============================================================================

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

/**
 * 检测当前是否以 Bun 编译二进制形式运行。
 * Bun 编译后的二进制文件中 `import.meta.url` 会包含 Bun 虚拟文件系统路径标识：
 * `"$bunfs"`、`"~BUN"` 或 `"%7EBUN"`
 */
export const isBunBinary =
	import.meta.url.includes("$bunfs") || import.meta.url.includes("~BUN") || import.meta.url.includes("%7EBUN");

/** 检测当前运行时是否为 Bun（包括编译二进制和 `bun run` 两种场景） */
export const isBunRuntime = !!process.versions.bun;

// =============================================================================
// 安装方式检测
// =============================================================================

/** 安装方式类型，表示包是通过哪种包管理器安装的 */
export type InstallMethod = "bun-binary" | "npm" | "pnpm" | "yarn" | "bun" | "unknown";

/** 自更新命令中的单个步骤 */
interface SelfUpdateCommandStep {
	command: string;
	args: string[];
	display: string;
}

/**
 * 自更新命令。
 * 当包名发生变更时（如从旧包名迁移到新包名），需要先卸载旧包再安装新包，
 * 此时 `steps` 包含多个步骤；否则 `steps` 为空，命令直接执行即可。
 */
export interface SelfUpdateCommand extends SelfUpdateCommandStep {
	steps?: SelfUpdateCommandStep[];
}

/**
 * 将安装步骤与可选的卸载步骤组合为完整的自更新命令。
 * 当卸载步骤存在时，`display` 会拼接两个步骤以便向用户展示完整操作流程。
 *
 * @param installStep - 安装新版本的命令步骤
 * @param uninstallStep - 卸载旧版本的命令步骤（包名变更时需要）
 */
function makeSelfUpdateCommand(
	installStep: SelfUpdateCommandStep,
	uninstallStep?: SelfUpdateCommandStep,
): SelfUpdateCommand {
	if (!uninstallStep) return installStep;
	return {
		...installStep,
		display: `${uninstallStep.display} && ${installStep.display}`,
		steps: [uninstallStep, installStep],
	};
}

/**
 * 构造单个命令步骤对象，自动处理参数中含空格时的引号转义。
 *
 * @param command - 要执行的命令
 * @param args - 命令参数列表
 */
function makeSelfUpdateCommandStep(command: string, args: string[]): SelfUpdateCommandStep {
	return {
		command,
		args,
		display: [command, ...args].map((arg) => (/\s/.test(arg) ? `"${arg}"` : arg)).join(" "),
	};
}

/**
 * 检测当前安装方式。
 * 通过分析模块路径和可执行文件路径中是否包含包管理器特征字符串来判断。
 * Bun 编译二进制优先级最高（直接返回），其余按 pnpm → yarn → bun → npm 的顺序匹配。
 */
export function detectInstallMethod(): InstallMethod {
	if (isBunBinary) {
		return "bun-binary";
	}

	// 将模块目录与可执行文件路径拼接后统一转小写、统一路径分隔符，便于特征匹配
	const resolvedPath = `${__dirname}\0${process.execPath || ""}`.toLowerCase().replace(/\\/g, "/");

	if (resolvedPath.includes("/pnpm/") || resolvedPath.includes("/.pnpm/")) {
		return "pnpm";
	}
	if (resolvedPath.includes("/yarn/") || resolvedPath.includes("/.yarn/")) {
		return "yarn";
	}
	if (isBunRuntime || resolvedPath.includes("/install/global/node_modules/")) {
		return "bun";
	}
	if (resolvedPath.includes("/npm/") || resolvedPath.includes("/node_modules/")) {
		return "npm";
	}

	return "unknown";
}

/**
 * 推断 npm 全局安装的根目录和前缀路径。
 * 通过分析包目录在 node_modules 层级中的位置来确定：
 * - 作用域包（如 @scope/pkg）的根目录比普通包多一层
 * - Linux/macOS 上 npm 全局安装通常位于 `<prefix>/lib/node_modules/`
 * - Windows 全局安装的路径形态与本地项目安装无法区分，因此不做推断以避免误判
 */
function getInferredNpmInstall(): { root: string; prefix: string } | undefined {
	const packageDir = getPackageDir();
	const path = process.platform === "win32" || packageDir.includes("\\") ? win32 : { basename, dirname };
	const parent = path.dirname(packageDir);
	let root: string | undefined;
	// 作用域包：node_modules/@scope/package → root 为 node_modules
	if (path.basename(parent).startsWith("@") && path.basename(path.dirname(parent)) === "node_modules") {
		root = path.dirname(parent);
	} else if (path.basename(parent) === "node_modules") {
		// 普通包：node_modules/package → root 为 node_modules
		root = parent;
	}
	if (!root) return undefined;
	const rootParent = path.dirname(root);
	// 标准 Unix npm 全局布局：<prefix>/lib/node_modules/
	if (path.basename(rootParent) === "lib") return { root, prefix: path.dirname(rootParent) };
	// Windows 全局 npm 前缀使用 `<prefix>\node_modules`，仅凭路径形态
	// 无法与本地项目安装区分。除非有 `npm root -g` 的证据，否则不推断
	// 不受支持的自定义 Windows 前缀。
	return undefined;
}

/**
 * 根据安装方式构造对应的自更新命令。
 * 当更新包名与已安装包名不同时（包名迁移场景），会先生成卸载旧包的步骤。
 *
 * @param method - 安装方式
 * @param installedPackageName - 当前已安装的包名
 * @param updatePackageName - 要更新到的包名（默认与已安装包名相同）
 * @param npmCommand - 自定义 npm 命令（用于通过环境变量覆盖默认 npm 命令的场景）
 */
function getSelfUpdateCommandForMethod(
	method: InstallMethod,
	installedPackageName: string,
	updatePackageName = installedPackageName,
	npmCommand?: string[],
): SelfUpdateCommand | undefined {
	switch (method) {
		case "bun-binary":
			// Bun 编译二进制不支持包管理器自动更新
			return undefined;
		case "pnpm":
			return makeSelfUpdateCommand(
				makeSelfUpdateCommandStep("pnpm", ["install", "-g", updatePackageName]),
				updatePackageName === installedPackageName
					? undefined
					: makeSelfUpdateCommandStep("pnpm", ["remove", "-g", installedPackageName]),
			);
		case "yarn":
			return makeSelfUpdateCommand(
				makeSelfUpdateCommandStep("yarn", ["global", "add", updatePackageName]),
				updatePackageName === installedPackageName
					? undefined
					: makeSelfUpdateCommandStep("yarn", ["global", "remove", installedPackageName]),
			);
		case "bun":
			return makeSelfUpdateCommand(
				makeSelfUpdateCommandStep("bun", ["install", "-g", updatePackageName]),
				updatePackageName === installedPackageName
					? undefined
					: makeSelfUpdateCommandStep("bun", ["uninstall", "-g", installedPackageName]),
			);
		case "npm": {
			const [command = "npm", ...npmArgs] = npmCommand ?? [];
			const inferred = npmCommand?.length ? undefined : getInferredNpmInstall();
			// 如果推断出了自定义前缀路径，通过 --prefix 参数传递给 npm
			const prefixArgs = [...npmArgs, ...(inferred ? ["--prefix", inferred.prefix] : [])];
			const installStep = makeSelfUpdateCommandStep(command, [...prefixArgs, "install", "-g", updatePackageName]);
			const uninstallStep =
				updatePackageName === installedPackageName
					? undefined
					: makeSelfUpdateCommandStep(command, [...prefixArgs, "uninstall", "-g", installedPackageName]);
			return makeSelfUpdateCommand(installStep, uninstallStep);
		}
		case "unknown":
			return undefined;
	}
}

/**
 * 执行外部命令并返回其标准输出。
 * 用于查询包管理器的全局根目录等信息。
 *
 * @param command - 要执行的命令
 * @param args - 命令参数
 * @param options.requireSuccess - 为 true 时，命令执行失败会抛出异常而非静默返回 undefined
 */
function readCommandOutput(
	command: string,
	args: string[],
	options: { requireSuccess?: boolean } = {},
): string | undefined {
	const result = spawnSync(command, args, {
		encoding: "utf-8",
		stdio: ["ignore", "pipe", "pipe"],
		shell: shouldUseWindowsShell(command),
	});
	if (result.status === 0) return result.stdout.trim() || undefined;
	if (options.requireSuccess) {
		const reason = result.error?.message || result.stderr.trim() || `exit code ${result.status ?? "unknown"}`;
		throw new Error(`Failed to run ${[command, ...args].join(" ")}: ${reason}`);
	}
	return undefined;
}

/**
 * 获取指定包管理器的全局包安装根目录列表。
 * 对于 npm，特殊处理了通过 `npmCommand` 传入 `bun` 命令的情况
 * （某些用户可能配置 `bun` 作为 npm 的替代命令）。
 * 返回多个候选路径以提高匹配成功率。
 *
 * @param method - 安装方式
 * @param _packageName - 包名（当前未使用，保留供未来扩展）
 * @param npmCommand - 自定义 npm 命令
 */
function getGlobalPackageRoots(method: InstallMethod, _packageName: string, npmCommand?: string[]): string[] {
	switch (method) {
		case "npm": {
			const configured = !!npmCommand?.length;
			const [command = "npm", ...npmArgs] = npmCommand ?? [];
			// 当用户配置了 bun 作为 npm 命令时，需要查询 bun 的全局安装路径
			if (configured && command === "bun") {
				const bunBin = readCommandOutput(command, [...npmArgs, "pm", "bin", "-g"], {
					requireSuccess: true,
				});
				const roots = [join(homedir(), ".bun", "install", "global", "node_modules")];
				if (bunBin) {
					roots.push(join(dirname(bunBin), "install", "global", "node_modules"));
				}
				return roots;
			}
			const root = readCommandOutput(command, [...npmArgs, "root", "-g"], {
				requireSuccess: configured,
			});
			const inferred = configured ? undefined : getInferredNpmInstall();
			return [root, inferred?.root].filter((x): x is string => !!x);
		}
		case "pnpm": {
			const root = readCommandOutput("pnpm", ["root", "-g"]);
			// pnpm 的全局根目录和其父目录都可能包含全局包
			return root ? [root, dirname(root)] : [];
		}
		case "yarn": {
			const dir = readCommandOutput("yarn", ["global", "dir"]);
			return dir ? [dir, join(dir, "node_modules")] : [];
		}
		case "bun": {
			const bunBin = readCommandOutput("bun", ["pm", "bin", "-g"]);
			const roots = [join(homedir(), ".bun", "install", "global", "node_modules")];
			if (bunBin) {
				roots.push(join(dirname(bunBin), "install", "global", "node_modules"));
			}
			return roots;
		}
		case "bun-binary":
		case "unknown":
			return [];
	}
}

/**
 * 将路径规范化为可用于比较的形式。
 * 解析符号链接（realpath）并将 Windows 路径转为小写，以实现跨平台路径比较。
 * 如果路径不存在或解析失败则返回 undefined。
 */
function normalizeExistingPathForComparison(path: string): string | undefined {
	const resolvedPath = resolve(path);
	if (!existsSync(resolvedPath)) {
		return undefined;
	}
	let normalizedPath: string;
	try {
		normalizedPath = realpathSync(resolvedPath);
	} catch {
		return undefined;
	}
	if (process.platform === "win32") {
		normalizedPath = normalizedPath.toLowerCase();
	}
	return normalizedPath;
}

/**
 * 检查包安装目录是否可写。
 * 自更新功能需要写入权限来更新包文件，同时检查包目录及其父目录的可写性。
 */
function isSelfUpdatePathWritable(): boolean {
	const packageDir = getPackageDir();
	try {
		accessSync(packageDir, constants.W_OK);
		accessSync(dirname(packageDir), constants.W_OK);
		return true;
	} catch {
		return false;
	}
}

/**
 * 判断当前安装是否由全局包管理器管理。
 * 通过比较包目录路径是否在包管理器的全局根目录下来判断。
 * 这是决定自更新功能是否可用的关键条件之一。
 */
function isManagedByGlobalPackageManager(method: InstallMethod, packageName: string, npmCommand?: string[]): boolean {
	const packageDir = normalizeExistingPathForComparison(getPackageDir());
	return (
		!!packageDir &&
		getGlobalPackageRoots(method, packageName, npmCommand).some((root) => {
			const normalizedRoot = normalizeExistingPathForComparison(root);
			return (
				!!normalizedRoot &&
				packageDir.startsWith(normalizedRoot.endsWith(sep) ? normalizedRoot : `${normalizedRoot}${sep}`)
			);
		})
	);
}

/**
 * 获取自更新命令。
 * 仅在同时满足以下条件时返回命令：
 * 1. 能检测到安装方式
 * 2. 安装由全局包管理器管理（非本地项目安装）
 * 3. 包安装目录具有写入权限
 *
 * @param packageName - 当前已安装的包名
 * @param npmCommand - 自定义 npm 命令（覆盖默认的 npm 行为）
 * @param updatePackageName - 要更新到的目标包名（用于包名迁移场景）
 */
export function getSelfUpdateCommand(
	packageName: string,
	npmCommand?: string[],
	updatePackageName = packageName,
): SelfUpdateCommand | undefined {
	const method = detectInstallMethod();
	const command = getSelfUpdateCommandForMethod(method, packageName, updatePackageName, npmCommand);
	if (!command || !isManagedByGlobalPackageManager(method, packageName, npmCommand) || !isSelfUpdatePathWritable()) {
		return undefined;
	}
	return command;
}

/**
 * 获取自更新不可用时的说明信息。
 * 根据不可用的原因返回不同的提示：
 * - Bun 编译二进制 → 提示从 GitHub Releases 下载
 * - 路径不可写 → 提示手动使用包管理器更新
 * - 非全局安装 → 提示使用原始安装方式进行更新
 *
 * @param packageName - 当前已安装的包名
 * @param npmCommand - 自定义 npm 命令
 * @param updatePackageName - 要更新到的目标包名
 */
export function getSelfUpdateUnavailableInstruction(
	packageName: string,
	npmCommand?: string[],
	updatePackageName = packageName,
): string {
	const method = detectInstallMethod();
	if (method === "bun-binary") {
		return `Download from: https://github.com/earendil-works/pi-mono/releases/latest`;
	}
	const command = getSelfUpdateCommandForMethod(method, packageName, updatePackageName, npmCommand);
	if (command) {
		if (isManagedByGlobalPackageManager(method, packageName, npmCommand) && !isSelfUpdatePathWritable()) {
			return `This installation is managed by a global ${method} install, but the install path is not writable. Update it yourself with: ${command.display}`;
		}
		return `This installation is not managed by a global ${method} install. Update it with the package manager, wrapper, or source checkout that provides it.`;
	}
	return `Update ${updatePackageName} using the package manager, wrapper, or source checkout that provides this installation.`;
}

/**
 * 获取更新提示信息。
 * 如果自更新可用则返回具体命令，否则返回不可用说明。
 * 用于向用户展示如何将应用更新到最新版本。
 *
 * @param packageName - 包名
 */
export function getUpdateInstruction(packageName: string): string {
	const method = detectInstallMethod();
	const command = getSelfUpdateCommandForMethod(method, packageName);
	if (command) {
		return `Run: ${command.display}`;
	}
	return getSelfUpdateUnavailableInstruction(packageName);
}

// =============================================================================
// 包资源路径（随可执行文件一起分发的资产）
// =============================================================================

/**
 * 获取包资产的基础目录（用于定位主题、package.json、README.md、CHANGELOG.md 等）。
 * - Bun 编译二进制：返回可执行文件所在目录
 * - Node.js（dist/ 模式）：返回 __dirname（即 dist/ 目录）
 * - tsx（src/ 模式）：返回父目录（即包根目录，因为 __dirname 是 src/）
 *
 * 支持通过环境变量 `PI_PACKAGE_DIR` 覆盖（适用于 Nix/Guix 等路径特殊的包管理器）。
 */
export function getPackageDir(): string {
	// 允许通过环境变量覆盖（适用于 Nix/Guix 等存储路径会变化的包管理器）
	const envDir = process.env.PI_PACKAGE_DIR;
	if (envDir) {
		if (envDir === "~") return homedir();
		if (envDir.startsWith("~/")) return homedir() + envDir.slice(1);
		return envDir;
	}

	if (isBunBinary) {
		// Bun 编译二进制：process.execPath 指向编译后的可执行文件
		return dirname(process.execPath);
	}
	// Node.js 模式：从 __dirname 向上查找直到找到 package.json
	let dir = __dirname;
	while (dir !== dirname(dir)) {
		if (existsSync(join(dir, "package.json"))) {
			return dir;
		}
		dir = dirname(dir);
	}
	// 兜底（理论上不应执行到这里）
	return __dirname;
}

/**
 * 获取内置主题目录路径。
 * - Bun 编译二进制：可执行文件旁的 theme/ 目录
 * - Node.js（dist/）：dist/modes/interactive/theme/
 * - tsx（src/）：src/modes/interactive/theme/
 */
export function getThemesDir(): string {
	if (isBunBinary) {
		return join(getPackageDir(), "theme");
	}
	// 主题位于 src/ 或 dist/ 目录下的 modes/interactive/theme/ 中
	const packageDir = getPackageDir();
	const srcOrDist = existsSync(join(packageDir, "src")) ? "src" : "dist";
	return join(packageDir, srcOrDist, "modes", "interactive", "theme");
}

/**
 * 获取 HTML 导出模板目录路径。
 * - Bun 编译二进制：可执行文件旁的 export-html/ 目录
 * - Node.js（dist/）：dist/core/export-html/
 * - tsx（src/）：src/core/export-html/
 */
export function getExportTemplateDir(): string {
	if (isBunBinary) {
		return join(getPackageDir(), "export-html");
	}
	const packageDir = getPackageDir();
	const srcOrDist = existsSync(join(packageDir, "src")) ? "src" : "dist";
	return join(packageDir, srcOrDist, "core", "export-html");
}

/** 获取 package.json 的路径 */
export function getPackageJsonPath(): string {
	return join(getPackageDir(), "package.json");
}

/** 获取 README.md 的路径 */
export function getReadmePath(): string {
	return resolve(join(getPackageDir(), "README.md"));
}

/** 获取文档目录的路径 */
export function getDocsPath(): string {
	return resolve(join(getPackageDir(), "docs"));
}

/** 获取示例目录的路径 */
export function getExamplesPath(): string {
	return resolve(join(getPackageDir(), "examples"));
}

/** 获取 CHANGELOG.md 的路径 */
export function getChangelogPath(): string {
	return resolve(join(getPackageDir(), "CHANGELOG.md"));
}

/**
 * 获取内置交互模式资产目录路径。
 * - Bun 编译二进制：可执行文件旁的 assets/ 目录
 * - Node.js（dist/）：dist/modes/interactive/assets/
 * - tsx（src/）：src/modes/interactive/assets/
 */
export function getInteractiveAssetsDir(): string {
	if (isBunBinary) {
		return join(getPackageDir(), "assets");
	}
	const packageDir = getPackageDir();
	const srcOrDist = existsSync(join(packageDir, "src")) ? "src" : "dist";
	return join(packageDir, srcOrDist, "modes", "interactive", "assets");
}

/**
 * 获取指定名称的捆绑交互模式资产文件的完整路径。
 *
 * @param name - 资产文件名
 */
export function getBundledInteractiveAssetPath(name: string): string {
	return join(getInteractiveAssetsDir(), name);
}

// =============================================================================
// 应用配置（从 package.json 的 piConfig 字段读取）
// =============================================================================

/** package.json 中需要读取的字段结构 */
interface PackageJson {
	name?: string;
	version?: string;
	piConfig?: {
		name?: string;
		configDir?: string;
	};
}

const pkg = JSON.parse(readFileSync(getPackageJsonPath(), "utf-8")) as PackageJson;

// 从 piConfig 中读取应用名称，支持白标定制（如将 "pi" 重命名为其他品牌名）
const piConfigName: string | undefined = pkg.piConfig?.name;
/** 包名，默认为 "@earendil-works/pi-coding-agent" */
export const PACKAGE_NAME: string = pkg.name || "@earendil-works/pi-coding-agent";
/** 应用名，影响配置目录名、环境变量前缀等。可被 piConfig.name 覆盖 */
export const APP_NAME: string = piConfigName || "pi";
/** 应用显示标题，未自定义时使用希腊字母 π */
export const APP_TITLE: string = piConfigName ? APP_NAME : "π";
/** 配置目录名，默认为 ".pi"（即 ~/.pi/） */
export const CONFIG_DIR_NAME: string = pkg.piConfig?.configDir || ".pi";
/** 当前版本号，从 package.json 读取，兜底为 "0.0.0" */
export const VERSION: string = pkg.version || "0.0.0";

// 环境变量名基于 APP_NAME 动态生成，例如 PI_CODING_AGENT_DIR 或 TAU_CODING_AGENT_DIR
export const ENV_AGENT_DIR = `${APP_NAME.toUpperCase()}_CODING_AGENT_DIR`;
/** 会话目录的环境变量名，用于覆盖默认的会话存储位置 */
export const ENV_SESSION_DIR = `${APP_NAME.toUpperCase()}_CODING_AGENT_SESSION_DIR`;

/**
 * 展开路径中的波浪号（~）为用户主目录。
 * 支持 `~`（主目录本身）和 `~/path`（主目录下的子路径）两种形式。
 *
 * @param path - 可能包含 ~ 前缀的路径
 */
export function expandTildePath(path: string): string {
	if (path === "~") return homedir();
	if (path.startsWith("~/")) return homedir() + path.slice(1);
	return path;
}

const DEFAULT_SHARE_VIEWER_URL = "https://pi.dev/session/";

/**
 * 获取会话分享查看器的 URL。
 * 可通过环境变量 `PI_SHARE_VIEWER_URL` 覆盖默认地址。
 *
 * @param gistId - GitHub Gist ID，作为 URL 锚点附加到查看器地址后
 */
export function getShareViewerUrl(gistId: string): string {
	const baseUrl = process.env.PI_SHARE_VIEWER_URL || DEFAULT_SHARE_VIEWER_URL;
	return `${baseUrl}#${gistId}`;
}

// =============================================================================
// 用户配置路径（~/.pi/agent/*）
// =============================================================================

/**
 * 获取代理配置目录路径（如 ~/.pi/agent/）。
 * 支持通过环境变量（如 `PI_CODING_AGENT_DIR`）覆盖默认位置。
 * 认证文件、模型配置、会话存储等均位于此目录下。
 */
export function getAgentDir(): string {
	const envDir = process.env[ENV_AGENT_DIR];
	if (envDir) {
		return expandTildePath(envDir);
	}
	return join(homedir(), CONFIG_DIR_NAME, "agent");
}

/** 获取用户自定义主题目录路径 */
export function getCustomThemesDir(): string {
	return join(getAgentDir(), "themes");
}

/** 获取模型配置文件（models.json）路径，用于自定义模型列表和参数 */
export function getModelsPath(): string {
	return join(getAgentDir(), "models.json");
}

/** 获取认证文件（auth.json）路径，存储各提供商的 OAuth 凭证 */
export function getAuthPath(): string {
	return join(getAgentDir(), "auth.json");
}

/** 获取用户设置文件（settings.json）路径 */
export function getSettingsPath(): string {
	return join(getAgentDir(), "settings.json");
}

/** 获取自定义工具目录路径 */
export function getToolsDir(): string {
	return join(getAgentDir(), "tools");
}

/** 获取托管二进制文件目录路径（用于存放 fd、rg 等辅助工具的可执行文件） */
export function getBinDir(): string {
	return join(getAgentDir(), "bin");
}

/** 获取提示模板目录路径 */
export function getPromptsDir(): string {
	return join(getAgentDir(), "prompts");
}

/** 获取会话存储目录路径 */
export function getSessionsDir(): string {
	return join(getAgentDir(), "sessions");
}

/** 获取调试日志文件路径 */
export function getDebugLogPath(): string {
	return join(getAgentDir(), `${APP_NAME}-debug.log`);
}
