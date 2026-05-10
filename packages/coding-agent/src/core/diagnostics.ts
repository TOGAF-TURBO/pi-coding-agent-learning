/**
 * @fileoverview Diagnostics — 资源加载诊断信息（冲突和错误）。
 *
 * 追踪技能、提示模板、扩展、主题加载时的冲突和错误，
 * 提供统一的诊断报告格式。
 */
export interface ResourceCollision {
	name: string; // skill name, command/tool/flag name, prompt name, theme name
	winnerPath: string;
	loserPath: string;
	winnerSource?: string; // e.g., "npm:foo", "git:...", "local"
	loserSource?: string;
}

export interface ResourceDiagnostic {
	type: "warning" | "error" | "collision";
	message: string;
	path?: string;
	collision?: ResourceCollision;
}
