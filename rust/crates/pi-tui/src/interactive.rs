//! 交互模式 — TUI + Agent 循环的集成。
//!
//! 对应 `packages/coding-agent/src/modes/interactive/interactive-mode.ts`。
//!
//! 架构：主 task 运行 TUI 事件循环，agent 在独立 tokio task 中运行。
//! ```text
//! ┌─────────────┐   tokio::mpsc    ┌──────────────┐
//! │  TUI task   │ ──── Command ──> │  Agent task   │
//! │  (渲染+输入) │ <── AppState ─── │  (LLM+工具)   │
//! └─────────────┘   Arc<RwLock>    └──────────────┘
//! ```

use std::path::PathBuf;
use std::sync::Arc;

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
use pi_types::session::SessionEntry;
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

    // 工具注册表（每次 agent 循环 clone）
    let tools = make_tools(&cfg.cwd);
    let session_dir = cfg.cwd.join(".piso").join("sessions");
    let model = cfg.model.clone();
    let api_key = cfg.api_key.clone();
    let api_type = cfg.api_type.clone();
    let base_url = cfg.base_url.clone();
    let system_prompt = cfg.system_prompt.clone();
    let cwd_str = cfg.cwd.to_string_lossy().to_string();

    // 命令 channel
    let (cmd_tx, mut cmd_rx) = mpsc::unbounded_channel::<Command>();

    // Agent task
    let agent_state = state.clone();
    let agent_handle = tokio::spawn(async move {
        while let Some(cmd) = cmd_rx.recv().await {
            match cmd {
                Command::Send { text } => {
                    run_agent_turn(
                        &text,
                        &agent_state,
                        &tools,
                        &session_dir,
                        &cwd_str,
                        &model,
                        &api_key,
                        &api_type,
                        &base_url,
                        &system_prompt,
                    )
                    .await;
                }
            }
        }
    });

    // TUI 事件循环
    let mut engine = TuiEngine::init()?;
    let mut input = InputEditor::new();
    let mut scroll_offset: usize = 0;

    loop {
        // 渲染
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

                match action {
                    Action::Submit => {
                        if !input.is_empty() {
                            let text = input.take();
                            scroll_offset = 0;
                            let _ = cmd_tx.send(Command::Send { text });
                        }
                    }
                    Action::Quit => break,
                    Action::Cancel => {
                        input.clear();
                    }
                    Action::ScrollUp => {
                        scroll_offset = scroll_offset.saturating_add(1);
                    }
                    Action::ScrollDown => {
                        scroll_offset = scroll_offset.saturating_sub(1);
                    }
                    Action::None => match key.code {
                        KeyCode::Char(c) => input.insert(c),
                        KeyCode::Backspace => input.backspace(),
                        KeyCode::Delete => input.delete(),
                        KeyCode::Left => input.move_left(),
                        KeyCode::Right => input.move_right(),
                        KeyCode::Home => input.move_home(),
                        KeyCode::End => input.move_end(),
                        KeyCode::Enter => input.insert('\n'),
                        _ => {}
                    },
                    _ => {}
                }
            }
            Event::Resize(_, _) => {}
            _ => {}
        }
    }

    // 清理
    drop(cmd_tx);
    agent_handle.abort();
    Ok(())
}

/// 运行一次 agent 循环。
async fn run_agent_turn(
    text: &str,
    state: &Arc<AppState>,
    tools: &ToolRegistry,
    session_dir: &std::path::Path,
    cwd: &str,
    model: &str,
    api_key: &str,
    api_type: &str,
    base_url: &Option<String>,
    system_prompt: &str,
) {
    state.push_user(text);
    state.set_state(AgentState::Thinking);

    // 创建会话
    let mgr = SessionManager::new(session_dir);
    if let Err(e) = mgr.ensure_dir().await {
        state.set_state(AgentState::Error(format!("Session dir: {e}")));
        return;
    }
    let session = match mgr.create(cwd).await {
        Ok(s) => s,
        Err(e) => {
            state.set_state(AgentState::Error(format!("Session: {e}")));
            return;
        }
    };

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

    // 创建流式回调
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
            StreamEvent::ToolCallEnd { name, .. } => {
                sink_state.set_state(AgentState::Thinking);
            }
            StreamEvent::Stop { .. } => {
                sink_state.finish_assistant();
            }
            _ => {}
        }
    }));

    let mut agent = AgentLoop::new(session, driver, tools.clone_for_agent(), model)
        .with_api_key(api_key)
        .with_system_prompt(system_prompt)
        .with_stream_sink(sink);

    if let Some(url) = base_url {
        agent = agent.with_base_url(url);
    }

    state.set_state(AgentState::Streaming);

    match agent.run(text).await {
        Ok(output) => {
            state.finish_assistant();
            state.set_state(AgentState::Idle);
            if output.tool_calls > 0 {
                state.set_state(AgentState::Idle);
            }
        }
        Err(e) => {
            state.finish_assistant();
            state.set_state(AgentState::Error(format!("{e}")));
        }
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
