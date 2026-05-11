//! RPC 模式 — JSON-over-stdio 协议，供 IDE 插件和外部工具控制 piso。
//!
//! 对应 `packages/coding-agent/src/modes/rpc/rpc-types.ts`。

#![allow(unused_assignments)]
//!
//! 协议：
//! - stdin: 每行一个 JSON 命令
//! - stdout: 每行一个 JSON 事件/响应
//! - stderr: 日志（不影响协议）
//!
//! 完整命令集：prompt, abort, get_state, get_messages, new_session,
//! set_model, cycle_model, get_available_models, steer, follow_up,
//! compact, bash, get_commands.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::mpsc;

use pi_agent::loop_engine::{AgentLoop, StreamSink};
use pi_llm::driver::{LlmDriver, StreamEvent};
use pi_session::manager::SessionManager;
use pi_session::JsonlSession;
use pi_tools::bash::BashTool;
use pi_tools::edit::EditTool;
use pi_tools::find::FindTool;
use pi_tools::grep::GrepTool;
use pi_tools::read::ReadTool;
use pi_tools::registry::ToolRegistry;
use pi_tools::write::WriteTool;

// ============================================================================
// RPC Commands (stdin → piso)
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum RpcCommand {
    /// 发送消息给 agent（同 TS 版 prompt）。
    #[serde(rename = "prompt")]
    Prompt {
        #[serde(default)]
        id: Option<String>,
        message: String,
    },
    /// 中止当前 agent 执行。
    #[serde(rename = "abort")]
    Abort {
        #[serde(default)]
        id: Option<String>,
    },
    /// 获取当前状态。
    #[serde(rename = "get_state")]
    GetState {
        #[serde(default)]
        id: Option<String>,
    },
    /// 获取消息列表。
    #[serde(rename = "get_messages")]
    GetMessages {
        #[serde(default)]
        id: Option<String>,
    },
    /// 创建新会话。
    #[serde(rename = "new_session")]
    NewSession {
        #[serde(default)]
        id: Option<String>,
    },
    /// 切换模型。
    #[serde(rename = "set_model")]
    SetModel {
        #[serde(default)]
        id: Option<String>,
        model: String,
    },
    /// 切换到下一个可用模型。
    #[serde(rename = "cycle_model")]
    CycleModel {
        #[serde(default)]
        id: Option<String>,
    },
    /// 获取可用模型列表。
    #[serde(rename = "get_available_models")]
    GetAvailableModels {
        #[serde(default)]
        id: Option<String>,
    },
    /// 注入流式引导（agent 运行时）。
    #[serde(rename = "steer")]
    Steer {
        #[serde(default)]
        id: Option<String>,
        message: String,
    },
    /// 排队下一条消息。
    #[serde(rename = "follow_up")]
    FollowUp {
        #[serde(default)]
        id: Option<String>,
        message: String,
    },
    /// 触发上下文压缩。
    #[serde(rename = "compact")]
    Compact {
        #[serde(default)]
        id: Option<String>,
    },
    /// 执行远程命令。
    #[serde(rename = "bash")]
    Bash {
        #[serde(default)]
        id: Option<String>,
        command: String,
    },
    /// 获取可用 slash 命令。
    #[serde(rename = "get_commands")]
    GetCommands {
        #[serde(default)]
        id: Option<String>,
    },
    /// 设置 thinking level。
    #[serde(rename = "set_thinking_level")]
    SetThinkingLevel {
        #[serde(default)]
        id: Option<String>,
        level: String,
    },
    /// 设置自动压缩。
    #[serde(rename = "set_auto_compaction")]
    SetAutoCompaction {
        #[serde(default)]
        id: Option<String>,
        enabled: bool,
    },
    /// 设置自动重试。
    #[serde(rename = "set_auto_retry")]
    SetAutoRetry {
        #[serde(default)]
        id: Option<String>,
        enabled: bool,
    },
    /// 扩展 UI 响应（IDE 回复扩展的 UI 请求）。
    #[serde(rename = "extension_ui_response")]
    ExtensionUiResponse {
        #[serde(default)]
        id: Option<String>,
        #[serde(flatten)]
        response: ExtensionUiResponseData,
    },
}

// ============================================================================
// RPC Events (piso → stdout)
// ============================================================================

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum RpcEvent {
    /// 文本增量。
    #[serde(rename = "text_delta")]
    TextDelta { text: String },
    /// 思考增量。
    #[serde(rename = "thinking_delta")]
    ThinkingDelta { thinking: String },
    /// 工具调用开始。
    #[serde(rename = "tool_call_start")]
    ToolCallStart { name: String },
    /// 工具结果。
    #[serde(rename = "tool_result")]
    ToolResult {
        name: String,
        output: String,
        is_error: bool,
    },
    /// Agent 状态变化。
    #[serde(rename = "state_change")]
    StateChange { state: String },
    /// 响应完成。
    #[serde(rename = "done")]
    Done { id: Option<String> },
    /// 状态响应。
    #[serde(rename = "state")]
    State {
        id: Option<String>,
        state: String,
        message_count: usize,
        session_id: String,
    },
    /// 消息列表响应。
    #[serde(rename = "messages")]
    Messages {
        id: Option<String>,
        messages: Vec<RpcMessage>,
    },
    /// 错误。
    #[serde(rename = "error")]
    Error { id: Option<String>, message: String },
    /// 可用模型列表。
    #[serde(rename = "available_models")]
    AvailableModels {
        id: Option<String>,
        models: Vec<ModelInfo>,
    },
    /// 当前模型。
    #[serde(rename = "current_model")]
    CurrentModel { id: Option<String>, model: String },
    /// Bash 输出。
    #[serde(rename = "bash_output")]
    BashOutput {
        id: Option<String>,
        output: String,
        exit_code: i32,
    },
    /// 命令列表。
    #[serde(rename = "commands")]
    Commands {
        id: Option<String>,
        commands: Vec<String>,
    },
    /// 压缩结果。
    #[serde(rename = "compact_result")]
    CompactResult {
        id: Option<String>,
        removed: usize,
        remaining: usize,
    },
    /// 排队消息确认。
    #[serde(rename = "follow_up_queued")]
    FollowUpQueued { id: Option<String> },
    /// 引导已注入。
    #[serde(rename = "steered")]
    Steered { id: Option<String> },
    /// Thinking level 已设置。
    #[serde(rename = "thinking_level_set")]
    ThinkingLevelSet { id: Option<String>, level: String },
    /// 自动压缩已设置。
    #[serde(rename = "auto_compaction_set")]
    AutoCompactionSet { id: Option<String>, enabled: bool },
    /// 自动重试已设置。
    #[serde(rename = "auto_retry_set")]
    AutoRetrySet { id: Option<String>, enabled: bool },
    /// 扩展 UI 请求（piso → IDE，请求用户交互）。
    #[serde(rename = "extension_ui_request")]
    ExtensionUiRequest {
        #[serde(default)]
        id: Option<String>,
        #[serde(flatten)]
        request: ExtensionUiRequestData,
    },
}

/// 模型信息。
#[derive(Debug, Serialize)]
pub struct ModelInfo {
    pub provider: String,
    pub id: String,
    pub name: String,
}

// ============================================================================
// Extension UI Protocol
// ============================================================================

/// 扩展 UI 请求数据（piso → IDE）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "ui_type")]
pub enum ExtensionUiRequestData {
    /// 选择列表。
    #[serde(rename = "select")]
    Select {
        title: String,
        options: Vec<String>,
    },
    /// 确认对话框。
    #[serde(rename = "confirm")]
    Confirm {
        title: String,
        message: String,
    },
    /// 文本输入。
    #[serde(rename = "input")]
    Input {
        title: String,
        placeholder: String,
    },
    /// 设置 widget 内容。
    #[serde(rename = "set_widget")]
    SetWidget {
        key: String,
        content: String,
    },
    /// 设置状态栏文本。
    #[serde(rename = "set_status")]
    SetStatus {
        key: String,
        text: String,
    },
}

/// 扩展 UI 响应数据（IDE → piso）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "ui_type")]
pub enum ExtensionUiResponseData {
    /// 选择结果。
    #[serde(rename = "select")]
    Select { selected_index: usize },
    /// 确认结果。
    #[serde(rename = "confirm")]
    Confirm { confirmed: bool },
    /// 输入结果。
    #[serde(rename = "input")]
    Input { value: String },
    /// 确认接收。
    #[serde(rename = "ack")]
    Ack,
}

/// RPC 消息摘要（用于 get_messages 响应）。
#[derive(Debug, Serialize)]
pub struct RpcMessage {
    pub role: String,
    pub content: String,
}

// ============================================================================
// Internal command
// ============================================================================

enum InternalCommand {
    Rpc(RpcCommand),
    /// Agent turn 完成。
    #[allow(dead_code)]
    AgentDone,
}

/// 运行 RPC 模式。
pub async fn run_rpc(
    model: String,
    api_key: String,
    api_type: String,
    base_url: Option<String>,
    system_prompt: String,
    cwd: PathBuf,
) -> Result<()> {
    let cwd_str = cwd.to_string_lossy().to_string();
    let tools = make_tools(&cwd);
    let cfg = crate::config::load_config(Some(&cwd));
    let session_dir = crate::config::session_dir_for_cwd(&cfg, &cwd);

    // 创建 session
    let mgr = SessionManager::new(&session_dir);
    mgr.ensure_dir().await?;
    let mut session = mgr.create(&cwd_str).await?;

    let (cmd_tx, mut cmd_rx) = mpsc::unbounded_channel::<InternalCommand>();
    let abort_flag = Arc::new(AtomicBool::new(false));

    // stdout writer
    let stdout = tokio::io::stdout();
    let mut stdout_writer = tokio::io::BufWriter::new(stdout);

    // Agent state
    #[allow(unused_assignments)]
    let mut agent_running = false;
    let mut current_model = model.clone();
    let mut follow_up_queue: Vec<String> = Vec::new();
    #[allow(unused_variables)]
    let mut thinking_level: Option<String> = None;
    #[allow(unused_variables)]
    let mut auto_compaction: bool = true;
    #[allow(unused_variables)]
    let mut auto_retry: bool = false;

    // 可用模型列表
    let available_models = crate::dispatch::collect_available_models(
        &crate::auth::AuthStorage::load(None).unwrap_or_else(|_| crate::auth::AuthStorage::empty()),
    );

    // stdin reader task
    let stdin_cmd_tx = cmd_tx.clone();
    let stdin_handle = tokio::spawn(async move {
        let stdin = tokio::io::stdin();
        let mut reader = BufReader::new(stdin);
        let mut line = String::new();

        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => break, // EOF
                Ok(_) => {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    match serde_json::from_str::<RpcCommand>(trimmed) {
                        Ok(cmd) => {
                            if stdin_cmd_tx.send(InternalCommand::Rpc(cmd)).is_err() {
                                break;
                            }
                        }
                        Err(e) => {
                            // 发送错误到 stdout
                            let err = RpcEvent::Error {
                                id: None,
                                message: format!("Invalid command: {e}"),
                            };
                            let _stdout = tokio::io::stdout();
                            let _ = serde_json::to_string(&err).map(|s| {
                                let _ = std::io::Write::write_all(
                                    &mut std::io::stdout(),
                                    format!("{}\n", s).as_bytes(),
                                );
                            });
                        }
                    }
                }
                Err(_) => break,
            }
        }
    });

    // 事件循环
    loop {
        let cmd = match cmd_rx.recv().await {
            Some(c) => c,
            None => break,
        };

        match cmd {
            InternalCommand::Rpc(rpc_cmd) => {
                match rpc_cmd {
                    RpcCommand::Prompt { id, message } => {
                        if agent_running {
                            emit(
                                &mut stdout_writer,
                                &RpcEvent::Error {
                                    id,
                                    message: "Agent is already running, abort first".to_string(),
                                },
                            )
                            .await?;
                            continue;
                        }
                        // 解析 @file 引用
                        let resolved_message =
                            pi_tools::fileref::resolve_file_refs(&message, &cwd).0;
                        agent_running = true;
                        abort_flag.store(false, Ordering::SeqCst);
                        emit(
                            &mut stdout_writer,
                            &RpcEvent::StateChange {
                                state: "running".to_string(),
                            },
                        )
                        .await?;

                        let driver = make_driver(&api_type);

                        // 创建 sink — 直接写 stdout（同步，因为 sink 回调不能是 async）
                        let sink = make_sync_sink();

                        // 准备 agent turn（用 dummy session swap）
                        let dummy_path = std::env::temp_dir()
                            .join(format!("piso-rpc-dummy-{}", std::process::id()));
                        let dummy_session = JsonlSession::create(&dummy_path, &cwd_str)
                            .await
                            .unwrap_or_else(|_| panic!("Failed to create dummy session"));

                        let mut agent = AgentLoop::new(
                            std::mem::replace(&mut session, dummy_session),
                            driver,
                            tools.clone_for_agent(),
                            &current_model,
                        )
                        .with_api_key(&api_key)
                        .with_system_prompt(&system_prompt)
                        .with_stream_sink(sink);

                        if let Some(url) = &base_url {
                            agent = agent.with_base_url(url);
                        }

                        // 运行 agent
                        match agent.run(&resolved_message).await {
                            Ok(_) => {}
                            Err(e) => {
                                emit(
                                    &mut stdout_writer,
                                    &RpcEvent::Error {
                                        id: id.clone(),
                                        message: format!("{e}"),
                                    },
                                )
                                .await?;
                            }
                        }

                        // 取回 session
                        session = agent.into_session();
                        agent_running = false;

                        emit(
                            &mut stdout_writer,
                            &RpcEvent::StateChange {
                                state: "idle".to_string(),
                            },
                        )
                        .await?;
                        emit(&mut stdout_writer, &RpcEvent::Done { id }).await?;

                        // 处理排队的 follow_up / steer 消息
                        while let Some(next_msg) = follow_up_queue.pop() {
                            agent_running = true;
                            emit(
                                &mut stdout_writer,
                                &RpcEvent::StateChange {
                                    state: "running".to_string(),
                                },
                            )
                            .await?;

                            let driver = make_driver(&api_type);
                            let sink2 = make_sync_sink();
                            let dummy_path = std::env::temp_dir()
                                .join(format!("piso-rpc-dummy-{}", std::process::id()));
                            let dummy_session = JsonlSession::create(&dummy_path, &cwd_str).await?;
                            let mut agent = AgentLoop::new(
                                std::mem::replace(&mut session, dummy_session),
                                driver,
                                tools.clone_for_agent(),
                                &current_model,
                            )
                            .with_api_key(&api_key)
                            .with_system_prompt(&system_prompt)
                            .with_stream_sink(sink2);
                            if let Some(url) = &base_url {
                                agent = agent.with_base_url(url);
                            }

                            let _ = agent.run(&next_msg).await;
                            session = agent.into_session();
                            agent_running = false;

                            emit(
                                &mut stdout_writer,
                                &RpcEvent::StateChange {
                                    state: "idle".to_string(),
                                },
                            )
                            .await?;
                            emit(&mut stdout_writer, &RpcEvent::Done { id: None }).await?;
                        }
                    }

                    RpcCommand::Abort { id } => {
                        abort_flag.store(true, Ordering::SeqCst);
                        agent_running = false;
                        emit(
                            &mut stdout_writer,
                            &RpcEvent::StateChange {
                                state: "idle".to_string(),
                            },
                        )
                        .await?;
                        emit(&mut stdout_writer, &RpcEvent::Done { id }).await?;
                    }

                    RpcCommand::GetState { id } => {
                        emit(
                            &mut stdout_writer,
                            &RpcEvent::State {
                                id,
                                state: if agent_running {
                                    "running".to_string()
                                } else {
                                    "idle".to_string()
                                },
                                message_count: session.len(),
                                session_id: session.id().to_string(),
                            },
                        )
                        .await?;
                    }

                    RpcCommand::GetMessages { id } => {
                        let messages: Vec<RpcMessage> = session
                            .entries()
                            .iter()
                            .filter_map(|e| {
                                if let pi_types::session::SessionEntry::Message(msg) = e {
                                    let text = msg
                                        .content
                                        .as_array()
                                        .map(|arr| {
                                            arr.iter()
                                                .filter_map(|b| {
                                                    if b.get("type").and_then(|v| v.as_str())
                                                        == Some("text")
                                                    {
                                                        b.get("text").and_then(|v| v.as_str())
                                                    } else {
                                                        None
                                                    }
                                                })
                                                .collect::<Vec<_>>()
                                                .join("\n")
                                        })
                                        .unwrap_or_default();
                                    if !text.is_empty() {
                                        Some(RpcMessage {
                                            role: msg.role.clone(),
                                            content: text,
                                        })
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            })
                            .collect();
                        emit(&mut stdout_writer, &RpcEvent::Messages { id, messages }).await?;
                    }

                    RpcCommand::NewSession { id } => {
                        let new_session = mgr.create(&cwd_str).await?;
                        session = new_session;
                        emit(
                            &mut stdout_writer,
                            &RpcEvent::State {
                                id,
                                state: "idle".to_string(),
                                message_count: 0,
                                session_id: session.id().to_string(),
                            },
                        )
                        .await?;
                    }

                    RpcCommand::SetModel {
                        id,
                        model: new_model,
                    } => {
                        current_model = new_model.clone();
                        emit(
                            &mut stdout_writer,
                            &RpcEvent::CurrentModel {
                                id,
                                model: new_model,
                            },
                        )
                        .await?;
                    }

                    RpcCommand::CycleModel { id } => {
                        if let Some(idx) = available_models
                            .iter()
                            .position(|(_, mid, _)| *mid == current_model)
                        {
                            let next_idx = (idx + 1) % available_models.len();
                            let (_, mid, name) = &available_models[next_idx];
                            current_model = mid.clone();
                            emit(
                                &mut stdout_writer,
                                &RpcEvent::CurrentModel {
                                    id,
                                    model: format!("{mid} ({name})"),
                                },
                            )
                            .await?;
                        } else {
                            emit(
                                &mut stdout_writer,
                                &RpcEvent::Error {
                                    id,
                                    message: format!(
                                        "Current model '{}' not in available list",
                                        current_model
                                    ),
                                },
                            )
                            .await?;
                        }
                    }

                    RpcCommand::GetAvailableModels { id } => {
                        let models: Vec<ModelInfo> = available_models
                            .iter()
                            .map(|(prov, mid, name)| ModelInfo {
                                provider: prov.clone(),
                                id: mid.clone(),
                                name: name.clone(),
                            })
                            .collect();
                        emit(
                            &mut stdout_writer,
                            &RpcEvent::AvailableModels { id, models },
                        )
                        .await?;
                    }

                    RpcCommand::Steer { id, message } => {
                        // 注入引导消息到排队列表前面
                        follow_up_queue.insert(0, message);
                        emit(&mut stdout_writer, &RpcEvent::Steered { id }).await?;
                    }

                    RpcCommand::FollowUp { id, message } => {
                        follow_up_queue.push(message);
                        emit(&mut stdout_writer, &RpcEvent::FollowUpQueued { id }).await?;
                    }

                    RpcCommand::Compact { id } => {
                        let total = session.len();
                        if total > 12 {
                            let removed = total - 12;
                            session.compact_keep_last(12).await?;
                            emit(
                                &mut stdout_writer,
                                &RpcEvent::CompactResult {
                                    id,
                                    removed,
                                    remaining: 12,
                                },
                            )
                            .await?;
                        } else {
                            emit(
                                &mut stdout_writer,
                                &RpcEvent::CompactResult {
                                    id,
                                    removed: 0,
                                    remaining: total,
                                },
                            )
                            .await?;
                        }
                    }

                    RpcCommand::Bash { id, command } => {
                        let output = tokio::process::Command::new("bash")
                            .arg("-c")
                            .arg(&command)
                            .output()
                            .await;
                        match output {
                            Ok(out) => {
                                let stdout_str = String::from_utf8_lossy(&out.stdout).to_string();
                                let stderr_str = String::from_utf8_lossy(&out.stderr).to_string();
                                let combined = if stderr_str.is_empty() {
                                    stdout_str
                                } else {
                                    format!("{stdout_str}\n{stderr_str}")
                                };
                                emit(
                                    &mut stdout_writer,
                                    &RpcEvent::BashOutput {
                                        id,
                                        output: combined,
                                        exit_code: out.status.code().unwrap_or(-1),
                                    },
                                )
                                .await?;
                            }
                            Err(e) => {
                                emit(
                                    &mut stdout_writer,
                                    &RpcEvent::Error {
                                        id,
                                        message: format!("Failed to execute: {e}"),
                                    },
                                )
                                .await?;
                            }
                        }
                    }

                    RpcCommand::GetCommands { id } => {
                        let commands = vec![
                            "/help".to_string(),
                            "/clear".to_string(),
                            "/compact".to_string(),
                            "/cost".to_string(),
                            "/usage".to_string(),
                            "/sessions".to_string(),
                            "/find".to_string(),
                            "/grep".to_string(),
                            "/new".to_string(),
                            "/reload".to_string(),
                            "/copy".to_string(),
                            "/fork".to_string(),
                            "/session".to_string(),
                            "/name".to_string(),
                            "/export".to_string(),
                            "/import".to_string(),
                            "/clone".to_string(),
                        ];
                        emit(&mut stdout_writer, &RpcEvent::Commands { id, commands }).await?;
                    }

                    RpcCommand::SetThinkingLevel { id, level } => {
                        thinking_level = Some(level.clone());
                        emit(
                            &mut stdout_writer,
                            &RpcEvent::ThinkingLevelSet { id, level },
                        )
                        .await?;
                    }

                    RpcCommand::SetAutoCompaction { id, enabled } => {
                        auto_compaction = enabled;
                        emit(
                            &mut stdout_writer,
                            &RpcEvent::AutoCompactionSet { id, enabled },
                        )
                        .await?;
                    }

                    RpcCommand::SetAutoRetry { id, enabled } => {
                        auto_retry = enabled;
                        emit(&mut stdout_writer, &RpcEvent::AutoRetrySet { id, enabled }).await?;
                    }
                    RpcCommand::ExtensionUiResponse { id, response: _ } => {
                        // Extension UI response — handled by extension system
                        emit(&mut stdout_writer, &RpcEvent::Done { id }).await?;
                    }
                }
            }
            InternalCommand::AgentDone => {
                agent_running = false;
            }
        }
    }

    stdin_handle.abort();
    Ok(())
}

/// 发送 RPC 事件到 stdout。
async fn emit<W: AsyncWriteExt + Unpin>(writer: &mut W, event: &RpcEvent) -> Result<()> {
    let json = serde_json::to_string(event)?;
    writer.write_all(json.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;
    Ok(())
}

/// 创建 LLM driver。
fn make_driver(api_type: &str) -> Box<dyn LlmDriver> {
    match api_type {
        "openai-completions" => Box::new(pi_llm::openai::OpenAiDriver::new()),
        "openai-responses" => Box::new(pi_llm::openai_responses::OpenAiResponsesDriver::new()),
        "azure-openai" => Box::new(pi_llm::azure::AzureOpenAiDriver::new()),
        "amazon-bedrock" | "bedrock" => Box::new(pi_llm::bedrock::BedrockDriver::new()),
        "google-vertex" | "vertex" => Box::new(pi_llm::vertex::VertexDriver::new()),
        "google-gemini" | "gemini" => Box::new(pi_llm::gemini::GeminiDriver::new()),
        _ => Box::new(pi_llm::providers::AnthropicDriver::new()),
    }
}

/// 构建工具注册表。
fn make_tools(cwd: &std::path::Path) -> ToolRegistry {
    let cwd_str = cwd.to_string_lossy().to_string();
    let tools = ToolRegistry::new();
    tools.register(BashTool::new(&cwd_str));
    tools.register(ReadTool::new());
    tools.register(WriteTool::new());
    tools.register(EditTool::new());
    tools.register(FindTool::new(&cwd_str));
    tools.register(GrepTool::new(&cwd_str));
    tools
}

/// 创建同步 sink（写 stdout）。
fn make_sync_sink() -> Arc<StreamSink> {
    Arc::new(Box::new(move |event| {
        let rpc_event = match event {
            StreamEvent::TextDelta { text } => Some(RpcEvent::TextDelta { text }),
            StreamEvent::ThinkingDelta { thinking } => Some(RpcEvent::ThinkingDelta { thinking }),
            StreamEvent::ToolCallStart { name, .. } => Some(RpcEvent::ToolCallStart { name }),
            StreamEvent::ToolResult {
                name,
                output,
                is_error,
                ..
            } => Some(RpcEvent::ToolResult {
                name,
                output,
                is_error,
            }),
            _ => None,
        };
        if let Some(ev) = rpc_event {
            if let Ok(json) = serde_json::to_string(&ev) {
                let _ = std::io::Write::write_all(
                    &mut std::io::stdout(),
                    format!("{}\n", json).as_bytes(),
                );
            }
        }
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_prompt_command() {
        let json = r#"{"type":"prompt","message":"hello","id":"1"}"#;
        let cmd: RpcCommand = serde_json::from_str(json).unwrap();
        match cmd {
            RpcCommand::Prompt { id, message } => {
                assert_eq!(id, Some("1".to_string()));
                assert_eq!(message, "hello");
            }
            _ => panic!("Expected Prompt"),
        }
    }

    #[test]
    fn deserialize_abort_command() {
        let json = r#"{"type":"abort"}"#;
        let cmd: RpcCommand = serde_json::from_str(json).unwrap();
        assert!(matches!(cmd, RpcCommand::Abort { id: None }));
    }

    #[test]
    fn deserialize_set_model() {
        let json = r#"{"type":"set_model","model":"gpt-4o"}"#;
        let cmd: RpcCommand = serde_json::from_str(json).unwrap();
        match cmd {
            RpcCommand::SetModel { model, .. } => assert_eq!(model, "gpt-4o"),
            _ => panic!("Expected SetModel"),
        }
    }

    #[test]
    fn deserialize_compact() {
        let json = r#"{"type":"compact"}"#;
        let cmd: RpcCommand = serde_json::from_str(json).unwrap();
        assert!(matches!(cmd, RpcCommand::Compact { .. }));
    }

    #[test]
    fn deserialize_bash() {
        let json = r#"{"type":"bash","command":"ls -la"}"#;
        let cmd: RpcCommand = serde_json::from_str(json).unwrap();
        match cmd {
            RpcCommand::Bash { command, .. } => assert_eq!(command, "ls -la"),
            _ => panic!("Expected Bash"),
        }
    }

    #[test]
    fn serialize_text_delta_event() {
        let event = RpcEvent::TextDelta {
            text: "hello".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("text_delta"));
        assert!(json.contains("hello"));
    }

    #[test]
    fn serialize_state_event() {
        let event = RpcEvent::State {
            id: Some("1".to_string()),
            state: "idle".to_string(),
            message_count: 5,
            session_id: "abc".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("state"));
        assert!(json.contains("idle"));
    }

    #[test]
    fn serialize_bash_output() {
        let event = RpcEvent::BashOutput {
            id: None,
            output: "file1.txt\nfile2.txt".to_string(),
            exit_code: 0,
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("bash_output"));
        assert!(json.contains("exit_code"));
    }

    #[test]
    fn deserialize_invalid_command() {
        let json = r#"{"type":"unknown"}"#;
        let result = serde_json::from_str::<RpcCommand>(json);
        assert!(result.is_err());
    }

    #[test]
    fn deserialize_set_thinking_level() {
        let json = r#"{"type":"set_thinking_level","level":"high"}"#;
        let cmd: RpcCommand = serde_json::from_str(json).unwrap();
        match cmd {
            RpcCommand::SetThinkingLevel { level, .. } => assert_eq!(level, "high"),
            _ => panic!("Expected SetThinkingLevel"),
        }
    }

    #[test]
    fn deserialize_set_auto_compaction() {
        let json = r#"{"type":"set_auto_compaction","enabled":true}"#;
        let cmd: RpcCommand = serde_json::from_str(json).unwrap();
        assert!(matches!(
            cmd,
            RpcCommand::SetAutoCompaction { enabled: true, .. }
        ));
    }

    #[test]
    fn deserialize_set_auto_retry() {
        let json = r#"{"type":"set_auto_retry","enabled":false}"#;
        let cmd: RpcCommand = serde_json::from_str(json).unwrap();
        assert!(matches!(
            cmd,
            RpcCommand::SetAutoRetry { enabled: false, .. }
        ));
    }

    #[test]
    fn serialize_thinking_level_set() {
        let event = RpcEvent::ThinkingLevelSet {
            id: None,
            level: "medium".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("thinking_level_set"));
    }

    #[test]
    fn serialize_auto_compaction_set() {
        let event = RpcEvent::AutoCompactionSet {
            id: None,
            enabled: true,
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("auto_compaction_set"));
    }

    #[test]
    fn serialize_extension_ui_request_select() {
        let event = RpcEvent::ExtensionUiRequest {
            id: Some("ext-1".to_string()),
            request: ExtensionUiRequestData::Select {
                title: "Choose file".to_string(),
                options: vec!["a.rs".to_string(), "b.rs".to_string()],
            },
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("extension_ui_request"));
        assert!(json.contains("Choose file"));
    }

    #[test]
    fn serialize_extension_ui_request_confirm() {
        let event = RpcEvent::ExtensionUiRequest {
            id: None,
            request: ExtensionUiRequestData::Confirm {
                title: "Delete?".to_string(),
                message: "Are you sure?".to_string(),
            },
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("confirm"));
    }

    #[test]
    fn serialize_extension_ui_request_input() {
        let event = RpcEvent::ExtensionUiRequest {
            id: None,
            request: ExtensionUiRequestData::Input {
                title: "Name".to_string(),
                placeholder: "Enter name...".to_string(),
            },
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("input"));
    }

    #[test]
    fn deserialize_extension_ui_response_select() {
        let json = r#"{"type":"extension_ui_response","ui_type":"select","selected_index":2}"#;
        let cmd: RpcCommand = serde_json::from_str(json).unwrap();
        assert!(matches!(
            cmd,
            RpcCommand::ExtensionUiResponse {
                response: ExtensionUiResponseData::Select { selected_index: 2 },
                ..
            }
        ));
    }

    #[test]
    fn deserialize_extension_ui_response_confirm() {
        let json = r#"{"type":"extension_ui_response","ui_type":"confirm","confirmed":true}"#;
        let cmd: RpcCommand = serde_json::from_str(json).unwrap();
        assert!(matches!(
            cmd,
            RpcCommand::ExtensionUiResponse {
                response: ExtensionUiResponseData::Confirm { confirmed: true },
                ..
            }
        ));
    }

    #[test]
    fn deserialize_extension_ui_response_input() {
        let json = r#"{"type":"extension_ui_response","ui_type":"input","value":"hello"}"#;
        let cmd: RpcCommand = serde_json::from_str(json).unwrap();
        assert!(matches!(
            cmd,
            RpcCommand::ExtensionUiResponse {
                response: ExtensionUiResponseData::Input { value },
                ..
            } if value == "hello"
        ));
    }

    #[test]
    fn deserialize_extension_ui_response_ack() {
        let json = r#"{"type":"extension_ui_response","ui_type":"ack"}"#;
        let cmd: RpcCommand = serde_json::from_str(json).unwrap();
        assert!(matches!(
            cmd,
            RpcCommand::ExtensionUiResponse {
                response: ExtensionUiResponseData::Ack,
                ..
            }
        ));
    }
}
