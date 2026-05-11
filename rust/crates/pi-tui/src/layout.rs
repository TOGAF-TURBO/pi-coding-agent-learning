//! 五区布局 — header / chat / status / editor / footer。
//!
//! 对应 TS 版本的 InteractiveMode 布局：
//! ```text
//! ┌─────────────────────────────────┐
//! │ header: model + session info    │  1 行
//! ├─────────────────────────────────┤
//! │                                 │
//! │ chat: 消息流（滚动区域）          │  flex
//! │                                 │
//! ├─────────────────────────────────┤
//! │ status: agent 状态 + token 计数  │  1 行
//! ├─────────────────────────────────┤
//! │ editor: 用户输入（可多行）        │  3-8 行
//! ├─────────────────────────────────┤
//! │ footer: 快捷键提示              │  1 行
//! └─────────────────────────────────┘
//! ```

use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// 布局区域索引。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Region {
    Header,
    Chat,
    Status,
    Editor,
    Footer,
}

/// 计算五区布局。
pub fn calculate(area: Rect, editor_height: u16) -> LayoutRegions {
    let editor_h = editor_height.max(3).min(area.height / 2);

    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),           // header
            Constraint::Min(3),              // chat (flex)
            Constraint::Length(1),           // status
            Constraint::Length(editor_h),    // editor
            Constraint::Length(1),           // footer
        ])
        .split(area);

    LayoutRegions {
        header: outer[0],
        chat: outer[1],
        status: outer[2],
        editor: outer[3],
        footer: outer[4],
    }
}

/// 计算后的布局区域。
#[derive(Debug, Clone, Copy)]
pub struct LayoutRegions {
    pub header: Rect,
    pub chat: Rect,
    pub status: Rect,
    pub editor: Rect,
    pub footer: Rect,
}

impl LayoutRegions {
    pub fn get(&self, region: Region) -> Rect {
        match region {
            Region::Header => self.header,
            Region::Chat => self.chat,
            Region::Status => self.status,
            Region::Editor => self.editor,
            Region::Footer => self.footer,
        }
    }
}
