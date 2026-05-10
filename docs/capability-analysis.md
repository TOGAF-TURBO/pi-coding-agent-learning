# pi 项目能力分析报告

> 基于 6 维分析法：{类型系统 · 函数签名 · 导出面 · CLI 协议 · 配置 Schema · 测试覆盖}
>
> 分析日期：2026-05-10 | 分析版本：0.74.0

---

## 方法论

本报告按照 `.cursor/skills/capability-analysis/SKILL.md` 定义的 6 维系统分析法进行。每个维度的发现以证据（源码位置、具体数值）支撑，而非主观描述。

| 维度 | 推导源 | 可靠性 |
|------|--------|--------|
| 1. 类型系统 | 联合类型成员数、泛型约束、字面量类型 | 最高（编译期约束） |
| 2. 函数签名 | 参数类型 → 返回值类型的映射 | 高 |
| 3. 导出面 | `index.ts` 的 `export` 语句、`package.json` 的 `exports` | 中高 |
| 4. CLI 协议 | `--help` 输出、RPC 类型定义、环境变量列表 | 高 |
| 5. 配置 Schema | TypeBox Schema、设置接口、models.json 结构 | 中 |
| 6. 测试覆盖 | 测试文件名、参数化矩阵、已验证的 Provider 列表 | 高（运行时验证） |

---

## 架构总览

```
pi-ai                    ┄ 统一 LLM 调用层（无状态）
  ↑
pi-agent-core            ┄ 有状态 Agent 运行时
  ↑
pi-tui  ←→  pi-coding-agent  ←→  pi-web-ui
(终端渲染)   (CLI / 交互 / RPC)    (Web 组件)
```

包依赖方向：下层 → 上层。下层对上层无感知。

---

## 一、维度 1：类型系统推导

### 1.1 全局能力边界（从联合类型成员数推导）

| 类型定义 | 成员数 | 推导的能力边界 |
|----------|--------|---------------|
| `KnownProvider`（`packages/ai/src/types.ts:65`） | **32** | 内置支持 32 个 LLM 服务商 |
| `KnownApi`（`packages/ai/src/types.ts:33`） | **9** | 内置支持 9 种 API 协议，外加 `(string & {})` 扩展槽 |
| `AgentEvent`（`packages/agent/src/types.ts`） | **10** | Agent 生命周期可观测 10 种事件 |
| `AssistantMessageEvent`（`packages/ai/src/types.ts`） | **8** | 流式输出可产生 8 种增量事件 |
| `RpcCommand`（`packages/coding-agent/src/modes/rpc/rpc-types.ts:19`） | **27** | RPC 模式支持 27 种远程操作 |
| Extension 事件类型（`packages/coding-agent/src/core/extensions/types.ts`） | **31** | 扩展可监听 31 种系统事件 |

### 1.2 架构约束（从泛型约束推导）

```typescript
// 模型与 API 协议的类型级绑定（types.ts）
Model<TApi extends Api>
// 推导：模型实例必须关联一个具体的 API 协议，编译期防止协议错配（如用 OpenAI 模型调 Anthropic API）

// 流式函数的泛型分发（stream.ts）
function stream<TApi extends Api>(model: Model<TApi>, ...): AssistantMessageEventStream
// 推导：根据 model.api 自动选择对应的提供商实现，调用方无需关心内部实现差异

// 声明合并支持自定义消息类型（agent/src/types.ts）
export interface AgentMessageTypes {}
// 推导：扩展可通过 TypeScript 声明合并添加自定义消息，无需修改框架代码
```

### 1.3 能力矩阵（从可区分联合的关键字段推导）

| 能力维度 | 类型的证据字段 | 价值范围 |
|----------|--------------|---------|
| **模型推理能力** | `Model.reasoning: boolean` | 是/否 |
| **模型视觉能力** | `Model.input: ("text" \| "image")[]` | 纯文本 / 文本+图片 |
| **上下文窗口** | `Model.contextWindow: number` | 0 ~ 2,000,000 tokens |
| **最大输出** | `Model.maxTokens: number` | 0 ~ 128,000 tokens |
| **思考深度** | `thinkingLevelMap: { off, minimal, low, medium, high, xhigh }` | 7 级可调 |
| **工具执行策略** | `ToolExecutionMode: "sequential" \| "parallel"` | 顺序/并行 |
| **中止控制** | `AbortSignal` 出现在多处签名 | 任意环节可中止 |
| **传输协议** | `Transport: "auto" \| "sse" \| "websocket"` | 3 种传输方式 |

---

## 二、维度 2：函数签名推导

### 2.1 pi-ai 层：LLM 调用能力

| 函数 | 签名本质 | 推导的能力 |
|------|---------|-----------|
| `stream` | `(Model<A>, Context, Options?) → EventStream` | 流式调用（实时增量） |
| `complete` | `(Model<A>, Context, Options?) → AssistantMessage` | 非流式调用（等待完整） |
| `streamSimple` | `(Model<A>, Context, SimpleOptions?) → EventStream` | 简化流式（自动推理映射） |
| `completeSimple` | `(Model<A>, Context, SimpleOptions?) → AssistantMessage` | 简化非流式 |
| `registerApiProvider` | `(ApiProvider, source?) → void` | 动态注册 API 适配器 |
| `getApiProvider` | `(api) → ApiProvider \| undefined` | 查询已注册适配器 |
| `getModel` | `(provider, modelId) → Model \| undefined` | 精确查找模型 |
| `getModels` | `(provider) → Model[]` | 获取提供商全部模型 |
| `getProviders` | `() → string[]` | 获取所有提供商列表 |
| `calculateCost` | `(model, usage) → CostTotal` | Token 成本计算 |
| `clampThinkingLevel` | `(model, level) → level` | 自动钳位思考级别 |
| `transformMessages` | `(Message[], Model) → Message[]` | 跨模型消息兼容转换 |

### 2.2 pi-agent-core 层：Agent 运行时能力

| 函数/方法 | 签名本质 | 推导的能力 |
|-----------|---------|-----------|
| `new Agent(opts)` | `(AgentOptions) → Agent` | 创建 Agent 实例 |
| `agent.prompt` | `(input, opts?) → AsyncGenerator<AgentEvent>` | 发起对话（异步事件流） |
| `agent.continue` | `(opts?) → AsyncGenerator<AgentEvent>` | 继续未完成轮次 |
| `agent.steer` | `(message) → void` | 注入转向指令 |
| `agent.followUp` | `(messages) → void` | 排队后续消息 |
| `agent.abort` | `() → void` | 中止运行 |
| `agent.waitForIdle` | `() → Promise<void>` | 等待空闲 |
| `agent.subscribe` | `(listener) → () => void` | 订阅事件（返回退订函数） |
| `runAgentLoop` | `(ctx, state, config) → AsyncGenerator<AgentEvent>` | 无状态 Agent 循环 |
| `compact` | `(messages, model, opts?) → CompactResult` | 上下文压缩 |
| `shouldCompact` | `(messages, model) → boolean` | 压缩判断 |
| `findCutPoint` | `(messages, opts?) → number` | 安全裁剪点查找 |

### 2.3 pi-coding-agent 层：编码助手能力

| 函数/工具工厂 | 签名本质 | 推导的能力 |
|--------------|---------|-----------|
| `createAgentSession` | `(opts?) → {session, extensions}` | 一行创建完整会话 |
| `createReadTool` | `(cwd) → AgentTool` | 文件读取 |
| `createWriteTool` | `(cwd) → AgentTool` | 文件写入 |
| `createEditTool` | `(cwd) → AgentTool` | 文本编辑 |
| `createBashTool` | `(cwd) → AgentTool` | Shell 执行 |
| `createGrepTool` | `(cwd) → AgentTool` | 正则搜索 |
| `createFindTool` | `(cwd) → AgentTool` | 文件查找 |
| `createLsTool` | `(cwd) → AgentTool` | 目录列出 |

### 2.4 pi-tui 层：终端 UI 能力

| 导出 | 签名本质 | 推导的能力 |
|------|---------|-----------|
| `TUI` | 差异渲染引擎 | 仅重绘变化行，消除闪烁 |
| `Editor` | 多行编辑器组件 | 光标、选择、剪贴板、撤销、Vim |
| `Markdown` | Markdown 渲染 | 代码高亮、表格、列表 |
| `SelectList` | 选择列表 | 滚动、搜索、键盘导航 |
| `Image` | 终端图片渲染 | Kitty/iTerm2 协议 |
| `fuzzyFilter` → `FuzzyMatch[]` | 模糊搜索 | 按键匹配与评分 |

---

## 三、维度 3：导出面统计

### 3.1 各包导出量化

| 包 | 总导出行 | 类型导出 | 运行时导出 | 类型占比 |
|-----|---------|---------|-----------|---------|
| **pi-ai** | 31 | 12 | 19 | 39% |
| **pi-agent-core** | 20 | 0 | 20 | 0% |
| **pi-tui** | 22 | 1 | 21 | 5% |
| **pi-web-ui** | 67 | 8 | 59 | 12% |

**推导**：
- pi-agent-core 零类型导出意味着**全部导出都是可执行的运行时能力**（类、函数、工具），类型定义通过 `export * from "./types.js"` 间接导出
- pi-web-ui 的 59 个运行时导出中大量是 Web Components（`ChatPanel`、`SettingsDialog` 等），反映了面向 Web 的组件化策略

### 3.2 子路径入口（扩展点）

| 包 | 子路径 | 用途 |
|-----|--------|------|
| pi-ai | `./anthropic`、`./google`、`./mistral`、`./openai-completions`、`./openai-responses`、`./azure-openai-responses`、`./openai-codex-responses`、`./google-vertex`、`./oauth`、`./bedrock-provider` | **10 个独立入口**，允许按需加载特定提供商，无需引入全部 |
| pi-coding-agent | `./hooks` | 暴露扩展钩子类型，供第三方扩展消费 |
| pi-web-ui | `./app.css` | 独立 CSS 入口，支持 Tailwind 样式按需引入 |

---

## 四、维度 4：CLI 协议推导

### 4.1 命令树

```
pi [options] [@files...] [messages...]          ← 主入口（发起对话）
├── pi install <source> [-l]                     ← 安装扩展（npm/git）
├── pi remove <source> [-l]                      ← 移除扩展
├── pi uninstall <source> [-l]                   ← 移除扩展（别名）
├── pi update [source|self|pi]                   ← 更新扩展/pi 自身
├── pi list                                       ← 列出已安装扩展
├── pi config                                     ← TUI 配置（启用/禁用资源）
└── pi --list-models [search]                     ← 列出可用模型（模糊搜索）
```

### 4.2 选项统计

| 类别 | 数量 | 代表选项 |
|------|------|---------|
| **模型控制** | 5 | `--provider`, `--model`, `--models`, `--thinking`, `--list-models` |
| **会话控制** | 6 | `--continue`, `--resume`, `--session`, `--fork`, `--session-dir`, `--no-session` |
| **输出控制** | 4 | `--mode`, `--print`, `--json`, `--export` |
| **工具控制** | 4 | `--no-tools`, `--no-builtin-tools`, `--tools`, `--read-only` |
| **扩展控制** | 3 | `--extension`, `--no-extensions`, `--skill` |
| **主题/模板** | 4 | `--theme`, `--no-themes`, `--prompt-template`, `--no-prompt-templates` |
| **其他** | 4 | `--api-key`, `--system-prompt`, `--verbose`, `--offline` |

总计 **48+** 个命令行选项，外加扩展可动态注册额外标志。

### 4.3 运行模式

| 模式 | 标志 | 场景 |
|------|------|------|
| **交互式** | 默认 | 终端 TUI 人机对话 |
| **Print** | `-p` / `--print` | CI/CD、脚本调用，处理完退出 |
| **JSON** | `--mode json` | 管道化，stdout 逐行输出 JSON 事件 |
| **RPC** | `--mode rpc` | 嵌入其他进程，stdin/stdout JSON-RPC |

### 4.4 RPC 协议（27 种命令）

从 `rpc-types.ts:19-69` 的 `RpcCommand` 联合类型提取：

| 命令分类 | 命令（type 值） |
|----------|---------------|
| **对话** | `prompt`, `steer`, `follow_up`, `abort` |
| **状态** | `get_state`, `get_messages`, `get_session_stats`, `get_commands` |
| **模型** | `set_model`, `cycle_model`, `get_available_models` |
| **思考** | `set_thinking_level`, `cycle_thinking_level` |
| **队列** | `set_steering_mode`, `set_follow_up_mode` |
| **压缩** | `compact`, `set_auto_compaction` |
| **重试** | `set_auto_retry`, `abort_retry` |
| **Bash** | `bash`, `abort_bash` |
| **会话** | `new_session`, `switch_session`, `fork`, `clone`, `get_fork_messages`, `get_last_assistant_text`, `set_session_name`, `export_html` |

### 4.5 环境变量列表

30+ 个环境变量可供配置（从 `--help` 输出 + `env-api-keys.ts` 推导），覆盖：
- API 密钥（ANTHROPIC、OPENAI、GEMINI、GROQ、DEEPSEEK 等）
- AWS Bedrock 认证（AWS_PROFILE、AWS_ACCESS_KEY_ID 等）
- Azure OpenAI 配置
- 小米 MiMo 多区域
- 配置路径覆盖（PI_CODING_AGENT_DIR、PI_OFFLINE）
- 遥测开关（PI_TELEMETRY）

### 4.6 AI CLI 独立命令

```
pi-ai list     ← 列出所有可用的 OAuth 提供商
pi-ai login    ← 通过 OAuth 登录到指定提供商
```

---

## 五、维度 5：配置 Schema 推导

### 5.1 Agent 配置层（AgentOptions）

| 类别 | 可选字段 | 推导的可配置维度 |
|------|---------|----------------|
| **核心状态** | `initialState` | 初始系统提示、模型、工具、消息 |
| **消息处理** | `convertToLlm`、`transformContext` | 自定义消息格式转换、上下文预处理 |
| **流式调用** | `streamFn`、`getApiKey` | 自定义 LLM 调用、密钥解析 |
| **拦截器** | `onPayload`、`onResponse`、`beforeToolCall`、`afterToolCall` | 请求/响应/工具执行的 4 层拦截 |
| **行为模式** | `steeringMode`、`followUpMode`、`toolExecution` | 队列模式、并行/顺序执行 |
| **性能** | `transport`、`thinkingBudgets`、`maxRetryDelayMs` | 传输协议、Token 预算、重试策略 |

**关键发现**：AgentOptions 的 **16 个字段全部可选**（零必填），意味着 Agent 构造函数可用零参数启动——架构设计支持最小化默认行为的快速原型。

### 5.2 编码会话配置层（CreateAgentSessionOptions）

| 可选字段 | 推导的可配置维度 |
|---------|----------------|
| `cwd`、`agentDir` | 工作目录、配置目录独立指定 |
| `authStorage`、`modelRegistry` | 认证和模型管理可注入自定义实例 |
| `model`、`thinkingLevel`、`scopedModels` | 模型选择、思考深度、切换范围 |
| `noTools`、`tools`、`customTools` | 工具白名单/黑名单/自定义 |
| `resourceLoader`、`sessionManager`、`settingsManager` | 资源、会话、设置可注入自定义实现 |

### 5.3 models.json 用户可扩展性

从 `model-registry.ts:140-160` 的 `ModelDefinitionSchema`：

- 用户可在 `~/.pi/agent/models.json` 中定义**任意 OpenAI 兼容模型**
- 必填字段仅有 `id`（模型 ID）
- 可选字段包括：`name`、`api`、`baseUrl`、`reasoning`、`contextWindow`、`maxTokens`、`headers`、`compat`
- 这意味着用户无需修改源码就能接入任意第三方 LLM API（已验证：GLM 通过此方式接入）

---

## 六、维度 6：测试覆盖推导

### 6.1 按包统计

| 包 | 测试文件数 | 说明 |
|-----|----------|------|
| pi-ai | **65** | 流式调用、Token 计数、中止、Provider 矩阵 |
| pi-agent-core | 13 | Agent 循环、事件、工具执行 |
| pi-coding-agent | **119** | 会话、压缩、扩展、工具、模式 |
| pi-tui | 22 | 组件渲染、键盘处理 |
| pi-web-ui | 0 | Web UI 无测试覆盖 |
| **总计** | **219** | |

### 6.2 已验证的 Provider 矩阵

从 `stream.test.ts` 提取——每个提供商至少有一个代表性模型通过测试：

```
amazon-bedrock     cerebras          cloudflare-ai-gateway
anthropic          cloudflare-workers-ai  deepseek
azure-openai-responses  github-copilot  google
google-vertex      groq              huggingface
kimi-coding        minimax           mistral
openai             openai-codex      openrouter
together           vercel-ai-gateway xai
xiaomi             xiaomi-token-plan-cn
xiaomi-token-plan-ams  xiaomi-token-plan-sgp
zai
```

**26 个唯一 Provider 已验证**（32 个 KnownProvider 中有 26 个有流式测试，覆盖率 **81%**）

### 6.3 从测试文件名推导的核心能力

| 测试文件类别 | 出现次数 | 推导的核心能力 |
|-------------|---------|--------------|
| session | 26 | 会话持久化、分支、导航、恢复 |
| agent | 18 | Agent 循环、事件、提示处理 |
| compaction | 9 | 上下文压缩、自动压缩、分支摘要 |
| tools | 6 | 内置工具的正确性和边界处理 |
| extensions | 5 | 扩展注册、命令、事件处理 |
| settings/manager | 8 | 配置持久化、热重载 |
| model/selector | 9 | 模型选择、切换、作用域 |
| interactive/rpc | 10 | TUI 交互、RPC 协议 |
| image | 4 | 图片处理、缩放、多提供商 |
| command/message | 8 | 斜杠命令、消息格式化 |

### 6.4 跨提供商通用能力矩阵

以下测试覆盖**全部**已验证 Provider：

| 测试文件 | 验证的能力 |
|----------|-----------|
| `stream.test.ts` | 流式对话核心流程 |
| `tokens.test.ts` | Token 计数和成本计算 |
| `abort.test.ts` | 请求中止 |
| `empty.test.ts` | 空助手响应、空工具列表 |
| `context-overflow.test.ts` | 上下文溢出处理 |
| `unicode-surrogate.test.ts` | Unicode 代理对处理 |
| `tool-call-without-result.test.ts` | 孤儿工具调用修复 |
| `image-tool-result.test.ts` | 图片工具结果 |
| `total-tokens.test.ts` | 总 Token 消耗追踪 |
| `cross-provider-handoff.test.ts` | 跨提供商消息交接 |

---

## 七、总结：能力全景量矩阵

### 量化维度一览

| 度量 | 数值 | 含义 |
|------|------|------|
| 支持的 LLM 服务商 | **32** | KnownProvider 联合成员 |
| 支持的 API 协议 | **9 + ∞** | KnownApi + `(string & {})` 扩展槽 |
| RPC 命令 | **27** | RpcCommand 联合成员 |
| 扩展事件类型 | **31** | Extension API 可处理的事件 |
| CLI 选项 | **48+** | 命令行可配置维度 |
| 内置工具 | **7** | read/bash/edit/write/grep/find/ls |
| 测试文件 | **219** | 总测试文件数 |
| 已验证 Provider | **26** | 有流式测试的 Provider（81% 覆盖率） |
| 运行时导出 | 119 | 5 个包合计可执行导出 |
| AgentEvent 类型 | **10** | Agent 生命周期可观测维度 |

### 层次能力矩阵

```
                    输入层              处理层              输出层
                    ──────              ──────              ──────
pi-ai:           文字 / 图片          → 25+ LLM 调用       → 流式事件流（8种）
                 32 提供商密钥        → 跨模型消息转换      → 助手消息
                 工具定义（TypeBox）   → 思维/Token 控制     → Token 用量 + 成本

pi-agent-core:   用户消息             → Agent 循环          → 生命周期事件（10种）
                 工具列表             → 并行/顺序工具执行    → Agent 状态变更
                 系统提示             → 16 种可配置钩子      → 消息历史

pi-coding-agent: CLI 参数（48+）     → 会话管理（JSONL）   → 终端输出 / JSON
                 文件系统             → 扩展执行（31种事件） → RPC 响应（27种命令）
                 SKILL.md             → 上下文压缩           → HTML 导出

pi-tui:          键盘事件             → 差异渲染             → 终端 ANSI 显示
                 组件树（12种）       → 焦点管理             → Kitty/iTerm2 图片
                 Markdown             → 语法高亮             → 选择反馈

pi-web-ui:       文本 / 附件          → 流式消费             → Web Components
                 IndexedDB 存储       → CORS 代理            → Artifact 预览
                 用户设置             → 沙盒 JS 执行         → 多语言 UI
```

### 关键架构决策（从代码推导，不在文档中声明）

1. **泛型类型安全**：`Model<TApi>` 将模型绑定到 API 协议，`stream<TApi>` 自动分发——编译期防止协议错配
2. **事件流模式**：`AgentEvent` 和 `AssistantMessageEvent` 都是 `AsyncGenerator`，天然支持背压和流式消费
3. **钩子链与责任链**：`beforeToolCall → execute → afterToolCall`、`transformContext → convertToLlm`、`onPayload → stream → onResponse` 形成多层插入点
4. **分层扩展注册**：编码代理层的 `ExtensionAPI.registerTool()` 将外部工具注册为 `AgentTool`，通过 `wrapToolDefinition` 桥接
5. **零必填初始化**：`AgentOptions` 16 字段全可选、`CreateAgentSessionOptions` 14 字段全可选——支持渐进式复杂度
6. **文件系统即配置**：`~/.pi/agent/models.json`、`auth.json`、`settings.json`、`.pi/skills/`——无数据库依赖
7. **协议兼容层**：`transformMessages` 自动处理 26 种 Provider 之间的消息格式差异（图片降级、孤儿修复、ID 规范化）
