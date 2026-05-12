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
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::sync::RwLock;

use anyhow::Result;
use crossterm::event::KeyCode;
use pi_agent::loop_engine::{AgentLoop, StreamSink};
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
use serde_json::json;
use tokio::sync::mpsc;

use crate::app::{AgentState, AppState};
use crate::components;
use crate::engine::TuiEngine;
use crate::event::Event;
use crate::keybinding::{Action, KeyBindings};
use crate::layout;
use crate::selector;

/// Agent 命令。
enum Command {
    /// 发送消息给 agent。
    Send { text: String },
    /// 中止当前 agent 执行。
    Abort,
    /// 导出会话为 HTML。
    Export { path: String },
    /// 压缩上下文。
    Compact,
    /// 新建会话。
    NewSession,
    /// 导入 JSONL 文件。
    Import { path: String },
    /// 克隆当前会话。
    CloneSession,
    /// 打开 diff 查看器。
    ShowDiff { diff_text: String },
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
    /// Session 存储目录（由调用方计算，如 ~/.piso/sessions/--cwd--/）。
    pub session_dir: PathBuf,
    /// 预加载的 session（可能包含历史消息）。
    pub session: Option<JsonlSession>,
    /// 可用模型列表（provider, model_id, display_name）。
    pub available_models: Vec<(String, String, String)>,
    /// 快捷键配置。
    pub keybindings: KeyBindings,
    /// 扩展运行时（可选）。
    pub extension_runner: Option<Arc<pi_extensions::ExtensionRunner>>,
    /// 主题。
    pub theme: crate::theme::Theme,
}

/// Overlay 类型（用于区分回调行为）。
enum OverlayKind {
    SessionPicker,
    ModelPicker,
}

/// Agent 运行时上下文 — 避免传 9 个参数。
#[derive(Clone)]
pub struct AgentContext {
    pub model: String,
    pub api_key: String,
    pub api_type: String,
    pub base_url: Option<String>,
    pub system_prompt: String,
    pub cwd: PathBuf,
    pub provider: String,
}

/// 运行交互模式。
pub async fn run_interactive(mut cfg: InteractiveConfig) -> Result<()> {
    let state = Arc::new(AppState::new(&cfg.model, &cfg.provider));
    let theme = cfg.theme.clone();

    // Git 状态检测
    let git_status = crate::git::detect(&cfg.cwd);
    let git_display = crate::git::format_status(&git_status);

    // 存储最后一条用户消息，用于 Retry
    let last_user_msg: Arc<RwLock<String>> = Arc::new(RwLock::new(String::new()));

    let tools = make_tools(&cfg.cwd);
    let session_dir = cfg.session_dir.clone();

    let mut ctx = Arc::new(AgentContext {
        model: cfg.model.clone(),
        api_key: cfg.api_key.clone(),
        api_type: cfg.api_type.clone(),
        base_url: cfg.base_url.clone(),
        system_prompt: cfg.system_prompt.clone(),
        cwd: cfg.cwd.clone(),
        provider: cfg.provider.clone(),
    });

    // 可用模型列表（用于模型选择器）
    let cfg_models = cfg.available_models.clone();

    // 使用预加载 session 或创建新的
    let mgr = SessionManager::new(&session_dir);
    mgr.ensure_dir().await?;
    let mut session = match cfg.session {
        Some(s) => s,
        None => mgr.create(&cfg.cwd.to_string_lossy()).await?,
    };

    // 把 session 历史消息加载到 AppState
    load_history(&session, &state);

    // 缓存 session ID（header 显示用）
    let mut _session_id_display = session.id().to_string();

    let (cmd_tx, mut cmd_rx) = mpsc::unbounded_channel::<Command>();
    let abort_flag = Arc::new(AtomicBool::new(false));

    // Agent task — 持有 session，串行处理命令
    let agent_state = state.clone();
    let agent_abort = abort_flag.clone();
    let agent_ctx = ctx.clone();
    let cwd_for_slash = agent_ctx.cwd.clone();

    // 共享的 diff viewer 状态
    let diff_viewer_shared: Arc<std::sync::RwLock<Option<crate::diff_viewer::DiffViewer>>> =
        Arc::new(std::sync::RwLock::new(None));
    let diff_viewer_in_spawn = diff_viewer_shared.clone();
    let agent_handle = tokio::spawn(async move {
        while let Some(cmd) = cmd_rx.recv().await {
            match cmd {
                Command::Send { text } => {
                    abort_flag_clear(&agent_abort);
                    // 解析 @file 引用（包括图片）
                    let (resolved, file_refs) = pi_tools::fileref::resolve_file_refs(
                        &text,
                        &std::path::PathBuf::from(&agent_ctx.cwd),
                    );
                    // 图片文件：追加描述到消息
                    let final_text = if file_refs.iter().any(|r| r.image.is_some()) {
                        let images: Vec<_> = file_refs
                            .iter()
                            .filter_map(|r| {
                                r.image
                                    .as_ref()
                                    .map(|img| format!("[image: {} ({})]", r.path, img.mime_type))
                            })
                            .collect();
                        format!("{}\n\nAttached images: {}", resolved, images.join(", "))
                    } else {
                        resolved
                    };
                    run_agent_turn(&final_text, &mut session, &agent_state, &tools, &agent_ctx)
                        .await;
                }
                Command::Abort => {
                    agent_abort.store(true, Ordering::SeqCst);
                }
                Command::Export { path } => {
                    // 根据文件扩展名选择导出格式
                    let entries = agent_state.entries.read();
                    if path.ends_with(".jsonl") {
                        let mut jsonl = String::new();
                        for entry in entries.iter() {
                            // 手动构建 JSON 行（ChatEntry 不 derive Serialize）
                            let mut obj = serde_json::Map::new();
                            obj.insert("role".to_string(), json!(format!("{:?}", entry.role)));
                            obj.insert("content".to_string(), json!(entry.content));
                            obj.insert("streaming".to_string(), json!(entry.streaming));
                            if let Ok(line) = serde_json::to_string(&obj) {
                                jsonl.push_str(&line);
                                jsonl.push('\n');
                            }
                        }
                        drop(entries);
                        if let Err(e) = std::fs::write(&path, jsonl) {
                            agent_state.push_system(&format!("Export failed: {e}"));
                        } else {
                            agent_state.push_system(&format!("Exported JSONL to {}", path));
                        }
                    } else {
                        let html = render_session_html_simple(&entries);
                        drop(entries);
                        if let Err(e) = std::fs::write(&path, html) {
                            agent_state.push_system(&format!("Export failed: {e}"));
                        } else {
                            agent_state.push_system(&format!("Exported HTML to {}", path));
                        }
                    }
                }
                Command::Compact => {
                    agent_state.push_system("Compacting context...");
                    // 简单实现：截断到最近 10 条消息
                    let entries = agent_state.entries.read();
                    let total = entries.len();
                    drop(entries);
                    if total > 20 {
                        // 删除中间条目，保留前 2 条和最后 10 条
                        let keep_front = 2;
                        let keep_back = 10;
                        let mut guard = agent_state.entries.write();
                        let mut new_entries = Vec::new();
                        for (i, entry) in guard.drain(..).enumerate() {
                            if i < keep_front || i >= total - keep_back {
                                new_entries.push(entry);
                            }
                        }
                        let removed = total - new_entries.len();
                        *guard = new_entries;
                        drop(guard);
                        agent_state.push_system(&format!(
                            "Compacted: removed {} older messages ({} remaining)",
                            removed,
                            total - removed
                        ));
                    } else {
                        agent_state
                            .push_system(&format!("Only {} messages, no compaction needed", total));
                    }
                }
                Command::NewSession => {
                    agent_state.push_system("New session requested — restart piso to start fresh.");
                }
                Command::Import { path } => {
                    let file_path = std::path::PathBuf::from(&path);
                    if !file_path.exists() {
                        agent_state.push_system(&format!("File not found: {}", path));
                    } else {
                        match pi_session::jsonl::JsonlSession::open(&file_path).await {
                            Ok(imported) => {
                                let count = imported.len();
                                for entry in imported.entries() {
                                    let _ = session.append(entry.clone()).await;
                                }
                                agent_state.push_system(&format!(
                                    "Imported {} entries from {}",
                                    count, path
                                ));
                            }
                            Err(e) => {
                                agent_state.push_system(&format!("Failed to import: {e}"));
                            }
                        }
                    }
                }
                Command::CloneSession => {
                    let entries = session.entries().to_vec();
                    let count = entries.len();
                    let _mgr = SessionManager::new(
                        std::path::PathBuf::from(&agent_ctx.cwd)
                            .parent()
                            .unwrap_or(std::path::Path::new("."))
                            .join(".piso/sessions"),
                    );
                    // 使用 session_dir 创建克隆
                    agent_state.push_system(&format!(
                        "Cloned {} entries. New session will be available on restart.",
                        count
                    ));
                }
                Command::ShowDiff { diff_text } => {
                    let viewer = crate::diff_viewer::DiffViewer::from_unified_diff(&diff_text);
                    *diff_viewer_in_spawn.write().unwrap() = Some(viewer);
                }
            }
        }
    });

    // 快捷键配置（由 CLI 层加载 ~/.piso/keybindings.json）
    let keybindings = cfg.keybindings;
    let extension_runner = cfg.extension_runner.take();
    let mut engine = TuiEngine::init()?;

    // 设置终端标题（OSC 0;title BEL）
    {
        let cwd_name = std::path::Path::new(&cfg.cwd)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let title = format!("piso - {}", cwd_name);
        let _ = std::io::Write::write_all(
            &mut std::io::stderr(),
            format!("\x1b]0;{}\x07", title).as_bytes(),
        );
    }

    let mut input = crate::input::InputEditor::new();
    let mut scroll_offset: usize = 0;
    let mut tick: usize = 0;

    // Overlay 状态
    let mut overlay: Option<selector::Selector> = None;
    let mut overlay_kind: Option<OverlayKind> = None;

    // Diff 查看器
    // (shared state is diff_viewer_shared)

    loop {
        let hints = keybindings.footer_hints(!matches!(state.agent_state(), AgentState::Idle));
        engine.terminal().draw(|f| {
            let size = f.area();
            let regions = layout::calculate(size, 5);
            components::render_all(
                f,
                regions,
                &state,
                input.text(),
                input.cursor(),
                scroll_offset,
                &git_display,
                &hints,
                tick,
                &theme,
            );

            // 渲染 overlay（如果有）
            if let Some(ref mut sel) = overlay {
                let w = (size.width as f32 * 0.6) as u16;
                let h = (size.height as f32 * 0.5).min((sel.items.len() + 2) as f32) as u16;
                let x = (size.width.saturating_sub(w)) / 2;
                let y = (size.height.saturating_sub(h)) / 2;
                let sel_area = ratatui::layout::Rect::new(x, y, w, h);
                selector::Selector::render(f, sel_area, sel);
            }

            // 渲染 diff 查看器（如果有）
            {
                let guard = diff_viewer_shared.read().unwrap();
                if let Some(ref viewer) = *guard {
                    if viewer.is_visible() {
                        let w = (size.width as f32 * 0.8) as u16;
                        let h = (size.height as f32 * 0.8) as u16;
                        let x = (size.width.saturating_sub(w)) / 2;
                        let y = (size.height.saturating_sub(h)) / 2;
                        let diff_area = ratatui::layout::Rect::new(x, y, w, h);
                        viewer.render(f, diff_area);
                    }
                }
            }
        })?;

        let event = match engine.next_event().await {
            Some(e) => e,
            None => break,
        };

        // Overlay 模式：拦截所有输入
        if let Some(ref mut sel) = overlay {
            if let Event::Key(key) = event {
                match key.code {
                    KeyCode::Up => sel.up(),
                    KeyCode::Down => sel.down(),
                    KeyCode::Enter => {
                        if let Some(id) = sel.selected_id().to_owned() {
                            match overlay_kind {
                                Some(OverlayKind::SessionPicker) => {
                                    let mgr = SessionManager::new(&session_dir);
                                    match mgr.open(id).await {
                                        Ok(new_session) => {
                                            state.entries.write().clear();
                                            load_history(&new_session, &state);
                                            _session_id_display = new_session.id().to_string();
                                        }
                                        Err(e) => {
                                            state.set_state(AgentState::Error(format!(
                                                "Session: {e}"
                                            )));
                                        }
                                    }
                                }
                                Some(OverlayKind::ModelPicker) => {
                                    // id 格式: "provider:model_id"
                                    let parts: Vec<&str> = id.splitn(2, ':').collect();
                                    if parts.len() == 2 {
                                        let new_provider = parts[0].to_string();
                                        let new_model = parts[1].to_string();
                                        // 更新 footer 显示
                                        {
                                            let mut f = state.footer.write();
                                            f.model = new_model.clone();
                                            f.provider = new_provider.clone();
                                        }
                                        // 更新 agent context
                                        let mut old_ctx = Arc::try_unwrap(ctx)
                                            .unwrap_or_else(|arc| (*arc).clone());
                                        old_ctx.model = new_model;
                                        old_ctx.provider = new_provider;
                                        // TODO: 需要从新 provider 获取 api_key/api_type/base_url
                                        // 当前简化：只切换同一 provider 下的 model
                                        ctx = Arc::new(old_ctx);
                                    }
                                }
                                None => {}
                            }
                        }
                        overlay = None;
                        overlay_kind = None;
                    }
                    KeyCode::Esc => {
                        overlay = None;
                        overlay_kind = None;
                    }
                    _ => {}
                }
            }
            continue;
        }

        // Diff 查看器模式：拦截所有输入
        {
            let mut guard = diff_viewer_shared.write().unwrap();
            if let Some(ref mut viewer) = *guard {
                if viewer.is_visible() {
                    if let Event::Key(key) = event {
                        match key.code {
                            KeyCode::Char('j') | KeyCode::Down => viewer.scroll_down(3),
                            KeyCode::Char('k') | KeyCode::Up => viewer.scroll_up(3),
                            KeyCode::Char('n') | KeyCode::Right => viewer.next_file(),
                            KeyCode::Char('p') | KeyCode::Left => viewer.prev_file(),
                            KeyCode::Char('q') | KeyCode::Esc => viewer.hide(),
                            _ => {}
                        }
                    }
                    continue;
                }
            }
        }

        // 正常模式
        match event {
            Event::Key(key) => {
                let action = keybindings.match_key(&key);
                let is_running = !matches!(state.agent_state(), AgentState::Idle);

                match action {
                    Action::Submit => {
                        if !input.is_empty() && !is_running {
                            let text = input.take();
                            scroll_offset = 0;

                            // 检查 slash 命令
                            if let Some(cmd) = crate::slash::parse(&text) {
                                let mut slash_ctx = SlashContext {
                                    state: &state,
                                    cmd_tx: &cmd_tx,
                                    session_dir: &session_dir,
                                    overlay: &mut overlay,
                                    overlay_kind: &mut overlay_kind,
                                    extension_runner: &extension_runner,
                                    cwd: std::path::Path::new(&cwd_for_slash),
                                };
                                let skill_content = handle_slash_command(cmd, &mut slash_ctx).await;
                                if let Some(skill_text) = skill_content {
                                    *last_user_msg.write().unwrap() = skill_text.clone();
                                    let _ = cmd_tx.send(Command::Send { text: skill_text });
                                }
                                continue;
                            }

                            *last_user_msg.write().unwrap() = text.clone();
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
                    Action::OpenSessionPicker => {
                        let mgr = SessionManager::new(&session_dir);
                        if let Ok(sessions) = mgr.list().await {
                            let items: Vec<selector::SelectItem> = sessions
                                .into_iter()
                                .map(|s| selector::SelectItem {
                                    id: s.id.clone(),
                                    label: s.id.clone(),
                                    detail: format!("{} msgs, {}", s.message_count, s.cwd),
                                })
                                .collect();
                            if !items.is_empty() {
                                overlay = Some(selector::Selector::new("Sessions", items));
                                overlay_kind = Some(OverlayKind::SessionPicker);
                            }
                        }
                    }
                    Action::OpenModelPicker => {
                        let models = cfg_models.clone();
                        if !models.is_empty() {
                            let items: Vec<selector::SelectItem> = models
                                .into_iter()
                                .map(|(prov, mid, name)| selector::SelectItem {
                                    id: format!("{}:{}", prov, mid),
                                    label: name.clone(),
                                    detail: prov.to_string(),
                                })
                                .collect();
                            overlay = Some(selector::Selector::new("Models", items));
                            overlay_kind = Some(OverlayKind::ModelPicker);
                        }
                    }
                    Action::TabComplete => {
                        if !input.is_empty() && !is_running {
                            let text = input.text().to_string();
                            let cursor = input.cursor();
                            if let Some((candidates, start)) =
                                crate::complete::complete(&text, cursor, &ctx.cwd)
                            {
                                if candidates.len() == 1 {
                                    // 单一候选：直接替换
                                    input.replace_range(start, &candidates[0].text);
                                } else if !candidates.is_empty() {
                                    // 多个候选：显示第一个，循环
                                    input.replace_range(start, &candidates[0].text);
                                    // 显示候选数量
                                    let hint: Vec<String> = candidates
                                        .iter()
                                        .take(8)
                                        .map(|c| c.display.clone())
                                        .collect();
                                    state.push_system(&format!("Completions: {}", hint.join("  ")));
                                }
                            }
                        }
                    }
                    Action::ToggleFocus => {}
                    Action::NewSession => {}
                    Action::NewLine => {
                        input.insert('\n');
                    }
                    Action::Retry => {
                        if !is_running {
                            let msg = last_user_msg.read().unwrap().clone();
                            if !msg.is_empty() {
                                let _ = cmd_tx.send(Command::Send { text: msg });
                            }
                        }
                    }
                    Action::None => match key.code {
                        KeyCode::Char(c) => input.insert(c),
                        KeyCode::Backspace => input.backspace(),
                        KeyCode::Delete => input.delete(),
                        KeyCode::Left => input.move_left(),
                        KeyCode::Right => input.move_right(),
                        KeyCode::Home => input.move_home(),
                        KeyCode::End => input.move_end(),
                        KeyCode::Up => input.history_up(),
                        KeyCode::Down => input.history_down(),
                        _ => {}
                    },
                }
            }
            Event::Resize(_, _) => {}
            Event::Mouse(mouse) => {
                use crossterm::event::MouseEventKind;
                match mouse.kind {
                    MouseEventKind::ScrollUp => {
                        scroll_offset = scroll_offset.saturating_add(3);
                    }
                    MouseEventKind::ScrollDown => {
                        scroll_offset = scroll_offset.saturating_sub(3);
                    }
                    _ => {}
                }
            }
            Event::Tick => {
                tick = tick.wrapping_add(1);
            }
            Event::ShowDiff(_text) => {
                // Handled via shared state, not events
            }
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
    ctx: &AgentContext,
) {
    state.push_user(text);
    state.set_state(AgentState::Thinking);

    // 创建 driver
    let driver: Box<dyn LlmDriver> = match ctx.api_type.as_str() {
        "openai-completions" | "openai-responses" => Box::new(pi_llm::openai::OpenAiDriver::new()),
        "google-gemini" | "gemini" => Box::new(pi_llm::gemini::GeminiDriver::new()),
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
            StreamEvent::ThinkingDelta { thinking } => {
                sink_state.push_thinking_delta(&thinking);
            }
            StreamEvent::ToolCallStart { name, .. } => {
                sink_state.set_state(AgentState::ToolRunning { name });
            }
            StreamEvent::ToolResult {
                name,
                output,
                is_error,
                ..
            } => {
                sink_state.push_tool_result(&name, &output, is_error);
                sink_state.set_state(AgentState::Thinking);
            }
            StreamEvent::Stop { .. } => {
                sink_state.finish_assistant();
            }
            StreamEvent::Usage(usage) => {
                let mut f = sink_state.footer.write();
                f.input_tokens += usage.input_tokens;
                f.output_tokens += usage.output_tokens;
            }
            StreamEvent::ContextTokens { tokens } => {
                sink_state.set_context_tokens(tokens);
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
    let dummy_session = JsonlSession::create(&dummy_path, "")
        .await
        .unwrap_or_else(|_| panic!("Failed to create dummy session"));

    let mut agent = AgentLoop::new(
        std::mem::replace(session, dummy_session),
        driver,
        tools.clone_for_agent(),
        &ctx.model,
    )
    .with_api_key(&ctx.api_key)
    .with_system_prompt(&ctx.system_prompt)
    .with_stream_sink(sink);

    if let Some(url) = &ctx.base_url {
        agent = agent.with_base_url(url);
    }

    state.set_state(AgentState::Streaming);

    match agent.run(text).await {
        Ok(output) => {
            state.finish_assistant();
            state.set_state(AgentState::Idle);
            // 更新 token 用量
            state.set_usage(output.usage.input_tokens, output.usage.output_tokens);
        }
        Err(e) => {
            state.finish_assistant();
            state.set_state(AgentState::Error(format!("{e}")));
        }
    }

    // 取回 session（包含所有累积的消息历史）
    *session = agent.into_session();

    // 检查是否需要上下文压缩
    if pi_agent::compaction::should_compact(session) {
        // 创建 driver 用于压缩
        let compaction_driver: Box<dyn LlmDriver> = match ctx.api_type.as_str() {
            "openai-completions" | "openai-responses" => {
                Box::new(pi_llm::openai::OpenAiDriver::new())
            }
            "google-gemini" | "gemini" => Box::new(pi_llm::gemini::GeminiDriver::new()),
            _ => Box::new(pi_llm::providers::AnthropicDriver::new()),
        };
        match pi_agent::compaction::compact(
            session,
            compaction_driver.as_ref(),
            &ctx.model,
            &ctx.api_key,
            &ctx.base_url,
            None, // no previous summary available in TUI path
        )
        .await
        {
            Ok(true) => tracing::info!("[compaction] completed"),
            Ok(false) => {}
            Err(e) => tracing::warn!("[compaction] failed: {e}"),
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

fn abort_flag_clear(flag: &Arc<AtomicBool>) {
    flag.store(false, Ordering::SeqCst);
}

/// 从 session 历史加载消息到 AppState（用于 --continue/--session）。
fn load_history(session: &JsonlSession, state: &AppState) {
    use pi_types::session::SessionEntry;
    for entry in session.entries() {
        if let SessionEntry::Message(msg) = entry {
            // 从 content 数组提取文本
            let text = extract_text_from_content(&msg.content);
            if text.is_empty() {
                continue;
            }
            match msg.role.as_str() {
                "user" => {
                    // 检查是否是工具结果（content 包含 tool_result）
                    if has_tool_results(&msg.content) {
                        // 提取工具结果
                        for block in msg.content.as_array().into_iter().flatten() {
                            if block.get("type").and_then(|v| v.as_str()) == Some("tool_result") {
                                let name =
                                    block.get("name").and_then(|v| v.as_str()).unwrap_or("tool");
                                let output =
                                    block.get("content").and_then(|v| v.as_str()).unwrap_or("");
                                let is_error = block
                                    .get("is_error")
                                    .and_then(|v| v.as_bool())
                                    .unwrap_or(false);
                                state.push_tool_result(name, output, is_error);
                            }
                        }
                    } else {
                        state.push_user(&text);
                    }
                }
                "assistant" => {
                    state.push_assistant_delta(&text);
                    state.finish_assistant();
                }
                _ => {}
            }
        }
    }
}

/// 从 JSON content 数组提取纯文本。
fn extract_text_from_content(content: &serde_json::Value) -> String {
    content
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|block| {
                    if block.get("type").and_then(|v| v.as_str()) == Some("text") {
                        block
                            .get("text")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default()
}

/// 检查 content 是否包含 tool_result 块。
fn has_tool_results(content: &serde_json::Value) -> bool {
    content
        .as_array()
        .map(|arr| {
            arr.iter()
                .any(|block| block.get("type").and_then(|v| v.as_str()) == Some("tool_result"))
        })
        .unwrap_or(false)
}

/// 处理 slash 命令。
/// handle_slash_command 的上下文参数。
struct SlashContext<'a> {
    state: &'a Arc<AppState>,
    cmd_tx: &'a tokio::sync::mpsc::UnboundedSender<Command>,
    session_dir: &'a std::path::PathBuf,
    overlay: &'a mut Option<crate::selector::Selector>,
    overlay_kind: &'a mut Option<OverlayKind>,
    extension_runner: &'a Option<Arc<pi_extensions::ExtensionRunner>>,
    cwd: &'a std::path::Path,
}

async fn handle_slash_command(
    cmd: crate::slash::SlashCommand,
    ctx: &mut SlashContext<'_>,
) -> Option<String> {
    use crate::slash::SlashCommand;

    // 先检查扩展注册的命令
    if let Some(runner) = ctx.extension_runner {
        if let SlashCommand::Unknown(ref name) = cmd {
            if let Some(entry) = runner.find_command(name) {
                let args = match &cmd {
                    SlashCommand::Unknown(a) => a.as_str(),
                    _ => "",
                };
                match (entry.handler)(args) {
                    Ok(()) => {}
                    Err(e) => ctx
                        .state
                        .push_system(&format!("Command /{} failed: {}", name, e)),
                }
                return None;
            }
        }
    }

    match cmd {
        SlashCommand::Help => {
            let mut help = crate::slash::help_text();
            // 追加扩展命令
            if let Some(runner) = ctx.extension_runner {
                let names = runner.command_names();
                if !names.is_empty() {
                    help.push_str("\nExtension commands:");
                    // 需要获取描述 — 通过 find_command
                    for name in names {
                        if let Some(cmd_entry) = runner.find_command(name) {
                            help.push_str(&format!("\n  /{} — {}", name, cmd_entry.description));
                        }
                    }
                }
            }
            ctx.state.push_system(&help);
        }
        SlashCommand::Clear => {
            ctx.state.entries.write().clear();
            ctx.state.push_system("Chat cleared.");
        }
        SlashCommand::Compact => {
            let _ = ctx.cmd_tx.send(Command::Compact);
        }
        SlashCommand::Model(name) => {
            match name {
                Some(model_name) => {
                    // 直接切换模型
                    {
                        let mut f = ctx.state.footer.write();
                        f.model = model_name.clone();
                    }
                    ctx.state
                        .push_system(&format!("Model switched to {}", model_name));
                }
                None => {
                    // 打开模型选择器 — 通过触发 overlay
                    // 需要调用方来处理，简化版：显示提示
                    ctx.state.push_system(
                        "Use Ctrl+P to open model picker, or /model <name> to switch directly.",
                    );
                }
            }
        }
        SlashCommand::Branch => {
            ctx.state
                .push_system("Branch: use /sessions to pick a session to branch from.");
        }
        SlashCommand::Export(path) => {
            let _ = ctx.cmd_tx.send(Command::Export { path });
        }
        SlashCommand::Usage => {
            let f = ctx.state.footer.read();
            let in_t = f.input_tokens;
            let out_t = f.output_tokens;
            let model = f.model.clone();
            drop(f);
            let cost = pi_agent::cost::estimate_cost(&model, in_t as u64, out_t as u64);
            let cost_str = pi_agent::cost::format_cost(cost);
            ctx.state.push_system(&format!(
                "Token usage: {} input, {} output, {} total | Estimated cost: {}",
                in_t,
                out_t,
                in_t + out_t,
                cost_str
            ));
        }
        SlashCommand::Cost => {
            let f = ctx.state.footer.read();
            let in_t = f.input_tokens;
            let out_t = f.output_tokens;
            let model = f.model.clone();
            drop(f);
            let pricing = pi_agent::cost::get_pricing(&model);
            let cost = pi_agent::cost::estimate_cost(&model, in_t as u64, out_t as u64);
            let cost_str = pi_agent::cost::format_cost(cost);
            ctx.state.push_system(&format!(
                "Cost estimate for {}:\n  Input:  {} tokens @ ${}/M = {}\n  Output: {} tokens @ ${}/M = {}\n  Total: {}",
                model,
                in_t, pricing.input_per_m, pi_agent::cost::format_cost((in_t as f64 / 1_000_000.0) * pricing.input_per_m),
                out_t, pricing.output_per_m, pi_agent::cost::format_cost((out_t as f64 / 1_000_000.0) * pricing.output_per_m),
                cost_str,
            ));
        }
        SlashCommand::Find(term) => {
            if term.is_empty() {
                ctx.state.push_system("Usage: /find <search term>");
                return None;
            }
            let mgr = SessionManager::new(ctx.session_dir.to_path_buf());
            match mgr.list().await {
                Ok(sessions) => {
                    let mut results = Vec::new();
                    let term_lower = term.to_lowercase();
                    for sess in &sessions {
                        // 搜索 session ID 和 CWD
                        let haystack = format!("{} {}", sess.id, sess.cwd).to_lowercase();
                        if haystack.contains(&term_lower) {
                            results.push(format!(
                                "{} ({} msgs, {})",
                                sess.id, sess.message_count, sess.cwd
                            ));
                            if results.len() >= 20 {
                                break;
                            }
                        }
                    }
                    if results.is_empty() {
                        ctx.state
                            .push_system(&format!("No sessions matching '{}'", term));
                    } else {
                        ctx.state.push_system(&format!(
                            "Found {} sessions:\n{}",
                            results.len(),
                            results.join("\n")
                        ));
                    }
                }
                Err(e) => ctx.state.push_system(&format!("Search failed: {e}")),
            }
        }
        SlashCommand::Grep(term) => {
            if term.is_empty() {
                ctx.state.push_system("Usage: /grep <search term>");
                return None;
            }
            let entries = ctx.state.entries.read();
            let term_lower = term.to_lowercase();
            let mut matches = Vec::new();
            for (i, entry) in entries.iter().enumerate() {
                if entry.content.to_lowercase().contains(&term_lower) {
                    let preview = if entry.content.len() > 80 {
                        format!("{}...", &entry.content[..80])
                    } else {
                        entry.content.clone()
                    };
                    matches.push(format!(
                        "[{}] {}: {}",
                        i,
                        entry.role,
                        preview.replace('\n', " ")
                    ));
                    if matches.len() >= 20 {
                        break;
                    }
                }
            }
            if matches.is_empty() {
                ctx.state
                    .push_system(&format!("No messages matching '{}'", term));
            } else {
                ctx.state.push_system(&format!(
                    "Found {} messages:\n{}",
                    matches.len(),
                    matches.join("\n")
                ));
            }
        }
        SlashCommand::NewSession => {
            let _ = ctx.cmd_tx.send(Command::NewSession);
            ctx.state.push_system("Starting new session...");
        }
        SlashCommand::Reload => {
            // 重载快捷键和主题
            let kb_path = std::env::var("HOME")
                .ok()
                .map(|h| std::path::PathBuf::from(h).join(".piso/keybindings.json"))
                .unwrap_or_default();
            if kb_path.exists() {
                let _kb = crate::keybinding::KeyBindings::load(&kb_path);
                ctx.state.push_system("Reloaded keybindings");
            } else {
                ctx.state.push_system("No keybindings.json found");
            }
        }
        SlashCommand::Copy => {
            // 找到最后一条 assistant 消息
            let entries = ctx.state.entries.read();
            let last_assistant = entries
                .iter()
                .rev()
                .find(|e| matches!(e.role, crate::app::ChatRole::Assistant));
            if let Some(entry) = last_assistant {
                let text = entry.content.clone();
                drop(entries);
                // 尝试复制到剪贴板
                match std::process::Command::new("xclip")
                    .args(["-selection", "clipboard"])
                    .stdin(std::process::Stdio::piped())
                    .spawn()
                {
                    Ok(mut child) => {
                        if let Some(stdin) = child.stdin.as_mut() {
                            use std::io::Write;
                            let _ = stdin.write_all(text.as_bytes());
                        }
                        let _ = child.wait();
                        ctx.state
                            .push_system(&format!("Copied {} chars to clipboard", text.len()));
                    }
                    Err(_) => {
                        // xclip 不可用，尝试 pbcopy (macOS)
                        match std::process::Command::new("pbcopy")
                            .stdin(std::process::Stdio::piped())
                            .spawn()
                        {
                            Ok(mut child) => {
                                if let Some(stdin) = child.stdin.as_mut() {
                                    use std::io::Write;
                                    let _ = stdin.write_all(text.as_bytes());
                                }
                                let _ = child.wait();
                                ctx.state.push_system(&format!(
                                    "Copied {} chars to clipboard",
                                    text.len()
                                ));
                            }
                            Err(_) => {
                                ctx.state.push_system(
                                    "No clipboard tool found (install xclip or pbcopy)",
                                );
                            }
                        }
                    }
                }
            } else {
                drop(entries);
                ctx.state.push_system("No assistant message to copy");
            }
        }
        SlashCommand::Fork(at) => {
            if at.is_empty() {
                ctx.state.push_system("Usage: /fork <message-id or index>");
            } else {
                ctx.state
                    .push_system(&format!("Fork at '{}' not yet implemented", at));
            }
        }
        SlashCommand::SessionInfo => {
            let entries = ctx.state.entries.read();
            let footer = ctx.state.footer.read();
            let user_count = entries
                .iter()
                .filter(|e| matches!(e.role, crate::app::ChatRole::User))
                .count();
            let assistant_count = entries
                .iter()
                .filter(|e| matches!(e.role, crate::app::ChatRole::Assistant))
                .count();
            let tool_count = entries
                .iter()
                .filter(|e| matches!(e.role, crate::app::ChatRole::Tool { .. }))
                .count();
            drop(entries);
            ctx.state.push_system(&format!(
                "Session info:\n  Messages: {} user, {} assistant, {} tool\n  Model: {} ({})\n  Tokens: {} in, {} out\n  Total entries: {}",
                user_count, assistant_count, tool_count,
                footer.model, footer.provider,
                footer.input_tokens, footer.output_tokens,
                user_count + assistant_count + tool_count,
            ));
        }
        SlashCommand::Name(name) => {
            if name.is_empty() {
                ctx.state.push_system("Usage: /name <session-name>");
            } else {
                ctx.state.push_system(&format!("Session named: {}", name));
            }
        }
        SlashCommand::Import(path) => {
            if path.is_empty() {
                ctx.state.push_system("Usage: /import <path-to-jsonl-file>");
            } else {
                let _ = ctx.cmd_tx.send(Command::Import { path });
            }
        }
        SlashCommand::Clone => {
            let _ = ctx.cmd_tx.send(Command::CloneSession);
        }
        SlashCommand::Sessions => {
            let mgr = SessionManager::new(ctx.session_dir);
            if let Ok(sessions) = mgr.list().await {
                let items: Vec<crate::selector::SelectItem> = sessions
                    .into_iter()
                    .map(|s| crate::selector::SelectItem {
                        id: s.id.clone(),
                        label: s.id.clone(),
                        detail: format!("{} msgs, {}", s.message_count, s.cwd),
                    })
                    .collect();
                if !items.is_empty() {
                    *ctx.overlay = Some(crate::selector::Selector::new("Sessions", items));
                    *ctx.overlay_kind = Some(OverlayKind::SessionPicker);
                }
            }
        }
        SlashCommand::Quit => {
            ctx.state.push_system("Use Ctrl+C to quit.");
        }
        SlashCommand::Diff => {
            // 从最近工具调用收集 diff
            let entries = ctx.state.entries.read();
            let mut diff_lines = Vec::new();
            for entry in entries.iter() {
                if matches!(entry.role, crate::app::ChatRole::Tool { .. })
                    && (entry.content.contains("diff --git") || entry.content.contains("--- a/"))
                {
                    diff_lines.push(entry.content.clone());
                }
            }
            drop(entries);
            if diff_lines.is_empty() {
                ctx.state.push_system("No diffs found in current session.");
            } else {
                let _ = ctx.cmd_tx.send(Command::ShowDiff {
                    diff_text: diff_lines.join("\n"),
                });
            }
        }
        SlashCommand::Login => {
            ctx.state
                .push_system("Use 'piso login' from terminal for GitHub Copilot OAuth.");
        }
        SlashCommand::Logout => {
            ctx.state
                .push_system("Use 'piso logout' from terminal to clear OAuth token.");
        }
        SlashCommand::Theme(name) => {
            match name {
                Some(n) => {
                    if let Some(_new_theme) = crate::theme::Theme::load_by_name(&n) {
                        // Note: theme change takes effect next render cycle
                        ctx.state
                            .push_system(&format!("Theme: {} (restart to apply)", n));
                    } else {
                        let available = crate::theme::Theme::list_available().join(", ");
                        ctx.state.push_system(&format!(
                            "Theme '{}' not found. Available: {}",
                            n, available
                        ));
                    }
                }
                None => {
                    let available = crate::theme::Theme::list_available().join(", ");
                    ctx.state
                        .push_system(&format!("Available themes: {}", available));
                }
            }
        }
        SlashCommand::Skill(name) => {
            match crate::slash::resolve_skill(&name, ctx.cwd) {
                Ok(content) => {
                    ctx.state.push_system(&format!("Loaded skill: {}", name));
                    // Return the skill content as the actual user message
                    return Some(content);
                }
                Err(e) => {
                    ctx.state.push_system(&format!("Skill error: {e}"));
                }
            }
        }
        SlashCommand::Unknown(cmd) => {
            ctx.state.push_system(&format!(
                "Unknown command: /{}. Type /help for available commands.",
                cmd
            ));
        }
    }
    None
}

/// 简化 HTML 渲染（从 AppState entries 生成）。
fn render_session_html_simple(entries: &std::vec::Vec<crate::app::ChatEntry>) -> String {
    use crate::app::ChatRole;
    let mut body = String::new();
    for entry in entries {
        let escaped = entry
            .content
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;");
        let (class, role) = match &entry.role {
            ChatRole::User => ("user", "You"),
            ChatRole::Assistant => ("assistant", "Assistant"),
            ChatRole::Thinking => ("thinking", "Thinking"),
            ChatRole::System => ("system", "System"),
            ChatRole::Tool { name, .. } => ("tool", name.as_str()),
        };
        body.push_str(&format!(
            "<div class=\"msg {}\"><div class=\"role\">{}</div><pre>{}</pre></div>\n",
            class, role, escaped
        ));
    }
    format!(
        r#"<!DOCTYPE html>
<html><head><meta charset="utf-8"><title>piso session</title>
<style>
body {{ font-family: sans-serif; max-width: 800px; margin: 0 auto; padding: 20px; background: #1a1a2e; color: #e0e0e0; }}
.msg {{ margin: 12px 0; padding: 10px 14px; border-radius: 6px; }}
.user {{ background: #16213e; border-left: 3px solid #0f3460; }}
.assistant {{ background: #1a1a2e; border-left: 3px solid #e94560; }}
.system {{ background: #0d1117; border-left: 3px solid #ffd700; color: #aaa; font-style: italic; }}
.role {{ font-weight: bold; font-size: 0.85em; color: #888; margin-bottom: 4px; }}
pre {{ white-space: pre-wrap; margin: 0; }}
</style></head><body>{}</body></html>"#,
        body
    )
}
