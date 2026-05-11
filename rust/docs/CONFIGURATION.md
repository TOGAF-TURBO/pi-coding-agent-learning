# piso 配置指南

piso 使用 `~/.piso/` 作为配置目录，与 TS 版 pi 的 `~/.pi/` 完全独立。

## 目录结构

```
~/.piso/
├── auth.json            # API key 存储
├── models.json          # Provider + model 定义
├── settings.json        # 全局设置
├── keybindings.json     # 快捷键自定义
├── sessions/            # 会话存储
│   └── --home-user-project--/
│       ├── 20260511-1556-83a5.jsonl
│       └── ...
└── skills/              # 全局技能
    └── my-skill/
        └── SKILL.md
```

## API 配置

### 方式一：环境变量

```bash
export ANTHROPIC_API_KEY=sk-...
export OPENAI_API_KEY=sk-...
export GLM_API_KEY=...
```

### 方式二：models.json

```json
{
  "providers": {
    "glm": {
      "name": "Zhipu GLM",
      "baseUrl": "https://open.bigmodel.cn/api/coding/paas/v4",
      "apiKey": "your-key",
      "api": "openai-completions",
      "models": [
        { "id": "glm-5.1", "name": "GLM-5.1 (旗舰)" },
        { "id": "glm-5", "name": "GLM-5 (基座)" },
        { "id": "glm-4.7", "name": "GLM-4.7 (推理)" }
      ]
    },
    "deepseek": {
      "name": "DeepSeek",
      "baseUrl": "https://api.deepseek.com",
      "apiKey": "your-key",
      "api": "openai-completions",
      "models": [
        { "id": "deepseek-chat", "name": "DeepSeek V3" }
      ]
    }
  }
}
```

支持的 `api` 类型：
- `openai-completions` — OpenAI, DeepSeek, GLM, Groq, Together, etc.
- `anthropic` — Anthropic Claude
- `google-gemini` — Google Gemini

### 方式三：auth.json

```json
{
  "anthropic": "sk-ant-...",
  "openai": "sk-..."
}
```

## 设置 (settings.json)

```json
{
  "model": "glm-5.1",
  "provider": "glm",
  "thinking": "enabled",
  "max_tokens": 16384
}
```

项目级配置放在 `<project>/.piso/settings.json`，覆盖全局设置。

## 快捷键 (keybindings.json)

```json
{
  "submit": { "modifiers": "ctrl", "key": "o" },
  "quit": { "modifiers": "ctrl", "key": "c" },
  "cancel": { "modifiers": "none", "key": "escape" },
  "open_session_picker": { "modifiers": "ctrl", "key": "s" },
  "open_model_picker": { "modifiers": "ctrl", "key": "p" },
  "tab_complete": { "modifiers": "none", "key": "tab" },
  "scroll_up": { "modifiers": "none", "key": "pageup" },
  "scroll_down": { "modifiers": "none", "key": "pagedown" }
}
```

`modifiers` 可选值：`none`, `ctrl`, `alt`, `shift`, `ctrl+alt`。
`key` 可选值：字母 (`a`-`z`), `enter`, `esc`, `tab`, `backspace`, `delete`, `home`, `end`, `pageup`, `pagedown`, `up`, `down`, `left`, `right`, `space`。

## Slash 命令

在输入框中输入以 `/` 开头的命令：

| 命令 | 别名 | 说明 |
|------|------|------|
| `/help` | `/h` `/?` | 显示帮助 |
| `/clear` | `/cls` | 清空聊天 |
| `/model [name]` | `/m` | 切换模型 |
| `/export [path]` | `/e` | 导出 HTML |
| `/usage` | `/u` | Token 用量 |
| `/sessions` | `/s` | 打开会话选择器 |
| `/compact` | | 压缩上下文 |
| `/quit` | `/q` | 退出 |

## CLI 用法

```bash
# 交互模式（TUI）
piso --provider glm --model glm-5.1

# 单次提问
piso -p "explain Rust ownership"

# 管道输入
echo "what is 2+2?" | piso --provider glm --model glm-5.1

# 恢复上次会话
piso --continue

# 打开指定会话
piso --session 20260511-1556-83a5

# 列出所有会话
piso --list-sessions

# 列出配置的模型
piso --list-models

# 导出会话为 HTML
piso --export output.html --continue

# 生成 shell 补全
piso completions bash > ~/.local/share/bash-completion/completions/piso
piso completions zsh > ~/.zfunc/_piso
piso completions fish > ~/.config/fish/completions/piso.fish
```

## Tab 补全

在输入框中输入路径片段后按 Tab：
- `./src/` → 列出目录内容
- `/home/user/` → 绝对路径补全
- `~/Documents/` → Home 目录展开

单一候选自动补全，多个候选显示列表并选第一个。

## 会话存储

会话存储在 `~/.piso/sessions/--<cwd-safe-path>--/` 下。
不同工作目录的会话相互隔离。

CWD 编码规则（与 TS 版 pi 兼容）：
```
/home/user/project → --home-user-project--
```
