//! TUI 事件类型 — crossterm 事件的统一抽象。

use crossterm::event::{KeyEvent, MouseEvent};
use std::fmt;

/// TUI 事件。
#[derive(Debug)]
pub enum Event {
    /// 键盘事件。
    Key(KeyEvent),
    /// 鼠标事件。
    Mouse(MouseEvent),
    /// 终端大小变化。
    Resize(u16, u16),
    /// 定时器 tick（用于状态栏刷新等）。
    Tick,
}

/// 聚焦区域 — 五区布局中哪个区域获得焦点。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusArea {
    Chat,
    Editor,
}

impl fmt::Display for FocusArea {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Chat => write!(f, "chat"),
            Self::Editor => write!(f, "editor"),
        }
    }
}
