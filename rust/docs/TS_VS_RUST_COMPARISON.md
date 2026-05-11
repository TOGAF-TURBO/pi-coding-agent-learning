# TypeScript (pi) vs Rust (piso) CLI 功能对比

> 仅从 CLI 功能角度对比，不含性能/体积等非功能性差异。
> 最后更新：2026-05-11（commit 0e21cea）

---

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
| `--session-dir` | ✅ | ✅ | 一致 |
| `--no-session` | ✅ | ✅ | 一致 |
| `--mode` | ✅ text/json/rpc | ❌ (自动推断) | **TS 显式模式切换** |
| `--print / -p` | ✅ | ✅ | 一致 |
| `--export` | ✅ 支持 HTML/JSONL | ✅ 仅 HTML | **TS 多了 JSONL 导出** |
| `--models` | ✅ glob/逗号分隔 | ✅ 逗号分隔过滤 Ctrl+P | 一致 |
| `--no-tools` | ✅ | ✅ | 一致 |
| `--no-builtin-tools` | ✅ | ✅ | 一致 |
| `--tools` | ✅ 逗号分隔白名单 | ✅ 逗号分隔白名单 | 一致 |
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
| `@file` 文件引用 | ✅ `@prompt.md @image.png` | ✅ `@file.txt` 文本内联 | **TS 多了图片支持** |
| 子命令 | ✅ install/remove/update/list/config | ✅ completions | **TS 有扩展管理子命令** |
| `unknownFlags` | ✅ 扩展可注册自定义 flag | ✅ `register_shortcut()` | TS 更灵活 |

**小结**：TS 有 22 个参数选项，piso 有 21 个。TS 独有约 7 个参数，piso 独有 4 个。

---

## 2. LLM Provider 支持

| Provider | TS (pi) | Rust (piso) | 差异 |
|----------|---------|-------------|------|
| Anthropic Claude | ✅ 原生 SSE | ✅ 原生 SSE | 一致 |
| OpenAI GPT | ✅ completions + responses | ✅ completions + responses | 一致 |
| Google Gemini | ✅ | ✅ | 一致 |
| Google Vertex | ✅ | ❌ | **TS 独有** |
| Azure OpenAI | ✅ responses | ✅ 部署映射 + api-key 头部 | 一致 |
| Amazon Bedrock | ✅ converse-stream | ✅ converse-stream + AWS env | 一致（piso 简化 Sigv4） |
| Mistral | ✅ | ✅ (via openai) | piso 通过兼容协议覆盖 |
| Cloudflare Workers AI | ✅ | ❌ | **TS 独有** |
| GitHub Copilot | ✅ OAuth + headers | ❌ | **TS 独有** |
| Groq | ✅ (via openai-completions) | ✅ (via openai) | 一致 |
| Together | ✅ (via openai) | ✅ (via openai) | 一致 |
| DeepSeek | ✅ (via openai) | ✅ (via openai) | 一致 |
| xAI Grok | ✅ (via openai) | ✅ (via openai) | 一致 |
| Fireworks | ✅ (via openai) | ✅ (via openai) | 一致 |
| OpenRouter | ✅ (via openai) | ✅ (via openai) | 一致 |
| GLM / 其他 OpenAI 兼容 | ✅ | ✅ | 一致 |
| 自定义 provider (扩展注册) | ✅ `registerProvider()` | ✅ `register_provider()` | 一致 |

**小结**：TS 有 ~17 个独立 provider 实现，piso 有 6 个 driver（Anthropic、OpenAI completions、OpenAI responses、Gemini、Azure、Bedrock）覆盖 ~30+ provider。

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
| `ls` | ✅ | ✅ 目录列表 + 排序 + 隐藏文件 | 一致 |
| diff 输出 | ✅ (edit 自动附带) | ✅ (edit/write 自动附带) | 一致 |

**小结**：TS 和 piso 均有 7 个内置工具。

---

## 4. Slash 命令

| 命令 | TS (pi) | Rust (piso) | 差异 |
|------|---------|-------------|------|
| `/help` | ✅ | ✅ | 一致 |
| `/quit` | ✅ | ✅ | 一致 |
| `/compact` | ✅ | ✅ | 一致 |
| `/export` | ✅ HTML + JSONL | ✅ 仅 HTML | **TS 多了 JSONL** |
| `/model` | ✅ 打开选择器 UI | ✅ 打开选择器 / 直接切换 | 一致 |
| `/new` | ✅ 新建会话 | ✅ | 一致 |
| `/reload` | ✅ 重载配置 | ✅ 重载 keybindings | 一致 |
| `/copy` | ✅ 复制到剪贴板 | ✅ xclip/pbcopy | 一致 |
| `/fork` | ✅ | ✅ 占位 | 一致 |
| `/session` | ✅ 显示会话统计 | ✅ | 一致 |
| `/name` | ✅ 设置会话名 | ✅ | 一致 |
| `/import` | ✅ 导入 JSONL 文件 | ❌ | **TS 独有** |
| `/share` | ✅ GitHub Gist 分享 | ❌ | **TS 独有** |
| `/clone` | ✅ | ❌ | **TS 独有** |
| `/tree` | ✅ 会话分支导航 | ❌ | **TS 独有** |
| `/resume` | ✅ | ❌ | **TS 独有** |
| `/settings` | ✅ 打开设置 UI | ❌ | **TS 独有** |
| `/scoped-models` | ✅ | ❌ | **TS 独有** |
| `/login` | ✅ | ❌ | **TS 独有** |
| `/logout` | ✅ | ❌ | **TS 独有** |
| `/hotkeys` | ✅ | ❌ | **TS 独有** |
| `/changelog` | ✅ | ❌ | **TS 独有** |
| `/clear` | ❌ | ✅ | **piso 独有** |
| `/usage` | ❌ | ✅ | **piso 独有** |
| `/cost` | ❌ | ✅ | **piso 独有** |
| `/find` | ❌ | ✅ 跨会话搜索 | **piso 独有** |
| `/grep` | ❌ | ✅ 当前会话搜索 | **piso 独有** |
| `/sessions` | ❌ | ✅ | **piso 独有** |
| `/branch` | ❌ | ✅ | **piso 独有** |

**小结**：TS 有 21 个 slash 命令，piso 有 17 个。TS 独有 10 个，piso 独有 6 个。

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
| @file 引用 | ✅ autocomplete | ✅ 解析 + 内联 | piso 解析但没有 Tab 补全 |
| Autocomplete | ✅ 792 行完整实现 | ✅ 混合补全（slash/@file/路径） | 一致（TS 更成熟） |
| Diff 查看器 | ✅ 交互式 diff | ❌ (仅内联 diff 文本) | **TS 独有** |
| 自定义编辑器组件 | ✅ Widget (above/below) | ✅ Hint 注入 | TS 更丰富 |
| Keybinding 配置 | ✅ JSON | ✅ JSON | 一致 |
| Theme 配置 | ✅ 多主题加载 | ✅ JSON | 一致 |
| Git 状态显示 | ❌ | ✅ header 显示分支 | **piso 独有** |
| Token 统计显示 | ✅ | ✅ | 一致 |
| Duration 计时 | ✅ | ✅ | 一致 |
| Context 使用率 | ✅ | ✅ | 一致 |
| 费用估算 | ❌ | ✅ | **piso 独有** |
| Tab 补全 | ✅ | ✅ slash命令 + @file + 路径 | piso 实现不同但覆盖度一致 |
| Ctrl+R 重试 | ❌ | ✅ | **piso 独有** |
| Shell completions | ❌ | ✅ bash/zsh/fish | **piso 独有** |
| 终端图片显示 | ✅ | ❌ | **TS 独有** |
| 倒计时显示 | ✅ | ❌ | **TS 独有** |

---

## 6. 扩展系统

| 能力 | TS (pi) | Rust (piso) | 差异 |
|------|---------|-------------|------|
| 事件订阅 | ✅ **30+ 事件类型** | ✅ **15+ 事件** | TS 仍有更多事件 |
| 工具注册 | ✅ schema + state | ✅ 闭包 handler | TS 更强大 |
| 命令注册 | ✅ | ✅ | 一致 |
| 消息渲染器 | ✅ 按类型注册 | ✅ 全局拦截 | TS 更灵活 |
| 编辑器 Widget | ✅ above/below 布局 | ✅ hint 文本 | **TS 更丰富** |
| 快捷键注册 | ✅ `registerShortcut()` | ✅ `register_shortcut()` | 一致 |
| CLI flag 注册 | ✅ `registerFlag()` | ❌ | **TS 独有** |
| Provider 注册 | ✅ `registerProvider()` | ✅ `register_provider()` | 一致 |
| 消息发送 | ✅ sendMessage/sendUserMessage | ✅ send_message/send_user_message | 一致 |
| 会话元数据 | ✅ setName/setLabel | ✅ set_session_name/set_label | 一致 |
| exec 命令 | ✅ | ❌ | **TS 独有** |
| Shared EventBus | ✅ | ✅ | 一致 |
| 动态加载 | ✅ JS/WASM (未来) | ❌ 仅 Rust 原生 | TS 可扩展性更好 |

---

## 7. RPC 模式

| 命令 | TS (pi) | Rust (piso) |
|------|---------|-------------|
| prompt | ✅ | ✅ |
| abort | ✅ | ✅ |
| get_state | ✅ | ✅ |
| new_session | ✅ | ✅ |
| get_messages | ✅ | ✅ |
| steer | ✅ | ✅ |
| follow_up | ✅ | ✅ |
| set_model | ✅ | ✅ |
| cycle_model | ✅ | ✅ |
| get_available_models | ✅ | ✅ |
| compact | ✅ | ✅ |
| bash | ✅ | ✅ |
| get_commands | ✅ | ✅ |
| set_thinking_level | ✅ | ❌ |
| set_auto_compaction | ✅ | ❌ |
| set_auto_retry | ✅ | ❌ |

**小结**：TS 有 16 个 RPC 命令，piso 有 13 个。piso 缺少 3 个（thinking level、auto compaction、auto retry）。

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
| 会话命名 | ✅ | ✅ /name | 一致 |
| 会话克隆 | ✅ | ❌ | **TS 独有** |
| 会话搜索 | ❌ | ✅ /find + /grep | **piso 独有** |
| @file 引用 | ✅ | ✅ 文本内联 | 一致 |

---

## 9. 测试覆盖

| 维度 | TS (pi) | Rust (piso) |
|------|---------|-------------|
| 单元测试 | ~150+ (跨多个包) | **244** |
| 集成测试 | TS 有 suite harness | TUI 通过 tmux E2E |
| 覆盖的 crate | 5 packages | 8 crates 全覆盖 |

---

## 汇总

| 维度 | TS (pi) | Rust (piso) | 差距 |
|------|---------|-------------|------|
| CLI 参数 | 22 个 | 21 个 | TS 多 1 个 |
| LLM Provider | ~17 独立实现 | 6 driver + 30+ 兼容 | TS 多 Vertex, Cloudflare, Copilot |
| 内置工具 | 7 个 | 7 个 | 一致 |
| Slash 命令 | 21 个 | 17 个 | TS 多 4 个 |
| TUI 组件 | ~35 个文件 | ~15 个文件 | TS 多 OAuth 登录, diff 查看器, 图片 |
| 扩展 API | 30+ 事件 + 10+ 注册 | 15+ 事件 + 9 注册 | TS 多 flag 注册, exec |
| RPC 命令 | 16 个 | 13 个 | TS 多 3 个 |
| 单元测试 | ~150+ | 244 | piso 更多 |
| 独有功能 | — | — | `/cost`, `/find`, `/grep`, Tab 补全, Git 状态, Ctrl+R 重试, `--init`, shell completions |

### piso 相对 TS 版的**剩余缺失**：

**P1 (重要缺失)**:
1. `/import` JSONL 导入
2. `/clone` 会话克隆
3. 更多 provider 原生实现（Vertex, Cloudflare, Copilot OAuth）
4. 交互式 Diff 查看器
5. 图片粘贴/显示
6. `register_flag()` 扩展 API
7. RPC: set_thinking_level, set_auto_compaction, set_auto_retry

**P2 (锦上添花)**:
8. `/share` Gist 分享
9. `/tree` 分支导航
10. OAuth 登录 UI
11. `/settings` 设置 UI
12. `/hotkeys` 快捷键列表
13. `/changelog` 更新日志
14. `--mode` 显式模式切换
15. `--skill` / `--theme` / `--prompt-template` 选项
16. JSONL 导出格式
17. 终端图片显示
18. JS/WASM 动态扩展加载
