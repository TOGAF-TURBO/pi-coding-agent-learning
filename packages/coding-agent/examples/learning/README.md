/**
 * @fileoverview 学习示例 README —— pi coding-agent 核心概念入门。
 *
 * 本目录包含面向初级程序员的 Agent Coding 学习示例。
 * 所有示例使用 faux（模拟）提供商，无需真实 API Key 即可运行。
 */

# pi Coding Agent 学习示例

面向初级程序员的 Agent Coding 核心概念入门。每个示例都是独立可运行的 TypeScript 脚本，使用 faux 提供商替代真实 LLM API，无需配置 API Key。

## 前置条件

- Node.js >= 18
- 在仓库根目录执行过 `npm install`

## 运行方式

```bash
# 在仓库根目录下运行
cd packages/coding-agent
npx tsx examples/learning/01-minimal-agent.ts
```

## 示例总览

| 文件 | 学习概念 | 涉及技能 |
|------|----------|----------|
| `01-minimal-agent.ts` | 创建 Agent、发送提示词、获取响应 | `createFauxSession`, `subscribe`, `prompt` |
| `02-custom-tool.ts` | 自定义工具注册与执行流程 | `customTools`, `ToolDefinition`, TypeBox Schema |
| `03-streaming-events.ts` | Agent 事件生命周期体系 | `subscribe`, 事件类型 (agent/turn/message/tool) |
| `04-agent-loop-trace.ts` | Agent 循环：多轮推理过程 | 工具调用反馈循环, `stopReason`, 多轮追踪 |
| `05-configuration-forks.ts` | ThinkingLevel 配置与行为对比 | `thinkingLevel`, 配置选项, 场景选择 |
| `_setup.ts` | 共享工具 —— faux 会话工厂 | `registerFauxProvider`, 认证和模型注册 |

## 示例详解

### 01-minimal-agent.ts — 最小化 Agent

**学习目标**：了解 Agent 的最基本使用方式。

创建一个 Agent 会话，发送一条提示词，获取响应。展示了：
- 使用 `createFauxSession()` 快速创建无需认证的 Agent
- 通过 `session.subscribe()` 监听流式输出
- 用 `session.prompt()` 发送用户消息

### 02-custom-tool.ts — 自定义工具

**学习目标**：理解工具注册和 LLM 调用工具的执行流程。

注册一个翻译工具，观察 LLM 如何发起工具调用、Agent 如何执行工具、结果如何返回。展示了：
- 用 TypeBox 的 `Type.Object()` 定义工具参数 Schema
- `ToolDefinition` 接口的 `name`、`description`、`parameters`、`execute` 四个核心字段
- `tool_execution_start` / `tool_execution_end` 事件

### 03-streaming-events.ts — 流式事件系统

**学习目标**：掌握 Agent 的事件驱动架构。

订阅所有事件类型，展示一轮对话中事件的完整触发顺序。展示了：
- 四层事件体系：Agent → Turn → Message → Tool
- 流式增量更新 (`message_update + text_delta`)
- 事件的生命周期和层次关系

### 04-agent-loop-trace.ts — Agent 循环追踪

**学习目标**：理解 Agent 的多轮推理循环。

通过两个自定义工具和多轮 faux 响应，展示 Agent 如何：
1. LLM 返回 tool_call → Agent 执行工具
2. 工具结果反馈给 LLM → LLM 再次返回 tool_call
3. 循环直到 LLM 返回 `stopReason="stop"`

关键概念：`stopReason="toolUse"` 继续循环，`stopReason="stop"` 终止循环。

### 05-configuration-forks.ts — 配置分叉

**学习目标**：理解 ThinkingLevel 的概念和选择策略。

依次创建 off、low、medium、high 四个等级的 Agent 会话，对比不同配置下的行为差异。展示了：
- 六个思考等级及其适用场景
- 如何通过 `createFauxSession({ thinkingLevel: "high" })` 配置
- 真实场景下的选择建议

## 核心概念速查

```typescript
import { fauxAssistantMessage, fauxToolCall, Type } from "@earendil-works/pi-ai";
import {
  createAgentSession,
  AuthStorage,
  ModelRegistry,
  SessionManager,
  SettingsManager,
  DefaultResourceLoader,
} from "@earendil-works/pi-coding-agent";

// Faux 会话工厂（所有示例的基础）
const { session, faux, cleanup } = await createFauxSession({
  systemPrompt: "你是一个助手",
  thinkingLevel: "medium",
  tools: ["read"],                    // 白名单工具
  customTools: [myTool],             // 自定义工具
});

// 预设响应
faux.setResponses([
  fauxAssistantMessage("回复文本"),
  fauxAssistantMessage([fauxToolCall("tool_name", { param: "value" })], { stopReason: "toolUse" }),
]);

// 订阅事件
session.subscribe((event) => {
  if (event.type === "message_update" && event.assistantMessageEvent.type === "text_delta") {
    process.stdout.write(event.assistantMessageEvent.delta);
  }
});

// 发送提示词
await session.prompt("帮我做某事");

// 清理
cleanup();
```

## 下一步

- 阅读 [SDK 示例](../sdk/) 了解更高级的 SDK 用法
- 阅读 [扩展示例](../extensions/) 了解如何为 Agent 编写扩展
- 查看 [SDK 文档](../sdk/README.md) 了解所有可用选项
