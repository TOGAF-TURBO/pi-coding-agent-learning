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
            let mut tick = tokio::time::interval(TICK_RATE);

            loop {
                tokio::select! {
                    _ = tick.tick() => {
                        if tx.send(Event::Tick).is_err() {
                            break;
                        }
                    }
                    _ = tokio::task::spawn_blocking(|| {
                        // 阻塞等待事件
                        let _ = event::read();
                    }) => {
                        // drain 所有 pending 事件
                        while event::poll(Duration::from_millis(0)).unwrap_or(false) {
                            match event::read() {
                                Ok(CrosstermEvent::Key(key)) => {
                                    if key.kind == event::KeyEventKind::Release {
                                        continue;
                                    }
                                    if tx.send(Event::Key(key)).is_err() {
                                        return;
                                    }
                                }
                                Ok(CrosstermEvent::Mouse(mouse)) => {
                                    if tx.send(Event::Mouse(mouse)).is_err() {
                                        return;
                                    }
                                }
                                Ok(CrosstermEvent::Resize(w, h)) => {
                                    if tx.send(Event::Resize(w, h)).is_err() {
                                        return;
                                    }
                                }
                                Ok(_) => {}
                                Err(_) => return,
                            }
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
        let _ = disable_raw_mode();
        let _ = crossterm::execute!(io::stderr(), LeaveAlternateScreen);
    }
}
