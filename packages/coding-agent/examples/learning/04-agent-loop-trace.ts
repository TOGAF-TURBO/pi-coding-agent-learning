/**
 * @fileoverview 学习示例 04 — Agent 循环追踪：展示多轮推理过程。
 *
 * 学习目标：
 *   1. 理解 Agent 循环：LLM 响应 → 工具执行 → 结果反馈 → LLM 再响应
 *   2. 了解 stopReason 如何控制循环的继续或终止
 *   3. 掌握如何在 Agent 循环中追踪每一轮的输入和输出
 *
 * Agent 循环流程：
 *   Turn 1: User prompt → LLM 返回 tool_call("step1")
 *            → Agent 执行工具 step1 → 结果返回给 LLM
 *   Turn 2: LLM 收到工具结果 → 返回 tool_call("step2")
 *            → Agent 执行工具 step2 → 结果返回给 LLM
 *   Turn 3: LLM 综合所有结果 → 返回最终文本回复 (stopReason="stop")
 *
 * 运行方式：npx tsx examples/learning/04-agent-loop-trace.ts
 */

import { fauxAssistantMessage, fauxToolCall, Type } from "@earendil-works/pi-ai";
import type { ToolDefinition } from "@earendil-works/pi-coding-agent";
import { createFauxSession } from "./_setup.js";

// ============================================================
// 第一步：定义两个自定义工具，用于演示多步推理
// ============================================================

let stepCounter = 0;

const step1Tool: ToolDefinition = {
	name: "get_file_list",
	label: "获取文件列表",
	description: "获取项目中的文件列表，返回一个文件路径数组。",
	parameters: Type.Object({
		directory: Type.String({ description: "要列出文件的目标目录" }),
	}),
	async execute(_toolCallId, params, _signal, _onUpdate, _ctx) {
		stepCounter++;
		const dir = (params as { directory: string }).directory;
		console.log(`  [Turn ${stepCounter}] 执行 get_file_list("${dir}")`);
		return {
			content: [{ type: "text" as const, text: JSON.stringify(["src/index.ts", "src/utils.ts", "package.json"]) }],
			details: { directory: dir },
		};
	},
};

const step2Tool: ToolDefinition = {
	name: "read_file_summary",
	label: "读取文件摘要",
	description: "读取指定文件的摘要信息，返回行数和导入列表。",
	parameters: Type.Object({
		path: Type.String({ description: "要读取的文件路径" }),
	}),
	async execute(_toolCallId, params, _signal, _onUpdate, _ctx) {
		stepCounter++;
		const path = (params as { path: string }).path;
		console.log(`  [Turn ${stepCounter}] 执行 read_file_summary("${path}")`);
		return {
			content: [{ type: "text" as const, text: `文件 ${path}: 120 行, 导入: react, lodash, typebox` }],
			details: { path },
		};
	},
};

// ============================================================
// 第二步：创建会话，注册两个工具
// ============================================================

const { session, faux, cleanup } = await createFauxSession({
	systemPrompt: "你是一个项目分析助手，按步骤收集信息后给出总结。",
	customTools: [step1Tool, step2Tool],
});

// ============================================================
// 第三步：编排 faux 的多轮响应，模拟 Agent 循环
// ============================================================

// Turn 1: LLM 决定先列出文件
const response1 = fauxAssistantMessage([fauxToolCall("get_file_list", { directory: "." })], { stopReason: "toolUse" });

// Turn 2: 收到文件列表后，LLM 决定读取某个文件的摘要
const response2 = fauxAssistantMessage([fauxToolCall("read_file_summary", { path: "src/index.ts" })], {
	stopReason: "toolUse",
});

// Turn 3: 综合所有信息，给出最终回复
const response3 = fauxAssistantMessage(
	"项目分析完成：当前目录有 3 个文件。源码入口为 src/index.ts（120行），依赖了 react、lodash、typebox。建议优先阅读 src/index.ts 了解项目入口。",
);

faux.setResponses([response1, response2, response3]);

// ============================================================
// 第四步：订阅事件，实时追踪循环
// ============================================================

console.log("=== Agent 循环追踪演示 ===\n");

let turnCount = 0;

session.subscribe((event) => {
	if (event.type === "turn_start") {
		turnCount++;
		console.log(`\n--- 第 ${turnCount} 轮开始 ---`);
	}

	if (event.type === "message_update" && event.assistantMessageEvent.type === "text_delta") {
		process.stdout.write(event.assistantMessageEvent.delta);
	}

	if (event.type === "tool_execution_start") {
		console.log(`  [循环] LLM 决定调用工具: ${event.toolName}`);
	}

	if (event.type === "tool_execution_end") {
		console.log(`  [循环] 工具执行完毕 → 结果将反馈给 LLM 进入下一轮`);
	}

	if (event.type === "turn_end") {
		console.log(`--- 第 ${turnCount} 轮结束 (stopReason 由 LLM 决定) ---`);
	}

	if (event.type === "agent_end") {
		console.log();
		console.log("=== 循环总结 ===");
		console.log(`共经过 ${turnCount} 轮推理，产生 ${event.messages.length} 条消息`);
		console.log();
		console.log("循环模式：");
		console.log("  stopReason='toolUse' → Agent 执行工具后继续循环");
		console.log("  stopReason='stop'   → 循环结束，返回最终回复");
	}
});

// ============================================================
// 第五步：发送提示词
// ============================================================

await session.prompt("请分析这个项目的文件结构和代码概况。");

// ============================================================
// 第六步：清理
// ============================================================

cleanup();
