/**
 * @fileoverview 学习示例 05 — 配置分叉：不同思考等级的行为对比。
 *
 * 学习目标：
 *   1. 理解 ThinkingLevel 的概念及其对 Agent 行为的影响
 *   2. 了解可用等级：off, minimal, low, medium, high, xhigh
 *   3. 掌握如何为不同场景选择合适的思考等级
 *
 * ThinkingLevel 说明：
 *   - off     : 不开启扩展思考，响应最快，适合简单任务
 *   - minimal : 最小思考，快速响应
 *   - low     : 低强度思考，适合日常编码任务
 *   - medium  : 中等思考，平衡速度与质量
 *   - high    : 高强度思考，适合复杂推理和分析
 *   - xhigh   : 极高强度思考，适合最复杂的架构决策
 *
 * 注意：本示例使用 faux 提供商演示配置差异。
 *       真实场景中，思考等级会影响 LLM 的推理深度和输出质量。
 *
 * 运行方式：npx tsx examples/learning/05-configuration-forks.ts
 */

import { fauxAssistantMessage } from "@earendil-works/pi-ai";
import { createFauxSession } from "./_setup.js";

// ============================================================
// 第一步：在不同思考等级下创建会话
// ============================================================

// 注意：isLoading 表示暂时不需要运行对话的会话
async function demonstrateThinkingLevel(
	label: string,
	level: "off" | "low" | "medium" | "high",
	expectedBehavior: string,
) {
	console.log(`\n=== 思考等级: ${level.toUpperCase()} ===`);
	console.log(`  预期行为: ${expectedBehavior}`);

	const { session, faux, cleanup } = await createFauxSession({
		thinkingLevel: level,
		systemPrompt: "你是一个分析助手。你的思考深度受 thinkingLevel 配置影响。",
	});

	// 设置一个简单的响应
	faux.setResponses([
		fauxAssistantMessage(`[${label}] 这是一个在 ${level} 思考等级下的回复。思考等级影响 LLM 如何分配推理资源。`),
	]);

	let output = "";
	session.subscribe((event) => {
		if (event.type === "message_update" && event.assistantMessageEvent.type === "text_delta") {
			output += event.assistantMessageEvent.delta;
			process.stdout.write(event.assistantMessageEvent.delta);
		}

		if (event.type === "agent_end") {
			console.log();
		}
	});

	await session.prompt("请根据你的思考等级能力，简要分析一段简单的代码。");
	cleanup();

	return { level, output };
}

// ============================================================
// 第二步：依次运行不同等级
// ============================================================

console.log("=== 思考等级配置对比 ===\n");
console.log("（使用 faux 提供商，所有响应均由预设内容提供）\n");

const results: Array<{ level: string; output: string }> = [];

// 从低到高依次演示
results.push(await demonstrateThinkingLevel("快速模式", "off", "跳过深度思考，快速给出直接答案"));

results.push(await demonstrateThinkingLevel("日常模式", "low", "短思考链，适合简单编码问题"));

results.push(await demonstrateThinkingLevel("平衡模式", "medium", "适中思考深度，平衡速度和质量"));

results.push(await demonstrateThinkingLevel("深度模式", "high", "扩展思考，适合复杂架构决策"));

// ============================================================
// 第三步：总结对比
// ============================================================

console.log();
console.log("=== 对比总结 ===");
console.log();
console.log("| 等级    | 适用场景           | 特点               |");
console.log("|---------|--------------------|--------------------|");
console.log("| off     | 简单问答、快速反馈  | 最低延迟，最直接    |");
console.log("| low     | 日常编码、小修改    | 轻量思考，快速响应  |");
console.log("| medium  | 代码审查、重构      | 平衡速度与深度      |");
console.log("| high    | 架构设计、复杂调试  | 深度推理，更全面    |");
console.log("| xhigh   | 极端复杂问题        | 最大推理预算        |");
console.log();
console.log("实际使用建议：");
console.log("  1. 编程时使用 medium —— 大多数情况下最佳选择");
console.log("  2. 快速修改时使用 low —— 降低延迟和成本");
console.log("  3. 架构决策时使用 high —— 获得更全面的分析");
console.log("  4. 交互模式下可用 Ctrl+T 动态切换等级");
