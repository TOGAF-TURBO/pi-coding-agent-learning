#!/usr/bin/env node
/**
 * @fileoverview Coding Agent 的命令行入口点。
 *
 * 职责：
 *   - 设置进程标题和环境变量
 *   - 配置全局 HTTP 代理（undici），禁用超时以支持本地 LLM 的长时间响应
 *   - 调用 main() 函数处理命令行参数
 *
 * 测试方式：npx tsx src/cli.ts [args...]
 */

import { EnvHttpProxyAgent, setGlobalDispatcher } from "undici";
import { APP_NAME } from "./config.js";
import { main } from "./main.js";

// 设置进程标题（在 ps/top 中显示应用名）
process.title = APP_NAME;
// 标记当前进程为 Coding Agent（其他模块可据此判断运行环境）
process.env.PI_CODING_AGENT = "true";
// 抑制 Node.js 弃用警告（SDK 可能使用已弃用的 API）
process.emitWarning = (() => {}) as typeof process.emitWarning;

// undici 默认 bodyTimeout/headersTimeout 为 300 秒
// 本地 LLM 长时间停顿（如 vLLM 缓冲大型工具调用）会超出该限制，
// 导致 SSE 流因 UND_ERR_BODY_TIMEOUT 中止。
// 此处禁用两者 —— 提供商 SDK 通过 retry.provider.timeoutMs 的
// AbortController 实现自己的超时机制。
setGlobalDispatcher(new EnvHttpProxyAgent({ bodyTimeout: 0, headersTimeout: 0 }));

// 传递命令行参数（去除 node 和脚本路径）
main(process.argv.slice(2));
