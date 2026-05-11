# piso — pi Rust 重构技术方案

> 本文档是 pi（TypeScript monorepo）的 Rust 重构计划。目标是创建一个
> 与原版 pi 理念兼容、性能更优、单二进制分发的 Rust 实现。

## 1. 设计哲学（继承自 pi）

### 1.1 核心原则

| 原则 | 含义 | Rust 中的体现 |
|------|------|--------------|
| **极简主义** | 只做必须的事，不过度工程 | 每个 crate 职责单一，避免 pi_agent_rust 的 hostcall_jit 过度设计 |
| **一切可扩展** | 工具、命令、渲染器、编辑器全部可替换 | `ExtensionApi` trait 暴露与 TS 版本完全一致的事件钩子 |
| **管道友好** | 支持 stdin/stdout 管道和 headless 模式 | `print` 和 `rpc` 模式零 TUI 依赖 |
| **键盘驱动** | 所有操作可通过键盘完成，keybinding 可配置 | keybinding 系统与 TS 版本格式兼容 |
| **会话持久化** | 会话是人类思考过程的记录，不能丢失 | JSONL 格式与 TS 版本双向兼容 |

### 1.2 不做什么

- **不做 Agent OS** — OpenFang 的定位是 OS，我们做的是编码工具
- **不做自建运行时** — 用 tokio，不造 asupersync
- **不做自建 TUI 框架** — 用 ratatui，不造 bubbletea wrapper
- **不做安全炫技** — 实用的命令安全检查，不做 taint tracking lattice
- **不做品牌绑定** — 支持所有 LLM provider，不绑定特定模型

## 2. 架构

### 2.1 Crate 依赖图

```
pi-types          ← 零 IO，纯类型（消息、工具、事件、配置）
  ↑
  ├── pi-llm      ← LLM provider 抽象（流式、工具调用、多路由）
  ├── pi-session  ← 会话管理（JSONL、分支、压缩）
  ├── pi-tools    ← 内置工具（bash、read、write、edit、find、grep）
  └── pi-extensions ← 扩展系统（事件总线、生命周期钩子）
        ↑
        ├── pi-tui    ← 终端 UI（ratatui、组件、主题）
        └── pi-agent  ← Agent 循环（编排 LLM + 工具 + 会话）
              ↑
              pi-cli  ← 入口（参数解析、模式分发）
```

### 2.2 与 TypeScript 版本的模块映射

| Rust crate | TypeScript 包 | 关键源文件 |
|-----------|-------------|----------|
| `pi-types` | `@earendil-works/pi-ai` (types) + `@earendil-works/pi-agent-core` (types) | `types.ts`, `harness/types.ts`, `settings-manager.ts` |
| `pi-llm` | `@earendil-works/pi-ai` (providers) | `stream.ts`, `providers/*.ts`, `transform-messages.ts` |
| `pi-session` | `@earendil-works/pi-agent-core` (session) | `harness/session/`, `compaction/`, `session-manager.ts` |
| `pi-tools` | `coding-agent` (tools) | `core/tools/{bash,read,write,edit,find,grep}.ts` |
| `pi-extensions` | `coding-agent` (extensions) | `core/extensions/`, `core/event-bus.ts` |
| `pi-tui` | `@earendil-works/pi-tui` + `coding-agent` (interactive) | `tui.ts`, `modes/interactive/` |
| `pi-agent` | `@earendil-works/pi-agent-core` (agent-loop) | `agent-loop.ts`, `agent-session.ts` |
| `pi-cli` | `coding-agent` (cli + main) | `cli.ts`, `main.ts`, `cli/args.ts` |

### 2.3 数据格式兼容性

**JSONL 会话格式**：与 TS 版本完全兼容，同一个会话文件可以在两个版本之间共享。

```jsonl
{"type":"message","id":"abc123","parentMessageId":null,"timestamp":"2026-05-10T12:00:00Z","role":"user","content":[{"type":"text","text":"fix the bug"}]}
{"type":"message","id":"def456","parentMessageId":"abc123","timestamp":"2026-05-10T12:00:01Z","role":"assistant","content":[{"type":"text","text":"I'll help you fix the bug."}]}
```

**配置格式**：`~/.pi/settings.json` 和 `.pi/settings.json` 共享同一格式。

**模型定义**：`~/.pi/agent/models.json` 格式兼容。

## 3. 分阶段实施计划

### Phase 1: 骨架 + 单工具闭环（目标：可编译、可运行一条 bash 命令）

**目标**：`piso -p "run ls" ` 能实际执行。

| 任务 | Crate | 工作量 |
|------|-------|-------|
| CLI 参数解析（clap derive） | pi-cli | 1 天 |
| 配置加载（TOML/JSON） | pi-cli | 1 天 |
| 认证管理（环境变量 + auth.json） | pi-cli | 1 天 |
| 消息类型定义 | pi-types | 2 天 |
| JSONL 会话读写 | pi-session | 2 天 |
| 单 provider 实现（Anthropic OpenAI-compatible） | pi-llm | 3 天 |
| SSE 流式解析 | pi-llm | 2 天 |
| bash 工具实现 | pi-tools | 2 天 |
| 工具注册表 | pi-tools | 1 天 |
| Agent 循环（LLM → 工具 → LLM 闭环） | pi-agent | 3 天 |
| Print 模式入口 | pi-cli | 1 天 |
| **合计** | | **~19 天** |

### Phase 2: 内置工具 + 多 Provider（目标：日常可用）

| 任务 | Crate | 工作量 |
|------|-------|-------|
| read 工具 | pi-tools | 2 天 |
| write 工具 | pi-tools | 1 天 |
| edit 工具（字符串替换） | pi-tools | 2 天 |
| find 工具 | pi-tools | 1 天 |
| grep 工具 | pi-tools | 2 天 |
| 输出截断策略 | pi-tools | 1 天 |
| OpenAI provider | pi-llm | 2 天 |
| Google provider | pi-llm | 2 天 |
| GLM/Zhipu provider | pi-llm | 1 天 |
| Provider 注册机制 | pi-llm | 1 天 |
| 消息转换（各 provider 格式差异） | pi-llm | 3 天 |
| 模型注册表 + 解析 | pi-agent | 2 天 |
| 会话管理器（create/resume/fork） | pi-session | 3 天 |
| 上下文压缩 | pi-session | 3 天 |
| 系统提示构建 | pi-agent | 2 天 |
| 技能加载 | pi-agent | 1 天 |
| `--continue` / `--resume` / `--fork` | pi-cli | 2 天 |
| `--list-models` | pi-cli | 1 天 |
| 管道 stdin 支持 | pi-cli | 0.5 天 |
| **合计** | | **~32 天** |

### Phase 3: TUI + 交互模式（目标：替代日常使用）

| 任务 | Crate | 工作量 |
|------|-------|-------|
| TUI 引擎（事件循环、alternate screen） | pi-tui | 3 天 |
| 五区布局（header/chat/status/editor/footer） | pi-tui | 3 天 |
| Markdown 渲染（代码高亮） | pi-tui | 3 天 |
| 输入组件（多行、历史、补全） | pi-tui | 3 天 |
| 流式消息渲染 | pi-tui | 2 天 |
| 工具执行结果渲染 | pi-tui | 2 天 |
| 思考块渲染 | pi-tui | 1 天 |
| Keybinding 系统 | pi-tui | 2 天 |
| 主题系统 | pi-tui | 1 天 |
| 会话选择器 | pi-tui | 2 天 |
| 模型选择器 | pi-tui | 1 天 |
| 设置界面 | pi-tui | 1 天 |
| InteractiveMode 编排 | pi-agent | 3 天 |
| Agent-TUI 通信（mpsc channel） | pi-agent | 2 天 |
| 页脚状态栏（模型、token、快捷键） | pi-tui | 1 天 |
| **合计** | | **~30 天** |

### Phase 4: 扩展系统 + 高级功能

| 任务 | Crate | 工作量 |
|------|-------|-------|
| EventBus（tokio broadcast） | pi-extensions | 2 天 |
| ExtensionApi trait（30+ 钩子方法） | pi-extensions | 3 天 |
| 扩展加载器（文件发现、排序） | pi-extensions | 2 天 |
| 扩展执行器（sandbox） | pi-extensions | 3 天 |
| 自定义工具注册 | pi-extensions | 1 天 |
| 自定义命令注册 | pi-extensions | 1 天 |
| 自定义消息渲染器 | pi-extensions | 2 天 |
| 自定义编辑器组件 | pi-extensions | 2 天 |
| RPC 模式（JSON-over-stdio） | pi-cli | 3 天 |
| HTML 导出 | pi-cli | 2 天 |
| LSP 集成（post-edit 诊断注入） | pi-agent | 3 天 |
| 自动补全（/command, @file） | pi-tui | 2 天 |
| 恢复/诊断命令 | pi-cli | 1 天 |
| Shell 自动补全脚本生成 | pi-cli | 0.5 天 |
| **合计** | | **~28 天** |

### 总工作量估算

| Phase | 内容 | 天数 | 里程碑 |
|-------|------|------|--------|
| 1 | 骨架 + 单工具闭环 | 19 | `piso -p "run ls"` 可工作 |
| 2 | 内置工具 + 多 Provider | 32 | 日常 print 模式可用 |
| 3 | TUI + 交互模式 | 30 | 日常交互模式可用 |
| 4 | 扩展系统 + 高级功能 | 28 | 功能对齐 TS 版本核心 |
| **总计** | | **~109 天** |

实际开发中 AI 辅助可压缩到 40-60 天。

## 4. 关键技术决策

### 4.1 为什么 8 个 crate 而不是 1 个

- **编译并行**：修改 TUI 不需要重新编译 LLM 层
- **依赖隔离**：pi-tui 需要 ratatui/crossterm，pi-llm 不需要；pi-cli 不需要 rusqlite
- **测试隔离**：每个 crate 的测试独立运行
- **边界清晰**：crate 之间的 pub 接口就是模块边界，编译器强制执行

对比 pi_agent_rust 的 1 个巨型 crate（257 文件）和 DeepSeek-TUI 的名义 14 crate 实则 1 个巨型 crate，我们选择真正的 crate 拆分。

### 4.2 为什么用 ratatui 而不是自建

pi 的 TS 版本自建了一个轻量 TUI 框架（`packages/tui/`），因为 TypeScript 生态中没有好的终端 UI 库。Rust 生态中 ratatui 已经是成熟标准：

- ratatui 有丰富的组件生态（ratatui-widgets、tui-widget）
- DeepSeek-TUI 和 pi_agent_rust 都用 ratatui
- 维护负担小，不需要自己处理终端转义序列

### 4.3 扩展系统：Rust 原生 vs JS/WASM

**Phase 1-3：只支持 Rust 原生扩展**。理由：

- pi 的扩展 API 有 30+ 钩子方法，JS bridge 的实现成本很高
- Rust 原生扩展零开销
- 初期不需要动态加载（编译时注册即可）

**Phase 4+：可选 JS runtime 嵌入**。当扩展生态成熟后，通过 Deno/V8 或 WASM 支持动态扩展，与 TS 版本的扩展格式兼容。

### 4.4 会话格式双向兼容

TS 版本的 JSONL 格式是 pi 的核心数据格式。Rust 版本必须能读写同一份会话文件：

- 两个版本可以在同一个项目上交替使用
- 迁移路径平滑：用户不需要导出/导入会话
- 测试策略：用 TS 版本生成的会话文件做 Rust 版本的解析测试

### 4.5 Provider 实现策略

不逐个 provider 手写 HTTP 请求。而是实现三种 "driver"：

1. **Anthropic driver**：原生 Anthropic Messages API
2. **OpenAI-compatible driver**：覆盖 OpenAI、DeepSeek、GLM、Groq、Together、Fireworks 等 15+ provider
3. **Google driver**：原生 Google Generative AI API

这与 TS 版本的三种 `Api` 类型（`anthropic`、`openai-completions`、`google`）对应。Bedrock、Azure 通过 OpenAI-compatible driver + 自定义 base URL 支持。

## 5. 测试策略

### 5.1 测试层次

```
Unit tests      每个 crate 内，纯逻辑测试（消息解析、工具输出截断、JSONL 读写）
Integration     crate 间集成（LLM driver → Agent loop → Tool registry）
Compatibility   TS/Rust 格式兼容性（JSONL 互操作、配置共享）
Snapshot        TUI 渲染 snapshot（ratatui::backend::TestBackend）
E2E             完整 agent 对话（使用 faux provider，无需真实 API key）
```

### 5.2 兼容性测试

关键测试用例：

```rust
#[test]
fn ts_session_is_readable_by_rust() {
    let session = JsonlSession::read("test/fixtures/ts-generated-session.jsonl");
    assert_eq!(session.entries().len(), 42);
}

#[test]
fn rust_session_is_readable_by_ts() {
    // 写入会话，然后用 serde_json 验证每行格式与 TS 版本一致
}
```

## 6. 项目结构

```
rust/
├── Cargo.toml              # workspace 定义
├── CLAUDE.md               # 本文件 — 开发指南
├── crates/
│   ├── pi-types/           # 核心类型（零 IO）
│   ├── pi-llm/             # LLM provider 抽象
│   ├── pi-session/         # 会话管理
│   ├── pi-tools/           # 内置工具
│   ├── pi-extensions/      # 扩展系统
│   ├── pi-tui/             # 终端 UI
│   ├── pi-agent/           # Agent 循环
│   └── pi-cli/             # CLI 入口
├── tests/
│   ├── compatibility/      # TS/Rust 格式兼容性测试
│   └── fixtures/           # 测试数据（含 TS 版本生成的会话文件）
└── benches/                # 性能基准
```

## 7. 与 TypeScript 版本的关系

### 7.1 不是替代，是并行

- TypeScript 版本继续维护和演进
- Rust 版本是独立的实现，共享数据格式和设计理念
- 两个版本可以在同一个项目上交替使用

### 7.2 功能对齐优先级

| 优先级 | 功能 | Phase |
|--------|------|-------|
| P0 | Agent 循环（LLM → 工具 → LLM） | 1 |
| P0 | bash 工具 | 1 |
| P0 | Print 模式（-p） | 1 |
| P0 | 会话持久化 | 1 |
| P1 | 全部 6 个内置工具 | 2 |
| P1 | 多 Provider 支持 | 2 |
| P1 | 会话恢复/分支 | 2 |
| P1 | 上下文压缩 | 2 |
| P2 | TUI 交互模式 | 3 |
| P2 | 流式渲染（思考块、工具输出） | 3 |
| P2 | Keybinding/主题 | 3 |
| P3 | 扩展系统 | 4 |
| P3 | RPC 模式 | 4 |
| P3 | LSP 集成 | 4 |
| P4 | Web UI 集成 | 未来 |
| P4 | JS 扩展兼容 | 未来 |
