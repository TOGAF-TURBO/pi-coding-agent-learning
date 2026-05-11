//! 主题 — 颜色和样式定义。
//!
//! 对应 `packages/tui/src/theme/` 的主题系统。

use ratatui::style::Color;

/// 主题配色。
#[derive(Debug, Clone)]
pub struct Theme {
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub error: Color,
    pub muted: Color,
    pub user_msg: Color,
    pub assistant_msg: Color,
    pub tool_msg: Color,
    pub border: Color,
    pub header_bg: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

impl Theme {
    /// 暗色主题（默认）。
    pub fn dark() -> Self {
        Self {
            primary: Color::Cyan,
            secondary: Color::Green,
            accent: Color::Magenta,
            error: Color::Red,
            muted: Color::DarkGray,
            user_msg: Color::Green,
            assistant_msg: Color::Cyan,
            tool_msg: Color::Magenta,
            border: Color::DarkGray,
            header_bg: Color::Black,
        }
    }

    /// 亮色主题。
    pub fn light() -> Self {
        Self {
            primary: Color::Blue,
            secondary: Color::Green,
            accent: Color::Magenta,
            error: Color::Red,
            muted: Color::Gray,
            user_msg: Color::Green,
            assistant_msg: Color::Blue,
            tool_msg: Color::Magenta,
            border: Color::Gray,
            header_bg: Color::White,
        }
    }
}
