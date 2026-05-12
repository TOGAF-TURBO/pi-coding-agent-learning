//! 四区布局 — chat / status / editor / footer。
//!
//! 与 TS 版对齐：无独立 header，模型信息在 footer 显示。
//! ```text
//! ┌─────────────────────────────────┐
//! │                                 │
//! │ chat: 消息流（滚动区域）          │  flex
//! │                                 │
//! ├─────────────────────────────────┤
//! │ status: agent 状态 + token 计数  │  1 行
//! ├─────────────────────────────────┤
//! │ editor: 用户输入（可多行）        │  3-8 行
//! ├─────────────────────────────────┤
//! │ footer: 模型 + 快捷键提示        │  1 行
//! └─────────────────────────────────┘
//! ```

use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// 布局区域索引。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Region {
    Chat,
    Status,
    Editor,
    Footer,
}

/// 左右边距（字符数）。
const HORIZONTAL_PADDING: u16 = 2;

/// 计算四区布局。
pub fn calculate(area: Rect, editor_height: u16) -> LayoutRegions {
    let editor_h = editor_height.max(3).min(area.height / 2);

    // 先水平内缩，留出左右边距
    let padded = Rect {
        x: area.x + HORIZONTAL_PADDING,
        y: area.y,
        width: area.width.saturating_sub(HORIZONTAL_PADDING * 2),
        height: area.height,
    };

    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),           // chat (flex)
            Constraint::Length(1),        // status
            Constraint::Length(editor_h), // editor
            Constraint::Length(1),        // footer
        ])
        .split(padded);

    LayoutRegions {
        chat: outer[0],
        status: outer[1],
        editor: outer[2],
        footer: outer[3],
    }
}

/// 计算后的布局区域。
#[derive(Debug, Clone, Copy)]
pub struct LayoutRegions {
    pub chat: Rect,
    pub status: Rect,
    pub editor: Rect,
    pub footer: Rect,
}

impl LayoutRegions {
    pub fn get(&self, region: Region) -> Rect {
        match region {
            Region::Chat => self.chat,
            Region::Status => self.status,
            Region::Editor => self.editor,
            Region::Footer => self.footer,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn four_regions_fit_in_24_rows() {
        let area = Rect::new(0, 0, 80, 24);
        let regions = calculate(area, 5);
        assert_eq!(regions.status.height, 1);
        assert_eq!(regions.editor.height, 5);
        assert_eq!(regions.footer.height, 1);
        // chat gets the rest
        assert!(regions.chat.height >= 15);
        // total must equal area height
        let total = regions.chat.height
            + regions.status.height
            + regions.editor.height
            + regions.footer.height;
        assert_eq!(total, 24);
    }

    #[test]
    fn editor_clamped_to_half_height() {
        let area = Rect::new(0, 0, 80, 10);
        let regions = calculate(area, 20); // request 20 but only 10 rows
        assert!(regions.editor.height <= 5); // clamped to half
    }
}
