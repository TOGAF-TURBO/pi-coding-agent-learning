#!/usr/bin/env node
/**
 * @fileoverview @earendil-works/pi-ai 包的命令行工具入口。
 *
 * 提供以下子命令：
 *   - login [provider]: 通过 OAuth 流程登录到指定提供商，支持交互式选择；
 *                       认证凭证保存到本地 auth.json 文件。
 *   - list:             列出所有已注册的 OAuth 提供商。
 *   - help:             显示用法说明和示例。
 *
 * 典型用法：
 *   npx @earendil-works/pi-ai login              # 交互式选择提供商并登录
 *   npx @earendil-works/pi-ai login anthropic    # 直接登录到 Anthropic
 *   npx @earendil-works/pi-ai list               # 查看可用提供商
 */

import { createInterface } from "node:readline";
import { existsSync, readFileSync, writeFileSync } from "fs";
import { getOAuthProvider, getOAuthProviders } from "./utils/oauth/index.js";
import type { OAuthCredentials, OAuthProviderId } from "./utils/oauth/types.js";

// 认证信息存储文件名
const AUTH_FILE = "auth.json";
// 获取所有可用的 OAuth 提供商列表
const PROVIDERS = getOAuthProviders();

/**
 * 通过 readline 封装的用户输入提示函数
 * @param rl - readline 接口实例
 * @param question - 提示用户的问题文本
 * @returns 用户输入的字符串
 */
function prompt(rl: ReturnType<typeof createInterface>, question: string): Promise<string> {
	return new Promise((resolve) => rl.question(question, resolve));
}

/**
 * 从 auth.json 文件加载已保存的认证信息
 * @returns 按提供商 ID 索引的认证信息对象，文件不存在或解析失败时返回空对象
 */
function loadAuth(): Record<string, { type: "oauth" } & OAuthCredentials> {
	if (!existsSync(AUTH_FILE)) return {};
	try {
		return JSON.parse(readFileSync(AUTH_FILE, "utf-8"));
	} catch {
		return {};
	}
}

/**
 * 将认证信息保存到 auth.json 文件
 * @param auth - 要保存的认证信息对象
 */
function saveAuth(auth: Record<string, { type: "oauth" } & OAuthCredentials>): void {
	writeFileSync(AUTH_FILE, JSON.stringify(auth, null, 2), "utf-8");
}

/**
 * 执行指定 OAuth 提供商的登录流程
 * 流程：显示授权 URL -> 等待用户完成浏览器授权 -> 保存凭证到本地文件
 * @param providerId - OAuth 提供商的标识符
 */
async function login(providerId: OAuthProviderId): Promise<void> {
	const provider = getOAuthProvider(providerId);
	if (!provider) {
		console.error(`Unknown provider: ${providerId}`);
		process.exit(1);
	}

	// 创建命令行交互接口
	const rl = createInterface({ input: process.stdin, output: process.stdout });
	const promptFn = (msg: string) => prompt(rl, `${msg} `);

	try {
		// 调用提供商的登录方法，传入回调函数处理不同阶段的交互
		const credentials = await provider.login({
			// 当需要用户在浏览器中打开授权页面时触发
			onAuth: (info) => {
				console.log(`\nOpen this URL in your browser:\n${info.url}`);
				if (info.instructions) console.log(info.instructions);
				console.log();
			},
			// 当需要用户在命令行中输入信息时触发（如授权码）
			onPrompt: async (p) => {
				return await promptFn(`${p.message}${p.placeholder ? ` (${p.placeholder})` : ""}:`);
			},
			// 显示进度信息
			onProgress: (msg) => console.log(msg),
		});

		// 将新获取的凭证合并到已有认证信息中并保存
		const auth = loadAuth();
		auth[providerId] = { type: "oauth", ...credentials };
		saveAuth(auth);

		console.log(`\nCredentials saved to ${AUTH_FILE}`);
	} finally {
		rl.close();
	}
}

/**
 * CLI 主入口函数
 * 解析命令行参数并分发到对应的子命令处理逻辑
 */
async function main(): Promise<void> {
	const args = process.argv.slice(2);
	const command = args[0];

	// 显示帮助信息（无参数、help、--help、-h 均触发）
	if (!command || command === "help" || command === "--help" || command === "-h") {
		const providerList = PROVIDERS.map((p) => `  ${p.id.padEnd(20)} ${p.name}`).join("\n");
		console.log(`Usage: npx @earendil-works/pi-ai <command> [provider]

Commands:
  login [provider]  Login to an OAuth provider
  list              List available providers

Providers:
${providerList}

Examples:
  npx @earendil-works/pi-ai login              # interactive provider selection
  npx @earendil-works/pi-ai login anthropic    # login to specific provider
  npx @earendil-works/pi-ai list               # list providers
`);
		return;
	}

	// list 子命令：列出所有可用的 OAuth 提供商
	if (command === "list") {
		console.log("Available OAuth providers:\n");
		for (const p of PROVIDERS) {
			console.log(`  ${p.id.padEnd(20)} ${p.name}`);
		}
		return;
	}

	// login 子命令：登录到指定的 OAuth 提供商
	if (command === "login") {
		let provider = args[1] as OAuthProviderId | undefined;

		// 如果没有指定提供商，则进入交互式选择模式
		if (!provider) {
			const rl = createInterface({ input: process.stdin, output: process.stdout });
			console.log("Select a provider:\n");
			for (let i = 0; i < PROVIDERS.length; i++) {
				console.log(`  ${i + 1}. ${PROVIDERS[i].name}`);
			}
			console.log();

			const choice = await prompt(rl, `Enter number (1-${PROVIDERS.length}): `);
			rl.close();

			// 解析用户选择的编号
			const index = parseInt(choice, 10) - 1;
			if (index < 0 || index >= PROVIDERS.length) {
				console.error("Invalid selection");
				process.exit(1);
			}
			provider = PROVIDERS[index].id;
		}

		// 验证指定的提供商是否有效
		if (!PROVIDERS.some((p) => p.id === provider)) {
			console.error(`Unknown provider: ${provider}`);
			console.error(`Use 'npx @earendil-works/pi-ai list' to see available providers`);
			process.exit(1);
		}

		console.log(`Logging in to ${provider}...`);
		await login(provider);
		return;
	}

	// 未知命令处理
	console.error(`Unknown command: ${command}`);
	console.error(`Use 'npx @earendil-works/pi-ai --help' for usage`);
	process.exit(1);
}

// 启动 CLI 并捕获未处理的异常
main().catch((err) => {
	console.error("Error:", err.message);
	process.exit(1);
});
