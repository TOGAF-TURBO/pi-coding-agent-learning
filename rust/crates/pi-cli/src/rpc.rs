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
//! 当前实现核心子集：prompt, abort, get_state, get_messages, new_session。

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

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
    ToolResult { name: String, output: String, is_error: bool },
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
    Error {
        id: Option<String>,
        message: String,
    },
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
    let _session_id = session.id().to_string();

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
                            let _ = serde_json::to_string(&err)
                                .map(|s| {
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
                            emit(&mut stdout_writer, &RpcEvent::Error {
                                id,
                                message: "Agent is already running, abort first".to_string(),
                            }).await?;
                            continue;
                        }
                        agent_running = true;
                        abort_flag.store(false, Ordering::SeqCst);
                        emit(&mut stdout_writer, &RpcEvent::StateChange {
                            state: "running".to_string(),
                        }).await?;

                        // 创建 driver
                        let driver = make_driver(&api_type);

                        // 创建 sink — 直接写 stdout（同步，因为 sink 回调不能是 async）
                        let _sink_id = id.clone();
                        let sink: Arc<StreamSink> = Arc::new(Box::new(move |event| {
                            let rpc_event = match event {
                                StreamEvent::TextDelta { text } => Some(RpcEvent::TextDelta { text }),
                                StreamEvent::ThinkingDelta { thinking } => Some(RpcEvent::ThinkingDelta { thinking }),
                                StreamEvent::ToolCallStart { name, .. } => Some(RpcEvent::ToolCallStart { name }),
                                StreamEvent::ToolResult { name, output, is_error, .. } => {
                                    Some(RpcEvent::ToolResult { name, output, is_error })
                                }
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
                        }));

                        // 准备 agent turn（用 dummy session swap）
                        let dummy_path = std::env::temp_dir().join(format!("piso-rpc-dummy-{}", std::process::id()));
                        let dummy_session = JsonlSession::create(&dummy_path, &cwd_str).await
                            .unwrap_or_else(|_| panic!("Failed to create dummy session"));

                        let mut agent = AgentLoop::new(
                            std::mem::replace(&mut session, dummy_session),
                            driver,
                            tools.clone_for_agent(),
                            &model,
                        )
                        .with_api_key(&api_key)
                        .with_system_prompt(&system_prompt)
                        .with_stream_sink(sink);

                        if let Some(url) = &base_url {
                            agent = agent.with_base_url(url);
                        }

                        // 运行 agent
                        match agent.run(&message).await {
                            Ok(_) => {}
                            Err(e) => {
                                emit(&mut stdout_writer, &RpcEvent::Error {
                                    id: id.clone(),
                                    message: format!("{e}"),
                                }).await?;
                            }
                        }

                        // 取回 session
                        session = agent.into_session();
                        agent_running = false;

                        emit(&mut stdout_writer, &RpcEvent::StateChange {
                            state: "idle".to_string(),
                        }).await?;
                        emit(&mut stdout_writer, &RpcEvent::Done { id }).await?;
                    }

                    RpcCommand::Abort { id } => {
                        abort_flag.store(true, Ordering::SeqCst);
                        agent_running = false;
                        emit(&mut stdout_writer, &RpcEvent::StateChange {
                            state: "idle".to_string(),
                        }).await?;
                        emit(&mut stdout_writer, &RpcEvent::Done { id }).await?;
                    }

                    RpcCommand::GetState { id } => {
                        emit(&mut stdout_writer, &RpcEvent::State {
                            id,
                            state: if agent_running { "running".to_string() } else { "idle".to_string() },
                            message_count: session.len(),
                            session_id: session.id().to_string(),
                        }).await?;
                    }

                    RpcCommand::GetMessages { id } => {
                        let messages: Vec<RpcMessage> = session.entries().iter()
                            .filter_map(|e| {
                                if let pi_types::session::SessionEntry::Message(msg) = e {
                                    let text = msg.content.as_array()
                                        .map(|arr| {
                                            arr.iter()
                                                .filter_map(|b| {
                                                    if b.get("type").and_then(|v| v.as_str()) == Some("text") {
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
                                        Some(RpcMessage { role: msg.role.clone(), content: text })
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
                        emit(&mut stdout_writer, &RpcEvent::State {
                            id,
                            state: "idle".to_string(),
                            message_count: 0,
                            session_id: session.id().to_string(),
                        }).await?;
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
        "openai-completions" | "openai-responses" => {
            Box::new(pi_llm::openai::OpenAiDriver::new())
        }
        "google-gemini" | "gemini" => {
            Box::new(pi_llm::gemini::GeminiDriver::new())
        }
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
