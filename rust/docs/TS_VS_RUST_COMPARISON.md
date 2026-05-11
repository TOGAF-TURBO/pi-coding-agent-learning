# TypeScript (pi) vs Rust (piso) CLI 功能对比

> 仅从 CLI 功能角度对比，不含性能/体积等非功能性差异。

## 1. CLI 参数

| 参数 | TS (pi) | Rust (piso) | 差异 |
|------|---------|-------------|------|
| `--provider` | ✅ | ✅ | 一致 |
| `--model` | ✅ 支持 `provider/id` 前缀和 `:thinking` 后缀 | ✅ 仅纯 ID | **TS 支持 `openai/gpt-4o:high` 语法** |
| `--api-key` | ✅ | ✅ | 一致 |
| `--base-url` | ❌ (通过 models.json) | ✅ | **piso 多了此参数** |
| `--thinking` | ✅ | ✅ | 一致 |
| `--system-prompt` | ✅ | ✅ | 一致 |
| `--append-system-prompt` | ✅ 可重复 | ✅ 可重复 | 一致 |
| `--continue / -c` | ✅ | ✅ | 一致 |
| `--resume / -r` | ✅ 交互式选择器 | ✅ 标记（无选择器） | **TS 有 UI 选择器** |
| `--fork` | ✅ | ✅ | 一致 |
| `--session` | ✅ 支持 partial UUID | ✅ 支持 ID 前缀 | 基本一致 |
| `--session-dir` | ✅ | ❌ | **TS 多了此参数** |
| `--no-session` | ✅ | ❌ | **TS 多了此参数** |
| `--mode` | ✅ text/json/rpc | ❌ (自动推断) | **TS 显式模式切换** |
| `--print / -p` | ✅ | ✅ | 一致 |
| `--export` | ✅ 支持 HTML/JSONL | ✅ 仅 HTML | **TS 多了 JSONL 导出** |
| `--models` | ✅ glob/逗号分隔 Ctrl+P 模型列表 | ❌ | **TS 多了模型列表限定** |
| `--no-tools` | ✅ | ✅ | 一致 |
| `--no-builtin-tools` | ✅ | ✅ | 一致 |
| `--tools` | ✅ 逗号分隔白名单 | ❌ | **TS 多了工具白名单** |
| `--extension / -e` | ✅ | ✅ | 一致 |
| `--no-extensions` | ✅ | ✅ | 一致 |
| `--skill` | ✅ 加载指定 skill | ❌ | **TS 多了此参数** |
| `--no-skills` | ✅ | ✅ | 一致 |
| `--prompt-template` | ✅ | ❌ | **TS 多了此参数** |
| `--no-prompt-templates` | ✅ | ✅ | 一致 |
| `--theme` | ✅ 加载指定 theme | ❌ | **TS 多了此参数** |
| `--no-themes` | ✅ | ❌ | **TS 多了此参数** |
| `--no-context-files` | ✅ | ✅ | 一致 |
| `--list-models` | ✅ 支持模糊搜索 | ✅ | **TS 多了搜索过滤** |
| `--list-sessions` | ❌ | ✅ | **piso 多了此参数** |
| `--offline` | ✅ | ✅ | 一致 |
| `--verbose` | ✅ | ✅ | 一致 |
| `--init` | ❌ | ✅ | **piso 多了配置初始化** |
| `@file` 文件引用 | ✅ `@prompt.md @image.png` | ❌ | **TS 多了文件引用语法** |
| 子命令 | ✅ install/remove/update/list/config | ✅ completions | **TS 有扩展管理子命令** |
| `unknownFlags` | ✅ 扩展可注册自定义 flag | ❌ | **TS 支持扩展 flag** |

**小结**：TS 有 22 个参数选项，piso 有 18 个。TS 独有约 12 个参数，piso 独有 3 个。

---

## 2. LLM Provider 支持

| Provider | TS (pi) | Rust (piso) | 差异 |
|----------|---------|-------------|------|
| Anthropic Claude | ✅ 原生 SSE | ✅ 原生 SSE | 一致 |
| OpenAI GPT | ✅ completions + responses | ✅ completions | **TS 多了 responses API** |
| Google Gemini | ✅ | ✅ | 一致 |
| Google Vertex | ✅ | ❌ | **TS 独有** |
| Azure OpenAI | ✅ responses | ❌ | **TS 独有** |
| Amazon Bedrock | ✅ converse-stream | ❌ | **TS 独有** |
| Mistral | ✅ | ❌ | **TS 独有** |
| Cloudflare Workers AI | ✅ | ❌ | **TS 独有** |
| GitHub Copilot | ✅ OAuth + headers | ❌ | **TS 独有** |
| Groq | ✅ (via openai-completions) | ✅ (via openai) | 一致 |
| Together | ✅ (via openai) | ✅ (via openai) | 一致 |
| DeepSeek | ✅ (via openai) | ✅ (via openai) | 一致 |
| xAI Grok | ✅ (via openai) | ✅ (via openai) | 一致 |
| Fireworks | ✅ (via openai) | ✅ (via openai) | 一致 |
| OpenRouter | ✅ (via openai) | ✅ (via openai) | 一致 |
| GLM / 其他 OpenAI 兼容 | ✅ | ✅ | 一致 |
| 自定义 provider (扩展注册) | ✅ `registerProvider()` | ❌ | **TS 独有** |

**小结**：TS 有 ~17 个独立 provider 实现，piso 有 3 个 driver 覆盖 ~30+ provider（OpenAI 兼容协议）。

---

## 3. 内置工具

| 工具 | TS (pi) | Rust (piso) | 差异 |
|------|---------|-------------|------|
| `read` | ✅ | ✅ | 一致 |
| `bash` | ✅ | ✅ | 一致 |
| `edit` | ✅ 精确替换 | ✅ 精确替换 | 一致 |
| `write` | ✅ | ✅ | 一致 |
| `find` | ✅ glob 模式 | ✅ glob 模式 | 一致 |
| `grep` | ✅ (默认关闭) | ✅ (默认开启) | 默认状态不同 |
| `ls` | ✅ | ❌ | **TS 独有** |
| diff 输出 | ✅ (edit 自动附带) | ✅ (edit/write 自动附带) | 一致 |

**小结**：TS 有 7 个内置工具，piso 有 6 个（缺少 `ls`）。

---

## 4. Slash 命令

| 命令 | TS (pi) | Rust (piso) | 差异 |
|------|---------|-------------|------|
| `/model` | ✅ 打开选择器 UI | ✅ 打开选择器 / 直接切换 | 一致 |
| `/compact` | ✅ | ✅ | 一致 |
| `/export` | ✅ HTML + JSONL | ✅ 仅 HTML | **TS 多了 JSONL** |
| `/import` | ✅ 导入 JSONL 文件 | ❌ | **TS 独有** |
| `/share` | ✅ GitHub Gist 分享 | ❌ | **TS 独有** |
| `/copy` | ✅ 复制到剪贴板 | ❌ | **TS 独有** |
| `/name` | ✅ 设置会话名 | ❌ | **TS 独有** |
| `/session` | ✅ 显示会话统计 | ❌ | **TS 独有** |
| `/fork` | ✅ | ❌ | **TS 独有** |
| `/clone` | ✅ | ❌ | **TS 独有** |
| `/tree` | ✅ 会话分支导航 | ❌ | **TS 独有** |
| `/new` | ✅ 新建会话 | ❌ (Ctrl+N 存在但未实现) | **TS 独有** |
| `/resume` | ✅ | ❌ | **TS 独有** |
| `/reload` | ✅ 重载配置 | ❌ | **TS 独有** |
| `/quit` | ✅ | ✅ | 一致 |
| `/settings` | ✅ 打开设置 UI | ❌ | **TS 独有** |
| `/scoped-models` | ✅ | ❌ | **TS 独有** |
| `/login` | ✅ | ❌ | **TS 独有** |
| `/logout` | ✅ | ❌ | **TS 独有** |
| `/hotkeys` | ✅ | ❌ | **TS 独有** |
| `/changelog` | ✅ | ❌ | **TS 独有** |
| `/help` | ✅ | ✅ | 一致 |
| `/clear` | ❌ (无内置) | ✅ | **piso 独有** |
| `/usage` | ❌ | ✅ | **piso 独有** |
| `/cost` | ❌ | ✅ | **piso 独有** |
| `/find` | ❌ | ✅ 跨会话搜索 | **piso 独有** |
| `/grep` | ❌ | ✅ 当前会话搜索 | **piso 独有** |
| `/sessions` | ❌ | ✅ | **piso 独有** |
| `/branch` | ❌ | ✅ | **piso 独有** |

**小结**：TS 有 21 个 slash 命令，piso 有 12 个。TS 独有 15 个，piso 独有 6 个。

---

## 5. TUI 交互

| 功能 | TS (pi) | Rust (piso) | 差异 |
|------|---------|-------------|------|
| 五区布局 | ✅ | ✅ | 一致 |
| Markdown 渲染 | ✅ 完整（含表格/链接） | ✅ 完整（含表格/链接/diff） | piso 多了 diff 高亮 |
| 代码块语言标签 | ✅ | ✅ | 一致 |
| 流式文本渲染 | ✅ | ✅ | 一致 |
| 思考块显示 | ✅ | ✅ | 一致 |
| 工具执行显示 | ✅ | ✅ | 一致 |
| 模型选择器 (Ctrl+P) | ✅ | ✅ | 一致 |
| 会话选择器 (Ctrl+S) | ✅ | ✅ | 一致 |
| Thinking level 选择器 | ✅ | ❌ | **TS 独有** |
| 设置选择器 | ✅ | ❌ | **TS 独有** |
| 配置选择器 | ✅ | ❌ | **TS 独有** |
| 登录对话框 | ✅ OAuth UI | ❌ | **TS 独有** |
| 图片粘贴 | ✅ 剪贴板图片 | ❌ | **TS 独有** |
| @file 引用 | ✅ autocomplete | ❌ | **TS 独有** |
| Autocomplete | ✅ 792 行完整实现 | ✅ 仅路径补全 | **TS 远超 piso** |
| Diff 查看器 | ✅ 交互式 diff | ❌ (仅内联 diff 文本) | **TS 独有** |
| 自定义编辑器组件 | ✅ Widget (above/below) | ✅ Hint 注入 | TS 更丰富 |
| Keybinding 配置 | ✅ JSON | ✅ JSON | 一致 |
| Theme 配置 | ✅ 多主题加载 | ✅ JSON | 一致 |
| Git 状态显示 | ❌ | ✅ header 显示分支 | **piso 独有** |
| Token 统计显示 | ✅ | ✅ | 一致 |
| Duration 计时 | ✅ | ✅ | 一致 |
| Context 使用率 | ✅ | ✅ | 一致 |
| 费用估算 | ❌ | ✅ | **piso 独有** |
| Tab 路径补全 | ❌ | ✅ | **piso 独有** |
| Ctrl+R 重试 | ❌ | ✅ | **piso 独有** |
| Shell completions | ❌ | ✅ bash/zsh/fish | **piso 独有** |
| 终端图片显示 | ✅ | ❌ | **TS 独有** |
| 倒计时显示 | ✅ | ❌ | **TS 独有** |

---

## 6. 扩展系统

| 能力 | TS (pi) | Rust (piso) | 差异 |
|------|---------|-------------|------|
| 事件订阅 | ✅ **30+ 事件类型** | ✅ **5 个生命周期钩子** | **TS 远超 piso** |
| 工具注册 | ✅ schema + state | ✅ 闭包 handler | TS 更强大 |
| 命令注册 | ✅ | ✅ | 一致 |
| 消息渲染器 | ✅ 按类型注册 | ✅ 全局拦截 | TS 更灵活 |
| 编辑器 Widget | ✅ above/below 布局 | ✅ hint 文本 | **TS 更丰富** |
| 快捷键注册 | ✅ `registerShortcut()` | ❌ | **TS 独有** |
| CLI flag 注册 | ✅ `registerFlag()` | ❌ | **TS 独有** |
| Provider 注册 | ✅ `registerProvider()` | ❌ | **TS 独有** |
| 消息发送 | ✅ sendMessage/sendUserMessage | ❌ | **TS 独有** |
| 会话元数据 | ✅ setName/setLabel | ❌ | **TS 独有** |
| exec 命令 | ✅ | ❌ | **TS 独有** |
| Shared EventBus | ✅ | ✅ | 一致 |
| 动态加载 | ✅ JS/WASM (未来) | ❌ 仅 Rust 原生 | TS 可扩展性更好 |

---

## 7. RPC 模式

| 命令 | TS (pi) | Rust (piso) |
|------|---------|-------------|
| prompt | ✅ | ✅ |
| steer | ✅ | ❌ |
| follow_up | ✅ | ❌ |
| abort | ✅ | ✅ |
| get_state | ✅ | ✅ |
| new_session | ✅ | ✅ |
| set_model | ✅ | ❌ |
| cycle_model | ✅ | ❌ |
| get_available_models | ✅ | ❌ |
| set_thinking_level | ✅ | ❌ |
| compact | ✅ | ❌ |
| set_auto_compaction | ✅ | ❌ |
| set_auto_retry | ✅ | ❌ |
| bash | ✅ | ❌ |
| get_messages | ✅ | ✅ |
| get_commands | ✅ | ❌ |

---

## 8. 会话管理

| 功能 | TS (pi) | Rust (piso) | 差异 |
|------|---------|-------------|------|
| JSONL 格式 | ✅ | ✅ 双向兼容 | 一致 |
| 多轮对话 | ✅ | ✅ | 一致 |
| 会话持久化 | ✅ `.pi/sessions/` | ✅ `.piso/sessions/` | 独立目录 |
| --continue 恢复 | ✅ | ✅ | 一致 |
| 会话选择器 | ✅ | ✅ | 一致 |
| 会话分支 | ✅ tree 导航 | ✅ create_branch | TS 更丰富 |
| 会话压缩 | ✅ | ✅ | 一致 |
| 会话导出 HTML | ✅ | ✅ | 一致 |
| 会话导入 JSONL | ✅ | ❌ | **TS 独有** |
| 会话分享 | ✅ GitHub Gist | ❌ | **TS 独有** |
| 会话命名 | ✅ | ❌ | **TS 独有** |
| 会话克隆 | ✅ | ❌ | **TS 独有** |
| 会话搜索 | ❌ | ✅ /find + /grep | **piso 独有** |

---

## 汇总

| 维度 | TS (pi) | Rust (piso) | piso 缺失关键项 |
|------|---------|-------------|-----------------|
| CLI 参数 | 22 个 | 18 个 | `--models`, `--tools`, `--session-dir`, `@file`, 子命令 |
| LLM Provider | ~17 独立实现 | 3 driver + 30+ 兼容 | Vertex, Azure, Bedrock, Mistral, Cloudflare, Copilot |
| 内置工具 | 7 个 | 6 个 | `ls` |
| Slash 命令 | 21 个 | 12 个 | `/share`, `/copy`, `/tree`, `/login`, `/import`, `/reload` 等 |
| TUI 组件 | ~35 个文件 | ~15 个文件 | OAuth 登录, diff 查看器, 图片, @file autocomplete |
| 扩展 API | 30+ 事件 + 10+ 注册 | 5 钩子 + 4 注册 | 事件粒度, provider/shortcut/flag 注册 |
| RPC 命令 | 16 个 | 5 个 | steer, follow_up, model 控制, bash, compact |
| 独有功能 | — | — | `/cost`, `/find`, `/grep`, Tab 补全, Git 状态, Ctrl+R 重试, `--init`, shell completions |

### piso 相对 TS 版的**缺失优先级排序**：

**P0 (核心缺失)**:
1. `@file` 文件引用 + image 支持
2. 完整 Autocomplete（命令/模型/file 混合补全）
3. 扩展事件粒度（至少 15 个核心事件）

**P1 (重要缺失)**:
4. `ls` 工具
5. 会话导入 `/import`
6. 更多 provider 原生实现（Vertex, Bedrock, Copilot）
7. Diff 查看器（交互式）
8. RPC 完整命令集

**P2 (锦上添花)**:
9. `/share` Gist 分享
10. `/tree` 分支导航
11. OAuth 登录 UI
12. 图片粘贴/显示
13. 扩展 provider 注册
14. 子命令 (install/remove-update)
