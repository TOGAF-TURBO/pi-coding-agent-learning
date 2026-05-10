/**
 * @fileoverview 学习示例 02 — 自定义工具注册与执行流程。
 *
 * 学习目标：
 *   1. 了解如何使用 TypeBox 定义工具的 JSON Schema 参数
 *   2. 掌握 customTools 选项注册自定义工具的方法
 *   3. 理解工具的执行流程：LLM 发起 tool_call → Agent 执行工具 → 结果返回 LLM
 *
 * 运行方式：npx tsx examples/learning/02-custom-tool.ts
 */

import { fauxAssistantMessage, fauxToolCall, Type } from "@earendil-works/pi-ai";
import type { ToolDefinition } from "@earendil-works/pi-coding-agent";
import { createFauxSession } from "./_setup.js";

// ============================================================
// 第一步：定义自定义工具 —— 单词翻译器
// ============================================================

// 翻译词典（模拟）
const dictionary: Record<string, string> = {
	hello: "你好",
	world: "世界",
	code: "代码",
	agent: "智能体",
	tool: "工具",
};

const translateTool: ToolDefinition = {
	name: "translate_to_chinese",
	label: "翻译成中文",
	description: "将英文单词翻译成中文。输入一个英文单词，返回其中文含义。",
	// 参数 Schema —— 告诉 LLM 这个工具接受什么输入
	parameters: Type.Object({
		word: Type.String({ description: "要翻译的英文单词，例如 'hello'" }),
	}),
	async execute(_toolCallId, params, _signal, _onUpdate, _ctx) {
		const word = (params as { word: string }).word.toLowerCase();
		const result = dictionary[word] ?? `(未找到 "${word}" 的翻译)`;
		console.log(`  [工具执行] translate_to_chinese("${word}") → "${result}"`);
		return {
			content: [{ type: "text" as const, text: result }],
			details: { word, result },
		};
	},
};

// ============================================================
// 第二步：创建带自定义工具的 Agent 会话
// ============================================================

const { session, faux, cleanup } = await createFauxSession({
	systemPrompt: "你是一个翻译助手。当用户想要翻译单词时，使用 translate_to_chinese 工具。",
	customTools: [translateTool],
});

// 预设 faux 的两轮响应：
//   第一轮：发起工具调用（要求翻译 "agent" 和 "tool"）
//   第二轮：收到工具结果后，给出最终回复
faux.setResponses([
	fauxAssistantMessage(
		[fauxToolCall("translate_to_chinese", { word: "agent" }), fauxToolCall("translate_to_chinese", { word: "tool" })],
		{ stopReason: "toolUse" },
	),
	fauxAssistantMessage("翻译结果：agent = 智能体，tool = 工具。两个词都是编程领域中常用的概念。"),
]);

// ============================================================
// 第三步：订阅事件，观察工具调用过程
// ============================================================

console.log("=== 翻译 Agent 示例 ===\n");

session.subscribe((event) => {
	// 助手文本输出
	if (event.type === "message_update" && event.assistantMessageEvent.type === "text_delta") {
		process.stdout.write(event.assistantMessageEvent.delta);
	}

	// 工具开始执行
	if (event.type === "tool_execution_start") {
		console.log(`  [事件] 工具开始执行: ${event.toolName}(${JSON.stringify(event.args)})`);
	}

	// 工具执行结束
	if (event.type === "tool_execution_end") {
		console.log(`  [事件] 工具执行结束: ${event.toolName}, 结果: ${JSON.stringify(event.result)}`);
	}

	// 对话回合结束
	if (event.type === "agent_end") {
		console.log();
		console.log("=== 对话完成 ===");
		console.log(
			`共 ${event.messages.length} 条消息，当前状态: ${JSON.stringify(session.state.messages.length > 0 ? "有对话历史" : "无对话历史")}`,
		);
	}
});

// ============================================================
// 第四步：发送需要翻译的提示词
// ============================================================

await session.prompt("请帮我翻译 agent 和 tool 这两个单词。");

// ============================================================
// 第五步：清理
// ============================================================

cleanup();
