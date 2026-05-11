//! TUI 引擎 — 事件循环、alternate screen、差异渲染。
//!
//! 负责：
//! - 初始化/恢复终端状态
//! - 从 crossterm 读取事件
//! - 调度事件到当前焦点组件
//! - 触发 ratatui 渲染

use std::io;
use std::time::Duration;

use crossterm::event::{self, Event as CrosstermEvent};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use crate::event::Event;

/// TUI tick rate（状态栏刷新间隔）。
const TICK_RATE: Duration = Duration::from_millis(250);

/// TUI 引擎。
pub struct TuiEngine {
    terminal: Terminal<CrosstermBackend<io::Stderr>>,
    event_rx: mpsc::UnboundedReceiver<Event>,
    event_tx: mpsc::UnboundedSender<Event>,
    reader_handle: JoinHandle<()>,
}

impl TuiEngine {
    /// 初始化 TUI：进入 alternate screen + raw mode。
    pub fn init() -> io::Result<Self> {
        enable_raw_mode()?;
        crossterm::execute!(io::stderr(), EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(io::stderr());
        let terminal = Terminal::new(backend)?;
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        let tx = event_tx.clone();
        let reader_handle = tokio::spawn(async move {
            // 用一个独立的 spawn_blocking task 持续 poll 事件
            // 比 select! + spawn_blocking 更可靠
            let tx = tx;
            let mut tick = tokio::time::interval(TICK_RATE);

            // 专用线程做 crossterm 事件读取
            let (key_tx, mut key_rx) = tokio::sync::mpsc::unbounded_channel::<Event>();
            let key_tx = key_tx;
            std::thread::spawn(move || loop {
                if event::poll(Duration::from_millis(100)).unwrap_or(false) {
                    match event::read() {
                        Ok(CrosstermEvent::Key(key)) => {
                            if key.kind == event::KeyEventKind::Release {
                                continue;
                            }
                            if key_tx.send(Event::Key(key)).is_err() {
                                break;
                            }
                        }
                        Ok(CrosstermEvent::Mouse(mouse)) => {
                            if key_tx.send(Event::Mouse(mouse)).is_err() {
                                break;
                            }
                        }
                        Ok(CrosstermEvent::Resize(w, h)) => {
                            if key_tx.send(Event::Resize(w, h)).is_err() {
                                break;
                            }
                        }
                        Ok(_) => {}
                        Err(_) => break,
                    }
                }
            });

            // 主循环：合并线程事件和 tick
            loop {
                tokio::select! {
                    ev = key_rx.recv() => {
                        match ev {
                            Some(event) => {
                                if tx.send(event).is_err() {
                                    break;
                                }
                            }
                            None => break,
                        }
                    }
                    _ = tick.tick() => {
                        if tx.send(Event::Tick).is_err() {
                            break;
                        }
                    }
                }
            }
        });

        Ok(Self {
            terminal,
            event_rx,
            event_tx,
            reader_handle,
        })
    }

    /// 获取下一个事件。
    pub async fn next_event(&mut self) -> Option<Event> {
        self.event_rx.recv().await
    }

    /// 获取可变终端引用（用于渲染）。
    pub fn terminal(&mut self) -> &mut Terminal<CrosstermBackend<io::Stderr>> {
        &mut self.terminal
    }

    /// 获取事件发送端（用于外部注入事件）。
    pub fn event_sender(&self) -> mpsc::UnboundedSender<Event> {
        self.event_tx.clone()
    }
}

impl Drop for TuiEngine {
    fn drop(&mut self) {
        self.reader_handle.abort();
        // 恢复终端标题
        let _ = std::io::Write::write_all(
            &mut std::io::stderr(),
            b"\x1b]0;\x07",
        );
        let _ = disable_raw_mode();
        let _ = crossterm::execute!(io::stderr(), LeaveAlternateScreen);
    }
}
