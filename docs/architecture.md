# Pi 项目架构与原理

本文档从整体到细节，系统说明 pi 项目的架构设计和运行机制。

---

## 总体架构

pi 是一个终端编码代理（Terminal Coding Agent），采用 monorepo 结构，由 5 个包组成，从底层到上层依次为：

```
┌─────────────────────────────────────────────┐
│         packages/coding-agent               │  ← CLI 入口 + 交互式 TUI
├─────────────────────────────────────────────┤
│         packages/agent                      │  ← Agent 运行时（工具调用、状态管理）
├─────────────────────────────────────────────┤
│         packages/ai                         │  ← 统一多提供商 LLM API
├─────────────────────────────────────────────┤
│         packages/tui                        │  ← 终端 UI 差分渲染引擎
├─────────────────────────────────────────────┤
│         packages/web-ui                     │  ← Web 端聊天组件
└─────────────────────────────────────────────┘
```

| 包名 | npm 包名 | 职责 |
|------|----------|------|
| `ai` | `@earendil-works/pi-ai` | 屏蔽不同 LLM 提供商的 API 差异，提供统一调用接口 |
| `agent` | `@earendil-works/pi-agent-core` | Agent 运行时，管理对话循环、工具调用和状态 |
| `tui` | `@earendil-works/pi-tui` | 终端 UI 差分渲染引擎 |
| `web-ui` | `@earendil-works/pi-web-ui` | Web 端 AI 聊天界面组件 |
| `coding-agent` | `@earendil-works/pi-coding-agent` | 编码代理主体，组合以上各层，提供 CLI 和交互式终端界面 |

---

## 各层原理

### 1. `packages/ai` — 统一 LLM 接口层

屏蔽不同 LLM 提供商的 API 差异，提供统一的流式调用接口。

- 支持 30+ 提供商（OpenAI、Anthropic、Google、DeepSeek、GLM 等）
- 每个提供商实现一个 `stream()` 函数，返回标准化的 `AssistantMessageEventStream`
- 统一事件类型：`text`（文本）、`tool_call`（工具调用）、`thinking`（推理）、`usage`（token 用量）、`stop`（结束）
- 消息格式转换：将内部统一的消息格式转换为各提供商需要的格式（如 OpenAI 的 `messages[]`、Anthropic 的 `content blocks`）
- 模型列表通过脚本自动生成（`scripts/generate-models.ts`）

**工作流程**：

```
内部统一消息格式 → 提供商消息转换 → HTTP/SSE 流式请求 → 解析响应 → 标准化事件流
```

**文件结构要点**：

- `src/types.ts`：核心类型定义（`Api`、`ApiOptionsMap`、`KnownProvider` 等联合类型）
- `src/providers/`：各提供商的实现文件
- `src/providers/register-builtins.ts`：通过懒加载注册内置提供商
- `src/env-api-keys.ts`：环境变量凭据检测
- `scripts/generate-models.ts`：模型列表生成（生成 `models.generated.ts`，不可直接修改）

### 2. `packages/agent` — Agent 运行时

管理 Agent 的对话循环、工具调用和状态。

- **对话循环**（Agentic Loop）：发送消息给 LLM → 收到响应 → 如果有工具调用则执行 → 将结果返回 LLM → 重复，直到 LLM 不再调用工具
- **工具注册与执行**：管理可用工具列表（内置 + 扩展注册的），调度工具执行
- **状态管理**：维护对话历史、消息树结构

**Agent Loop 伪代码**：

```ts
while (true) {
  const response = await llm.stream(messages, tools);
  messages.push(response);

  if (response.tool_calls.length > 0) {
    for (const call of response.tool_calls) {
      const result = await executeTool(call);
      messages.push(result);
    }
  } else {
    break; // LLM 不再调用工具，循环结束
  }
}
```

### 3. `packages/tui` — 终端 UI 引擎

在终端中高效渲染复杂 UI。

- **差分渲染**：不重绘整个屏幕，只更新变化的部分（类似 React 的虚拟 DOM diff，但针对终端）
- 组件化：提供声明式 UI 组件（列表、文本、输入框、状态栏等）
- 处理终端转义序列、颜色、鼠标事件等

### 4. `packages/web-ui` — Web UI 组件

提供 Web 端的 AI 聊天界面组件，可用于浏览器环境。

### 5. `packages/coding-agent` — 编码代理主体

将以上各层组合成可用的编码助手，是用户直接交互的入口。

**四个内置工具**：

| 工具 | 作用 |
|------|------|
| `read` | 读取文件内容（支持文本和图片） |
| `bash` | 执行 shell 命令 |
| `edit` | 精确文本替换编辑文件 |
| `write` | 创建或覆盖文件 |

**会话管理**：

- 会话以 JSONL 格式存储，每条记录有 `id` 和 `parentId`，形成**树结构**
- 支持分支（`/tree`）、分叉（`/fork`）、克隆（`/clone`）
- **上下文压缩（Compaction）**：长对话超出上下文窗口时，自动摘要旧消息，完整的原始历史仍保留在 JSONL 文件中

**四种运行模式**：

| 模式 | 说明 |
|------|------|
| 交互式（默认） | 终端 TUI，实时对话 |
| Print（`-p`） | 输出响应后退出 |
| JSON（`--mode json`） | 以 JSON 行输出所有事件 |
| RPC（`--mode rpc`） | 通过 stdin/stdout 的 JSONL 协议，用于进程集成 |

---

## 一次用户请求的完整执行流程

```
用户在终端输入 "帮我重构 src/foo.ts"
        │
        ▼
  TUI 编辑器接收输入，Enter 提交
        │
        ▼
  coding-agent 将消息加入会话历史
        │
        ▼
  agent core 启动 agentic loop:
        │
        ├─→ ai 包将消息格式化（+ system prompt + 工具定义）
        │       │
        │       ▼
        │   发送给 LLM 流式请求
        │       │
        │       ▼
        │   LLM 返回: "我来先读取这个文件" + tool_call(read, "src/foo.ts")
        │       │
        │       ▼
        │   TUI 实时显示 LLM 的文本和工具调用
        │       │
        │       ▼
        │   agent 执行 read 工具 → 返回文件内容
        │       │
        │       ▼
        │   第二轮: 消息 + 文件内容 → LLM
        │       │
        │       ▼
        │   LLM 返回: 分析 + tool_call(edit, {oldText, newText})
        │       │
        │       ▼
        │   agent 执行 edit 工具 → 文件被修改
        │       │
        │       ▼
        │   第三轮: 编辑结果 → LLM
        │       │
        │       ▼
        │   LLM 返回: "重构完成"（无工具调用）
        │       │
        │       ▼
        │   agentic loop 结束
        │
        ▼
  会话历史自动保存到 ~/.pi/agent/sessions/
```

---

## 扩展系统

pi 的核心理念是**最小核心 + 极致可扩展**。核心只做四件事：LLM 调用、工具执行、会话管理、终端渲染。其他一切通过扩展系统实现。

### Extensions（扩展）

TypeScript 模块，可注册：

- 自定义工具（或替换内置工具）
- 自定义命令、快捷键
- 事件处理器
- UI 组件（替换编辑器、状态栏、页脚、覆盖层等）

```ts
export default function (pi: ExtensionAPI) {
  pi.registerTool({ name: "deploy", ... });
  pi.registerCommand("stats", { ... });
  pi.on("tool_call", async (event, ctx) => { ... });
}
```

放置于 `~/.pi/agent/extensions/`、`.pi/extensions/` 或 pi 包中。

### Skills（技能）

按需加载的能力描述，遵循 [Agent Skills 标准](https://agentskills.io)。Markdown 文件，指导模型执行特定任务。

```markdown
<!-- ~/.pi/agent/skills/my-skill/SKILL.md -->
# My Skill
Use this skill when the user asks about X.

## Steps
1. Do this
2. Then that
```

放置于 `~/.pi/agent/skills/`、`~/.agents/skills/`、`.pi/skills/` 或 pi 包中。

### Prompt Templates（提示模板）

可复用的提示模板，用 `/name` 展开。支持 `{{variable}}` 占位符。

放置于 `~/.pi/agent/prompts/`、`.pi/prompts/` 或 pi 包中。

### Themes（主题）

UI 主题定制，支持热重载。内置 `dark` 和 `light`。

放置于 `~/.pi/agent/themes/`、`.pi/themes/` 或 pi 包中。

### Pi Packages（包）

将扩展、技能、模板和主题打包为 npm 或 git 包，通过 `pi install` 安装，通过 `pi config` 启用/禁用。

---

## 上下文与配置加载

### System Prompt 构成

1. **默认系统提示**（可被 `.pi/SYSTEM.md` 替换）
2. **AGENTS.md / CLAUDE.md**：从 `~/.pi/agent/` 和当前目录向上遍历加载
3. **已激活的 Skills**：技能描述会追加到系统提示
4. **`APPEND_SYSTEM.md`**：追加内容而不替换

### 配置层级

| 文件 | 作用域 |
|------|--------|
| `~/.pi/agent/settings.json` | 全局 |
| `.pi/settings.json` | 项目级（覆盖全局） |
| `~/.pi/agent/keybindings.json` | 快捷键 |
| `~/.pi/agent/auth.json` | 认证信息 |
| `~/.pi/agent/models.json` | 自定义提供商/模型 |

---

## 设计哲学

pi 不内置以下功能，而是让用户通过扩展系统自行实现或安装第三方包：

- **不内置子代理** → 通过 Extensions 或 tmux 实现
- **不内置计划模式** → 通过 Extensions 实现
- **不内置权限弹窗** → 在容器中运行，或通过 Extensions 实现自定义确认流程
- **不内置后台 Bash** → 使用 tmux
- **不依赖 MCP** → 用 Skills 或 Extensions 替代
- **不内置 TODO** → 用文件或 Extensions 实现

这使得 pi 不会强迫用户适应固定工作流，而是让用户根据需要拼装自己的编码助手。

---

## 相关文档

- [会话格式](session-format.md)
- [上下文压缩](compaction.md)
- [扩展开发](extensions.md)
- [技能系统](skills.md)
- [提示模板](prompt-templates.md)
- [主题定制](themes.md)
- [包管理](packages.md)
- [SDK 集成](sdk.md)
- [RPC 模式](rpc.md)
- [提供商标配置](providers.md)
- [模型管理](models.md)
- [自定义提供商](custom-provider.md)
- [快捷键](keybindings.md)
- [设置项](settings.md)
