//! 可配置快捷键 — 从 JSON 文件加载键绑定。
//!
//! 用户可在 `~/.piso/keybindings.json` 中自定义快捷键，
//! 未配置的动作使用内置默认值。

use std::path::Path;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::{Deserialize, Serialize};

/// 应用级快捷键动作。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Submit,
    Quit,
    Cancel,
    ToggleFocus,
    /// Tab 补全。
    TabComplete,
    ScrollUp,
    ScrollDown,
    NewSession,
    OpenSessionPicker,
    OpenModelPicker,
    /// 重试上一个 prompt。
    Retry,
    None,
}

/// 键绑定描述（JSON 可序列化）。
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KeyBinding {
    /// 修饰键组合，如 "ctrl", "alt", "ctrl+alt", "none"。
    pub modifiers: String,
    /// 键码，如 "o", "c", "enter", "esc", "pageup"。
    pub key: String,
}

/// 键绑定映射表。
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct KeyBindings {
    pub submit: Option<KeyBinding>,
    pub quit: Option<KeyBinding>,
    pub cancel: Option<KeyBinding>,
    pub toggle_focus: Option<KeyBinding>,
    pub scroll_up: Option<KeyBinding>,
    pub scroll_down: Option<KeyBinding>,
    pub new_session: Option<KeyBinding>,
    pub open_session_picker: Option<KeyBinding>,
    pub open_model_picker: Option<KeyBinding>,
    pub tab_complete: Option<KeyBinding>,
    /// 重试上一个 prompt。
    pub retry: Option<KeyBinding>,
}

/// 默认键绑定（与 TS 版 pi 兼容）。
impl KeyBindings {
    pub fn defaults() -> Self {
        Self {
            submit: Some(KeyBinding { modifiers: "ctrl".into(), key: "o".into() }),
            quit: Some(KeyBinding { modifiers: "ctrl".into(), key: "c".into() }),
            cancel: Some(KeyBinding { modifiers: "none".into(), key: "esc".into() }),
            toggle_focus: None,
            scroll_up: Some(KeyBinding { modifiers: "none".into(), key: "pageup".into() }),
            scroll_down: Some(KeyBinding { modifiers: "none".into(), key: "pagedown".into() }),
            new_session: Some(KeyBinding { modifiers: "ctrl".into(), key: "n".into() }),
            open_session_picker: Some(KeyBinding { modifiers: "ctrl".into(), key: "s".into() }),
            open_model_picker: Some(KeyBinding { modifiers: "ctrl".into(), key: "p".into() }),
            tab_complete: Some(KeyBinding { modifiers: "none".into(), key: "tab".into() }),
            retry: Some(KeyBinding { modifiers: "ctrl".into(), key: "r".into() }),
        }
    }

    /// 从 JSON 文件加载，缺失字段使用默认值。
    pub fn load(path: &Path) -> Self {
        let mut bindings = Self::defaults();
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(overrides) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(obj) = overrides.as_object() {
                    if let Some(v) = obj.get("submit") {
                        if let Ok(kb) = serde_json::from_value::<KeyBinding>(v.clone()) {
                            bindings.submit = Some(kb);
                        }
                    }
                    if let Some(v) = obj.get("quit") {
                        if let Ok(kb) = serde_json::from_value::<KeyBinding>(v.clone()) {
                            bindings.quit = Some(kb);
                        }
                    }
                    if let Some(v) = obj.get("cancel") {
                        if let Ok(kb) = serde_json::from_value::<KeyBinding>(v.clone()) {
                            bindings.cancel = Some(kb);
                        }
                    }
                    if let Some(v) = obj.get("scroll_up") {
                        if let Ok(kb) = serde_json::from_value::<KeyBinding>(v.clone()) {
                            bindings.scroll_up = Some(kb);
                        }
                    }
                    if let Some(v) = obj.get("scroll_down") {
                        if let Ok(kb) = serde_json::from_value::<KeyBinding>(v.clone()) {
                            bindings.scroll_down = Some(kb);
                        }
                    }
                    if let Some(v) = obj.get("new_session") {
                        if let Ok(kb) = serde_json::from_value::<KeyBinding>(v.clone()) {
                            bindings.new_session = Some(kb);
                        }
                    }
                    if let Some(v) = obj.get("open_session_picker") {
                        if let Ok(kb) = serde_json::from_value::<KeyBinding>(v.clone()) {
                            bindings.open_session_picker = Some(kb);
                        }
                    }
                    if let Some(v) = obj.get("open_model_picker") {
                        if let Ok(kb) = serde_json::from_value::<KeyBinding>(v.clone()) {
                            bindings.open_model_picker = Some(kb);
                        }
                    }
                }
            }
        }
        bindings
    }

    /// 构建匹配表：解析后的 (KeyCode, KeyModifiers) → Action。
    fn build_match_table(&self) -> Vec<(KeyCode, KeyModifiers, Action)> {
        let mut table = Vec::new();
        let fields: &[(&Option<KeyBinding>, Action)] = &[
            (&self.submit, Action::Submit),
            (&self.quit, Action::Quit),
            (&self.cancel, Action::Cancel),
            (&self.toggle_focus, Action::ToggleFocus),
            (&self.scroll_up, Action::ScrollUp),
            (&self.scroll_down, Action::ScrollDown),
            (&self.new_session, Action::NewSession),
            (&self.open_session_picker, Action::OpenSessionPicker),
            (&self.open_model_picker, Action::OpenModelPicker),
            (&self.tab_complete, Action::TabComplete),
            (&self.retry, Action::Retry),
        ];
        for (kb, action) in fields {
            if let Some(binding) = kb {
                if let (Some(code), mods) = parse_binding(binding) {
                    table.push((code, mods, action.clone()));
                }
            }
        }
        table
    }

    /// 匹配按键事件。
    pub fn match_key(&self, key: &KeyEvent) -> Action {
        let table = self.build_match_table();
        for (code, mods, action) in &table {
            if &key.code == code && &key.modifiers == mods {
                return action.clone();
            }
        }
        // Ctrl+Enter 也作为 Submit 的备选（硬编码，不可覆盖）
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Enter {
            return Action::Submit;
        }
        Action::None
    }

    /// 生成 footer 显示的快捷键提示文本。
    pub fn footer_hints(&self, is_running: bool) -> Vec<(&'static str, String)> {
        if is_running {
            let cancel_key = self.cancel.as_ref()
                .map(|kb| kb.display()).unwrap_or_else(|| "Esc".to_string());
            let quit_key = self.quit.as_ref()
                .map(|kb| kb.display()).unwrap_or_else(|| "Ctrl+C".to_string());
            vec![
                ("key", format!(" {}", cancel_key)),
                ("label", " Cancel  ".to_string()),
                ("key", format!(" {}", quit_key)),
                ("label", " Quit".to_string()),
            ]
        } else {
            let submit_key = self.submit.as_ref()
                .map(|kb| kb.display()).unwrap_or_else(|| "Ctrl+O".to_string());
            let quit_key = self.quit.as_ref()
                .map(|kb| kb.display()).unwrap_or_else(|| "Ctrl+C".to_string());
            let scroll_up_key = self.scroll_up.as_ref()
                .map(|kb| kb.display()).unwrap_or_else(|| "PgUp".to_string());
            let scroll_dn_key = self.scroll_down.as_ref()
                .map(|kb| kb.display()).unwrap_or_else(|| "PgDn".to_string());
            let session_key = self.open_session_picker.as_ref()
                .map(|kb| kb.display()).unwrap_or_else(|| "Ctrl+S".to_string());
            let model_key = self.open_model_picker.as_ref()
                .map(|kb| kb.display()).unwrap_or_else(|| "Ctrl+P".to_string());
            let retry_key = self.retry.as_ref()
                .map(|kb| kb.display()).unwrap_or_else(|| "Ctrl+R".to_string());
            vec![
                ("key", format!(" {}", submit_key)),
                ("label", " Send  ".to_string()),
                ("key", format!(" {}", quit_key)),
                ("label", " Quit  ".to_string()),
                ("key", format!("{}/{}", scroll_up_key, scroll_dn_key)),
                ("label", " Scroll  ".to_string()),
                ("key", format!(" {}", session_key)),
                ("label", " Sessions  ".to_string()),
                ("key", format!(" {}", model_key)),
                ("label", " Models  ".to_string()),
                ("key", format!(" {}", retry_key)),
                ("label", " Retry".to_string()),
            ]
        }
    }
}

impl KeyBinding {
    /// 显示文本。
    fn display(&self) -> String {
        let mods = self.modifiers.to_lowercase();
        if mods == "none" {
            capitalize_first(&self.key)
        } else {
            format!("{}+{}", capitalize_first(&mods), capitalize_first(&self.key))
        }
    }
}

fn capitalize_first(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

/// 解析键绑定为 crossterm 类型。
fn parse_binding(kb: &KeyBinding) -> (Option<KeyCode>, KeyModifiers) {
    let mods = parse_modifiers(&kb.modifiers);
    let code = parse_keycode(&kb.key);
    (code, mods)
}

fn parse_modifiers(s: &str) -> KeyModifiers {
    let mut mods = KeyModifiers::NONE;
    for part in s.to_lowercase().split('+') {
        match part.trim() {
            "ctrl" | "control" => mods.insert(KeyModifiers::CONTROL),
            "alt" => mods.insert(KeyModifiers::ALT),
            "shift" => mods.insert(KeyModifiers::SHIFT),
            "super" | "meta" => mods.insert(KeyModifiers::SUPER),
            _ => {}
        }
    }
    mods
}

fn parse_keycode(s: &str) -> Option<KeyCode> {
    match s.to_lowercase().as_str() {
        "enter" | "return" => Some(KeyCode::Enter),
        "esc" | "escape" => Some(KeyCode::Esc),
        "tab" => Some(KeyCode::Tab),
        "backspace" | "bs" => Some(KeyCode::Backspace),
        "delete" | "del" => Some(KeyCode::Delete),
        "home" => Some(KeyCode::Home),
        "end" => Some(KeyCode::End),
        "pageup" | "pgup" => Some(KeyCode::PageUp),
        "pagedown" | "pgdn" => Some(KeyCode::PageDown),
        "up" => Some(KeyCode::Up),
        "down" => Some(KeyCode::Down),
        "left" => Some(KeyCode::Left),
        "right" => Some(KeyCode::Right),
        "space" => Some(KeyCode::Char(' ')),
        // 单字符
        c if c.len() == 1 => Some(KeyCode::Char(c.chars().next().unwrap())),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_bindings_match() {
        let kb = KeyBindings::defaults();
        let key = KeyEvent::new(KeyCode::Char('o'), KeyModifiers::CONTROL);
        assert_eq!(kb.match_key(&key), Action::Submit);

        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        assert_eq!(kb.match_key(&key), Action::Cancel);

        let key = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert_eq!(kb.match_key(&key), Action::Quit);
    }

    #[test]
    fn custom_binding_overrides_default() {
        let json = r#"{"submit": {"modifiers": "alt", "key": "enter"}}"#;
        let overrides: serde_json::Value = serde_json::from_str(json).unwrap();
        let mut kb = KeyBindings::defaults();
        if let Some(v) = overrides.get("submit") {
            if let Ok(binding) = serde_json::from_value::<KeyBinding>(v.clone()) {
                kb.submit = Some(binding);
            }
        }

        // Alt+Enter 应匹配 Submit
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::ALT);
        assert_eq!(kb.match_key(&key), Action::Submit);

        // Ctrl+O 不再匹配（被覆盖）
        let key = KeyEvent::new(KeyCode::Char('o'), KeyModifiers::CONTROL);
        assert_eq!(kb.match_key(&key), Action::None);
    }

    #[test]
    fn parse_keycode_variants() {
        assert_eq!(parse_keycode("enter"), Some(KeyCode::Enter));
        assert_eq!(parse_keycode("Esc"), Some(KeyCode::Esc));
        assert_eq!(parse_keycode("pgup"), Some(KeyCode::PageUp));
        assert_eq!(parse_keycode("a"), Some(KeyCode::Char('a')));
        assert_eq!(parse_keycode("space"), Some(KeyCode::Char(' ')));
    }

    #[test]
    fn model_picker_binding() {
        let kb = KeyBindings::defaults();
        let key = KeyEvent::new(KeyCode::Char('p'), KeyModifiers::CONTROL);
        assert_eq!(kb.match_key(&key), Action::OpenModelPicker);
    }

    #[test]
    fn footer_hints_format() {
        let kb = KeyBindings::defaults();
        let idle_hints = kb.footer_hints(false);
        assert!(idle_hints.iter().any(|(t, s)| *t == "key" && s.contains("Ctrl+O")));
        let running_hints = kb.footer_hints(true);
        assert!(running_hints.iter().any(|(t, s)| *t == "key" && s.contains("Esc")));
    }
}
