/**
 * @fileoverview 学习示例 03 — 流式事件系统：观察 Agent 完整生命周期。
 *
 * 学习目标：
 *   1. 理解 Agent 的事件驱动架构和事件类型体系
 *   2. 掌握 subscribe 机制监听各类生命周期事件
 *   3. 了解一轮对话中事件的触发顺序和层次结构
 *
 * 事件层级（从外到内）：
 *   agent_start
 *     turn_start
 *       message_start (user)
 *       message_end   (user)
 *       message_start (assistant)
 *         message_update × N  ← 流式增量
 *       message_end   (assistant)
 *       [ message_start/message_end (tool) ]  ← 如果有工具调用
 *       [ tool_execution_start/tool_execution_end ]
 *     turn_end
 *   agent_end
 *
 * 运行方式：npx tsx examples/learning/03-streaming-events.ts
 */

import { fauxAssistantMessage, fauxToolCall } from "@earendil-works/pi-ai";
import { createFauxSession } from "./_setup.js";

// ============================================================
// 第一步：创建会话
// ============================================================

const { session, faux, cleanup } = await createFauxSession({
	systemPrompt: "你是一个演示助手，用于展示事件系统。",
});

// 预设两轮 faux 响应，以触发工具执行生命周期事件
faux.setResponses([
	fauxAssistantMessage([fauxToolCall("read", { path: "/demo/hello.txt" })], { stopReason: "toolUse" }),
	fauxAssistantMessage("演示完成！你已经看到了 Agent 的完整事件生命周期。"),
]);

// ============================================================
// 第二步：订阅所有事件类型并记录
// ============================================================

const eventLog: string[] = [];

session.subscribe((event) => {
	const prefix = `  [${event.type}]`;

	switch (event.type) {
		// === Agent 生命周期 ===
		case "agent_start":
			eventLog.push(`${prefix} Agent 开始运行`);
			break;

		case "agent_end":
			eventLog.push(`${prefix} Agent 运行结束，共 ${event.messages.length} 条消息`);
			break;

		// === 轮次生命周期 ===
		case "turn_start":
			eventLog.push(`${prefix} 新对话轮次开始`);
			break;

		case "turn_end":
			eventLog.push(`${prefix} 对话轮次结束，工具结果数: ${event.toolResults.length}`);
			break;

		// === 消息生命周期 ===
		case "message_start":
			eventLog.push(`${prefix} 消息开始 [角色: ${event.message.role}]`);
			break;

		case "message_update":
			// 流式增量 —— 实时输出文本
			if (event.assistantMessageEvent.type === "text_delta") {
				process.stdout.write(event.assistantMessageEvent.delta);
			} else if (event.assistantMessageEvent.type === "thinking_delta") {
				process.stdout.write(`(思考: ${event.assistantMessageEvent.delta})`);
			}
			break;

		case "message_end":
			eventLog.push(`${prefix} 消息结束 [角色: ${event.message.role}]`);
			break;

		// === 工具执行生命周期 ===
		case "tool_execution_start":
			eventLog.push(`${prefix} 工具开始: ${event.toolName}(${JSON.stringify(event.args)})`);
			break;

		case "tool_execution_update":
			eventLog.push(`${prefix} 工具更新: ${event.toolName}, 部分结果: ${JSON.stringify(event.partialResult)}`);
			break;

		case "tool_execution_end":
			eventLog.push(`${prefix} 工具结束: ${event.toolName}, 是否错误: ${event.isError}`);
			break;
	}
});

// ============================================================
// 第三步：发送提示词
// ============================================================

console.log("=== 事件生命周期演示 ===\n");
console.log("（助手回复如下）\n");

await session.prompt("请演示完整的 Agent 事件生命周期。使用 read 工具读取一个文件。");

// ============================================================
// 第四步：展示事件顺序
// ============================================================

console.log();
console.log("=== 事件触发顺序 ===");
for (const entry of eventLog) {
	console.log(entry);
}

console.log();
console.log("=== 事件类型说明 ===");
console.log("  agent_start/agent_end            Agent 整体生命周期");
console.log("  turn_start/turn_end              单轮对话（一个用户问题 + 助手回复）");
console.log("  message_start/message_update/message_end  单条消息的流式生命周期");
console.log("  tool_execution_start/tool_execution_end   工具调用的执行生命周期");

cleanup();
