//! 主题系统 — 从 JSON 加载和切换主题。
//!
//! 内置 dark/light 主题，支持 ~/.piso/theme.json 自定义。

use std::path::Path;

use ratatui::style::Color;
use serde::{Deserialize, Serialize};

/// 主题配色。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Theme {
    pub primary: ColorDef,
    pub secondary: ColorDef,
    pub accent: ColorDef,
    pub error: ColorDef,
    pub muted: ColorDef,
    pub user_msg: ColorDef,
    pub assistant_msg: ColorDef,
    pub tool_msg: ColorDef,
    pub border: ColorDef,
    pub header_bg: ColorDef,
    /// 代码块前景色。
    pub code_fg: ColorDef,
    /// 行内代码前景色。
    pub inline_code_fg: ColorDef,
    /// 标题颜色（# 标题）。
    pub heading_fg: ColorDef,
}

/// 可序列化的颜色定义 — 颜色名或 #rrggbb。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ColorDef(pub String);

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

impl Theme {
    /// 暗色主题（默认）。
    pub fn dark() -> Self {
        Self {
            primary: ColorDef("cyan".into()),
            secondary: ColorDef("green".into()),
            accent: ColorDef("magenta".into()),
            error: ColorDef("red".into()),
            muted: ColorDef("dark_gray".into()),
            user_msg: ColorDef("green".into()),
            assistant_msg: ColorDef("cyan".into()),
            tool_msg: ColorDef("magenta".into()),
            border: ColorDef("dark_gray".into()),
            header_bg: ColorDef("black".into()),
            code_fg: ColorDef("gray".into()),
            inline_code_fg: ColorDef("yellow".into()),
            heading_fg: ColorDef("yellow".into()),
        }
    }

    /// 亮色主题。
    pub fn light() -> Self {
        Self {
            primary: ColorDef("blue".into()),
            secondary: ColorDef("green".into()),
            accent: ColorDef("magenta".into()),
            error: ColorDef("red".into()),
            muted: ColorDef("gray".into()),
            user_msg: ColorDef("green".into()),
            assistant_msg: ColorDef("blue".into()),
            tool_msg: ColorDef("magenta".into()),
            border: ColorDef("gray".into()),
            header_bg: ColorDef("white".into()),
            code_fg: ColorDef("dark_gray".into()),
            inline_code_fg: ColorDef("yellow".into()),
            heading_fg: ColorDef("blue".into()),
        }
    }

    /// 从 JSON 文件加载，缺失字段使用默认值。
    pub fn load(path: &Path) -> Self {
        let mut theme = Self::dark();
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(overrides) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(obj) = overrides.as_object() {
                    if let Some(v) = obj.get("theme") {
                        if v.as_str() == Some("light") {
                            theme = Self::light();
                        }
                    }
                    // 逐字段覆盖
                    macro_rules! try_override {
                        ($field:ident) => {
                            if let Some(v) = obj.get(stringify!($field)) {
                                if let Ok(cd) = serde_json::from_value::<ColorDef>(v.clone()) {
                                    theme.$field = cd;
                                }
                            }
                        };
                    }
                    try_override!(primary);
                    try_override!(secondary);
                    try_override!(accent);
                    try_override!(error);
                    try_override!(muted);
                    try_override!(user_msg);
                    try_override!(assistant_msg);
                    try_override!(tool_msg);
                    try_override!(border);
                    try_override!(header_bg);
                    try_override!(code_fg);
                    try_override!(inline_code_fg);
                    try_override!(heading_fg);
                }
            }
        }
        theme
    }

    /// 将 ColorDef 转为 ratatui Color。
    pub fn resolve(&self, def: &ColorDef) -> Color {
        let s = &def.0;
        // 以 # 开头 = hex
        if s.starts_with('#') && s.len() == 7 {
            let r = u8::from_str_radix(&s[1..3], 16).unwrap_or(0);
            let g = u8::from_str_radix(&s[3..5], 16).unwrap_or(0);
            let b = u8::from_str_radix(&s[5..7], 16).unwrap_or(0);
            return Color::Rgb(r, g, b);
        }
        match s.to_lowercase().as_str() {
            "red" => Color::Red,
            "green" => Color::Green,
            "blue" => Color::Blue,
            "cyan" => Color::Cyan,
            "magenta" => Color::Magenta,
            "yellow" => Color::Yellow,
            "white" => Color::White,
            "black" => Color::Black,
            "dark_gray" | "darkgray" => Color::DarkGray,
            "gray" | "grey" => Color::Gray,
            "light_red" | "lightred" => Color::LightRed,
            "light_green" | "lightgreen" => Color::LightGreen,
            "light_blue" | "lightblue" => Color::LightBlue,
            "light_cyan" | "lightcyan" => Color::LightCyan,
            "light_magenta" | "lightmagenta" => Color::LightMagenta,
            "light_yellow" | "lightyellow" => Color::LightYellow,
            _ => Color::Reset,
        }
    }

    /// 获取 assistant 消息的 base style 颜色。
    pub fn assistant_color(&self) -> Color {
        self.resolve(&self.assistant_msg)
    }

    /// 获取代码块颜色。
    pub fn code_color(&self) -> Color {
        self.resolve(&self.code_fg)
    }

    /// 获取行内代码颜色。
    pub fn inline_code_color(&self) -> Color {
        self.resolve(&self.inline_code_fg)
    }

    /// 获取标题颜色。
    pub fn heading_color(&self) -> Color {
        self.resolve(&self.heading_fg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_dark() {
        let t = Theme::default();
        assert_eq!(t.primary.0, "cyan");
    }

    #[test]
    fn resolve_named_colors() {
        let t = Theme::dark();
        assert_eq!(t.resolve(&ColorDef("red".into())), Color::Red);
        assert_eq!(t.resolve(&ColorDef("cyan".into())), Color::Cyan);
        assert_eq!(t.resolve(&ColorDef("dark_gray".into())), Color::DarkGray);
    }

    #[test]
    fn resolve_hex_colors() {
        let t = Theme::dark();
        assert_eq!(t.resolve(&ColorDef("#ff8800".into())), Color::Rgb(255, 136, 0));
        // 6-char hex without # is NOT hex, it's a named color that resolves to Reset
    }

    #[test]
    fn load_from_json() {
        let json = "{\"theme\":\"light\",\"primary\":\"#ff0000\"}";
        let dir = std::env::temp_dir().join("piso_test_theme");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("theme.json");
        std::fs::write(&path, json).unwrap();

        let t = Theme::load(&path);
        assert_eq!(t.primary.0, "#ff0000");
        assert_eq!(t.border.0, "gray"); // light theme default
    }

    #[test]
    fn load_missing_file_uses_defaults() {
        let t = Theme::load(Path::new("/tmp/piso_nonexistent_theme_xyz.json"));
        assert_eq!(t.primary.0, "cyan"); // dark default
    }

    #[test]
    fn convenience_accessors() {
        let t = Theme::dark();
        assert_eq!(t.assistant_color(), Color::Cyan);
        assert_eq!(t.code_color(), Color::Gray);
        assert_eq!(t.inline_code_color(), Color::Yellow);
        assert_eq!(t.heading_color(), Color::Yellow);
    }
}
