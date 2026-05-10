/**
 * @fileoverview @earendil-works/pi-ai 包的公共 API 导出入口。
 *
 * 导出内容分类：
 *   - 核心类型：types.ts 中的所有类型（Api、Model、Message、Context、StreamFunction 等）
 *   - 流式调用：stream()、complete()、streamSimple()、completeSimple()
 *   - API 注册表：registerApiProvider()、getApiProvider() 等
 *   - 模型管理：getModel()、getProviders()、calculateCost() 等
 *   - 提供商选项类型：各提供商的特定选项接口（AnthropicOptions、OpenAICompletionsOptions 等）
 *   - OAuth 工具：OAuth 认证相关的类型和接口
 *   - 工具函数：JSON 解析、类型校验、溢出处理、诊断等
 */

export type { Static, TSchema } from "typebox";
export { Type } from "typebox";

// API 注册表
export * from "./api-registry.js";
// 环境变量 API 密钥检测
export * from "./env-api-keys.js";
// 图像生成模型定义
export * from "./image-models.js";
// 图像生成 API
export * from "./images.js";
// 图像生成 API 注册表
export * from "./images-api-registry.js";
// 模型查询与成本计算
export * from "./models.js";

// 各提供商的特定选项类型（仅导出类型，不导出运行时代码）
export type { BedrockOptions, BedrockThinkingDisplay } from "./providers/amazon-bedrock.js";
export type { AnthropicEffort, AnthropicOptions, AnthropicThinkingDisplay } from "./providers/anthropic.js";
export type { AzureOpenAIResponsesOptions } from "./providers/azure-openai-responses.js";
// 测试用 faux 提供商（包含运行时代码）
export * from "./providers/faux.js";
export type { GoogleOptions } from "./providers/google.js";
export type { GoogleThinkingLevel } from "./providers/google-shared.js";
export type { GoogleVertexOptions } from "./providers/google-vertex.js";
// 图像生成提供商注册
export * from "./providers/images/register-builtins.js";
export type { MistralOptions } from "./providers/mistral.js";
export type {
	OpenAICodexResponsesOptions,
	OpenAICodexWebSocketDebugStats,
} from "./providers/openai-codex-responses.js";
export type { OpenAICompletionsOptions } from "./providers/openai-completions.js";
export type { OpenAIResponsesOptions } from "./providers/openai-responses.js";
// 内置提供商注册（触发副作用注册）
export * from "./providers/register-builtins.js";
// 会话资源
export * from "./session-resources.js";
// 流式调用入口函数
export * from "./stream.js";
// 核心类型定义
export * from "./types.js";
// 诊断工具
export * from "./utils/diagnostics.js";
// 事件流实现
export * from "./utils/event-stream.js";
// JSON 安全解析
export * from "./utils/json-parse.js";
// OAuth 认证相关类型
export type {
	OAuthAuthInfo,
	OAuthCredentials,
	OAuthLoginCallbacks,
	OAuthPrompt,
	OAuthProvider,
	OAuthProviderId,
	OAuthProviderInfo,
	OAuthProviderInterface,
	OAuthSelectOption,
	OAuthSelectPrompt,
} from "./utils/oauth/types.js";
// 上下文溢出处理
export * from "./utils/overflow.js";
// TypeBox 辅助工具
export * from "./utils/typebox-helpers.js";
// 类型和 Schema 验证
export * from "./utils/validation.js";
