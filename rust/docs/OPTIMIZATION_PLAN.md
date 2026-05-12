# piso 优化方案

> 基于 `pi-engineering-analysis.md` 17 维度对照，精确识别 Rust 版差距并制定实施计划。

---

## 差距总览

| # | 差距 | 影响 | 难度 | 优先级 |
|---|------|------|------|--------|
| G1 | 工具并行/串行执行 | 性能（多工具调用 3-10x 延迟） | 中 | **P0** |
| G2 | 结构化压缩摘要 | Agent 长会话记忆质量 | 中 | **P0** |
| G3 | 进程树清理 | 安全（孤儿进程泄漏） | 小 | **P0** |
| G4 | 运行时参数校验 | 健壮性（LLM 输出错误参数不报错） | 小 | **P1** |
| G5 | terminate 信号 | 控制（工具无法主动停止循环） | 小 | **P1** |
| G6 | steering 内层轮询 | 控制（无法打断当前轮次） | 中 | **P1** |
| G7 | system prompt date 注入 | 准确性（时间幻觉） | 极小 | **P1** |
| G8 | 图片 resize + 降级 | 兼容性（大图浪费 token，不支持时无提示） | 小 | **P1** |
| G9 | 模型变更 JSONL 记录 | 可靠性（重放时上下文不一致） | 极小 | **P2** |
| G10 | Skill 运行时展开 | 便利性（无法在会话中切换 skill） | 小 | **P2** |
| G11 | JSONL 压缩实际写入 | 真正节省 token（当前仅 log） | 中 | **P2** |
| G12 | Extension transformContext/convertToLlm | 扩展性 | 中 | **P2** |

---

## P0：核心差距（影响基本可用性）

### G1：工具并行/串行执行

**现状**：`loop_engine.rs` 工具按顺序 for 循环执行。
**TS 参考**：`agent-loop.ts:430-460` 三阶段并行：预检 → Promise.all 并发 → 按序输出。

**方案**：在 `loop_engine.rs` 的 `run()` 方法中：

```rust
// 当前：
for tc in &response.tool_calls {
    let result = self.execute_tool(&tc.name, tc.input.clone()).await;
    // ...
}

// 改为：
let tool_calls = &response.tool_calls;

// 阶段一：判断是否有 sequential 工具
let has_sequential = tool_calls.iter().any(|tc|
    self.tools.is_sequential(&tc.name)
);

if has_sequential || tool_calls.len() == 1 {
    // 顺序执行（现有逻辑）
    for tc in tool_calls { ... }
} else {
    // 阶段一：并行执行所有工具
    let futures: Vec<_> = tool_calls.iter().map(|tc| {
        self.execute_tool(&tc.name, tc.input.clone())
    }).collect();
    let results = futures::future::join_all(futures).await;

    // 阶段二：按原始顺序处理结果
    for (i, result) in results.into_iter().enumerate() {
        // emit events, push tool_results
    }
}
```

**改动文件**：
- `crates/pi-tools/src/registry.rs`：`ToolRegistry::is_sequential(name) -> bool`（新增 1 方法）
- `crates/pi-types/src/tool.rs`：`ToolDefinition` 增加 `execution_mode: ExecutionMode` 字段
- `crates/pi-agent/src/loop_engine.rs`：替换顺序 for 为并行逻辑
- 新增测试：3 个并行工具调用的执行顺序和结果

**预估**：~80 行改动，3 个新测试。

---

### G2：结构化压缩摘要

**现状**：`compaction.rs` 使用通用 "Summarize this conversation" prompt，无 UPDATE 模式。
**TS 参考**：`compaction.ts:466-525` 有两套 prompt（SUMMARIZATION + UPDATE_SUMMARIZATION），严格模板格式。

**方案**：

```rust
const SUMMARIZATION_PROMPT: &str = r#"The messages above are a conversation to summarize.
Create a structured context checkpoint summary using this EXACT format:

## Goal
[What is the user trying to accomplish?]

## Constraints & Preferences
- [Any constraints or requirements]

## Progress
### Done
- [x] [Completed tasks]

### In Progress
- [ ] [Current work]

### Blocked
- [Issues preventing progress]

## Key Decisions
- **[Decision]**: [Rationale]

## Next Steps
1. [Ordered list]

## Critical Context
- [Data, paths, references needed to continue]

Keep each section concise. Preserve exact file paths, function names, error messages."#;

const UPDATE_SUMMARIZATION_PROMPT: &str = r#"The messages above are NEW messages to merge
into the existing summary in <previous-summary> tags.

RULES:
- PRESERVE all existing information
- ADD new progress, decisions, context
- UPDATE Progress: move In Progress → Done when completed
- UPDATE Next Steps based on current state
- Preserve exact file paths, function names, error messages

Use the SAME EXACT format as the previous summary."#;
```

同时修改 `compact()` 函数：
- 接收 `previous_summary: Option<&str>` 参数
- 有旧摘要时用 UPDATE prompt + `<previous-summary>` 包裹
- 将摘要存入 session（作为 compaction entry 追加到 JSONL）

**改动文件**：
- `crates/pi-agent/src/compaction.rs`：替换 prompt + 增加 UPDATE 逻辑 + 实际写入 session
- `crates/pi-agent/src/loop_engine.rs`：`compact_context()` 传入 previous_summary
- 新增测试：prompt 模板正确性、UPDATE vs FULL 分支

**预估**：~100 行改动，4 个新测试。

---

### G3：进程树清理

**现状**：`bash.rs` 用 `tokio::process::Command`，超时后进程变孤儿。
**TS 参考**：`shell.ts:187` 使用 `kill(-pid, SIGKILL)` 杀进程组。

**方案**：设置进程组 + 超时后杀进程组：

```rust
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;

// 在 Command 创建时：
let mut cmd = Command::new("bash");
cmd.arg("-c").arg(&command).current_dir(&self.cwd);
// 设置新进程组（Linux/macOS）
unsafe { cmd.pre_exec(|| { libc::setsid(); Ok(()) }); }

let mut child = cmd.spawn()?;

// 超时时杀进程组：
let pgid = Pid::from_raw(-child.id().unwrap() as i32);
signal::kill(pgid, Signal::SIGKILL).ok();
child.kill().await.ok();
```

需要添加 `nix` crate 依赖（仅 Unix），Windows 用 `taskkill /F /T /PID`。

**改动文件**：
- `crates/pi-tools/Cargo.toml`：添加 `nix = { version = "0.29", optional = true }`
- `crates/pi-tools/src/bash.rs`：重写为 spawn + 进程组 + 超时清理
- 新增测试：超时后确认无孤儿进程

**预估**：~60 行改动，2 个新测试。

---

## P1：重要改进（影响开发体验）

### G4：运行时参数校验

**现状**：工具参数从 LLM JSON 直接传给 execute()，无校验。
**TS 参考**：`validation.ts:296` 使用 TypeBox Value.Check 校验，失败抛错让 LLM 重试。

**方案**：在 `ToolDefinition` 上增加 JSON Schema 校验：

```rust
// pi-types/src/tool.rs 新增：
pub fn validate_input(&self, input: &Value) -> Result<Value, String> {
    // 基础校验：required 字段检查
    if let Some(required) = self.parameters.get("required").and_then(|r| r.as_array()) {
        let props = input.as_object()
            .ok_or_else(|| "tool arguments must be a JSON object".to_string())?;
        for field in required {
            if let Some(name) = field.as_str() {
                if !props.contains_key(name) {
                    return Err(format!(
                        "missing required parameter '{}' for tool '{}'",
                        name, self.name
                    ));
                }
            }
        }
    }

    // 类型校验：properties 中声明的类型
    if let Some(properties) = self.parameters.get("properties").and_then(|p| p.as_object()) {
        let input_obj = input.as_object().unwrap();
        for (key, schema) in properties {
            if let Some(value) = input_obj.get(key) {
                if let Some(expected_type) = schema.get("type").and_then(|t| t.as_str()) {
                    let actual_type = json_type_of(value);
                    if actual_type != expected_type {
                        return Err(format!(
                            "parameter '{}' expected type '{}', got '{}'",
                            key, expected_type, actual_type
                        ));
                    }
                }
            }
        }
    }

    Ok(input.clone())
}

fn json_type_of(v: &Value) -> &'static str {
    match v {
        Value::String(_) => "string",
        Value::Number(_) => "number",
        Value::Bool(_) => "boolean",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
        Value::Null => "null",
    }
}
```

在 `loop_engine.rs` 执行工具前调用：
```rust
// 在 execute_tool_inner 之前：
if let Some(tool_def) = self.tools.get_definition(&name) {
    if let Err(e) = tool_def.validate_input(&input) {
        return Ok(ToolResult { output: e, is_error: true, .. });
    }
}
```

**改动文件**：
- `crates/pi-types/src/tool.rs`：增加 `validate_input()` + `json_type_of()`
- `crates/pi-tools/src/registry.rs`：增加 `get_definition()` 方法
- `crates/pi-agent/src/loop_engine.rs`：执行前校验
- 新增测试：缺失必填字段、类型错误、正常通过

**预估**：~60 行改动，4 个新测试。

---

### G5：terminate 信号

**现状**：工具无法主动停止 Agent 循环。
**TS 参考**：工具返回 `terminate: true`，所有工具都 terminate 时停止。

**方案**：

```rust
// ToolResult 增加字段：
pub struct ToolResult {
    pub tool_use_id: String,
    pub output: String,
    pub is_error: bool,
    pub duration_ms: Option<u64>,
    pub terminate: bool,  // 新增：工具请求终止循环
}

// loop_engine.rs 检查：
let should_terminate: bool = tool_results.iter().all(|r| {
    if let ContentBlock::ToolResult(tr) = r { tr.terminate } else { false }
});
if should_terminate {
    break; // 跳出内层 ReAct 循环
}
```

**改动文件**：
- `crates/pi-types/src/tool.rs`：`ToolResult` 增加 `terminate: bool`
- `crates/pi-agent/src/loop_engine.rs`：检查 terminate
- **无需修改**：现有工具默认 terminate=false（serde default）

**预估**：~15 行改动，1 个新测试。

---

### G6：steering 内层轮询

**现状**：仅 followUp 队列（RPC 模式），无内层 steering 轮询。
**TS 参考**：内层循环末尾轮询 steering，外层末尾轮询 followUp。

**方案**：在 `loop_engine.rs` 的 `run()` 方法中增加双层循环 + steering channel：

```rust
pub fn with_steering_rx(mut self, rx: tokio::sync::mpsc::UnboundedReceiver<String>) -> Self {
    self.steering_rx = Some(rx);
    self
}

// run() 中：
let mut pending = Vec::new();

'outer: loop {
    let mut has_tool_calls = true;

    while has_tool_calls || !pending.is_empty() {
        // 消耗 steering 消息
        if !pending.is_empty() {
            for msg in pending.drain(..) {
                self.session.append(/* user message */);
            }
        }

        let response = self.call_llm(request).await?;
        // ... tool execution ...

        // 内层末尾：非阻塞轮询 steering
        while let Ok(msg) = self.steering_rx.try_recv() {
            pending.push(msg);
        }
    }

    // 外层末尾：阻塞等待 followUp
    if let Some(ref mut rx) = self.follow_up_rx {
        match rx.recv().await {
            Some(msg) => pending.push(msg),
            None => break 'outer,
        }
    } else {
        break 'outer;
    }
}
```

**改动文件**：
- `crates/pi-agent/src/loop_engine.rs`：双层循环 + steering/followUp channel
- `crates/pi-cli/src/rpc.rs`：steer → steering_tx.send()

**预估**：~80 行改动，2 个新测试。

---

### G7：system prompt date 注入

**现状**：system prompt 无日期注入。
**TS 参考**：`system-prompt.ts:97` 注入 `Current date: 2026-05-12`。

**方案**：

```rust
// system_prompt.rs build() 方法：
let date = chrono::Utc::now().format("%Y-%m-%d").to_string();
parts.push(format!("Current date: {date}"));
parts.push(format!("Current working directory: {}", self.cwd));
```

**改动文件**：
- `crates/pi-agent/src/system_prompt.rs`：3 行
- 新增测试：验证 build() 包含 "Current date:"

**预估**：~5 行改动，1 个新测试。

---

### G8：图片 resize + 降级提示

**现状**：图片直接 base64 编码，无 resize，无降级。
**TS 参考**：resize 到 2000x2000，不支持图片时返回文本提示。

**方案**：
- Resize：添加 `image` crate，缩放到 max 2000x2000
- 降级：在 `fileref.rs` 的 resolve 时检查模型是否支持图片（需要传入 model info）

简化方案（不引入图片处理依赖）：在 `fileref.rs` 中限制图片大小，超过 1MB 时跳过并返回提示。

```rust
// fileref.rs
const MAX_IMAGE_BYTES: usize = 1024 * 1024; // 1MB
if metadata.len() as usize > MAX_IMAGE_BYTES {
    return FileRef {
        path: path.display().to_string(),
        content: format!("[image too large: {} bytes, max {} bytes]",
            metadata.len(), MAX_IMAGE_BYTES),
        image: None,
    };
}
```

**改动文件**：
- `crates/pi-tools/src/fileref.rs`：大小限制 + 降级提示
- 新增测试：大文件跳过

**预估**：~20 行改动，1 个新测试。

---

## P2：完善性改进

### G9：模型变更 JSONL 记录

在 `loop_engine.rs` 切换模型时，向 session 追加一条 `model_change` entry。
**预估**：~15 行，1 个测试。

### G10：Skill 运行时展开

在 TUI 交互模式中解析 `/skill:name` 命令，展开为 SKILL.md 内容。
**预估**：~30 行，2 个测试。

### G11：JSONL 压缩实际写入

修改 session 格式支持：写入 compaction entry + 标记旧 entry 为 archived。使用 `compaction` 类型 entry 保存摘要，读取时跳过 archived 范围。
**预估**：~60 行，3 个测试。

### G12：Extension transformContext/convertToLlm

在 `ExtensionApi` trait 增加两个方法，在 `loop_engine.rs` 的 `call_llm` 前后调用。
**预估**：~40 行，2 个测试。

---

## 实施顺序

```
Phase 1（1-2 天）— P0 三项：
  G7 date 注入 (5 行)
  → G3 进程树清理 (60 行)
  → G5 terminate 信号 (15 行)
  → G4 参数校验 (60 行)
  → G1 并行执行 (80 行)
  → G2 结构化压缩 (100 行)

Phase 2（1 天）— P1 剩余：
  G8 图片限制 (20 行)
  → G6 steering (80 行)

Phase 3（半天）— P2 完善：
  G9 模型记录 (15 行)
  → G10 Skill 展开 (30 行)
  → G11 JSONL 压缩写入 (60 行)
  → G12 Extension hooks (40 行)
```

总预估：**~600 行新增/修改代码**，**~25 个新测试**。

完成后 piso Rust 版与 TS 版能力对齐度将从 72% 提升至 **95%+**。
