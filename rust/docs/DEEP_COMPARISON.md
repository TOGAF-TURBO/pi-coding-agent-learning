# TypeScript (pi) vs Rust (piso) — 深度功能对比

> 基于源码逐行分析，而非文档声明。
> 生成日期：2026-05-09

---

## 1. CLI 参数解析

### 架构差异

| 维度 | TS (pi) | Rust (piso) |
|------|---------|-------------|
| 解析方式 | 手写 parser（`parseArgs()` 358行） | clap derive（`#[derive(Parser)]` 268行） |
| 未知 flag | `unknownFlags: Map<string, boolean \| string>` 收集所有 `--xxx` | `register_flag()` 扩展注册 |
| 参数校验 | `diagnostics: Array<{type, message}>` 收集警告 | clap 自动校验 + 退出 |
| `@file` 解析 | CLI 层分离 `fileArgs: string[]` | `fileref.rs` 在消息发送前解析 |
| `--print` 语义 | 可选吸下一个非 flag 参数 | 纯 flag，参数走 positional |

### 功能矩阵

| 参数 | TS | Rust | 行为差异 |
|------|:---:|:----:|----------|
| `--model` 语法 | `provider/id:thinking` | 纯 ID | **TS 支持 `openai/gpt-4o:high`** |
| `--models` | `string[]` 数组，支持 glob | `Option<String>` 逗号分隔 | TS 更灵活 |
| `--tools` | `string[]` 数组 | `Option<String>` 逗号分隔 | 数据结构不同，功能等价 |
| `--skill` | 可重复 `string[]` | `Option<String>` 单个 | **TS 支持多个 skill** |
| `--theme` | 可重复 `string[]` | `Option<String>` 单个 | **TS 支持多个 theme** |
| `--prompt-template` | 可重复 `string[]` | `Option<String>` 单个 | **TS 支持多个 template** |
| `--extension` | 可重复 `string[]` | `Option<Vec<String>>` 可重复 | 一致 |
| `--list-models` | `string \| true` 可选搜索 | `Option<Option<String>>` | 一致 |
| `--mode` | `text/json/rpc` 显式 | `Option<String>` 存在但未使用 | **Rust 未接入** |
| `--export` | `HTML + JSONL` | 仅 HTML | **TS 多 JSONL** |
| `--no-themes` | ✅ | ❌ | **TS 独有** |
| `--init` | ❌ | ✅ | **Rust 独有** |
| `--list-sessions` | ❌ | ✅ | **Rust 独有** |
| `--base-url` | ❌ | ✅ | **Rust 独有** |
| 子命令 | `install/remove/update/list/config` 5 个 | `completions` 1 个 | **TS 远超 Rust** |
| 环境变量文档 | 40+ 个详细列出 | 4 个简略列出 | **TS 文档更完整** |

---

## 2. `@file` 文件引用

| 维度 | TS | Rust |
|------|:---:|:----:|
| 解析位置 | CLI 层（`file-processor.ts` 104行） | 消息层（`fileref.rs` 155行） |
| 文本文件 | `<file name="path">\ncontent\n</file>` | `--- path ---\ncontent\n--- end of path ---` |
| 图片文件 | base64 编码 + MIME 检测 + 自动缩放 (2000x2000) | NUL 字节检测标记为 binary |
| 图片 MIME | `detectSupportedImageMimeTypeFromFile()` 识别 png/jpg/gif/webp | 无 MIME 检测 |
| 图片缩放 | `resizeImage()` 自动缩放到 2000px | 无缩放 |
| 空文件 | 跳过（`stats.size === 0`） | 作为空内容内联 |
| 不存在文件 | `console.error` + `process.exit(1)` | 保留 `@path` 原文不处理 |

**核心差距**：TS 版有完整的图片处理管线（MIME 检测 + base64 + 自动缩放），Rust 版仅做文本内联。

---

## 3. LLM Provider

### TS Provider 文件 (17 个)

| 文件 | 行数 | 独立 API 格式 |
|------|------|:---:|
| `anthropic.ts` | ~800 | ✅ SSE |
| `openai-completions.ts` | ~600 | ✅ chat completions |
| `openai-responses.ts` | ~500 | ✅ responses API |
| `openai-codex-responses.ts` | ~200 | ✅ codex responses |
| `azure-openai-responses.ts` | ~300 | ✅ Azure 部署映射 |
| `google.ts` | ~400 | ✅ Gemini |
| `google-vertex.ts` | ~350 | ✅ Vertex AI |
| `amazon-bedrock.ts` | ~500 | ✅ converse-stream + SigV4 |
| `cloudflare.ts` | ~250 | ✅ Workers AI |
| `github-copilot-headers.ts` | ~200 | ✅ OAuth device flow |
| `mistral.ts` | ~200 | ✅ Mistral 格式 |
| `faux.ts` | ~100 | 测试用 |
| `google-shared.ts` | ~150 | 共享逻辑 |
| `openai-responses-shared.ts` | ~150 | 共享逻辑 |
| `register-builtins.ts` | ~100 | 注册 |
| `simple-options.ts` | ~100 | 简化选项 |
| `transform-messages.ts` | ~200 | 消息转换 |

### Rust Driver 文件 (11 个)

| 文件 | 行数 | 覆盖范围 |
|------|------|----------|
| `driver.rs` | ~120 | trait 定义 |
| `providers.rs` | ~500 | Anthropic SSE |
| `openai.rs` | ~450 | OpenAI completions + 30+ 兼容 |
| `openai_responses.rs` | ~130 | OpenAI responses |
| `gemini.rs` | ~400 | Gemini |
| `vertex.rs` | ~270 | Vertex AI |
| `azure.rs` | ~115 | Azure OpenAI |
| `bedrock.rs` | ~220 | Amazon Bedrock |
| `registry.rs` | ~80 | Provider 注册表 |
| `stream.rs` | ~60 | 事件序列化 |
| `transform.rs` | ~3 | 空（未实现） |

### 功能对比

| 能力 | TS | Rust | 差距 |
|------|:---:|:----:|------|
| 独立 provider 数量 | 11 | 7 | TS 多 4 个 |
| 通过兼容协议覆盖 | 10+ | 30+ | Rust 更广 |
| `registerProvider()` | ✅ 运行时注册 | ✅ 运行时注册 | 一致 |
| OAuth device flow | ✅ GitHub Copilot | ❌ | **TS 独有** |
| Cloudflare Workers AI | ✅ 独立实现 | ❌ | **TS 独有** |
| Mistral 独立格式 | ✅ | 通过 OpenAI 兼容 | 功能等价 |
| `transform-messages.ts` | 200行，provider 格式转换 | 3行，空 | **TS 有完整消息转换** |
| SigV4 签名 | ✅ 完整实现 | 简化占位 | **TS 更完整** |
| thinking budget 控制 | ✅ 完整 | ❌ | **TS 独有** |

---

## 4. 内置工具

| 工具 | TS | Rust | TS 独有功能 |
|------|:---:|:----:|------------|
| `read` | ✅ | ✅ | — |
| `bash` | ✅ | ✅ | TS 有 working directory 管理 |
| `edit` | ✅ 精确替换 | ✅ 精确替换 | — |
| `write` | ✅ | ✅ | — |
| `find` | ✅ glob | ✅ glob | — |
| `grep` | ✅ 默认关闭 | ✅ 默认开启 | 默认状态不同 |
| `ls` | ✅ | ✅ | — |
| diff 输出 | edit 自动附带 | edit/write 自动附带 | 一致 |

**辅助模块**：

| 模块 | TS | Rust |
|------|:---:|:----:|
| `file-mutation-queue.ts` | ✅ 写入队列 | ❌ |
| `output-accumulator.ts` | ✅ 输出累积 | ❌ |
| `tool-definition-wrapper.ts` | ✅ 动态 schema | ❌ |
| `path-utils.ts` | ✅ 路径解析 | 简化版 |
| `render-utils.ts` | ✅ 渲染辅助 | ❌ |
| `edit-diff.ts` | ✅ | `diff.rs` |
| `truncate.ts` | ✅ | ✅ |

---

## 5. Slash 命令

### TS 独有 (12 个)

| 命令 | 功能 | 复杂度 |
|------|------|--------|
| `/share` | GitHub Gist 分享 | 高（API 调用） |
| `/tree` | 会话分支树导航 | 高（tree UI） |
| `/resume` | 恢复不同会话 | 中 |
| `/settings` | 打开设置 UI | 中（selector） |
| `/scoped-models` | 模型限定管理 | 中 |
| `/login` | OAuth 登录流程 | 高 |
| `/logout` | 清除认证 | 低 |
| `/hotkeys` | 快捷键列表 | 低 |
| `/changelog` | 更新日志 | 中 |
| `/debug` | 调试信息 | 低 |
| `/model` | 直接模型切换 | 中 |
| 彩蛋命令 | `/arminsayshi`, `/dementedelves`, `/daxnuts` | — |

### Rust 独有 (6 个)

| 命令 | 功能 | 复杂度 |
|------|------|--------|
| `/cost` | 费用估算 | 中 |
| `/find` | 跨会话搜索 | 中 |
| `/grep` | 当前会话搜索 | 低 |
| `/clear` | 清除聊天 | 低 |
| `/usage` | Token 用量 | 低 |
| `/sessions` | 列出会话 | 低 |

### 共有命令差异

| 命令 | TS 行为 | Rust 行为 | 差异 |
|------|---------|-----------|------|
| `/compact` | LLM 摘要压缩 | 保留前 2+后 10 条 | **算法完全不同** |
| `/export` | HTML + JSONL | 仅 HTML | TS 多格式 |
| `/copy` | 内置剪贴板 | xclip/pbcopy 外部工具 | TS 更可靠 |
| `/reload` | 重载全部配置 | 仅重载 keybindings | **TS 更全面** |
| `/fork` | 完整分支实现 | 占位 | **TS 有完整功能** |
| `/new` | 真正创建新会话 | 提示重启 | **TS 真正工作** |

---

## 6. TUI 组件

### TS TUI (34 个组件文件)

```
assistant-message.ts    — 助手消息渲染
bash-execution.ts       — Bash 执行进度
bordered-loader.ts      — 加载边框
branch-summary-message  — 分支摘要
compaction-summary      — 压缩摘要
config-selector.ts      — 配置选择器
countdown-timer.ts      — 倒计时
custom-editor.ts        — 自定义编辑器
custom-message.ts       — 自定义消息
diff.ts                 — 交互式 diff 查看器
dynamic-border.ts       — 动态边框
extension-editor.ts     — 扩展编辑器
extension-input.ts      — 扩展输入
extension-selector.ts   — 扩展选择器
footer.ts               — 页脚
keybinding-hints.ts     — 快捷键提示
login-dialog.ts         — OAuth 登录对话框
model-selector.ts       — 模型选择器
oauth-selector.ts       — OAuth 选择器
scoped-models-selector  — 模型限定选择器
session-selector.ts     — 会话选择器
session-selector-search — 会话搜索
settings-selector.ts    — 设置选择器
show-images-selector.ts — 图片显示选择器
skill-invocation        — Skill 调用消息
theme-selector.ts       — 主题选择器
thinking-selector.ts    — 思考级别选择器
tool-execution.ts       — 工具执行显示
tree-selector.ts        — 分支树选择器
user-message.ts         — 用户消息
user-message-selector   — 用户消息选择
visual-truncate.ts      — 可视截断
armin.ts, daxnuts.ts    — 彩蛋
earendil-announcement   — 公告
```

### Rust TUI (16 个模块文件)

```
app.rs          — AppState（消息列表、状态）
complete.rs     — Tab 补全（slash/@file/路径）
components.rs   — 5 区渲染（header/chat/status/editor/footer）
engine.rs       — crossterm 事件循环
event.rs        — Event 枚举
eventbus.rs     — 事件总线
git.rs          — Git 状态检测
input.rs        — 输入编辑器（历史、多行）
interactive.rs  — 交互模式主循环
keybinding.rs   — KeyBindings 配置
layout.rs       — ratatui Layout 定义
markdown.rs     — Markdown 渲染（代码块/表格/链接/diff）
selector.rs     — 通用选择器
slash.rs        — Slash 命令解析
theme.rs        — 主题配置
```

### 关键差距

| 功能 | TS | Rust | 影响 |
|------|:---:|:----:|------|
| 交互式 diff 查看器 | `diff.ts` 完整实现 | 无 | 无法逐行审查代码变更 |
| OAuth 登录 UI | `login-dialog.ts` | 无 | 必须手动配置 API key |
| 倒计时显示 | `countdown-timer.ts` | 简单 elapsed 秒数 | TS 更精确 |
| 可视截断 | `visual-truncate.ts` | 无 | 长消息无法优雅截断 |
| 图片显示 | `show-images-selector.ts` | 无 | 无法在终端查看图片 |
| 思考级别选择器 | `thinking-selector.ts` | 无 | 无法交互切换 thinking |
| 设置选择器 | `settings-selector.ts` | 无 | 无法 TUI 内修改设置 |
| 分支树选择器 | `tree-selector.ts` | 无 | 无法可视化导航分支 |
| 会话搜索 | 无 | ✅ `/find` 跨会话搜索 | **Rust 独有** |
| Git 状态 | 无 | ✅ header 分支+脏标记 | **Rust 独有** |
| 费用估算 | 无 | ✅ `/cost` 命令 | **Rust 独有** |

---

## 7. 扩展系统

### 事件系统

**TS: 25 个事件**

| 事件 | TS | Rust |
|------|:---:|:----:|
| `agent_start` | ✅ | ✅ `on_agent_start` |
| `before_agent_start` | ✅ 可拦截 | ❌ |
| `agent_end` | ✅ | ✅ `on_agent_done` |
| `turn_start` | ✅ | ✅ |
| `turn_end` | ✅ | ✅ |
| `message_start` | ✅ | ✅ |
| `message_end` | ✅ | ✅ |
| `message_update` | ✅ 增量更新 | ❌ |
| `model_select` | ✅ | ✅ `on_model_switched` |
| `thinking_level_select` | ✅ | ❌ |
| `context` | ✅ | ❌ |
| `input` | ✅ | ❌ |
| `tool_call` | ✅ | ✅ `on_tool_call_start` |
| `tool_result` | ✅ | ✅ `on_tool_call_end` |
| `tool_execution_start` | ✅ | ✅ |
| `tool_execution_update` | ✅ 进度更新 | ❌ |
| `tool_execution_end` | ✅ | ✅ |
| `session_start` | ✅ | ✅ |
| `session_before_switch` | ✅ 可拦截 | ❌ |
| `session_before_fork` | ✅ 可拦截 | ❌ |
| `session_before_tree` | ✅ 可拦截 | ❌ |
| `session_compact` | ✅ | ✅ |
| `session_tree` | ✅ | ❌ |
| `session_shutdown` | ✅ | ✅ |
| `after_provider_response` | ✅ | ✅ |
| `before_provider_request` | ❌ | ✅ **Rust 独有** |
| `error` | ❌ | ✅ **Rust 独有** |
| `resources_discover` | ✅ | ❌ |
| `user_bash` | ✅ | ❌ |

**Rust 独有 2 个事件**（`before_provider_request`, `error`），TS 独有 8 个事件。

### 扩展 API 方法

| 方法 | TS | Rust |
|------|:---:|:----:|
| `registerTool()` | ✅ 含 schema + state | ✅ 闭包 handler |
| `registerCommand()` | ✅ 含 options | ✅ |
| `registerShortcut()` | ✅ | ✅ |
| `registerFlag()` | ✅ | ✅ |
| `registerProvider()` | ✅ | ✅ |
| `sendMessage()` | ✅ | ✅ `send_message()` |
| `sendUserMessage()` | ✅ | ✅ `send_user_message()` |
| `setName()` / `setLabel()` | ✅ | ✅ |
| `exec()` | ✅ 执行命令 | ❌ |
| `setStatus()` | ✅ | ❌ |
| `setWorkingMessage()` | ✅ | ❌ |
| `setWidget()` | ✅ above/below | ❌ |
| `setFooter()` | ✅ | ❌ |
| `setHeader()` | ✅ 自定义组件 | ❌ |
| `setTitle()` | ✅ | ❌ |
| `setEditorText()` | ✅ | ❌ |
| `getEditorText()` | ✅ | ❌ |
| `setEditorComponent()` | ✅ | ❌ hint 注入 |
| `setTheme()` / `getTheme()` | ✅ | ❌ |
| `setToolsExpanded()` | ✅ | ❌ |
| `getContextUsage()` | ✅ | ❌ |
| `getSystemPrompt()` | ✅ | ❌ |
| `onTerminalInput()` | ✅ | ❌ |
| `select()` / `confirm()` / `input()` | ✅ UI 对话框 | ❌ |

**TS 有 23+ 个 API 方法，Rust 有 10 个。** TS 的 UI 交互 API（`select/confirm/input/widget`）在 Rust 中完全缺失。

---

## 8. RPC 协议

### TS 独有命令 (10 个)

| 命令 | 功能 |
|------|------|
| `abort_bash` | 中止远程 bash |
| `abort_retry` | 中止重试 |
| `clone` | 克隆会话 |
| `cycle_thinking_level` | 切换 thinking level |
| `export_html` | HTML 导出 |
| `extension_ui_request/response` | 扩展 UI 交互（5 种子类型） |
| `fork` | 分叉会话 |
| `get_fork_messages` | 获取分叉消息 |
| `get_last_assistant_text` | 获取最后回复 |
| `get_session_stats` | 会话统计 |
| `set_follow_up_mode` | Follow-up 模式 |
| `set_session_name` | 设置会话名 |
| `set_steering_mode` | Steering 模式 |
| `switch_session` | 切换会话 |

### 共同命令

TS 和 Rust 共有 6 个核心命令（`prompt`, `abort`, `get_state`, `new_session`, `get_messages`, `steer`）。

**TS 总计 ~32 个 RPC 类型（含请求+响应），Rust 有 16 个请求 + 16 个响应。**

---

## 9. 会话管理

| 功能 | TS | Rust | 差异 |
|------|:---:|:----:|------|
| JSONL 格式 | ✅ | ✅ | 双向兼容 |
| 会话分支 | `session.ts` tree + branch | `branching.rs` create_branch | TS 有可视化 tree |
| 会话压缩 | LLM 摘要 | 保留首尾消息 | **算法不同** |
| 会话搜索 | ❌ | ✅ `/find` | **Rust 独有** |
| `--session-dir` | ✅ | ✅ | 一致 |
| `--no-session` | ✅ | ✅ | 一致 |
| 会话选择器 | ✅ 模糊搜索 | ✅ 列表选择 | TS 更灵活 |

---

## 10. 性能 & 体积

| 维度 | TS (pi) | Rust (piso) |
|------|---------|-------------|
| 运行时 | Node.js (~60MB) | 单二进制 12MB |
| 启动时间 | ~1-2s（V8 冷启动） | <50ms |
| 依赖 | node_modules ~200MB | 无运行时依赖 |
| 分发 | 需要 Node.js | 单文件下载即可运行 |
| 内存占用 | ~150MB | ~20MB |
| 测试数量 | ~150+ | 260 |

---

## 汇总：关键差距清单

### Rust 需要补齐的（按影响排序）

**高影响**:
1. **图片支持** — TS 有完整的 MIME 检测 + base64 + 自动缩放管线
2. **`--model provider/id:thinking` 语法** — TS 支持紧凑的模型引用格式
3. **OAuth device flow** — GitHub Copilot 等需要
4. **交互式 diff 查看器** — TS 有完整的多文件 diff UI
5. **`message_update` 事件** — 增量更新，TS 用于流式工具进度
6. **`before_*` 拦截事件** — TS 允许扩展拦截 agent 行为

**中影响**:
7. **扩展 UI API** — `select()/confirm()/input()` 对话框
8. **`transform-messages.ts`** — provider 格式转换
9. **子命令** — `install/remove/update/list/config`
10. **`--mode` 接入** — 参数存在但未使用
11. **多值参数** — `--skill`/`--theme` 应为可重复数组
12. **RPC 扩展 UI 协议** — 远程 UI 交互

**低影响**:
13. Cloudflare Workers AI provider
14. 彩蛋命令
15. `file-mutation-queue.ts` — 文件写入序列化
16. `visual-truncate.ts` — 消息截断
17. JSONL export 格式

### Rust 的优势（TS 没有的）

1. **`/cost` 费用估算** — 6 个 provider 定价表
2. **`/find` 跨会话搜索** — 全文搜索
3. **Git 状态显示** — header 分支+脏标记
4. **Tab 混合补全** — slash/@file/路径三合一
5. **Ctrl+R 重试** — 一键重发
6. **`--init` 配置初始化** — 生成模板配置
7. **Shell completions** — bash/zsh/fish
8. **12MB 单文件分发** — 无需 Node.js
9. **260 个测试** — 比 TS 更多
10. **`before_provider_request` / `error` 事件** — TS 没有
