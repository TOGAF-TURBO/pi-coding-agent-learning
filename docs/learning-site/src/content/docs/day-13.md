---
title: "Day 13: RPC 协议"
---

**学习目标**：理解无头模式的 stdin/stdout JSON 协议

## 阅读清单

| 顺序 | 文件 | 关键行 | 要理解的概念 | 术语 |
|------|------|--------|-------------|------|
| 1 | `packages/coding-agent/src/modes/rpc/rpc-types.ts:1` | `RpcCommand` 联合类型 | 27 种 RPC 命令的数据结构 | RPC 命令 |
| 2 | `packages/coding-agent/src/modes/rpc/rpc-mode.ts:48` | `runRpcMode()` | RPC 模式的完整启动流程 | RPC 模式 |
| 3 | `packages/coding-agent/src/modes/rpc/rpc-mode.ts:371` | `handleCommand()` | 命令分发到各个处理器 | RPC 命令 |
| 4 | `packages/coding-agent/src/modes/rpc/jsonl.ts:1` | `attachJsonlLineReader()` | JSON 行的流式解析 | JSON 行协议 |

## 通信协议

RPC 模式使用简单的**JSONL 协议**通过 stdin/stdout 通信：

```
→ stdin:  {"type": "prompt", "messages": [...]}\n
→ stdin:  {"type": "abort"}\n
← stdout: {"type": "text_delta", "content": "我..."}\n
← stdout: {"type": "text_delta", "content": "来帮..."}\n
← stdout: {"type": "done"}\n
```

## 命令分发

`handleCommand()` 是命令分发中心。收到 stdin 上的 JSON 命令后，根据 `type` 字段路由到对应的处理器函数。27 种命令覆盖了：

- **会话操作**：创建、切换、重命名、列出会话
- **消息操作**：发送消息、中断生成
- **工具操作**：执行工具、获取工具列表
- **状态查询**：获取当前状态、流式事件订阅

## 使用场景

RPC 模式是 pi 作为无头 Agent 的集成接口。编辑器插件（VS Code、Neovim）和 Web UI 通过 stdin/stdout JSON 协议与 pi 进程通信，无需关心 TUI 实现细节。
