//! 交互模式 — TUI + Agent 循环的集成。
//!
//! 对应 `packages/coding-agent/src/modes/interactive/interactive-mode.ts`。
//!
//! 架构：主 task 运行 TUI 事件循环，收到 Submit 后启动 agent 子 task。

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use crossterm::event::KeyCode;

use crate::app::{AgentState, AppState};
use crate::components;
use crate::engine::TuiEngine;
use crate::event::Event;
use crate::input::InputEditor;
use crate::keybinding::{self, Action};
use crate::layout;

/// 运行交互模式。
///
/// `agent_runner` 是一个闭包，接收用户文本，运行 agent 循环，
/// 并通过 AppState 更新 UI 状态。
pub async fn run_interactive(
    model: String,
    provider: String,
    mut agent_runner: Box<dyn FnMut(String, Arc<AppState>) + Send>,
) -> Result<()> {
    let state = Arc::new(AppState::new(&model, &provider));
    let mut engine = TuiEngine::init()?;
    let mut input = InputEditor::new();

    loop {
        // 渲染
        engine.terminal().draw(|f| {
            let size = f.area();
            let regions = layout::calculate(size, 5);
            components::render_all(f, regions, &state, input.text());
        })?;

        // 事件处理
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
                            let s = state.clone();
                            // 同步运行 agent（阻塞当前 task）
                            // TODO: 改为 spawn + channel 以保持 UI 响应
                            agent_runner(text, s);
                        }
                    }
                    Action::Quit => break,
                    Action::Cancel => {
                        if !input.is_empty() {
                            input.clear();
                        }
                    }
                    Action::None => {
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
                    _ => {}
                }
            }
            Event::Resize(_, _) => {}
            _ => {}
        }
    }

    Ok(())
}
