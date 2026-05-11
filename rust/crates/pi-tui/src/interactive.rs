//! 交互模式 — TUI + Agent 循环的集成。
//!
//! 对应 `packages/coding-agent/src/modes/interactive/interactive-mode.ts`。
//!
//! 架构：
//! ```text
//! ┌─────────────┐   tokio::mpsc    ┌──────────────┐
//! │  TUI task   │ ──── Command ──> │  Agent task   │
//! │  (渲染+输入) │ <── AppState ─── │  (LLM+工具)   │
//! └─────────────┘   Arc<RwLock>    │  1 session    │
//!                                   └──────────────┘
//! ```
//! 关键：agent task 持有唯一的 JsonlSession，多次 turn 复用同一 session，
//! LLM 拥有完整的多轮对话上下文。

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::{Context, Result, anyhow};
use crossterm::event::KeyCode;
use pi_agent::loop_engine::{AgentLoop, StreamSink};
use pi_agent::system_prompt::SystemPromptBuilder;
use pi_llm::driver::LlmDriver;
use pi_session::manager::SessionManager;
use pi_session::JsonlSession;
use pi_tools::bash::BashTool;
use pi_tools::edit::EditTool;
use pi_tools::find::FindTool;
use pi_tools::grep::GrepTool;
use pi_tools::read::ReadTool;
use pi_tools::registry::ToolRegistry;
use pi_tools::write::WriteTool;
use tokio::sync::mpsc;

use crate::app::{AgentState, AppState};
use crate::components;
use crate::engine::TuiEngine;
use crate::event::Event;
use crate::input::InputEditor;
use crate::keybinding::{self, Action};
use crate::layout;

/// Agent 命令。
enum Command {
    /// 发送消息给 agent。
    Send { text: String },
    /// 中止当前 agent 执行。
    Abort,
}

/// 交互模式配置。
pub struct InteractiveConfig {
    pub model: String,
    pub provider: String,
    pub api_key: String,
    pub api_type: String,
    pub base_url: Option<String>,
    pub cwd: PathBuf,
    pub system_prompt: String,
}

/// 运行交互模式。
pub async fn run_interactive(cfg: InteractiveConfig) -> Result<()> {
    let state = Arc::new(AppState::new(&cfg.model, &cfg.provider));

    let tools = make_tools(&cfg.cwd);
    let session_dir = cfg.cwd.join(".piso").join("sessions");
    let model = cfg.model.clone();
    let api_key = cfg.api_key.clone();
    let api_type = cfg.api_type.clone();
    let base_url = cfg.base_url.clone();
    let system_prompt = cfg.system_prompt.clone();
    let cwd_str = cfg.cwd.to_string_lossy().to_string();

    // 创建唯一 session
    let mgr = SessionManager::new(&session_dir);
    mgr.ensure_dir().await?;
    let mut session = mgr.create(&cwd_str).await?;

    let (cmd_tx, mut cmd_rx) = mpsc::unbounded_channel::<Command>();
    let abort_flag = Arc::new(AtomicBool::new(false));

    // Agent task — 持有 session，串行处理命令
    let agent_state = state.clone();
    let agent_abort = abort_flag.clone();
    let agent_handle = tokio::spawn(async move {
        while let Some(cmd) = cmd_rx.recv().await {
            match cmd {
                Command::Send { text } => {
                    abort_flag_clear(&agent_abort);
                    run_agent_turn(
                        &text,
                        &mut session,
                        &agent_state,
                        &tools,
                        &model,
                        &api_key,
                        &api_type,
                        &base_url,
                        &system_prompt,
                    )
                    .await;
                }
                Command::Abort => {
                    agent_abort.store(true, Ordering::SeqCst);
                }
            }
        }
    });

    // TUI 事件循环
    let mut engine = TuiEngine::init()?;
    let mut input = InputEditor::new();
    let mut scroll_offset: usize = 0;

    loop {
        engine.terminal().draw(|f| {
            let size = f.area();
            let regions = layout::calculate(size, 5);
            components::render_all(f, regions, &state, input.text(), scroll_offset);
        })?;

        let event = match engine.next_event().await {
            Some(e) => e,
            None => break,
        };

        match event {
            Event::Key(key) => {
                let action = keybinding::match_key(&key);
                let is_running = !matches!(state.agent_state(), AgentState::Idle);

                match action {
                    Action::Submit => {
                        if !input.is_empty() && !is_running {
                            let text = input.take();
                            scroll_offset = 0;
                            let _ = cmd_tx.send(Command::Send { text });
                        }
                    }
                    Action::Quit => break,
                    Action::Cancel => {
                        if is_running {
                            let _ = cmd_tx.send(Command::Abort);
                            state.set_state(AgentState::Idle);
                        } else if !input.is_empty() {
                            input.clear();
                        }
                    }
                    Action::ScrollUp => {
                        scroll_offset = scroll_offset.saturating_add(5);
                    }
                    Action::ScrollDown => {
                        scroll_offset = scroll_offset.saturating_sub(5);
                    }
                    Action::None => {
                        if !is_running {
                            match key.code {
                                KeyCode::Char(c) => input.insert(c),
                                KeyCode::Backspace => input.backspace(),
                                KeyCode::Delete => input.delete(),
                                KeyCode::Left => input.move_left(),
                                KeyCode::Right => input.move_right(),
                                KeyCode::Home => input.move_home(),
                                KeyCode::End => input.move_end(),
                                KeyCode::Enter => input.insert('\n'),
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }
            Event::Resize(_, _) => {}
            _ => {}
        }
    }

    drop(cmd_tx);
    agent_handle.abort();
    Ok(())
}

/// 运行一次 agent turn（使用共享 session）。
async fn run_agent_turn(
    text: &str,
    session: &mut JsonlSession,
    state: &Arc<AppState>,
    tools: &ToolRegistry,
    model: &str,
    api_key: &str,
    api_type: &str,
    base_url: &Option<String>,
    system_prompt: &str,
) {
    state.push_user(text);
    state.set_state(AgentState::Thinking);

    // 创建 driver
    let driver: Box<dyn LlmDriver> = match api_type {
        "openai-completions" | "openai-responses" => {
            Box::new(pi_llm::openai::OpenAiDriver::new())
        }
        "google-gemini" | "gemini" => {
            Box::new(pi_llm::gemini::GeminiDriver::new())
        }
        _ => Box::new(pi_llm::providers::AnthropicDriver::new()),
    };

    // 流式回调
    let sink_state = state.clone();
    let sink: Arc<StreamSink> = Arc::new(Box::new(move |event| {
        use pi_llm::driver::StreamEvent;
        match event {
            StreamEvent::TextDelta { text } => {
                sink_state.push_assistant_delta(&text);
            }
            StreamEvent::ToolCallStart { name, .. } => {
                sink_state.set_state(AgentState::ToolRunning { name });
            }
            StreamEvent::ToolResult { name, output, is_error, .. } => {
                sink_state.push_tool_result(&name, &output, is_error);
                sink_state.set_state(AgentState::Thinking);
            }
            StreamEvent::Stop { .. } => {
                sink_state.finish_assistant();
            }
            _ => {}
        }
    }));

    // 注意：AgentLoop::new takes ownership of session。
    // 我们先用临时 session，run 后再取回。
    // 但这样会丢失已有消息。
    //
    // 正确做法：克隆 session path，创建新的空 AgentLoop 但复用已有条目。
    // AgentLoop 的 session 字段需要知道之前的历史才能 build_messages。
    //
    // 所以必须让 AgentLoop 借用 session 而不是拥有。
    // 但 run(&mut self) + session 追加操作需要 &mut session。
    //
    // 当前方案：创建临时 session，run 结束后把条目复制回来。
    // 但这样会导致不一致。
    //
    // 最简方案：直接把 session 给 AgentLoop，run 结束后用 into_session() 取回。

    // 临时 dummy session 用于 std::mem::replace
    // AgentLoop takes ownership of session，run 后通过 into_session() 取回
    let dummy_path = std::env::temp_dir().join(format!("piso-dummy-{}", std::process::id()));
    let dummy_session = JsonlSession::create(&dummy_path, "").await
        .unwrap_or_else(|_| panic!("Failed to create dummy session"));

    let mut agent = AgentLoop::new(
        std::mem::replace(session, dummy_session),
        driver,
        tools.clone_for_agent(),
        model,
    )
    .with_api_key(api_key)
    .with_system_prompt(system_prompt)
    .with_stream_sink(sink);

    if let Some(url) = base_url {
        agent = agent.with_base_url(url);
    }

    state.set_state(AgentState::Streaming);

    match agent.run(text).await {
        Ok(_output) => {
            state.finish_assistant();
            state.set_state(AgentState::Idle);
        }
        Err(e) => {
            state.finish_assistant();
            state.set_state(AgentState::Error(format!("{e}")));
        }
    }

    // 取回 session（包含所有累积的消息历史）
    *session = agent.into_session();
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

fn abort_flag_clear(flag: &Arc<AtomicBool>) {
    flag.store(false, Ordering::SeqCst);
}
