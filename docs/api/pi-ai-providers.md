## :toolbox: Functions

- [formatThrownValue](#gear-formatthrownvalue)
- [extractDiagnosticError](#gear-extractdiagnosticerror)
- [createAssistantMessageDiagnostic](#gear-createassistantmessagediagnostic)
- [appendAssistantMessageDiagnostic](#gear-appendassistantmessagediagnostic)
- [createAssistantMessageEventStream](#gear-createassistantmessageeventstream)
- [findEnvKeys](#gear-findenvkeys)
- [findEnvKeys](#gear-findenvkeys)
- [findEnvKeys](#gear-findenvkeys)
- [getEnvApiKey](#gear-getenvapikey)
- [getEnvApiKey](#gear-getenvapikey)
- [getEnvApiKey](#gear-getenvapikey)
- [getModel](#gear-getmodel)
- [getProviders](#gear-getproviders)
- [getModels](#gear-getmodels)
- [calculateCost](#gear-calculatecost)
- [getSupportedThinkingLevels](#gear-getsupportedthinkinglevels)
- [clampThinkingLevel](#gear-clampthinkinglevel)
- [modelsAreEqual](#gear-modelsareequal)
- [headersToRecord](#gear-headerstorecord)
- [repairJson](#gear-repairjson)
- [parseJsonWithRepair](#gear-parsejsonwithrepair)
- [parseStreamingJson](#gear-parsestreamingjson)
- [sanitizeSurrogates](#gear-sanitizesurrogates)
- [isCloudflareProvider](#gear-iscloudflareprovider)
- [resolveCloudflareBaseUrl](#gear-resolvecloudflarebaseurl)
- [inferCopilotInitiator](#gear-infercopilotinitiator)
- [hasCopilotVisionInput](#gear-hascopilotvisioninput)
- [buildCopilotDynamicHeaders](#gear-buildcopilotdynamicheaders)
- [buildBaseOptions](#gear-buildbaseoptions)
- [clampReasoning](#gear-clampreasoning)
- [adjustMaxTokensForThinking](#gear-adjustmaxtokensforthinking)
- [transformMessages](#gear-transformmessages)
- [streamAnthropic](#gear-streamanthropic)
- [streamSimpleAnthropic](#gear-streamsimpleanthropic)
- [streamOpenAICompletions](#gear-streamopenaicompletions)
- [streamSimpleOpenAICompletions](#gear-streamsimpleopenaicompletions)
- [convertMessages](#gear-convertmessages)
- [shortHash](#gear-shorthash)
- [convertResponsesMessages](#gear-convertresponsesmessages)
- [convertResponsesTools](#gear-convertresponsestools)
- [processResponsesStream](#gear-processresponsesstream)
- [streamOpenAIResponses](#gear-streamopenairesponses)
- [streamSimpleOpenAIResponses](#gear-streamsimpleopenairesponses)
- [isThinkingPart](#gear-isthinkingpart)
- [retainThoughtSignature](#gear-retainthoughtsignature)
- [requiresToolCallId](#gear-requirestoolcallid)
- [convertMessages](#gear-convertmessages)
- [convertTools](#gear-converttools)
- [mapToolChoice](#gear-maptoolchoice)
- [mapStopReason](#gear-mapstopreason)
- [mapStopReasonString](#gear-mapstopreasonstring)
- [streamGoogle](#gear-streamgoogle)
- [streamSimpleGoogle](#gear-streamsimplegoogle)
- [streamMistral](#gear-streammistral)
- [streamSimpleMistral](#gear-streamsimplemistral)
- [streamBedrock](#gear-streambedrock)
- [streamSimpleBedrock](#gear-streamsimplebedrock)
- [registerApiProvider](#gear-registerapiprovider)
- [getApiProvider](#gear-getapiprovider)
- [getApiProviders](#gear-getapiproviders)
- [unregisterApiProviders](#gear-unregisterapiproviders)
- [clearApiProviders](#gear-clearapiproviders)
- [fauxText](#gear-fauxtext)
- [fauxThinking](#gear-fauxthinking)
- [fauxToolCall](#gear-fauxtoolcall)
- [fauxAssistantMessage](#gear-fauxassistantmessage)
- [registerFauxProvider](#gear-registerfauxprovider)
- [streamAzureOpenAIResponses](#gear-streamazureopenairesponses)
- [streamSimpleAzureOpenAIResponses](#gear-streamsimpleazureopenairesponses)
- [streamGoogleVertex](#gear-streamgooglevertex)
- [streamSimpleGoogleVertex](#gear-streamsimplegooglevertex)
- [registerSessionResourceCleanup](#gear-registersessionresourcecleanup)
- [cleanupSessionResources](#gear-cleanupsessionresources)
- [streamOpenAICodexResponses](#gear-streamopenaicodexresponses)
- [streamSimpleOpenAICodexResponses](#gear-streamsimpleopenaicodexresponses)
- [getOpenAICodexWebSocketDebugStats](#gear-getopenaicodexwebsocketdebugstats)
- [resetOpenAICodexWebSocketDebugStats](#gear-resetopenaicodexwebsocketdebugstats)
- [closeOpenAICodexWebSocketSessions](#gear-closeopenaicodexwebsocketsessions)
- [setBedrockProviderModule](#gear-setbedrockprovidermodule)
- [registerBuiltInApiProviders](#gear-registerbuiltinapiproviders)
- [resetApiProviders](#gear-resetapiproviders)

### :gear: formatThrownValue

| Function | Type |
| ---------- | ---------- |
| `formatThrownValue` | `(value: unknown) => string` |

### :gear: extractDiagnosticError

| Function | Type |
| ---------- | ---------- |
| `extractDiagnosticError` | `(error: unknown) => DiagnosticErrorInfo` |

### :gear: createAssistantMessageDiagnostic

| Function | Type |
| ---------- | ---------- |
| `createAssistantMessageDiagnostic` | `(type: string, error: unknown, details?: Record<string, unknown> or undefined) => AssistantMessageDiagnostic` |

### :gear: appendAssistantMessageDiagnostic

| Function | Type |
| ---------- | ---------- |
| `appendAssistantMessageDiagnostic` | `<T extends { diagnostics?: AssistantMessageDiagnostic[]; }>(message: T, diagnostic: AssistantMessageDiagnostic) => void` |

### :gear: createAssistantMessageEventStream

Factory function for AssistantMessageEventStream (for use in extensions)

| Function | Type |
| ---------- | ---------- |
| `createAssistantMessageEventStream` | `() => AssistantMessageEventStream` |

### :gear: findEnvKeys

Find configured environment variables that can provide an API key for a provider.

This only reports actual API key variables. It intentionally excludes ambient
credential sources such as AWS profiles, AWS IAM credentials, and Google
Application Default Credentials.

| Function | Type |
| ---------- | ---------- |
| `findEnvKeys` | `{ (provider: KnownProvider): string[] or undefined; (provider: string): string[] or undefined; }` |

### :gear: findEnvKeys

Find configured environment variables that can provide an API key for a provider.

This only reports actual API key variables. It intentionally excludes ambient
credential sources such as AWS profiles, AWS IAM credentials, and Google
Application Default Credentials.

| Function | Type |
| ---------- | ---------- |
| `findEnvKeys` | `{ (provider: KnownProvider): string[] or undefined; (provider: string): string[] or undefined; }` |

### :gear: findEnvKeys

Find configured environment variables that can provide an API key for a provider.

This only reports actual API key variables. It intentionally excludes ambient
credential sources such as AWS profiles, AWS IAM credentials, and Google
Application Default Credentials.

| Function | Type |
| ---------- | ---------- |
| `findEnvKeys` | `{ (provider: KnownProvider): string[] or undefined; (provider: string): string[] or undefined; }` |

### :gear: getEnvApiKey

Get API key for provider from known environment variables, e.g. OPENAI_API_KEY.

Will not return API keys for providers that require OAuth tokens.

| Function | Type |
| ---------- | ---------- |
| `getEnvApiKey` | `{ (provider: KnownProvider): string or undefined; (provider: string): string or undefined; }` |

### :gear: getEnvApiKey

Get API key for provider from known environment variables, e.g. OPENAI_API_KEY.

Will not return API keys for providers that require OAuth tokens.

| Function | Type |
| ---------- | ---------- |
| `getEnvApiKey` | `{ (provider: KnownProvider): string or undefined; (provider: string): string or undefined; }` |

### :gear: getEnvApiKey

Get API key for provider from known environment variables, e.g. OPENAI_API_KEY.

Will not return API keys for providers that require OAuth tokens.

| Function | Type |
| ---------- | ---------- |
| `getEnvApiKey` | `{ (provider: KnownProvider): string or undefined; (provider: string): string or undefined; }` |

### :gear: getModel

按提供商和模型 ID 获取模型定义

| Function | Type |
| ---------- | ---------- |
| `getModel` | `<TProvider extends KnownProvider, TModelId extends keyof (typeof MODELS)[TProvider]>(provider: TProvider, modelId: TModelId) => Model<ModelApi<TProvider, TModelId>>` |

Parameters:

* `provider`: - 提供商 ID
* `modelId`: - 模型 ID


Returns:

模型定义对象，支持完整的类型推断

### :gear: getProviders

获取所有已注册的提供商列表

| Function | Type |
| ---------- | ---------- |
| `getProviders` | `() => KnownProvider[]` |

Returns:

提供商 ID 数组

### :gear: getModels

获取指定提供商的所有模型

| Function | Type |
| ---------- | ---------- |
| `getModels` | `<TProvider extends KnownProvider>(provider: TProvider) => Model<ModelApi<TProvider, keyof { readonly "amazon-bedrock": { readonly "amazon.nova-2-lite-v1:0": { id: string; name: string; api: "bedrock-converse-stream"; ... 6 more ...; maxTokens: number; }; ... 91 more ...; readonly "zai.glm-5": { ...; }; }; ... 30 mor...` |

Parameters:

* `provider`: - 提供商 ID


Returns:

该提供商下所有模型的数组

### :gear: calculateCost

根据 Token 用量和模型费率计算实际成本
费率单位为美元/百万 Token，计算结果直接写入 usage.cost

| Function | Type |
| ---------- | ---------- |
| `calculateCost` | `<TApi extends Api>(model: Model<TApi>, usage: Usage) => { input: number; output: number; cacheRead: number; cacheWrite: number; total: number; }` |

Parameters:

* `model`: - 模型定义（含费率信息）
* `usage`: - Token 用量统计，计算结果会修改其 cost 字段


Returns:

成本明细

### :gear: getSupportedThinkingLevels

获取模型支持的思考级别列表
不支持推理的模型仅返回 ["off"]；支持推理的模型根据 thinkingLevelMap 过滤

| Function | Type |
| ---------- | ---------- |
| `getSupportedThinkingLevels` | `<TApi extends Api>(model: Model<TApi>) => ModelThinkingLevel[]` |

Parameters:

* `model`: - 目标模型


Returns:

该模型支持的思考级别数组

### :gear: clampThinkingLevel

将请求的思考级别限制到模型支持的范围内
如果请求的级别不被支持，优先向上查找更高的级别，其次向下回退

| Function | Type |
| ---------- | ---------- |
| `clampThinkingLevel` | `<TApi extends Api>(model: Model<TApi>, level: ModelThinkingLevel) => ModelThinkingLevel` |

Parameters:

* `model`: - 目标模型
* `level`: - 请求的思考级别


Returns:

调整后的有效思考级别

### :gear: modelsAreEqual

比较两个模型是否相同（基于 id 和 provider 双重判断）
任一参数为 null/undefined 时返回 false

| Function | Type |
| ---------- | ---------- |
| `modelsAreEqual` | `<TApi extends Api>(a: Model<TApi> or null or undefined, b: Model<TApi> or null or undefined) => boolean` |

Parameters:

* `a`: - 第一个模型
* `b`: - 第二个模型


Returns:

两个模型是否相同

### :gear: headersToRecord

| Function | Type |
| ---------- | ---------- |
| `headersToRecord` | `(headers: Headers) => Record<string, string>` |

### :gear: repairJson

Repairs malformed JSON string literals by:
- escaping raw control characters inside strings
- doubling backslashes before invalid escape characters

| Function | Type |
| ---------- | ---------- |
| `repairJson` | `(json: string) => string` |

### :gear: parseJsonWithRepair

| Function | Type |
| ---------- | ---------- |
| `parseJsonWithRepair` | `<T>(json: string) => T` |

### :gear: parseStreamingJson

Attempts to parse potentially incomplete JSON during streaming.
Always returns a valid object, even if the JSON is incomplete.

| Function | Type |
| ---------- | ---------- |
| `parseStreamingJson` | `<T = Record<string, unknown>>(partialJson: string or undefined) => T` |

Parameters:

* `partialJson`: The partial JSON string from streaming


Returns:

Parsed object or empty object if parsing fails

### :gear: sanitizeSurrogates

Removes unpaired Unicode surrogate characters from a string.

Unpaired surrogates (high surrogates 0xD800-0xDBFF without matching low surrogates 0xDC00-0xDFFF,
or vice versa) cause JSON serialization errors in many API providers.

Valid emoji and other characters outside the Basic Multilingual Plane use properly paired
surrogates and will NOT be affected by this function.

| Function | Type |
| ---------- | ---------- |
| `sanitizeSurrogates` | `(text: string) => string` |

Parameters:

* `text`: - The text to sanitize


Returns:

The sanitized text with unpaired surrogates removed

Examples:

// Valid emoji (properly paired surrogates) are preserved
sanitizeSurrogates("Hello 🙈 World") // => "Hello 🙈 World"

// Unpaired high surrogate is removed
const unpaired = String.fromCharCode(0xD83D); // high surrogate without low
sanitizeSurrogates(`Text ${unpaired} here`) // => "Text  here"


### :gear: isCloudflareProvider

| Function | Type |
| ---------- | ---------- |
| `isCloudflareProvider` | `(provider: string) => boolean` |

### :gear: resolveCloudflareBaseUrl

Substitute `{VAR}` placeholders in a Cloudflare baseUrl from process.env.

| Function | Type |
| ---------- | ---------- |
| `resolveCloudflareBaseUrl` | `(model: Model<Api>) => string` |

### :gear: inferCopilotInitiator

| Function | Type |
| ---------- | ---------- |
| `inferCopilotInitiator` | `(messages: Message[]) => "user" or "agent"` |

### :gear: hasCopilotVisionInput

| Function | Type |
| ---------- | ---------- |
| `hasCopilotVisionInput` | `(messages: Message[]) => boolean` |

### :gear: buildCopilotDynamicHeaders

| Function | Type |
| ---------- | ---------- |
| `buildCopilotDynamicHeaders` | `(params: { messages: Message[]; hasImages: boolean; }) => Record<string, string>` |

### :gear: buildBaseOptions

将简化流式选项转换为标准流式选项

映射规则：
  - maxTokens 默认取模型上限与 32000 的较小值，防止过大的输出请求
  - apiKey 优先使用显式传入的参数，其次使用 options 中的值
  - 其余字段直接透传

| Function | Type |
| ---------- | ---------- |
| `buildBaseOptions` | `(model: Model<Api>, options?: SimpleStreamOptions or undefined, apiKey?: string or undefined) => StreamOptions` |

Parameters:

* `model`: - 当前使用的模型（用于获取 maxTokens 上限）
* `options`: - 简化流式选项（可选）
* `apiKey`: - 外部提供的 API 密钥（优先级高于 options.apiKey）


Returns:

完整的标准流式选项对象

### :gear: clampReasoning

将 xhigh 思考级别降级为 high

仅部分模型家族支持 xhigh 级别，此函数用于不支持的提供商中将
xhigh 安全降级为 high，避免请求被拒绝。

| Function | Type |
| ---------- | ---------- |
| `clampReasoning` | `(effort: ThinkingLevel or undefined) => "minimal" or "low" or "medium" or "high" or undefined` |

Parameters:

* `effort`: - 原始思考级别


Returns:

降级后的思考级别（xhigh → high，其余不变）

### :gear: adjustMaxTokensForThinking

在启用思考模式时调整输出 Token 预算

思考模式的 Token 消耗分为两部分：思考过程（内部推理）和最终输出。
此函数根据思考级别分配预算，并在总预算超出模型上限时缩减思考预算，
确保至少保留 minOutputTokens 给最终输出。

计算方式：
  1. 根据思考级别确定默认思考预算（如 high = 16384 tokens）
  2. 总预算 = 基础输出 + 思考预算，但不超过模型上限
  3. 如果总预算不够，优先保证输出空间，缩减思考预算

| Function | Type |
| ---------- | ---------- |
| `adjustMaxTokensForThinking` | `(baseMaxTokens: number, modelMaxTokens: number, reasoningLevel: ThinkingLevel, customBudgets?: ThinkingBudgets or undefined) => { maxTokens: number; thinkingBudget: number; }` |

Parameters:

* `baseMaxTokens`: - 基础输出 Token 上限
* `modelMaxTokens`: - 模型允许的最大输出 Token 数
* `reasoningLevel`: - 当前思考级别
* `customBudgets`: - 自定义思考预算（覆盖默认值）


Returns:

调整后的 maxTokens 和 thinkingBudget

### :gear: transformMessages

消息转换主入口 — 对对话历史进行跨提供商兼容性处理

处理步骤：
  1. 图片降级：不支持视觉的模型，图片替换为占位文本
  2. 思考块处理：跨模型时，加密思考内容被移除；明文思考转为普通文本
  3. 工具调用 ID 规范化：跨提供商时，通过 normalizeToolCallId 回调统一 ID 格式
  4. 孤儿工具调用修复：为缺少对应结果的工具调用插入合成错误结果
  5. 错误/中止消息过滤：跳过 stopReason 为 error 或 aborted 的助手消息

| Function | Type |
| ---------- | ---------- |
| `transformMessages` | `<TApi extends Api>(messages: Message[], model: Model<TApi>, normalizeToolCallId?: ((id: string, model: Model<TApi>, source: AssistantMessage) => string) or undefined) => Message[]` |

Parameters:

* `messages`: - 原始消息列表
* `model`: - 目标模型（决定转换策略）
* `normalizeToolCallId`: - 可选的工具调用 ID 规范化函数
不同提供商对工具调用 ID 的格式有不同要求：
- OpenAI Responses API 生成的 ID 可达 450+ 字符且含特殊字符
- Anthropic 要求 ID 匹配 ^[a-zA-Z0-9_-]+$ 且最长 64 字符
通过此回调函数进行格式转换，确保跨提供商兼容


Returns:

转换后的消息列表，可直接传递给 LLM API

### :gear: streamAnthropic

| Function | Type |
| ---------- | ---------- |
| `streamAnthropic` | `StreamFunction<"anthropic-messages", AnthropicOptions>` |

### :gear: streamSimpleAnthropic

| Function | Type |
| ---------- | ---------- |
| `streamSimpleAnthropic` | `StreamFunction<"anthropic-messages", SimpleStreamOptions>` |

### :gear: streamOpenAICompletions

标准流式调用 — 向 OpenAI Completions API 发起流式请求

完整的 SSE 流式响应处理流程：
  1. 创建 OpenAI 客户端，配置 API Key、Base URL 和请求头
  2. 构建请求参数（消息转换、工具转换、思考模式配置等）
  3. 发起流式请求，逐块解析 SSE 事件
  4. 维护内容块状态机（文本块、思考块、工具调用块）
  5. 发出标准化事件到 AssistantMessageEventStream

错误处理：捕获所有异常，将错误信息写入助手消息并发出 error 事件。

| Function | Type |
| ---------- | ---------- |
| `streamOpenAICompletions` | `StreamFunction<"openai-completions", OpenAICompletionsOptions>` |

### :gear: streamSimpleOpenAICompletions

简化流式调用 — 将 SimpleStreamOptions 映射为 OpenAICompletionsOptions

处理思考级别的钳位（clamp）和工具选择参数的透传，
然后委托给 streamOpenAICompletions。

| Function | Type |
| ---------- | ---------- |
| `streamSimpleOpenAICompletions` | `StreamFunction<"openai-completions", SimpleStreamOptions>` |

### :gear: convertMessages

将内部消息格式转换为 OpenAI Chat Completions API 的消息格式

转换规则：
  - 系统提示：根据 compat 决定使用 system 还是 developer 角色
  - 用户消息：文本直接传递，图片转为 image_url 内容块
  - 助手消息：文本、思考块、工具调用分别转换
  - 工具结果：转为 tool 角色，图片作为附加用户消息发送
  - 特殊处理：空助手消息跳过、工具调用 ID 规范化

| Function | Type |
| ---------- | ---------- |
| `convertMessages` | `(model: Model<"openai-completions">, context: Context, compat: ResolvedOpenAICompletionsCompat) => ChatCompletionMessageParam[]` |

Parameters:

* `model`: - 目标模型
* `context`: - 包含消息和工具的对话上下文
* `compat`: - 已解析的兼容性设置


Returns:

OpenAI Chat Completions 格式的消息数组

### :gear: shortHash

Fast deterministic hash to shorten long strings

| Function | Type |
| ---------- | ---------- |
| `shortHash` | `(str: string) => string` |

### :gear: convertResponsesMessages

| Function | Type |
| ---------- | ---------- |
| `convertResponsesMessages` | `<TApi extends Api>(model: Model<TApi>, context: Context, allowedToolCallProviders: ReadonlySet<string>, options?: ConvertResponsesMessagesOptions or undefined) => ResponseInput` |

### :gear: convertResponsesTools

| Function | Type |
| ---------- | ---------- |
| `convertResponsesTools` | `(tools: Tool<TSchema>[], options?: ConvertResponsesToolsOptions or undefined) => Tool[]` |

### :gear: processResponsesStream

| Function | Type |
| ---------- | ---------- |
| `processResponsesStream` | `<TApi extends Api>(openaiStream: AsyncIterable<ResponseStreamEvent>, output: AssistantMessage, stream: AssistantMessageEventStream, model: Model<...>, options?: OpenAIResponsesStreamOptions or undefined) => Promise<...>` |

### :gear: streamOpenAIResponses

Generate function for OpenAI Responses API

| Function | Type |
| ---------- | ---------- |
| `streamOpenAIResponses` | `StreamFunction<"openai-responses", OpenAIResponsesOptions>` |

### :gear: streamSimpleOpenAIResponses

| Function | Type |
| ---------- | ---------- |
| `streamSimpleOpenAIResponses` | `StreamFunction<"openai-responses", SimpleStreamOptions>` |

### :gear: isThinkingPart

Determines whether a streamed Gemini `Part` should be treated as "thinking".

Protocol note (Gemini / Vertex AI thought signatures):
- `thought: true` is the definitive marker for thinking content (thought summaries).
- `thoughtSignature` is an encrypted representation of the model's internal thought process
  used to preserve reasoning context across multi-turn interactions.
- `thoughtSignature` can appear on ANY part type (text, functionCall, etc.) - it does NOT
  indicate the part itself is thinking content.
- For non-functionCall responses, the signature appears on the last part for context replay.
- When persisting/replaying model outputs, signature-bearing parts must be preserved as-is;
  do not merge/move signatures across parts.

See: https://ai.google.dev/gemini-api/docs/thought-signatures

| Function | Type |
| ---------- | ---------- |
| `isThinkingPart` | `(part: Pick<Part, "thought" or "thoughtSignature">) => boolean` |

### :gear: retainThoughtSignature

Retain thought signatures during streaming.

Some backends only send `thoughtSignature` on the first delta for a given part/block; later deltas may omit it.
This helper preserves the last non-empty signature for the current block.

Note: this does NOT merge or move signatures across distinct response parts. It only prevents
a signature from being overwritten with `undefined` within the same streamed block.

| Function | Type |
| ---------- | ---------- |
| `retainThoughtSignature` | `(existing: string or undefined, incoming: string or undefined) => string or undefined` |

### :gear: requiresToolCallId

Models via Google APIs that require explicit tool call IDs in function calls/responses.

| Function | Type |
| ---------- | ---------- |
| `requiresToolCallId` | `(modelId: string) => boolean` |

### :gear: convertMessages

Convert internal messages to Gemini Content[] format.

| Function | Type |
| ---------- | ---------- |
| `convertMessages` | `<T extends GoogleApiType>(model: Model<T>, context: Context) => Content[]` |

### :gear: convertTools

Convert tools to Gemini function declarations format.

By default uses `parametersJsonSchema` which supports full JSON Schema (including
anyOf, oneOf, const, etc.). Set `useParameters` to true to use the legacy `parameters`
field instead (OpenAPI 3.03 Schema). This is needed for Cloud Code Assist with Claude
models, where the API translates `parameters` into Anthropic's `input_schema`.

| Function | Type |
| ---------- | ---------- |
| `convertTools` | `(tools: Tool<TSchema>[], useParameters?: boolean) => { functionDeclarations: Record<string, unknown>[]; }[] or undefined` |

### :gear: mapToolChoice

Map tool choice string to Gemini FunctionCallingConfigMode.

| Function | Type |
| ---------- | ---------- |
| `mapToolChoice` | `(choice: string) => FunctionCallingConfigMode` |

### :gear: mapStopReason

Map Gemini FinishReason to our StopReason.

| Function | Type |
| ---------- | ---------- |
| `mapStopReason` | `(reason: FinishReason) => StopReason` |

### :gear: mapStopReasonString

Map string finish reason to our StopReason (for raw API responses).

| Function | Type |
| ---------- | ---------- |
| `mapStopReasonString` | `(reason: string) => StopReason` |

### :gear: streamGoogle

| Function | Type |
| ---------- | ---------- |
| `streamGoogle` | `StreamFunction<"google-generative-ai", GoogleOptions>` |

### :gear: streamSimpleGoogle

| Function | Type |
| ---------- | ---------- |
| `streamSimpleGoogle` | `StreamFunction<"google-generative-ai", SimpleStreamOptions>` |

### :gear: streamMistral

Stream responses from Mistral using `chat.stream`.

| Function | Type |
| ---------- | ---------- |
| `streamMistral` | `StreamFunction<"mistral-conversations", MistralOptions>` |

### :gear: streamSimpleMistral

Maps provider-agnostic `SimpleStreamOptions` to Mistral options.

| Function | Type |
| ---------- | ---------- |
| `streamSimpleMistral` | `StreamFunction<"mistral-conversations", SimpleStreamOptions>` |

### :gear: streamBedrock

| Function | Type |
| ---------- | ---------- |
| `streamBedrock` | `StreamFunction<"bedrock-converse-stream", BedrockOptions>` |

### :gear: streamSimpleBedrock

| Function | Type |
| ---------- | ---------- |
| `streamSimpleBedrock` | `StreamFunction<"bedrock-converse-stream", SimpleStreamOptions>` |

### :gear: registerApiProvider

注册一个 API 提供商到全局注册表

| Function | Type |
| ---------- | ---------- |
| `registerApiProvider` | `<TApi extends Api, TOptions extends StreamOptions>(provider: ApiProvider<TApi, TOptions>, sourceId?: string or undefined) => void` |

Parameters:

* `provider`: - 提供商实例，包含 API 标识符和流式函数
* `sourceId`: - 可选的来源标识，用于后续按来源批量注销


### :gear: getApiProvider

按 API 标识符查询已注册的提供商

| Function | Type |
| ---------- | ---------- |
| `getApiProvider` | `(api: Api) => ApiProviderInternal or undefined` |

Parameters:

* `api`: - API 标识符


Returns:

对应的提供商内部结构，未注册时返回 undefined

### :gear: getApiProviders

获取全部已注册的提供商列表

| Function | Type |
| ---------- | ---------- |
| `getApiProviders` | `() => ApiProviderInternal[]` |

Returns:

提供商内部结构数组

### :gear: unregisterApiProviders

按来源标识注销所有匹配的 API 提供商

| Function | Type |
| ---------- | ---------- |
| `unregisterApiProviders` | `(sourceId: string) => void` |

Parameters:

* `sourceId`: - 来源标识，注册时传入的 sourceId 相同的条目将被移除


### :gear: clearApiProviders

清空全部已注册的 API 提供商

| Function | Type |
| ---------- | ---------- |
| `clearApiProviders` | `() => void` |

### :gear: fauxText

| Function | Type |
| ---------- | ---------- |
| `fauxText` | `(text: string) => TextContent` |

### :gear: fauxThinking

| Function | Type |
| ---------- | ---------- |
| `fauxThinking` | `(thinking: string) => ThinkingContent` |

### :gear: fauxToolCall

| Function | Type |
| ---------- | ---------- |
| `fauxToolCall` | `(name: string, arguments_: Record<string, any>, options?: { id?: string or undefined; }) => ToolCall` |

### :gear: fauxAssistantMessage

| Function | Type |
| ---------- | ---------- |
| `fauxAssistantMessage` | `(content: string or FauxContentBlock or FauxContentBlock[], options?: { stopReason?: StopReason or undefined; errorMessage?: string or undefined; responseId?: string or undefined; timestamp?: number or undefined; }) => AssistantMessage` |

### :gear: registerFauxProvider

| Function | Type |
| ---------- | ---------- |
| `registerFauxProvider` | `(options?: RegisterFauxProviderOptions) => FauxProviderRegistration` |

### :gear: streamAzureOpenAIResponses

Generate function for Azure OpenAI Responses API

| Function | Type |
| ---------- | ---------- |
| `streamAzureOpenAIResponses` | `StreamFunction<"azure-openai-responses", AzureOpenAIResponsesOptions>` |

### :gear: streamSimpleAzureOpenAIResponses

| Function | Type |
| ---------- | ---------- |
| `streamSimpleAzureOpenAIResponses` | `StreamFunction<"azure-openai-responses", SimpleStreamOptions>` |

### :gear: streamGoogleVertex

| Function | Type |
| ---------- | ---------- |
| `streamGoogleVertex` | `StreamFunction<"google-vertex", GoogleVertexOptions>` |

### :gear: streamSimpleGoogleVertex

| Function | Type |
| ---------- | ---------- |
| `streamSimpleGoogleVertex` | `StreamFunction<"google-vertex", SimpleStreamOptions>` |

### :gear: registerSessionResourceCleanup

| Function | Type |
| ---------- | ---------- |
| `registerSessionResourceCleanup` | `(cleanup: SessionResourceCleanup) => () => void` |

### :gear: cleanupSessionResources

| Function | Type |
| ---------- | ---------- |
| `cleanupSessionResources` | `(sessionId?: string or undefined) => void` |

### :gear: streamOpenAICodexResponses

| Function | Type |
| ---------- | ---------- |
| `streamOpenAICodexResponses` | `StreamFunction<"openai-codex-responses", OpenAICodexResponsesOptions>` |

### :gear: streamSimpleOpenAICodexResponses

| Function | Type |
| ---------- | ---------- |
| `streamSimpleOpenAICodexResponses` | `StreamFunction<"openai-codex-responses", SimpleStreamOptions>` |

### :gear: getOpenAICodexWebSocketDebugStats

| Function | Type |
| ---------- | ---------- |
| `getOpenAICodexWebSocketDebugStats` | `(sessionId: string) => OpenAICodexWebSocketDebugStats or undefined` |

### :gear: resetOpenAICodexWebSocketDebugStats

| Function | Type |
| ---------- | ---------- |
| `resetOpenAICodexWebSocketDebugStats` | `(sessionId?: string or undefined) => void` |

### :gear: closeOpenAICodexWebSocketSessions

| Function | Type |
| ---------- | ---------- |
| `closeOpenAICodexWebSocketSessions` | `(sessionId?: string or undefined) => void` |

### :gear: setBedrockProviderModule

替换 Bedrock 提供商的模块实现

用于浏览器等无法使用 Node.js 动态 import 的环境，
允许外部直接注入已加载的模块。

| Function | Type |
| ---------- | ---------- |
| `setBedrockProviderModule` | `(module: BedrockProviderModule) => void` |

Parameters:

* `module`: - Bedrock 提供商模块


### :gear: registerBuiltInApiProviders

注册所有内置 API 提供商到全局注册表

将 9 个内置提供商按其 API 标识符注册，每个提供商提供
标准流式函数和简化流式函数。此函数在模块加载时自动调用一次，
也可通过 resetApiProviders 重新调用。

| Function | Type |
| ---------- | ---------- |
| `registerBuiltInApiProviders` | `() => void` |

### :gear: resetApiProviders

重置 API 注册表：清空所有已注册提供商，然后重新注册内置提供商

用于测试环境或需要恢复默认状态时调用。

| Function | Type |
| ---------- | ---------- |
| `resetApiProviders` | `() => void` |


## :wrench: Constants

- [MODELS](#gear-models)
- [CLOUDFLARE_WORKERS_AI_BASE_URL](#gear-cloudflare_workers_ai_base_url)
- [CLOUDFLARE_AI_GATEWAY_COMPAT_BASE_URL](#gear-cloudflare_ai_gateway_compat_base_url)
- [CLOUDFLARE_AI_GATEWAY_OPENAI_BASE_URL](#gear-cloudflare_ai_gateway_openai_base_url)
- [CLOUDFLARE_AI_GATEWAY_ANTHROPIC_BASE_URL](#gear-cloudflare_ai_gateway_anthropic_base_url)
- [streamAnthropic](#gear-streamanthropic)
- [streamSimpleAnthropic](#gear-streamsimpleanthropic)
- [streamAzureOpenAIResponses](#gear-streamazureopenairesponses)
- [streamSimpleAzureOpenAIResponses](#gear-streamsimpleazureopenairesponses)
- [streamGoogle](#gear-streamgoogle)
- [streamSimpleGoogle](#gear-streamsimplegoogle)
- [streamGoogleVertex](#gear-streamgooglevertex)
- [streamSimpleGoogleVertex](#gear-streamsimplegooglevertex)
- [streamMistral](#gear-streammistral)
- [streamSimpleMistral](#gear-streamsimplemistral)
- [streamOpenAICodexResponses](#gear-streamopenaicodexresponses)
- [streamSimpleOpenAICodexResponses](#gear-streamsimpleopenaicodexresponses)
- [streamOpenAICompletions](#gear-streamopenaicompletions)
- [streamSimpleOpenAICompletions](#gear-streamsimpleopenaicompletions)
- [streamOpenAIResponses](#gear-streamopenairesponses)
- [streamSimpleOpenAIResponses](#gear-streamsimpleopenairesponses)

### :gear: MODELS

| Constant | Type |
| ---------- | ---------- |
| `MODELS` | `{ readonly "amazon-bedrock": { readonly "amazon.nova-2-lite-v1:0": { id: string; name: string; api: "bedrock-converse-stream"; provider: string; baseUrl: string; reasoning: false; input: ("text" or "image")[]; cost: { input: number; output: number; cacheRead: number; cacheWrite: number; }; contextWindow: number; maxT...` |

### :gear: CLOUDFLARE_WORKERS_AI_BASE_URL

Workers AI direct endpoint.

| Constant | Type |
| ---------- | ---------- |
| `CLOUDFLARE_WORKERS_AI_BASE_URL` | `"https://api.cloudflare.com/client/v4/accounts/{CLOUDFLARE_ACCOUNT_ID}/ai/v1"` |

### :gear: CLOUDFLARE_AI_GATEWAY_COMPAT_BASE_URL

AI Gateway Unified API. https://developers.cloudflare.com/ai-gateway/usage/unified-api/

| Constant | Type |
| ---------- | ---------- |
| `CLOUDFLARE_AI_GATEWAY_COMPAT_BASE_URL` | `"https://gateway.ai.cloudflare.com/v1/{CLOUDFLARE_ACCOUNT_ID}/{CLOUDFLARE_GATEWAY_ID}/compat"` |

### :gear: CLOUDFLARE_AI_GATEWAY_OPENAI_BASE_URL

AI Gateway → OpenAI passthrough. Used until /compat supports /v1/responses.

| Constant | Type |
| ---------- | ---------- |
| `CLOUDFLARE_AI_GATEWAY_OPENAI_BASE_URL` | `"https://gateway.ai.cloudflare.com/v1/{CLOUDFLARE_ACCOUNT_ID}/{CLOUDFLARE_GATEWAY_ID}/openai"` |

### :gear: CLOUDFLARE_AI_GATEWAY_ANTHROPIC_BASE_URL

AI Gateway → Anthropic passthrough.

| Constant | Type |
| ---------- | ---------- |
| `CLOUDFLARE_AI_GATEWAY_ANTHROPIC_BASE_URL` | `"https://gateway.ai.cloudflare.com/v1/{CLOUDFLARE_ACCOUNT_ID}/{CLOUDFLARE_GATEWAY_ID}/anthropic"` |

### :gear: streamAnthropic

| Constant | Type |
| ---------- | ---------- |
| `streamAnthropic` | `StreamFunction<"anthropic-messages", AnthropicOptions>` |

### :gear: streamSimpleAnthropic

| Constant | Type |
| ---------- | ---------- |
| `streamSimpleAnthropic` | `StreamFunction<"anthropic-messages", SimpleStreamOptions>` |

### :gear: streamAzureOpenAIResponses

| Constant | Type |
| ---------- | ---------- |
| `streamAzureOpenAIResponses` | `StreamFunction<"azure-openai-responses", AzureOpenAIResponsesOptions>` |

### :gear: streamSimpleAzureOpenAIResponses

| Constant | Type |
| ---------- | ---------- |
| `streamSimpleAzureOpenAIResponses` | `StreamFunction<"azure-openai-responses", SimpleStreamOptions>` |

### :gear: streamGoogle

| Constant | Type |
| ---------- | ---------- |
| `streamGoogle` | `StreamFunction<"google-generative-ai", GoogleOptions>` |

### :gear: streamSimpleGoogle

| Constant | Type |
| ---------- | ---------- |
| `streamSimpleGoogle` | `StreamFunction<"google-generative-ai", SimpleStreamOptions>` |

### :gear: streamGoogleVertex

| Constant | Type |
| ---------- | ---------- |
| `streamGoogleVertex` | `StreamFunction<"google-vertex", GoogleVertexOptions>` |

### :gear: streamSimpleGoogleVertex

| Constant | Type |
| ---------- | ---------- |
| `streamSimpleGoogleVertex` | `StreamFunction<"google-vertex", SimpleStreamOptions>` |

### :gear: streamMistral

| Constant | Type |
| ---------- | ---------- |
| `streamMistral` | `StreamFunction<"mistral-conversations", MistralOptions>` |

### :gear: streamSimpleMistral

| Constant | Type |
| ---------- | ---------- |
| `streamSimpleMistral` | `StreamFunction<"mistral-conversations", SimpleStreamOptions>` |

### :gear: streamOpenAICodexResponses

| Constant | Type |
| ---------- | ---------- |
| `streamOpenAICodexResponses` | `StreamFunction<"openai-codex-responses", OpenAICodexResponsesOptions>` |

### :gear: streamSimpleOpenAICodexResponses

| Constant | Type |
| ---------- | ---------- |
| `streamSimpleOpenAICodexResponses` | `StreamFunction<"openai-codex-responses", SimpleStreamOptions>` |

### :gear: streamOpenAICompletions

| Constant | Type |
| ---------- | ---------- |
| `streamOpenAICompletions` | `StreamFunction<"openai-completions", OpenAICompletionsOptions>` |

### :gear: streamSimpleOpenAICompletions

| Constant | Type |
| ---------- | ---------- |
| `streamSimpleOpenAICompletions` | `StreamFunction<"openai-completions", SimpleStreamOptions>` |

### :gear: streamOpenAIResponses

| Constant | Type |
| ---------- | ---------- |
| `streamOpenAIResponses` | `StreamFunction<"openai-responses", OpenAIResponsesOptions>` |

### :gear: streamSimpleOpenAIResponses

| Constant | Type |
| ---------- | ---------- |
| `streamSimpleOpenAIResponses` | `StreamFunction<"openai-responses", SimpleStreamOptions>` |


## :factory: EventStream

### Methods

- [push](#gear-push)
- [end](#gear-end)
- [result](#gear-result)

#### :gear: push

| Method | Type |
| ---------- | ---------- |
| `push` | `(event: T) => void` |

#### :gear: end

| Method | Type |
| ---------- | ---------- |
| `end` | `(result?: R or undefined) => void` |

#### :gear: result

| Method | Type |
| ---------- | ---------- |
| `result` | `() => Promise<R>` |

## :factory: AssistantMessageEventStream

## :tropical_drink: Interfaces

- [DiagnosticErrorInfo](#gear-diagnosticerrorinfo)
- [AssistantMessageDiagnostic](#gear-assistantmessagediagnostic)
- [ThinkingBudgets](#gear-thinkingbudgets)
- [ProviderResponse](#gear-providerresponse)
- [StreamOptions](#gear-streamoptions)
- [ImagesOptions](#gear-imagesoptions)
- [SimpleStreamOptions](#gear-simplestreamoptions)
- [TextSignatureV1](#gear-textsignaturev1)
- [TextContent](#gear-textcontent)
- [ThinkingContent](#gear-thinkingcontent)
- [ImageContent](#gear-imagecontent)
- [ToolCall](#gear-toolcall)
- [Usage](#gear-usage)
- [UserMessage](#gear-usermessage)
- [AssistantMessage](#gear-assistantmessage)
- [ToolResultMessage](#gear-toolresultmessage)
- [ImagesContext](#gear-imagescontext)
- [AssistantImages](#gear-assistantimages)
- [Tool](#gear-tool)
- [Context](#gear-context)
- [OpenAICompletionsCompat](#gear-openaicompletionscompat)
- [OpenAIResponsesCompat](#gear-openairesponsescompat)
- [AnthropicMessagesCompat](#gear-anthropicmessagescompat)
- [OpenRouterRouting](#gear-openrouterrouting)
- [VercelGatewayRouting](#gear-vercelgatewayrouting)
- [Model](#gear-model)
- [ImagesModel](#gear-imagesmodel)
- [AnthropicOptions](#gear-anthropicoptions)
- [OpenAICompletionsOptions](#gear-openaicompletionsoptions)
- [OpenAIResponsesStreamOptions](#gear-openairesponsesstreamoptions)
- [ConvertResponsesMessagesOptions](#gear-convertresponsesmessagesoptions)
- [ConvertResponsesToolsOptions](#gear-convertresponsestoolsoptions)
- [OpenAIResponsesOptions](#gear-openairesponsesoptions)
- [GoogleOptions](#gear-googleoptions)
- [MistralOptions](#gear-mistraloptions)
- [BedrockOptions](#gear-bedrockoptions)
- [ApiProvider](#gear-apiprovider)
- [FauxModelDefinition](#gear-fauxmodeldefinition)
- [RegisterFauxProviderOptions](#gear-registerfauxprovideroptions)
- [FauxProviderRegistration](#gear-fauxproviderregistration)
- [AzureOpenAIResponsesOptions](#gear-azureopenairesponsesoptions)
- [GoogleVertexOptions](#gear-googlevertexoptions)
- [OpenAICodexResponsesOptions](#gear-openaicodexresponsesoptions)
- [OpenAICodexWebSocketDebugStats](#gear-openaicodexwebsocketdebugstats)

### :gear: DiagnosticErrorInfo



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `name` | `string or undefined` |  |
| `message` | `string` |  |
| `stack` | `string or undefined` |  |
| `code` | `string or number or undefined` |  |


### :gear: AssistantMessageDiagnostic



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `type` | `string` |  |
| `timestamp` | `number` |  |
| `error` | `DiagnosticErrorInfo or undefined` |  |
| `details` | `Record<string, unknown> or undefined` |  |


### :gear: ThinkingBudgets

各思考级别的 Token 预算（仅适用于基于 Token 控制思考的提供商）

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `minimal` | `number or undefined` |  |
| `low` | `number or undefined` |  |
| `medium` | `number or undefined` |  |
| `high` | `number or undefined` |  |


### :gear: ProviderResponse

提供商 HTTP 响应的封装

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `status` | `number` |  |
| `headers` | `Record<string, string>` |  |


### :gear: StreamOptions

流式调用的完整选项
所有提供商共享的基础选项，各提供商可从中提取自己支持的字段

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `temperature` | `number or undefined` | 生成温度，控制输出的随机性，0.0 为确定性输出 |
| `maxTokens` | `number or undefined` | 最大输出 Token 数 |
| `signal` | `AbortSignal or undefined` | 中止信号，用于取消正在进行的流式请求 |
| `apiKey` | `string or undefined` | API 密钥，覆盖默认凭据 |
| `transport` | `Transport or undefined` | 首选传输协议，不支持此选项的提供商会忽略 |
| `cacheRetention` | `CacheRetention or undefined` | 提示缓存保留偏好，提供商标映射到各自支持的值 默认值："short" |
| `sessionId` | `string or undefined` | 可选的会话标识符，用于支持基于会话的缓存 提供商可用于启用提示缓存、请求路由等会话感知功能 |
| `onPayload` | `((payload: unknown, model: Model<Api>) => unknown) or undefined` | 发送前的请求体拦截/替换回调 返回 undefined 保持请求体不变，返回新值则替换原始请求体 |
| `onResponse` | `((response: ProviderResponse, model: Model<Api>) => void or Promise<void>) or undefined` | HTTP 响应接收后的回调（在响应体流被消费之前触发） |
| `headers` | `Record<string, string> or undefined` | 自定义 HTTP 请求头，与提供商默认请求头合并，可覆盖默认值 部分提供商不支持（如 AWS Bedrock 使用 SDK 认证） |
| `timeoutMs` | `number or undefined` | HTTP 请求超时时间（毫秒），如 OpenAI 和 Anthropic SDK 默认 10 分钟 |
| `maxRetries` | `number or undefined` | 最大重试次数，如 OpenAI 和 Anthropic SDK 默认 2 次 |
| `maxRetryDelayMs` | `number or undefined` | 服务端请求长时间等待时的最大延迟上限（毫秒） 超过此值则立即失败并返回包含请求延迟的错误，允许上层重试逻辑处理 默认值：60000（60 秒），设为 0 禁用上限 |
| `metadata` | `Record<string, unknown> or undefined` | 附加到 API 请求的元数据 提供商提取自己理解的字段，忽略其余（如 Anthropic 使用 user_id 进行滥用追踪） |


### :gear: ImagesOptions

图像生成调用的选项
结构与 StreamOptions 类似但用于图像生成场景

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `signal` | `AbortSignal or undefined` |  |
| `apiKey` | `string or undefined` |  |
| `onPayload` | `((payload: unknown, model: ImagesModel<ImagesApi>) => unknown) or undefined` | 发送前的请求体拦截/替换回调 |
| `onResponse` | `((response: ProviderResponse, model: ImagesModel<ImagesApi>) => void or Promise<void>) or undefined` | HTTP 响应接收后的回调 |
| `headers` | `Record<string, string> or undefined` | 自定义 HTTP 请求头 |
| `timeoutMs` | `number or undefined` | HTTP 请求超时时间（毫秒） |
| `maxRetries` | `number or undefined` | 最大重试次数 |
| `maxRetryDelayMs` | `number or undefined` | 服务端请求长时间等待时的最大延迟上限（毫秒） 默认值：60000（60 秒），设为 0 禁用上限 |
| `metadata` | `Record<string, unknown> or undefined` | 附加到 API 请求的元数据 |


### :gear: SimpleStreamOptions

简化流式调用选项，在 StreamOptions 基础上增加推理控制参数
用于 streamSimple() 和 completeSimple() 等简化调用入口

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `reasoning` | `ThinkingLevel or undefined` | 推理/思考级别，统一映射到各提供商的具体参数 |
| `thinkingBudgets` | `ThinkingBudgets or undefined` | 自定义各思考级别的 Token 预算（仅适用于基于 Token 控制的提供商） |


### :gear: TextSignatureV1

文本签名（V1 版本），用于标识文本内容的阶段

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `v` | `1` |  |
| `id` | `string` |  |
| `phase` | `"commentary" or "final_answer" or undefined` |  |


### :gear: TextContent

文本内容块

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `type` | `"text"` |  |
| `text` | `string` |  |
| `textSignature` | `string or undefined` | 文本签名，如 OpenAI Responses 的 message metadata |


### :gear: ThinkingContent

推理/思考内容块
包含模型的推理过程文本，部分提供商会返回签名用于多轮对话连续性

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `type` | `"thinking"` |  |
| `thinking` | `string` |  |
| `thinkingSignature` | `string or undefined` | 推理签名，如 OpenAI Responses 的 reasoning item ID，用于多轮对话传递 |
| `redacted` | `boolean or undefined` | 是否被安全过滤器编辑（脱敏） 编辑后的加密载荷存储在 thinkingSignature 中，可传回 API 保持多轮连续性 |


### :gear: ImageContent

图像内容块

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `type` | `"image"` |  |
| `data` | `string` | Base64 编码的图像数据 |
| `mimeType` | `string` | MIME 类型（如 "image/jpeg"、"image/png"） |


### :gear: ToolCall

工具调用内容块
表示 LLM 请求调用一个工具

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `type` | `"toolCall"` |  |
| `id` | `string` | 工具调用的唯一标识符，用于将工具结果与调用配对 |
| `name` | `string` | 要调用的工具名称 |
| `arguments` | `Record<string, any>` | 传递给工具的参数 |
| `thoughtSignature` | `string or undefined` | Google 特有：用于复用思考上下文的不透明签名 |


### :gear: Usage

Token 用量和成本统计

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `input` | `number` |  |
| `output` | `number` |  |
| `cacheRead` | `number` |  |
| `cacheWrite` | `number` |  |
| `totalTokens` | `number` | 总 Token 数（input + output，不含缓存） |
| `cost` | `{ input: number; output: number; cacheRead: number; cacheWrite: number; total: number; }` | 各项成本，单位为美元 |


### :gear: UserMessage

用户消息

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `role` | `"user"` |  |
| `content` | `string or (TextContent or ImageContent)[]` | 文本内容，或文本/图像混合内容数组 |
| `timestamp` | `number` | Unix 时间戳（毫秒） |


### :gear: AssistantMessage

助手消息
包含 LLM 的完整响应，可能同时包含文本、推理内容和工具调用

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `role` | `"assistant"` |  |
| `content` | `(TextContent or ThinkingContent or ToolCall)[]` | 内容数组，可包含文本、推理和工具调用 |
| `api` | `Api` | 使用的 API 标识符 |
| `provider` | `string` | 使用的提供商标识符 |
| `model` | `string` | 请求的模型 ID |
| `responseModel` | `string or undefined` | 实际响应的模型 ID（当与请求不同时，如 OpenRouter auto 路由） |
| `responseId` | `string or undefined` | 提供商特定的响应/消息标识符 |
| `diagnostics` | `AssistantMessageDiagnostic[] or undefined` | 脱敏后的提供商/运行时诊断信息（用于失败和恢复的分析） |
| `usage` | `Usage` | Token 用量统计 |
| `stopReason` | `StopReason` | 停止原因 |
| `errorMessage` | `string or undefined` | 错误消息（stopReason 为 error/aborted 时存在） |
| `timestamp` | `number` | Unix 时间戳（毫秒） |


### :gear: ToolResultMessage

工具结果消息
将工具执行的结果返回给 LLM，完成一轮工具调用循环

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `role` | `"toolResult"` |  |
| `toolCallId` | `string` | 对应的 ToolCall ID |
| `toolName` | `string` | 工具名称 |
| `content` | `(TextContent or ImageContent)[]` | 工具输出内容，支持文本和图像 |
| `details` | `TDetails or undefined` | 工具特定的附加详情 |
| `isError` | `boolean` | 是否为错误结果 |
| `timestamp` | `number` | Unix 时间戳（毫秒） |


### :gear: ImagesContext

图像生成的输入上下文

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `input` | `ImagesInputContent[]` |  |


### :gear: AssistantImages

图像生成的完整响应结果

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `api` | `ImagesApi` |  |
| `provider` | `string` |  |
| `model` | `string` |  |
| `output` | `ImagesOutputContent[]` |  |
| `responseId` | `string or undefined` |  |
| `usage` | `Usage or undefined` |  |
| `stopReason` | `ImagesStopReason` |  |
| `errorMessage` | `string or undefined` |  |
| `timestamp` | `number` | Unix 时间戳（毫秒） |


### :gear: Tool

工具定义，描述一个可供 LLM 调用的工具
使用 TypeBox 的 TSchema 定义参数的 JSON Schema

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `name` | `string` | 工具名称 |
| `description` | `string` | 工具描述，帮助 LLM 决定何时调用此工具 |
| `parameters` | `TParameters` | 参数的 JSON Schema 定义 |


### :gear: Context

对话上下文，包含 LLM 调用所需的全部输入信息

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `systemPrompt` | `string or undefined` | 系统提示词 |
| `messages` | `Message[]` | 消息历史（用户、助手、工具结果的交替序列） |
| `tools` | `Tool<TSchema>[] or undefined` | 可用工具列表，不提供则 LLM 不会调用工具 |


### :gear: OpenAICompletionsCompat

OpenAI Chat Completions 兼容 API 的兼容性设置
用于覆盖基于 URL 的自动检测，适配不同的 OpenAI 兼容提供商

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `supportsStore` | `boolean or undefined` | 是否支持 store 字段。默认从 URL 自动检测 |
| `supportsDeveloperRole` | `boolean or undefined` | 是否支持 developer 角色（而非 system）。默认从 URL 自动检测 |
| `supportsReasoningEffort` | `boolean or undefined` | 是否支持 reasoning_effort 参数。默认从 URL 自动检测 |
| `supportsUsageInStreaming` | `boolean or undefined` | 是否支持 stream_options: { include_usage: true } 获取流式 Token 用量。默认 true |
| `maxTokensField` | `"max_completion_tokens" or "max_tokens" or undefined` | 使用哪个字段限制最大 Token 数。默认从 URL 自动检测 |
| `requiresToolResultName` | `boolean or undefined` | 工具结果是否需要 name 字段。默认从 URL 自动检测 |
| `requiresAssistantAfterToolResult` | `boolean or undefined` | 工具结果后是否需要中间插入一条助手消息。默认从 URL 自动检测 |
| `requiresThinkingAsText` | `boolean or undefined` | 是否需要将思考块转换为带 <thinking> 标签的文本块。默认从 URL 自动检测 |
| `requiresReasoningContentOnAssistantMessages` | `boolean or undefined` | 启用推理时，历史助手消息是否必须包含空的 reasoning_content 字段。默认从 URL 自动检测 |
| `thinkingFormat` | `"openai" or "deepseek" or "openrouter" or "zai" or "together" or "qwen" or "qwen-chat-template" or undefined` | 推理/思考参数的格式 - "openai"：使用 reasoning_effort - "openrouter"：使用 reasoning: { effort } - "deepseek"：使用 thinking: { type } + reasoning_effort - "together"：使用 reasoning: { enabled } + reasoning_effort - "zai"：使用顶层 enable_thinking: boolean - "qwen"：使用顶层 enable_thinking: boolean - "qwen-chat-template"：使用 chat_template_kwargs.enable_thinking 默认 "openai" |
| `openRouterRouting` | `OpenRouterRouting or undefined` | OpenRouter 特有的路由偏好，仅在 baseUrl 指向 OpenRouter 时使用 |
| `vercelGatewayRouting` | `VercelGatewayRouting or undefined` | Vercel AI Gateway 路由偏好，仅在 baseUrl 指向 Vercel AI Gateway 时使用 |
| `zaiToolStream` | `boolean or undefined` | z.ai 是否支持顶层 tool_stream: true 流式工具调用增量。默认 false |
| `supportsStrictMode` | `boolean or undefined` | 是否支持工具定义中的 strict 字段。默认 true |
| `cacheControlFormat` | `"anthropic" or undefined` | 提示缓存的缓存控制格式 "anthropic" 表示在系统提示、最后一个工具定义和最后一条用户/助手文本内容上 应用 Anthropic 风格的 cache_control 标记 |
| `sendSessionAffinityHeaders` | `boolean or undefined` | 缓存启用时是否发送会话亲和性请求头（session_id、x-client-request-id、x-session-affinity）。默认 false |
| `supportsLongCacheRetention` | `boolean or undefined` | 是否支持长期缓存保留（如 prompt_cache_retention: "24h" 或 Anthropic 的 cache_control.ttl: "1h"）。默认 true |


### :gear: OpenAIResponsesCompat

OpenAI Responses API 的兼容性设置

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `sendSessionIdHeader` | `boolean or undefined` | 缓存启用时是否发送 OpenAI session_id 缓存亲和性请求头。默认 true |
| `supportsLongCacheRetention` | `boolean or undefined` | 是否支持 prompt_cache_retention: "24h"。默认 true |


### :gear: AnthropicMessagesCompat

Anthropic Messages 兼容 API 的兼容性设置

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `supportsEagerToolInputStreaming` | `boolean or undefined` | 是否支持按工具设置 eager_input_streaming 为 false 时，Anthropic 提供商会省略 tools[].eager_input_streaming 并发送旧版 fine-grained-tool-streaming-2025-05-14 beta 请求头 默认 true |
| `supportsLongCacheRetention` | `boolean or undefined` | 是否支持 Anthropic 长期缓存保留（cache_control.ttl: "1h"）。默认 true |


### :gear: OpenRouterRouting

OpenRouter 提供商路由偏好
控制请求路由到哪些上游提供商

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `allow_fallbacks` | `boolean or undefined` | 是否允许备用提供商处理请求。默认 true |
| `require_parameters` | `boolean or undefined` | 是否仅路由到支持请求中所有参数的提供商。默认 false |
| `data_collection` | `"deny" or "allow" or undefined` | 数据收集策略。"allow"（默认）允许可能存储/训练数据的提供商；"deny" 仅使用不收集用户数据的提供商 |
| `zdr` | `boolean or undefined` | 是否限制路由到仅 ZDR（零数据保留）端点 |
| `enforce_distillable_text` | `boolean or undefined` | 是否限制路由到仅允许文本蒸馏的模型 |
| `order` | `string[] or undefined` | 按顺序尝试的提供商名称/别名列表，不可用时依次回退 |
| `only` | `string[] or undefined` | 仅允许的提供商名称/别名白名单 |
| `ignore` | `string[] or undefined` | 要跳过的提供商名称/别名黑名单 |
| `quantizations` | `string[] or undefined` | 按量化级别筛选提供商（如 ["fp16", "bf16", "fp8", "fp6", "int8", "int4", "fp4", "fp32"]） |
| `sort` | `string or { by?: string or undefined; partition?: string or null or undefined; } or undefined` | 排序策略 可以是字符串（如 "price"、"throughput"、"latency"）或包含 by 和 partition 的对象 |
| `max_price` | `{ prompt?: string or number or undefined; completion?: string or number or undefined; image?: string or number or undefined; audio?: string or number or undefined; request?: string or number or undefined; } or undefined` | 每百万 Token 的最大价格上限（美元） |
| `preferred_min_throughput` | `number or { p50?: number or undefined; p75?: number or undefined; p90?: number or undefined; p99?: number or undefined; } or undefined` | 首选最小吞吐量（Token/秒），可以是数字（应用于 p50）或包含分位数的对象 |
| `preferred_max_latency` | `number or { p50?: number or undefined; p75?: number or undefined; p90?: number or undefined; p99?: number or undefined; } or undefined` | 首选最大延迟（秒），可以是数字（应用于 p50）或包含分位数的对象 |


References:

* [https://openrouter.ai/docs/guides/routing/provider-selection](https://openrouter.ai/docs/guides/routing/provider-selection)


### :gear: VercelGatewayRouting

Vercel AI Gateway 路由偏好
控制网关将请求路由到哪些上游提供商

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `only` | `string[] or undefined` | 仅使用的提供商别名白名单（如 ["bedrock", "anthropic"]） |
| `order` | `string[] or undefined` | 按顺序尝试的提供商别名列表（如 ["anthropic", "openai"]） |


References:

* [https://vercel.com/docs/ai-gateway/models-and-providers/provider-options](https://vercel.com/docs/ai-gateway/models-and-providers/provider-options)


### :gear: Model

统一模型定义接口
包含模型的完整元信息，用于路由、计费和能力判断

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `id` | `string` | 模型 ID（如 "gpt-4o"、"claude-3-5-sonnet-20241022"） |
| `name` | `string` | 人类可读的模型名称 |
| `api` | `TApi` | 模型使用的 API 协议 |
| `provider` | `string` | 模型所属提供商 |
| `baseUrl` | `string` | API 基础 URL |
| `reasoning` | `boolean` | 模型是否支持推理/思考功能 |
| `thinkingLevelMap` | `Partial<Record<ModelThinkingLevel, string or null>> or undefined` | 将统一思考级别映射到提供商/模型特定的值 缺失的键使用提供商默认值，null 表示该级别不被支持 |
| `input` | `("text" or "image")[]` | 模型支持的输入类型 |
| `cost` | `{ input: number; output: number; cacheRead: number; cacheWrite: number; }` | 模型使用成本（美元/百万 Token） |
| `contextWindow` | `number` | 上下文窗口大小（Token 数） |
| `maxTokens` | `number` | 最大输出 Token 数 |
| `headers` | `Record<string, string> or undefined` | 附加到请求的自定义 HTTP 头 |
| `compat` | `(TApi extends "openai-completions" ? OpenAICompletionsCompat : TApi extends "openai-responses" ? OpenAIResponsesCompat : TApi extends "anthropic-messages" ? AnthropicMessagesCompat : never) or undefined` | 兼容性覆盖配置 根据不同的 API 协议自动选择对应的兼容性接口 不设置时，从 baseUrl 自动检测 |


### :gear: ImagesModel

图像生成模型定义
继承 Model 的大部分字段，但针对图像生成场景调整了部分类型

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `api` | `TApi` |  |
| `provider` | `string` |  |
| `output` | `("text" or "image")[]` | 模型支持的输出类型 |


### :gear: AnthropicOptions



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `thinkingEnabled` | `boolean or undefined` | Enable extended thinking. For Opus 4.6 and Sonnet 4.6: uses adaptive thinking (model decides when/how much to think). For older models: uses budget-based thinking with thinkingBudgetTokens. |
| `thinkingBudgetTokens` | `number or undefined` | Token budget for extended thinking (older models only). Ignored for Opus 4.6 and Sonnet 4.6, which use adaptive thinking. |
| `effort` | `AnthropicEffort or undefined` | Effort level for adaptive thinking (Opus 4.6+ and Sonnet 4.6). Controls how much thinking Claude allocates: - "max": Always thinks with no constraints (Opus 4.6 only) - "xhigh": Highest reasoning level (Opus 4.7) - "high": Always thinks, deep reasoning (default) - "medium": Moderate thinking, may skip for simple queries - "low": Minimal thinking, skips for simple tasks Ignored for older models. |
| `thinkingDisplay` | `AnthropicThinkingDisplay or undefined` | Controls how thinking content is returned in API responses. - "summarized": Thinking blocks contain summarized thinking text (default here). - "omitted": Thinking blocks return an empty thinking field; the encrypted signature still travels back for multi-turn continuity. Use for faster time-to-first-text-token when your UI does not surface thinking.  Note: Anthropic's API default for Claude Opus 4.7 and Claude Mythos Preview is "omitted". We default to "summarized" here to keep behavior consistent with older Claude 4 models. Set this explicitly to "omitted" to opt in. |
| `interleavedThinking` | `boolean or undefined` |  |
| `toolChoice` | `"auto" or "none" or "any" or { type: "tool"; name: string; } or undefined` |  |
| `client` | `Anthropic or undefined` | Pre-built Anthropic client instance. When provided, skips internal client construction entirely. Use this to inject alternative SDK clients such as `AnthropicVertex` that shares the same messaging API. |


### :gear: OpenAICompletionsOptions

OpenAI Completions 提供商的扩展选项

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `toolChoice` | `"auto" or "none" or "required" or { type: "function"; function: { name: string; }; } or undefined` | 工具选择策略：auto=自动决定，none=禁用，required=必须调用，或指定函数名 |
| `reasoningEffort` | `"minimal" or "low" or "medium" or "high" or "xhigh" or undefined` | 推理强度级别 |


### :gear: OpenAIResponsesStreamOptions



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `serviceTier` | `"auto" or "default" or "flex" or "scale" or "priority" or null or undefined` |  |
| `resolveServiceTier` | `((responseServiceTier: "auto" or "default" or "flex" or "scale" or "priority" or null or undefined, requestServiceTier: "auto" or "default" or "flex" or "scale" or "priority" or null or undefined) => "auto" or ... 5 more ... or undefined) or undefined` |  |
| `applyServiceTierPricing` | `((usage: Usage, serviceTier: "auto" or "default" or "flex" or "scale" or "priority" or null or undefined) => void) or undefined` |  |


### :gear: ConvertResponsesMessagesOptions



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `includeSystemPrompt` | `boolean or undefined` |  |


### :gear: ConvertResponsesToolsOptions



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `strict` | `boolean or null or undefined` |  |


### :gear: OpenAIResponsesOptions



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `reasoningEffort` | `"minimal" or "low" or "medium" or "high" or "xhigh" or undefined` |  |
| `reasoningSummary` | `"auto" or "detailed" or "concise" or null or undefined` |  |
| `serviceTier` | `"auto" or "default" or "flex" or "scale" or "priority" or null or undefined` |  |


### :gear: GoogleOptions



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `toolChoice` | `"auto" or "none" or "any" or undefined` |  |
| `thinking` | `{ enabled: boolean; budgetTokens?: number or undefined; level?: GoogleThinkingLevel or undefined; } or undefined` |  |


### :gear: MistralOptions



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `toolChoice` | `"auto" or "none" or "any" or "required" or { type: "function"; function: { name: string; }; } or undefined` |  |
| `promptMode` | `"reasoning" or undefined` |  |
| `reasoningEffort` | `MistralReasoningEffort or undefined` |  |


### :gear: BedrockOptions



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `region` | `string or undefined` |  |
| `profile` | `string or undefined` |  |
| `toolChoice` | `"auto" or "none" or "any" or { type: "tool"; name: string; } or undefined` |  |
| `reasoning` | `ThinkingLevel or undefined` |  |
| `thinkingBudgets` | `ThinkingBudgets or undefined` |  |
| `interleavedThinking` | `boolean or undefined` |  |
| `thinkingDisplay` | `BedrockThinkingDisplay or undefined` | Controls how Claude's thinking content is returned in responses. - "summarized": Thinking blocks contain summarized thinking text (default here). - "omitted": Thinking content is redacted but the signature still travels back for multi-turn continuity, reducing time-to-first-text-token.  Note: Anthropic's API default for Claude Opus 4.7 and Mythos Preview is "omitted". We default to "summarized" here to keep behavior consistent with older Claude 4 models. Only applies to Claude models on Bedrock. |
| `requestMetadata` | `Record<string, string> or undefined` | Key-value pairs attached to the inference request for cost allocation tagging. Keys: max 64 chars, no `aws:` prefix. Values: max 256 chars. Max 50 pairs. Tags appear in AWS Cost Explorer split cost allocation data. see: https://docs.aws.amazon.com/bedrock/latest/APIReference/API_runtime_ConverseStream.html |
| `bearerToken` | `string or undefined` | Bearer token for Bedrock API key authentication. When set, bypasses SigV4 signing and sends Authorization: Bearer <token> instead. Requires `bedrock:CallWithBearerToken` IAM permission on the token's identity. Set via AWS_BEARER_TOKEN_BEDROCK env var or pass directly. see: https://docs.aws.amazon.com/service-authorization/latest/reference/list_amazonbedrock.html |


### :gear: ApiProvider

外部可用的 API 提供商接口，保留完整的泛型类型信息

| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `api` | `TApi` |  |
| `stream` | `StreamFunction<TApi, TOptions>` |  |
| `streamSimple` | `StreamFunction<TApi, SimpleStreamOptions>` |  |


### :gear: FauxModelDefinition



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `id` | `string` |  |
| `name` | `string or undefined` |  |
| `reasoning` | `boolean or undefined` |  |
| `input` | `("text" or "image")[] or undefined` |  |
| `cost` | `{ input: number; output: number; cacheRead: number; cacheWrite: number; } or undefined` |  |
| `contextWindow` | `number or undefined` |  |
| `maxTokens` | `number or undefined` |  |


### :gear: RegisterFauxProviderOptions



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `api` | `string or undefined` |  |
| `provider` | `string or undefined` |  |
| `models` | `FauxModelDefinition[] or undefined` |  |
| `tokensPerSecond` | `number or undefined` |  |
| `tokenSize` | `{ min?: number or undefined; max?: number or undefined; } or undefined` |  |


### :gear: FauxProviderRegistration



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `api` | `string` |  |
| `models` | `[Model<string>, ...Model<string>[]]` |  |
| `state` | `{ callCount: number; }` |  |
| `setResponses` | `(responses: FauxResponseStep[]) => void` |  |
| `appendResponses` | `(responses: FauxResponseStep[]) => void` |  |
| `getPendingResponseCount` | `() => number` |  |
| `unregister` | `() => void` |  |


### :gear: AzureOpenAIResponsesOptions



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `reasoningEffort` | `"minimal" or "low" or "medium" or "high" or "xhigh" or undefined` |  |
| `reasoningSummary` | `"auto" or "detailed" or "concise" or null or undefined` |  |
| `azureApiVersion` | `string or undefined` |  |
| `azureResourceName` | `string or undefined` |  |
| `azureBaseUrl` | `string or undefined` |  |
| `azureDeploymentName` | `string or undefined` |  |


### :gear: GoogleVertexOptions



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `toolChoice` | `"auto" or "none" or "any" or undefined` |  |
| `thinking` | `{ enabled: boolean; budgetTokens?: number or undefined; level?: GoogleThinkingLevel or undefined; } or undefined` |  |
| `project` | `string or undefined` |  |
| `location` | `string or undefined` |  |


### :gear: OpenAICodexResponsesOptions



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `reasoningEffort` | `"none" or "minimal" or "low" or "medium" or "high" or "xhigh" or undefined` |  |
| `reasoningSummary` | `"auto" or "off" or "detailed" or "concise" or "on" or null or undefined` |  |
| `serviceTier` | `"auto" or "default" or "flex" or "scale" or "priority" or null or undefined` |  |
| `textVerbosity` | `"low" or "medium" or "high" or undefined` |  |


### :gear: OpenAICodexWebSocketDebugStats



| Property | Type | Description |
| ---------- | ---------- | ---------- |
| `requests` | `number` |  |
| `connectionsCreated` | `number` |  |
| `connectionsReused` | `number` |  |
| `cachedContextRequests` | `number` |  |
| `storeTrueRequests` | `number` |  |
| `fullContextRequests` | `number` |  |
| `deltaRequests` | `number` |  |
| `lastInputItems` | `number` |  |
| `lastDeltaInputItems` | `number or undefined` |  |
| `lastPreviousResponseId` | `string or undefined` |  |
| `websocketFailures` | `number` |  |
| `sseFallbacks` | `number` |  |
| `websocketFallbackActive` | `boolean or undefined` |  |
| `lastWebSocketError` | `string or undefined` |  |


## :cocktail: Types

- [KnownApi](#gear-knownapi)
- [Api](#gear-api)
- [KnownImagesApi](#gear-knownimagesapi)
- [ImagesApi](#gear-imagesapi)
- [KnownProvider](#gear-knownprovider)
- [Provider](#gear-provider)
- [KnownImagesProvider](#gear-knownimagesprovider)
- [ImagesProvider](#gear-imagesprovider)
- [ThinkingLevel](#gear-thinkinglevel)
- [ModelThinkingLevel](#gear-modelthinkinglevel)
- [ThinkingLevelMap](#gear-thinkinglevelmap)
- [CacheRetention](#gear-cacheretention)
- [Transport](#gear-transport)
- [ProviderStreamOptions](#gear-providerstreamoptions)
- [ProviderImagesOptions](#gear-providerimagesoptions)
- [StreamFunction](#gear-streamfunction)
- [ImagesFunction](#gear-imagesfunction)
- [StopReason](#gear-stopreason)
- [Message](#gear-message)
- [ImagesInputContent](#gear-imagesinputcontent)
- [ImagesOutputContent](#gear-imagesoutputcontent)
- [ImagesStopReason](#gear-imagesstopreason)
- [AssistantMessageEvent](#gear-assistantmessageevent)
- [AnthropicEffort](#gear-anthropiceffort)
- [AnthropicThinkingDisplay](#gear-anthropicthinkingdisplay)
- [GoogleThinkingLevel](#gear-googlethinkinglevel)
- [BedrockThinkingDisplay](#gear-bedrockthinkingdisplay)
- [ApiStreamFunction](#gear-apistreamfunction)
- [ApiStreamSimpleFunction](#gear-apistreamsimplefunction)
- [FauxContentBlock](#gear-fauxcontentblock)
- [FauxResponseFactory](#gear-fauxresponsefactory)
- [FauxResponseStep](#gear-fauxresponsestep)
- [SessionResourceCleanup](#gear-sessionresourcecleanup)

### :gear: KnownApi

内置已知的 API 协议标识符
每种标识符对应一套不同的请求/响应格式

| Type | Type |
| ---------- | ---------- |
| `KnownApi` | `| "openai-completions" // OpenAI Chat Completions API（使用最广泛，众多兼容提供商） or "mistral-conversations" // Mistral Conversations API or "openai-responses" // OpenAI Responses API（新一代 API） or "azure-openai-responses" // Azure 托管的 OpenAI Responses API or "openai-codex-responses" // OpenAI Codex Responses API or "anthropic-messages" // Anthropic Messages API（Claude 系列模型） or "bedrock-converse-stream" // AWS Bedrock Converse Stream API or "google-generative-ai" // Google Generative AI API（Gemini 系列模型） or "google-vertex` |

### :gear: Api

API 标识符，支持已知标识符和自定义字符串
`KnownApi` 提供类型提示和校验，`(string & {})` 允许扩展自定义 API

| Type | Type |
| ---------- | ---------- |
| `Api` | `KnownApi or (string and {})` |

### :gear: KnownImagesApi

内置已知的图像生成 API 标识符

| Type | Type |
| ---------- | ---------- |
| `KnownImagesApi` | `openrouter-images` |

### :gear: ImagesApi

图像生成 API 标识符

| Type | Type |
| ---------- | ---------- |
| `ImagesApi` | `KnownImagesApi or (string and {})` |

### :gear: KnownProvider

内置已知的 LLM 提供商

| Type | Type |
| ---------- | ---------- |
| `KnownProvider` | `| "amazon-bedrock" or "anthropic" or "google" or "google-vertex" or "openai" or "azure-openai-responses" or "openai-codex" or "deepseek" or "github-copilot" or "xai" or "groq" or "cerebras" or "openrouter" or "vercel-ai-gateway" or "zai" or "mistral" or "minimax" or "minimax-cn" or "moonshotai" or "moonshotai-cn" or "huggingface" or "fireworks" or "together" or "opencode" or "opencode-go" or "kimi-coding" or "cloudflare-workers-ai" or "cloudflare-ai-gateway" or "xiaomi" or "xiaomi-token-plan-cn" or "xiaomi-token-plan-ams" or "xiaomi-token-plan-sgp` |

### :gear: Provider

提供商标识符，支持已知和自定义提供商

| Type | Type |
| ---------- | ---------- |
| `Provider` | `KnownProvider or string` |

### :gear: KnownImagesProvider

内置已知的图像生成提供商

| Type | Type |
| ---------- | ---------- |
| `KnownImagesProvider` | `openrouter` |

### :gear: ImagesProvider

图像生成提供商标识符

| Type | Type |
| ---------- | ---------- |
| `ImagesProvider` | `KnownImagesProvider or string` |

### :gear: ThinkingLevel

思考深度级别（不含关闭选项）

| Type | Type |
| ---------- | ---------- |
| `ThinkingLevel` | `minimal" or "low" or "medium" or "high" or "xhigh` |

### :gear: ModelThinkingLevel

模型思考级别，在 ThinkingLevel 基础上增加 "off"（关闭思考）

| Type | Type |
| ---------- | ---------- |
| `ModelThinkingLevel` | `off" or ThinkingLevel` |

### :gear: ThinkingLevelMap

思考级别到提供商特定值的映射表
缺失的键使用提供商默认值，null 表示该级别不被支持

| Type | Type |
| ---------- | ---------- |
| `ThinkingLevelMap` | `Partial<Record<ModelThinkingLevel, string or null>>` |

### :gear: CacheRetention

提示缓存保留策略
- "none"：不缓存
- "short"：短期缓存（默认）
- "long"：长期缓存（如 Anthropic 的 1 小时 TTL）

| Type | Type |
| ---------- | ---------- |
| `CacheRetention` | `none" or "short" or "long` |

### :gear: Transport

HTTP 传输协议类型
- "sse"：Server-Sent Events
- "websocket"：WebSocket
- "websocket-cached"：带缓存的 WebSocket
- "auto"：自动选择

| Type | Type |
| ---------- | ---------- |
| `Transport` | `sse" or "websocket" or "websocket-cached" or "auto` |

### :gear: ProviderStreamOptions

提供商特定的流式调用选项，在 StreamOptions 基础上允许任意额外字段

| Type | Type |
| ---------- | ---------- |
| `ProviderStreamOptions` | `StreamOptions and Record<string, unknown>` |

### :gear: ProviderImagesOptions

提供商特定的图像生成选项

| Type | Type |
| ---------- | ---------- |
| `ProviderImagesOptions` | `ImagesOptions and Record<string, unknown>` |

### :gear: StreamFunction

泛型流式调用函数类型

契约：
- 必须返回 AssistantMessageEventStream
- 请求/模型/运行时失败应编码在返回的流中，而非抛出异常
- 错误终止必须产生一个 stopReason 为 "error" 或 "aborted" 的 AssistantMessage，
  通过流协议发出

| Type | Type |
| ---------- | ---------- |
| `StreamFunction` | `( model: Model<TApi>, context: Context, options?: TOptions, ) => AssistantMessageEventStream` |

### :gear: ImagesFunction

泛型图像生成函数类型
与 StreamFunction 不同，图像生成是异步非流式的

| Type | Type |
| ---------- | ---------- |
| `ImagesFunction` | `( model: ImagesModel<TApi>, context: ImagesContext, options?: TOptions, ) => Promise<AssistantImages>` |

### :gear: StopReason

流式响应的停止原因
- "stop"：正常结束
- "length"：达到最大 Token 数
- "toolUse"：LLM 请求调用工具（需要继续循环）
- "error"：发生错误
- "aborted"：请求被中止

| Type | Type |
| ---------- | ---------- |
| `StopReason` | `stop" or "length" or "toolUse" or "error" or "aborted` |

### :gear: Message

消息联合类型，包含用户消息、助手消息和工具结果消息

| Type | Type |
| ---------- | ---------- |
| `Message` | `UserMessage or AssistantMessage or ToolResultMessage` |

### :gear: ImagesInputContent

| Type | Type |
| ---------- | ---------- |
| `ImagesInputContent` | `TextContent or ImageContent` |

### :gear: ImagesOutputContent

| Type | Type |
| ---------- | ---------- |
| `ImagesOutputContent` | `TextContent or ImageContent` |

### :gear: ImagesStopReason

图像生成的停止原因

| Type | Type |
| ---------- | ---------- |
| `ImagesStopReason` | `stop" or "error" or "aborted` |

### :gear: AssistantMessageEvent

助手消息事件协议，定义 AssistantMessageEventStream 中的事件类型

流的生命周期：
1. `start`：流开始，携带初始 partial AssistantMessage
2. 增量更新阶段（text/thinking/toolcall 的 start -> delta -> end）
3. 终止事件（二选一）：
   - `done`：成功完成，携带最终 AssistantMessage
   - `error`：失败或中止，携带 stopReason 为 "error"/"aborted" 的 AssistantMessage

| Type | Type |
| ---------- | ---------- |
| `AssistantMessageEvent` | `| { type: "start"; partial: AssistantMessage } or { type: "text_start"; contentIndex: number; partial: AssistantMessage } or { type: "text_delta"; contentIndex: number; delta: string; partial: AssistantMessage } or { type: "text_end"; contentIndex: number; content: string; partial: AssistantMessage } or { type: "thinking_start"; contentIndex: number; partial: AssistantMessage } or { type: "thinking_delta"; contentIndex: number; delta: string; partial: AssistantMessage } or { type: "thinking_end"; contentIndex: number; content: string; partial: AssistantMessage } or { type: "toolcall_start"; contentIndex: number; partial: AssistantMessage } or { type: "toolcall_delta"; contentIndex: number; delta: string; partial: AssistantMessage } or { type: "toolcall_end"; contentIndex: number; toolCall: ToolCall; partial: AssistantMessage } or { type: "done"; reason: Extract<StopReason, "stop" or "length" or "toolUse">; message: AssistantMessage } or { type: "error"; reason: Extract<StopReason, "aborted" or "error">; error: AssistantMessage }` |

### :gear: AnthropicEffort

| Type | Type |
| ---------- | ---------- |
| `AnthropicEffort` | `low" or "medium" or "high" or "xhigh" or "max` |

### :gear: AnthropicThinkingDisplay

| Type | Type |
| ---------- | ---------- |
| `AnthropicThinkingDisplay` | `summarized" or "omitted` |

### :gear: GoogleThinkingLevel

Thinking level for Gemini 3 models.
Mirrors Google's ThinkingLevel enum values.

| Type | Type |
| ---------- | ---------- |
| `GoogleThinkingLevel` | `THINKING_LEVEL_UNSPECIFIED" or "MINIMAL" or "LOW" or "MEDIUM" or "HIGH` |

### :gear: BedrockThinkingDisplay

| Type | Type |
| ---------- | ---------- |
| `BedrockThinkingDisplay` | `summarized" or "omitted` |

### :gear: ApiStreamFunction

类型擦除后的标准流式调用函数签名

| Type | Type |
| ---------- | ---------- |
| `ApiStreamFunction` | `( model: Model<Api>, context: Context, options?: StreamOptions, ) => AssistantMessageEventStream` |

Parameters:

* `model`: - 使用的模型实例
* `context`: - 对话上下文
* `options`: - 流式调用选项


Returns:

助手消息事件流

### :gear: ApiStreamSimpleFunction

类型擦除后的简化流式调用函数签名（使用 SimpleStreamOptions）

| Type | Type |
| ---------- | ---------- |
| `ApiStreamSimpleFunction` | `( model: Model<Api>, context: Context, options?: SimpleStreamOptions, ) => AssistantMessageEventStream` |

Parameters:

* `model`: - 使用的模型实例
* `context`: - 对话上下文
* `options`: - 简化流式调用选项


Returns:

助手消息事件流

### :gear: FauxContentBlock

| Type | Type |
| ---------- | ---------- |
| `FauxContentBlock` | `TextContent or ThinkingContent or ToolCall` |

### :gear: FauxResponseFactory

| Type | Type |
| ---------- | ---------- |
| `FauxResponseFactory` | `( context: Context, options: StreamOptions or undefined, state: { callCount: number }, model: Model<string>, ) => AssistantMessage or Promise<AssistantMessage>` |

### :gear: FauxResponseStep

| Type | Type |
| ---------- | ---------- |
| `FauxResponseStep` | `AssistantMessage or FauxResponseFactory` |

### :gear: SessionResourceCleanup

| Type | Type |
| ---------- | ---------- |
| `SessionResourceCleanup` | `(sessionId?: string) => void` |

