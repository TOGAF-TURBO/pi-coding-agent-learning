/**
 * @fileoverview System Prompt — 将技能列表格式化为系统提示的一部分。
 *
 * 将可见技能（未设置 disableModelInvocation）格式化为 XML 结构的技能列表，
 * 嵌入到系统提示中供模型了解可用的技能及其用途。
 *
 * 输出格式：
 *   <available_skills>
 *     <skill>
 *       <name>技能名</name>
 *       <description>描述</description>
 *       <location>文件路径</location>
 *     </skill>
 *     ...
 *   </available_skills>
 */

import type { Skill } from "./types.js";

/**
 * 将技能列表格式化为系统提示文本
 *
 * 过滤掉 disableModelInvocation=true 的技能，将剩余技能格式化为
 * XML 结构。包含使用说明，指导模型在匹配时通过 read 工具加载技能文件。
 * 技能为空时返回空字符串（不注入任何内容）。
 *
 * @param skills - 所有已加载的技能
 * @returns 格式化后的系统提示文本，或空字符串
 */
export function formatSkillsForSystemPrompt(skills: Skill[]): string {
	const visibleSkills = skills.filter((skill) => !skill.disableModelInvocation);
	if (visibleSkills.length === 0) return "";

	const lines = [
		"The following skills provide specialized instructions for specific tasks.",
		"Use the read tool to load a skill's file when the task matches its description.",
		"When a skill file references a relative path, resolve it against the skill directory (parent of SKILL.md / dirname of the path) and use that absolute path in tool commands.",
		"",
		"<available_skills>",
	];

	for (const skill of visibleSkills) {
		lines.push("  <skill>");
		lines.push(`    <name>${escapeXml(skill.name)}</name>`);
		lines.push(`    <description>${escapeXml(skill.description)}</description>`);
		lines.push(`    <location>${escapeXml(skill.filePath)}</location>`);
		lines.push("  </skill>");
	}

	lines.push("</available_skills>");
	return lines.join("\n");
}

/** 转义 XML 特殊字符，防止技能内容中的 HTML/XML 破坏系统提示结构 */
function escapeXml(value: string): string {
	return value
		.replace(/&/g, "&amp;")
		.replace(/</g, "&lt;")
		.replace(/>/g, "&gt;")
		.replace(/"/g, "&quot;")
		.replace(/'/g, "&apos;");
}
