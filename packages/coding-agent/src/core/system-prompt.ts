/**
 * @fileoverview 系统提示构建与项目上下文加载
 *
 * 本模块负责构建发送给 LLM 的完整系统提示。系统提示是 pi 编码代理的核心配置，
 * 它定义了代理的身份、可用工具列表、行为准则和项目特定的上下文信息。
 *
 * 主要功能：
 * - `buildSystemPrompt`: 根据传入选项构建完整的系统提示字符串
 * - 支持自定义提示或使用默认提示模板
 * - 自动附加项目上下文文件和技能描述
 * - 根据可用工具动态生成工具使用准则
 *
 * 设计要点：
 * - 当提供自定义提示时，仅附加日期和工作目录，保留用户对提示的完全控制
 * - 技能信息只在 read 工具可用时才附加，因为技能依赖文件读取能力
 * - 工具列表只展示提供了摘要的工具，避免暴露未实现的工具
 */

import { getDocsPath, getExamplesPath, getReadmePath } from "../config.js";
import { formatSkillsForPrompt, type Skill } from "./skills.js";

export interface BuildSystemPromptOptions {
	/** 自定义系统提示（替换默认提示） */
	customPrompt?: string;
	/** 选择的工具列表。默认: [read, bash, edit, write] */
	selectedTools?: string[];
	/** 可选的工具摘要，以工具名为键 */
	toolSnippets?: Record<string, string>;
	/** 附加到默认系统提示准则的额外准则条目 */
	promptGuidelines?: string[];
	/** 追加到系统提示末尾的文本 */
	appendSystemPrompt?: string;
	/** 工作目录路径 */
	cwd: string;
	/** 预加载的上下文文件 */
	contextFiles?: Array<{ path: string; content: string }>;
	/** 预加载的技能列表 */
	skills?: Skill[];
}

/**
 * 构建包含工具列表、行为准则和项目上下文的完整系统提示
 *
 * @param options - 系统提示构建选项
 * @returns 完整的系统提示字符串
 */
export function buildSystemPrompt(options: BuildSystemPromptOptions): string {
	const {
		customPrompt,
		selectedTools,
		toolSnippets,
		promptGuidelines,
		appendSystemPrompt,
		cwd,
		contextFiles: providedContextFiles,
		skills: providedSkills,
	} = options;
	const resolvedCwd = cwd;
	// 统一路径分隔符为正斜杠，确保跨平台一致性
	const promptCwd = resolvedCwd.replace(/\\/g, "/");

	const now = new Date();
	const year = now.getFullYear();
	const month = String(now.getMonth() + 1).padStart(2, "0");
	const day = String(now.getDate()).padStart(2, "0");
	const date = `${year}-${month}-${day}`;

	const appendSection = appendSystemPrompt ? `\n\n${appendSystemPrompt}` : "";

	const contextFiles = providedContextFiles ?? [];
	const skills = providedSkills ?? [];

	// 自定义提示分支：保留用户的提示内容，仅追加必要的上下文信息
	if (customPrompt) {
		let prompt = customPrompt;

		if (appendSection) {
			prompt += appendSection;
		}

		// 追加项目上下文文件
		if (contextFiles.length > 0) {
			prompt += "\n\n# Project Context\n\n";
			prompt += "Project-specific instructions and guidelines:\n\n";
			for (const { path: filePath, content } of contextFiles) {
				prompt += `## ${filePath}\n\n${content}\n\n`;
			}
		}

		// 仅在 read 工具可用时附加技能信息（技能依赖文件读取能力）
		const customPromptHasRead = !selectedTools || selectedTools.includes("read");
		if (customPromptHasRead && skills.length > 0) {
			prompt += formatSkillsForPrompt(skills);
		}

		// 日期和工作目录放在最后，确保 LLM 优先关注前面的内容
		prompt += `\nCurrent date: ${date}`;
		prompt += `\nCurrent working directory: ${promptCwd}`;

		return prompt;
	}

	// 获取 pi 文档和示例的绝对路径
	const readmePath = getReadmePath();
	const docsPath = getDocsPath();
	const examplesPath = getExamplesPath();

	// 构建工具列表。只有提供了摘要的工具才会展示给 LLM，避免暴露未实现的工具
	const tools = selectedTools || ["read", "bash", "edit", "write"];
	const visibleTools = tools.filter((name) => !!toolSnippets?.[name]);
	const toolsList =
		visibleTools.length > 0 ? visibleTools.map((name) => `- ${name}: ${toolSnippets![name]}`).join("\n") : "(none)";

	// 根据实际可用的工具动态生成行为准则，使用 Set 去重
	const guidelinesList: string[] = [];
	const guidelinesSet = new Set<string>();
	const addGuideline = (guideline: string): void => {
		if (guidelinesSet.has(guideline)) {
			return;
		}
		guidelinesSet.add(guideline);
		guidelinesList.push(guideline);
	};

	const hasBash = tools.includes("bash");
	const hasGrep = tools.includes("grep");
	const hasFind = tools.includes("find");
	const hasLs = tools.includes("ls");
	const hasRead = tools.includes("read");

	// 文件探索准则：根据可用工具组合推荐最优的文件搜索方式
	if (hasBash && !hasGrep && !hasFind && !hasLs) {
		addGuideline("Use bash for file operations like ls, rg, find");
	} else if (hasBash && (hasGrep || hasFind || hasLs)) {
		addGuideline("Prefer grep/find/ls tools over bash for file exploration (faster, respects .gitignore)");
	}

	// 合并用户自定义的准则
	for (const guideline of promptGuidelines ?? []) {
		const normalized = guideline.trim();
		if (normalized.length > 0) {
			addGuideline(normalized);
		}
	}

	// 始终包含的基础准则
	addGuideline("Be concise in your responses");
	addGuideline("Show file paths clearly when working with files");

	const guidelines = guidelinesList.map((g) => `- ${g}`).join("\n");

	let prompt = `You are an expert coding assistant operating inside pi, a coding agent harness. You help users by reading files, executing commands, editing code, and writing new files.

Available tools:
${toolsList}

In addition to the tools above, you may have access to other custom tools depending on the project.

Guidelines:
${guidelines}

Pi documentation (read only when the user asks about pi itself, its SDK, extensions, themes, skills, or TUI):
- Main documentation: ${readmePath}
- Additional docs: ${docsPath}
- Examples: ${examplesPath} (extensions, custom tools, SDK)
- When asked about: extensions (docs/extensions.md, examples/extensions/), themes (docs/themes.md), skills (docs/skills.md), prompt templates (docs/prompt-templates.md), TUI components (docs/tui.md), keybindings (docs/keybindings.md), SDK integrations (docs/sdk.md), custom providers (docs/custom-provider.md), adding models (docs/models.md), pi packages (docs/packages.md)
- When working on pi topics, read the docs and examples, and follow .md cross-references before implementing
- Always read pi .md files completely and follow links to related docs (e.g., tui.md for TUI API details)`;

	if (appendSection) {
		prompt += appendSection;
	}

	// 追加项目上下文文件
	if (contextFiles.length > 0) {
		prompt += "\n\n# Project Context\n\n";
		prompt += "Project-specific instructions and guidelines:\n\n";
		for (const { path: filePath, content } of contextFiles) {
			prompt += `## ${filePath}\n\n${content}\n\n`;
		}
	}

	// 仅在 read 工具可用时附加技能信息
	if (hasRead && skills.length > 0) {
		prompt += formatSkillsForPrompt(skills);
	}

	// 日期和工作目录放在最后
	prompt += `\nCurrent date: ${date}`;
	prompt += `\nCurrent working directory: ${promptCwd}`;

	return prompt;
}
