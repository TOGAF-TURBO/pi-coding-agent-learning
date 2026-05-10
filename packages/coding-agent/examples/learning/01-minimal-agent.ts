/**
 * @fileoverview 学习示例 01 — 最小化 Agent：创建 Agent 并发送提示词。
 *
 * 学习目标：
 *   1. 了解如何使用 faux 提供商创建不需要真实 API Key 的 Agent
 *   2. 掌握 Agent 会话的基本生命周期：创建 → 发送提示词 → 获取响应
 *   3. 理解 subscribe 机制如何监听 Agent 的输出流
 *
 * 运行方式：npx tsx examples/learning/01-minimal-agent.ts
 */

import { fauxAssistantMessage } from "@earendil-works/pi-ai";
import { createFauxSession } from "./_setup.js";

// ============================================================
// 第一步：创建 faux 会话（不依赖真实 API Key）
// ============================================================

const { session, faux, cleanup } = await createFauxSession({
	systemPrompt: "你是一个乐于助人的编码助手，回答要简洁。",
});

// 预设 faux 的响应 —— 无论 Agent 问什么，faux 都会返回这条消息
faux.setResponses([fauxAssistantMessage("你好！我是一个编码助手，可以帮你阅读和修改代码。有什么我可以帮你的吗？")]);

// ============================================================
// 第二步：订阅事件流，实时查看 Agent 的回复
// ============================================================

console.log("=== 发送提示词 ===\n");

session.subscribe((event) => {
	// message_update + text_delta = 助手正在逐字输出
	if (event.type === "message_update" && event.assistantMessageEvent.type === "text_delta") {
		process.stdout.write(event.assistantMessageEvent.delta);
	}

	// agent_end = 一轮对话结束
	if (event.type === "agent_end") {
		console.log();
		console.log();
		console.log("=== 对话完成 ===");
		console.log(`共产生 ${event.messages.length} 条消息`);
		console.log();
		// 遍历所有消息，展示消息结构
		for (const msg of event.messages) {
			console.log(`  [${msg.role}]`, msg.role === "user" ? msg.content : "(助手回复见上方)");
		}
	}
});

// ============================================================
// 第三步：发送提示词，触发一轮 Agent 对话
// ============================================================

await session.prompt("你好，请介绍一下你自己。");

// ============================================================
// 第四步：清理资源
// ============================================================

cleanup();
