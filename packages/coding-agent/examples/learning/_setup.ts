/**
 * @fileoverview 学习示例共享工具 — 基于 faux 提供商的 Agent 会话工厂。
 *
 * 本文件为所有学习示例提供统一的 faux 会话创建函数，
 * 避免在每个示例中重复注册 providers、配置认证等样板代码。
 * 所有示例均不使用真实 API Key，不发起真实 HTTP 请求。
 *
 * @module examples/learning/_setup
 */

import type { ThinkingLevel } from "@earendil-works/pi-agent-core";
import { type FauxProviderRegistration, registerFauxProvider } from "@earendil-works/pi-ai";
import {
	type AgentSession,
	AuthStorage,
	createAgentSession,
	DefaultResourceLoader,
	ModelRegistry,
	SessionManager,
	SettingsManager,
	type ToolDefinition,
} from "@earendil-works/pi-coding-agent";

export interface FauxSession {
	session: AgentSession;
	faux: FauxProviderRegistration;
	cleanup: () => void;
}

export interface CreateFauxSessionOptions {
	systemPrompt?: string;
	thinkingLevel?: ThinkingLevel;
	tools?: string[];
	customTools?: ToolDefinition[];
}

export async function createFauxSession(opts: CreateFauxSessionOptions = {}): Promise<FauxSession> {
	const faux = registerFauxProvider();
	faux.setResponses([]);
	const model = faux.getModel();

	const authStorage = AuthStorage.inMemory();
	authStorage.setRuntimeApiKey(model.provider, "faux-key");

	const modelRegistry = ModelRegistry.inMemory(authStorage);
	modelRegistry.registerProvider(model.provider, {
		baseUrl: model.baseUrl,
		apiKey: "faux-key",
		api: faux.api,
		models: faux.models.map((m) => ({
			id: m.id,
			name: m.name,
			api: m.api,
			reasoning: m.reasoning,
			input: m.input,
			cost: m.cost,
			contextWindow: m.contextWindow,
			maxTokens: m.maxTokens,
			baseUrl: m.baseUrl,
		})),
	});

	const resourceLoader = new DefaultResourceLoader({
		cwd: process.cwd(),
		agentDir: "/tmp/pi-learning-agent",
		noExtensions: true,
		noSkills: true,
		noPromptTemplates: true,
		noThemes: true,
		noContextFiles: true,
		systemPromptOverride: () => opts.systemPrompt ?? "你是一个乐于助人的编码助手，回答要简洁。",
	});
	await resourceLoader.reload();

	const { session } = await createAgentSession({
		model,
		authStorage,
		modelRegistry,
		resourceLoader,
		thinkingLevel: opts.thinkingLevel,
		tools: opts.tools,
		customTools: opts.customTools,
		sessionManager: SessionManager.inMemory(),
		settingsManager: SettingsManager.inMemory(),
	});

	return {
		session,
		faux,
		cleanup() {
			session.dispose();
			faux.unregister();
		},
	};
}
