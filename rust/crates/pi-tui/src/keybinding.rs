//! 快捷键系统 — 键绑定定义和匹配。
//!
//! 对应 `packages/coding-agent/src/core/keybindings.ts`。

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// 应用级快捷键动作。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// 提交输入。
    Submit,
    /// 退出。
    Quit,
    /// 取消当前操作。
    Cancel,
    /// 切换焦点（chat ↔ editor）。
    ToggleFocus,
    /// 向上滚动。
    ScrollUp,
    /// 向下滚动。
    ScrollDown,
    /// 新建会话。
    NewSession,
    /// 无操作。
    None,
}

/// 匹配快捷键。
pub fn match_key(key: &KeyEvent) -> Action {
    match (key.modifiers, key.code) {
        // Ctrl+O: 提交（与 TS 版本一致）
        (KeyModifiers::CONTROL, KeyCode::Char('o')) => Action::Submit,
        // Ctrl+C: 退出
        (KeyModifiers::CONTROL, KeyCode::Char('c')) => Action::Quit,
        // Escape: 取消
        (KeyModifiers::NONE, KeyCode::Esc) => Action::Cancel,
        // Tab: 切换焦点
        (KeyModifiers::NONE, KeyCode::Tab) => Action::ToggleFocus,
        // PageUp/Down: 滚动
        (KeyModifiers::NONE, KeyCode::PageUp) => Action::ScrollUp,
        (KeyModifiers::NONE, KeyCode::PageDown) => Action::ScrollDown,
        // Enter 在编辑器中换行，Ctrl+Enter 也提交（备选）
        (KeyModifiers::CONTROL, KeyCode::Enter) => Action::Submit,
        _ => Action::None,
    }
}
