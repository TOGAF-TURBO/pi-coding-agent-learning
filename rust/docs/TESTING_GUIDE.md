# piso 内部测试指南

> piso 是 pi 的 Rust 版本——一个终端 AI 编码助手。单二进制文件，无依赖，支持 TUI 交互模式。

## 1. 获取二进制文件

从构建产物中获取：

```bash
# 文件位置（构建后）
rust/target/release/piso

# 拷贝到 PATH 中任意目录即可使用
cp rust/target/release/piso ~/.local/bin/piso
```

验证：

```bash
piso --version
# 输出: piso 0.1.0
```

**系统要求**：Linux x86-64，glibc 2.17+（CentOS 7+、Ubuntu 18.04+ 均可）。无其他依赖。

## 2. 配置 LLM Provider

有三种方式（按优先级排列）。

### 方式 A：配置文件（推荐）

创建 `~/.piso/models.json`：

```json
{
  "providers": {
    "glm": {
      "name": "Zhipu GLM",
      "baseUrl": "https://open.bigmodel.cn/api/coding/paas/v4",
      "apiKey": "你的API密钥",
      "api": "openai-completions",
      "models": [
        { "id": "glm-5.1", "name": "GLM-5.1" }
      ]
    }
  }
}
```

`api` 字段决定使用哪个驱动：

| api 值 | 适用 Provider |
|--------|--------------|
| `openai-completions` | GLM、DeepSeek、Groq、OpenRouter、Together 等 OpenAI 兼容 API |
| `anthropic-messages` | Anthropic Claude |
| `google-gemini` | Google Gemini |

可配置多个 provider，piso 会根据 `--provider` 参数选择。

### 方式 B：环境变量

```bash
export GLM_API_KEY="你的API密钥"
```

支持的环境变量：

| 环境变量 | Provider |
|---------|----------|
| `ANTHROPIC_API_KEY` | Anthropic |
| `OPENAI_API_KEY` | OpenAI |
| `GLM_API_KEY` | GLM (智谱) |
| `DEEPSEEK_API_KEY` | DeepSeek |
| `GOOGLE_API_KEY` / `GEMINI_API_KEY` | Google Gemini |
| `GROQ_API_KEY` | Groq |

### 方式 C：命令行参数（零配置）

```bash
piso \
  --provider glm \
  --model glm-5.1 \
  --api-key "你的API密钥" \
  --base-url "https://open.bigmodel.cn/api/coding/paas/v4/chat/completions" \
  -p "your prompt"
```

## 3. 验证连通性

```bash
# 有配置文件时
piso --provider glm --model glm-5.1 -p "say hello"

# 查看已配置的 provider
piso --list-models
```

正常输出示例：

```
hello
```

## 4. 启动 TUI 交互模式

```bash
piso --provider glm --model glm-5.1
```

进入后你会看到五区布局：

```
 piso glm-5.1 (glm)  20260511-124 [Ctrl+O submit, Ctrl+C quit]

                      ← header：模型 + 会话 ID
                      ← chat：消息流（滚动区域）

 STREAMING            ← status：当前状态
────────────────────── ← editor：用户输入
Type a message...
                      ← footer：快捷键提示
 Ctrl+O Send   Ctrl+C Quit   PgUp/PgDn Scroll
```

### 4.1 基本操作

| 操作 | 按键 | 说明 |
|------|------|------|
| 输入文字 | 直接打字 | 支持 Unicode（中文、emoji） |
| 换行 | `Enter` | 多行输入 |
| 发送 | `Ctrl+O` | 提交消息给 LLM |
| 取消/中止 | `Esc` | 清空输入 或 中止正在运行的 agent |
| 向上滚动 | `PageUp` | 查看历史消息 |
| 向下滚动 | `PageDown` | 回到最新消息 |
| 退出 | `Ctrl+C` | 退出 TUI，终端恢复 |

### 4.2 状态指示

| 状态 | 含义 |
|------|------|
| `READY` (绿) | 空闲，等待输入 |
| `STREAMING` (青) | 正在接收 LLM 文本 |
| `THINKING` (黄) | 等待 LLM 响应 |
| `Running bash...` (紫) | 正在执行工具 |
| `ERR: ...` (红) | 出错 |

### 4.3 对话示例

```
You
   create a file /tmp/hello.txt with content "hello piso"

write
   Wrote 11 bytes, 1 lines to /tmp/hello.txt

read
        1       hello piso

Assistant
   File created and verified.
```

工具调用结果自动显示在 chat 中。LLM 可以连续调用多个工具（最多 50 轮）。

### 4.4 多轮记忆

piso 在一次 TUI 会话中保持完整的对话历史。LLM 能记住之前说过的内容：

```
You: my secret number is 42
Assistant: Got it, 42.

You: what is my secret number?
Assistant: 42.
```

### 4.5 恢复上次会话

```bash
# 继续上次对话（历史消息会加载到 TUI）
piso --provider glm --model glm-5.1 -c
```

## 5. 其他运行模式

### Print 模式（非交互）

```bash
# 单次问答，输出到 stdout
piso --provider glm --model glm-5.1 -p "what is 2+2"

# 带工具调用
piso --provider glm --model glm-5.1 -p "read /etc/hostname"

# 管道友好
echo "list files in /tmp" | piso --provider glm --model glm-5.1 -p -
```

**注意**：`--provider`、`--model` 等选项必须放在 `-p` 之前。

### RPC 模式（JSON-over-stdio）

供 IDE 插件或脚本调用：

```bash
piso --provider glm --model glm-5.1 --mode rpc
```

stdin 输入 JSON 命令，stdout 输出 JSON 事件：

```bash
echo '{"type":"prompt","message":"say ok"}' | piso --mode rpc
# stdout: {"type":"state_change","state":"running"}
# stdout: {"type":"text_delta","text":"ok"}
# stdout: {"type":"done","id":null}
```

支持的命令：`prompt`、`abort`、`get_state`、`get_messages`、`new_session`。

### 查看历史会话

```bash
piso --list-sessions
```

## 6. 配置目录结构

```
~/.piso/
  ├── models.json        ← provider 配置（优先）
  └── auth.json          ← API key 存储（可选）

<cwd>/.piso/
  ├── sessions/          ← 会话 JSONL 文件
  │   └── 20260511-1203-xxxx/
  │       └── session.jsonl
  └── skills/            ← 技能定义（可选）
```

piso 使用独立的 `~/.piso/` 目录，不会与 TS 版本的 `~/.pi/` 冲突。但会向后兼容读取 `~/.pi/agent/models.json`。

## 7. 内置工具

| 工具 | 功能 | 示例 |
|------|------|------|
| `bash` | 执行 shell 命令 | `run ls -la` |
| `read` | 读取文件（带行号） | `read the file main.rs` |
| `write` | 写入文件 | `create a file with content ...` |
| `edit` | 精确替换文件内容 | `change "hello" to "world" in main.rs` |
| `find` | 查找文件 | `find all .rs files in src/` |
| `grep` | 搜索文件内容 | `search for "TODO" in all files` |

禁用工具：

```bash
piso --provider glm --model glm-5.1 -n -p "just chat, no tools"
```

## 8. AGENTS.md 上下文

如果工作目录或其父目录包含 `AGENTS.md`、`CLAUDE.md` 等文件，piso 会自动注入到系统提示中：

```bash
# 在项目根目录启动时，AGENTS.md 内容自动加载
cd /your/project && piso --provider glm --model glm-5.1
```

禁用：

```bash
piso --provider glm --model glm-5.1 --no-context-files
```

## 9. 常见问题

### Q: 启动报 "No API key found"

确认配置了 API key（三种方式见第 2 节）。检查：

```bash
piso --list-models
```

### Q: `--provider` 参数没生效

**`--provider` 必须放在 `-p` 之前**：

```bash
# 正确
piso --provider glm --model glm-5.1 -p "hello"

# 错误 — --provider 被当作消息文本
piso -p "hello" --provider glm --model glm-5.1
```

### Q: TUI 退出后终端显示异常

piso 在退出时会恢复终端状态。如果异常（如进程被 kill），手动修复：

```bash
reset
```

### Q: LLM 返回 401 / 403

检查 API key 是否正确。如果是 OpenAI 兼容 provider，确认 `api` 字段是 `"openai-completions"` 且 `baseUrl` 正确。

### Q: 工具执行超时

bash 工具有 30 秒超时。长时间运行的命令会被自动终止。

### Q: 中文显示乱码

确认终端编码为 UTF-8：

```bash
echo $LANG
# 应输出类似 en_US.UTF-8 或 zh_CN.UTF-8
```

## 10. 快速启动模板

### 智谱 GLM

```bash
# 1. 创建配置
mkdir -p ~/.piso
cat > ~/.piso/models.json << 'EOF'
{
  "providers": {
    "glm": {
      "name": "Zhipu GLM",
      "baseUrl": "https://open.bigmodel.cn/api/coding/paas/v4",
      "apiKey": "在此填入你的API密钥",
      "api": "openai-completions",
      "models": [{ "id": "glm-5.1", "name": "GLM-5.1" }]
    }
  }
}
EOF

# 2. 测试
piso --provider glm --model glm-5.1 -p "hello"

# 3. 启动 TUI
piso --provider glm --model glm-5.1
```

### Anthropic Claude

```bash
export ANTHROPIC_API_KEY="sk-ant-..."
piso --model claude-sonnet-4-20250514 -p "hello"
piso --model claude-sonnet-4-20250514
```

### DeepSeek

```bash
mkdir -p ~/.piso
cat > ~/.piso/models.json << 'EOF'
{
  "providers": {
    "deepseek": {
      "name": "DeepSeek",
      "baseUrl": "https://api.deepseek.com",
      "apiKey": "在此填入你的API密钥",
      "api": "openai-completions",
      "models": [{ "id": "deepseek-chat", "name": "DeepSeek V3" }]
    }
  }
}
EOF

piso --provider deepseek --model deepseek-chat
```
