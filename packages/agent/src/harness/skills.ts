/**
 * @fileoverview Skills — 技能文件的发现、加载和展开机制。
 *
 * 技能（Skill）是从 SKILL.md 文件加载的可选能力描述，用于在特定任务场景下
 * 指导模型执行。每个技能目录包含一个 SKILL.md 文件，其 YAML frontmatter
 * 定义名称和描述，正文为技能指令内容。
 *
 * 加载流程：
 *   1. 递归遍历指定目录，查找 SKILL.md 文件
 *   2. 解析 YAML frontmatter（名称、描述、disable-model-invocation 标志）
 *   3. 验证名称格式（小写字母、数字、连字符）和描述长度
 *   4. 遵循 .gitignore / .ignore / .fdignore 规则过滤
 *
 * 技能使用：
 *   - 技能列表注入系统提示（通过 system-prompt.ts）
 *   - 用户通过 /skillname 命令激活，内容展开为提示发送
 *   - expandSkillCommand() 将技能内容包装为 XML 标签
 */

import ignore from "ignore";
import { parse } from "yaml";
import type { ExecutionEnv, Skill } from "./types.js";

// 技能名称和描述的长度限制
const MAX_NAME_LENGTH = 64;
const MAX_DESCRIPTION_LENGTH = 1024;

// 遵循的忽略文件名（与 ripgrep 等工具一致）
const IGNORE_FILE_NAMES = [".gitignore", ".ignore", ".fdignore"];

type IgnoreMatcher = ReturnType<typeof ignore>;

/**
 * 技能加载过程中的诊断信息
 *
 * 目前只包含警告级别（如名称格式错误、描述过长等），
 * 不会阻止其他技能的加载。
 */
export interface SkillDiagnostic {
	/** 诊断严重性，当前只有 warning */
	type: "warning";
	/** 人类可读的诊断消息 */
	message: string;
	/** 关联的文件路径 */
	path: string;
}

/** SKILL.md 的 YAML frontmatter 结构 */
interface SkillFrontmatter {
	name?: string;
	description?: string;
	/** 设为 true 时技能不出现在模型可见的列表中 */
	"disable-model-invocation"?: boolean;
	[key: string]: unknown;
}

/**
 * 将技能展开为提示文本
 *
 * 将技能内容包装为 XML 标签，包含名称和文件路径属性。
 * 路径信息帮助模型理解技能文件的位置，以便正确解析相对路径引用。
 *
 * @param skill - 技能对象
 * @param additionalInstructions - 可选的附加指令（追加在技能内容后）
 * @returns 展开后的提示文本
 */
export function expandSkillCommand(skill: Skill, additionalInstructions?: string): string {
	const skillBlock = `<skill name="${skill.name}" location="${skill.filePath}">\nReferences are relative to ${dirnameEnvPath(skill.filePath)}.\n\n${skill.content}\n</skill>`;
	return additionalInstructions ? `${skillBlock}\n\n${additionalInstructions}` : skillBlock;
}

/**
 * 从一个或多个目录加载技能
 *
 * 递归遍历目录，加载 SKILL.md 文件和根目录下的 .md 文件，
 * 遵循忽略文件规则，返回加载的技能列表和诊断信息。
 * 不存在的输入目录会被静默跳过。
 *
 * @param env - 执行环境（提供文件系统访问）
 * @param dirs - 目录路径或路径数组
 * @returns 技能列表和诊断信息
 */
export async function loadSkills(
	env: ExecutionEnv,
	dirs: string | string[],
): Promise<{ skills: Skill[]; diagnostics: SkillDiagnostic[] }> {
	const skills: Skill[] = [];
	const diagnostics: SkillDiagnostic[] = [];
	for (const dir of Array.isArray(dirs) ? dirs : [dirs]) {
		const rootInfo = await safeFileInfo(env, dir);
		if (!rootInfo || (await resolveKind(env, rootInfo)) !== "directory") continue;
		const result = await loadSkillsFromDirInternal(env, rootInfo.path, true, ignore(), rootInfo.path);
		skills.push(...result.skills);
		diagnostics.push(...result.diagnostics);
	}
	return { skills, diagnostics };
}

/**
 * 从带来源标签的目录加载技能
 *
 * 每个输入目录关联一个 source 值（来源标签），加载后 source 保留在
 * 每个技能和诊断上。agent 包不解释 source 值，由应用层定义来源语义。
 * 例如，应用可以用 source 区分内置技能和用户技能。
 */
export async function loadSourcedSkills<TSource>(
	env: ExecutionEnv,
	inputs: Array<{ path: string; source: TSource }>,
): Promise<{
	skills: Array<{ skill: Skill; source: TSource }>;
	diagnostics: Array<SkillDiagnostic & { source: TSource }>;
}> {
	const skills: Array<{ skill: Skill; source: TSource }> = [];
	const diagnostics: Array<SkillDiagnostic & { source: TSource }> = [];
	for (const input of inputs) {
		const result = await loadSkills(env, input.path);
		for (const skill of result.skills) skills.push({ skill, source: input.source });
		for (const diagnostic of result.diagnostics) diagnostics.push({ ...diagnostic, source: input.source });
	}
	return { skills, diagnostics };
}

/**
 * 递归加载目录中的技能（内部实现）
 *
 * 加载策略：
 *   1. 如果目录包含 SKILL.md，加载它并立即返回（不继续搜索子目录）
 *   2. 否则递归处理子目录（跳过 . 开头的和 node_modules）
 *   3. 在根目录下（includeRootFiles=true）也加载直接的 .md 文件作为技能
 *   4. 在每个目录层级应用忽略规则
 *
 * @param env - 执行环境
 * @param dir - 当前目录路径
 * @param includeRootFiles - 是否将根目录下的 .md 文件作为技能加载
 * @param ignoreMatcher - 累积的忽略规则匹配器
 * @param rootDir - 遍历的根目录（用于计算相对路径）
 */
async function loadSkillsFromDirInternal(
	env: ExecutionEnv,
	dir: string,
	includeRootFiles: boolean,
	ignoreMatcher: IgnoreMatcher,
	rootDir: string,
): Promise<{ skills: Skill[]; diagnostics: SkillDiagnostic[] }> {
	const skills: Skill[] = [];
	const diagnostics: SkillDiagnostic[] = [];

	if (!(await env.exists(dir))) return { skills, diagnostics };
	const dirInfo = await safeFileInfo(env, dir);
	if (!dirInfo || (await resolveKind(env, dirInfo)) !== "directory") return { skills, diagnostics };

	// 在当前目录加载忽略规则
	await addIgnoreRules(env, ignoreMatcher, dir, rootDir);

	let entries: Awaited<ReturnType<ExecutionEnv["listDir"]>>;
	try {
		entries = await env.listDir(dir);
	} catch {
		return { skills, diagnostics };
	}

	// 优先检查 SKILL.md — 如果存在，加载并停止递归
	for (const entry of entries) {
		if (entry.name !== "SKILL.md") continue;
		const fullPath = entry.path;
		const kind = await resolveKind(env, entry);
		if (kind !== "file") continue;
		const relPath = relativeEnvPath(rootDir, fullPath);
		if (ignoreMatcher.ignores(relPath)) continue;

		const result = await loadSkillFromFile(env, fullPath);
		if (result.skill) skills.push(result.skill);
		diagnostics.push(...result.diagnostics);
		return { skills, diagnostics };
	}

	// 无 SKILL.md，递归处理子目录和根目录下的 .md 文件
	for (const entry of entries.sort((a, b) => a.name.localeCompare(b.name))) {
		if (entry.name.startsWith(".") || entry.name === "node_modules") continue;
		const fullPath = entry.path;
		const kind = await resolveKind(env, entry);
		if (!kind) continue;

		// 检查忽略规则（目录需要加尾部斜杠）
		const relPath = relativeEnvPath(rootDir, fullPath);
		const ignorePath = kind === "directory" ? `${relPath}/` : relPath;
		if (ignoreMatcher.ignores(ignorePath)) continue;

		if (kind === "directory") {
			const result = await loadSkillsFromDirInternal(env, fullPath, false, ignoreMatcher, rootDir);
			skills.push(...result.skills);
			diagnostics.push(...result.diagnostics);
			continue;
		}

		// 仅在根目录下加载 .md 文件作为技能
		if (kind !== "file" || !includeRootFiles || !entry.name.endsWith(".md")) continue;
		const result = await loadSkillFromFile(env, fullPath);
		if (result.skill) skills.push(result.skill);
		diagnostics.push(...result.diagnostics);
	}

	return { skills, diagnostics };
}

/**
 * 从忽略文件加载忽略规则
 *
 * 支持 .gitignore、.ignore 和 .fdignore 格式。
 * 规则相对于 rootDir，目录层级的规则会加上适当的前缀。
 */
async function addIgnoreRules(env: ExecutionEnv, ig: IgnoreMatcher, dir: string, rootDir: string): Promise<void> {
	const relativeDir = relativeEnvPath(rootDir, dir);
	const prefix = relativeDir ? `${relativeDir}/` : "";

	for (const filename of IGNORE_FILE_NAMES) {
		const ignorePath = joinEnvPath(dir, filename);
		const info = await safeFileInfo(env, ignorePath);
		if (info?.kind !== "file") continue;
		try {
			const content = await env.readTextFile(ignorePath);
			const patterns = content
				.split(/\r?\n/)
				.map((line) => prefixIgnorePattern(line, prefix))
				.filter((line): line is string => Boolean(line));
			if (patterns.length > 0) ig.add(patterns);
		} catch {}
	}
}

/**
 * 为忽略模式添加目录前缀
 *
 * 将相对模式转换为相对于 rootDir 的模式，处理否定（!）和
 * 根级（/）前缀。空行和注释被过滤。
 */
function prefixIgnorePattern(line: string, prefix: string): string | null {
	const trimmed = line.trim();
	if (!trimmed) return null;
	if (trimmed.startsWith("#") && !trimmed.startsWith("\\#")) return null;

	let pattern = line;
	let negated = false;
	if (pattern.startsWith("!")) {
		negated = true;
		pattern = pattern.slice(1);
	} else if (pattern.startsWith("\\!")) {
		pattern = pattern.slice(1);
	}
	if (pattern.startsWith("/")) pattern = pattern.slice(1);
	const prefixed = prefix ? `${prefix}${pattern}` : pattern;
	return negated ? `!${prefixed}` : prefixed;
}

/**
 * 从 SKILL.md 文件加载技能
 *
 * 解析 YAML frontmatter，验证名称和描述，构造 Skill 对象。
 * 缺少描述的文件返回 null（不会被注册为技能）。
 */
async function loadSkillFromFile(
	env: ExecutionEnv,
	filePath: string,
): Promise<{ skill: Skill | null; diagnostics: SkillDiagnostic[] }> {
	const diagnostics: SkillDiagnostic[] = [];
	try {
		const rawContent = await env.readTextFile(filePath);
		const { frontmatter, body } = parseFrontmatter<SkillFrontmatter>(rawContent);
		const skillDir = dirnameEnvPath(filePath);
		const parentDirName = basenameEnvPath(skillDir);

		for (const error of validateDescription(frontmatter.description)) {
			diagnostics.push({ type: "warning", message: error, path: filePath });
		}

		const name = frontmatter.name || parentDirName;
		for (const error of validateName(name, parentDirName)) {
			diagnostics.push({ type: "warning", message: error, path: filePath });
		}

		// 描述为空时不注册技能
		if (!frontmatter.description || frontmatter.description.trim() === "") {
			return { skill: null, diagnostics };
		}

		return {
			skill: {
				name,
				description: frontmatter.description,
				content: body,
				filePath,
				disableModelInvocation: frontmatter["disable-model-invocation"] === true,
			},
			diagnostics,
		};
	} catch (error) {
		const message = error instanceof Error ? error.message : "failed to parse skill file";
		diagnostics.push({ type: "warning", message, path: filePath });
		return { skill: null, diagnostics };
	}
}

/**
 * 验证技能名称
 *
 * 规则：必须与父目录名匹配，只允许小写字母、数字和连字符，
 * 长度不超过 64 字符，不能以连字符开头/结尾，不能有连续连字符。
 */
function validateName(name: string, parentDirName: string): string[] {
	const errors: string[] = [];
	if (name !== parentDirName) errors.push(`name "${name}" does not match parent directory "${parentDirName}"`);
	if (name.length > MAX_NAME_LENGTH) errors.push(`name exceeds ${MAX_NAME_LENGTH} characters (${name.length})`);
	if (!/^[a-z0-9-]+$/.test(name)) {
		errors.push("name contains invalid characters (must be lowercase a-z, 0-9, hyphens only)");
	}
	if (name.startsWith("-") || name.endsWith("-")) errors.push("name must not start or end with a hyphen");
	if (name.includes("--")) errors.push("name must not contain consecutive hyphens");
	return errors;
}

/** 验证技能描述（必须非空且不超过 1024 字符） */
function validateDescription(description: string | undefined): string[] {
	const errors: string[] = [];
	if (!description || description.trim() === "") {
		errors.push("description is required");
	} else if (description.length > MAX_DESCRIPTION_LENGTH) {
		errors.push(`description exceeds ${MAX_DESCRIPTION_LENGTH} characters (${description.length})`);
	}
	return errors;
}

/**
 * 解析 YAML frontmatter
 *
 * 从内容中提取 `---` 包围的 YAML 头部和正文。
 * 无 frontmatter 时返回空对象和完整内容。
 */
function parseFrontmatter<T extends Record<string, unknown>>(content: string): { frontmatter: T; body: string } {
	const normalized = content.replace(/\r\n/g, "\n").replace(/\r/g, "\n");
	if (!normalized.startsWith("---")) return { frontmatter: {} as T, body: normalized };
	const endIndex = normalized.indexOf("\n---", 3);
	if (endIndex === -1) return { frontmatter: {} as T, body: normalized };
	const yamlString = normalized.slice(4, endIndex);
	const body = normalized.slice(endIndex + 4).trim();
	return { frontmatter: (parse(yamlString) ?? {}) as T, body };
}

// ============================================================================
// 路径工具函数（不依赖 Node.js path 模块，兼容浏览器环境）
// ============================================================================

/** 安全获取文件信息（失败返回 undefined 而非抛异常） */
async function safeFileInfo(
	env: ExecutionEnv,
	path: string,
): Promise<Awaited<ReturnType<ExecutionEnv["fileInfo"]>> | undefined> {
	try {
		return await env.fileInfo(path);
	} catch {
		return undefined;
	}
}

/**
 * 解析文件系统对象的实际类型
 *
 * 处理符号链接：先通过 realPath 跟踪链接，再判断目标类型。
 */
async function resolveKind(
	env: ExecutionEnv,
	info: Awaited<ReturnType<ExecutionEnv["fileInfo"]>>,
): Promise<"file" | "directory" | undefined> {
	if (info.kind === "file" || info.kind === "directory") return info.kind;
	try {
		const realPath = await env.realPath(info.path);
		const target = await env.fileInfo(realPath);
		return target.kind === "file" || target.kind === "directory" ? target.kind : undefined;
	} catch {
		return undefined;
	}
}

/** 拼接路径（不依赖 Node.js path.join，兼容浏览器） */
function joinEnvPath(base: string, child: string): string {
	return `${base.replace(/\/+$/, "")}/${child.replace(/^\/+/, "")}`;
}

/** 获取目录名（不依赖 Node.js path.dirname，兼容浏览器） */
function dirnameEnvPath(path: string): string {
	const normalized = path.replace(/\/+$/, "");
	const slashIndex = normalized.lastIndexOf("/");
	return slashIndex <= 0 ? "/" : normalized.slice(0, slashIndex);
}

/** 获取文件名（不依赖 Node.js path.basename，兼容浏览器） */
function basenameEnvPath(path: string): string {
	const normalized = path.replace(/\/+$/, "");
	const slashIndex = normalized.lastIndexOf("/");
	return slashIndex === -1 ? normalized : normalized.slice(slashIndex + 1);
}

/** 计算相对路径（不依赖 Node.js path.relative，兼容浏览器） */
function relativeEnvPath(root: string, path: string): string {
	const normalizedRoot = root.replace(/\/+$/, "");
	const normalizedPath = path.replace(/\/+$/, "");
	if (normalizedPath === normalizedRoot) return "";
	return normalizedPath.startsWith(`${normalizedRoot}/`)
		? normalizedPath.slice(normalizedRoot.length + 1)
		: normalizedPath.replace(/^\/+/, "");
}
