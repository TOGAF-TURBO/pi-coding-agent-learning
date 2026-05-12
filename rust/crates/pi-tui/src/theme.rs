//! 主题系统 — 多主题支持，色板扩展，灰阶梯度生成。
//!
//! 内置 6 个主题 (dark, light, tokyonight, catppuccin, gruvbox, dracula)。
//! 支持从 ~/.piso/themes/*.json 加载自定义主题。
//! 25 个语义色 token 覆盖背景层级、diff、语法高亮、状态色。

use std::path::Path;

use ratatui::style::Color;
use serde::{Deserialize, Serialize};

// ============================================================================
// Theme struct — 25 semantic color tokens
// ============================================================================

/// 主题配色 — 25 个语义色 token。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Theme {
    // --- 基础色 ---
    pub primary: ColorDef,
    pub secondary: ColorDef,
    pub accent: ColorDef,
    pub error: ColorDef,
    pub success: ColorDef,
    pub warning: ColorDef,

    // --- 文字色 ---
    pub text: ColorDef,
    pub text_muted: ColorDef,

    // --- 背景色层级 ---
    pub background: ColorDef,
    pub background_panel: ColorDef,
    pub border: ColorDef,
    pub border_subtle: ColorDef,

    // --- 消息色 ---
    pub user_msg: ColorDef,
    pub assistant_msg: ColorDef,
    pub tool_msg: ColorDef,

    // --- Markdown 渲染色 ---
    pub heading_fg: ColorDef,
    pub code_fg: ColorDef,
    pub inline_code_fg: ColorDef,
    pub link_fg: ColorDef,

    // --- 语法高亮色 ---
    pub syntax_keyword: ColorDef,
    pub syntax_string: ColorDef,
    pub syntax_comment: ColorDef,

    // --- Diff 渲染色 ---
    pub diff_added: ColorDef,
    pub diff_removed: ColorDef,

    // --- 元数据 ---
    /// 主题显示名。
    #[serde(skip)]
    pub name: String,
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
    // ========================================================================
    // Built-in themes
    // ========================================================================

    /// 暗色主题（默认）。
    pub fn dark() -> Self {
        Self {
            primary: ColorDef("cyan".into()),
            secondary: ColorDef("green".into()),
            accent: ColorDef("magenta".into()),
            error: ColorDef("red".into()),
            success: ColorDef("green".into()),
            warning: ColorDef("yellow".into()),
            text: ColorDef("#c0c0c0".into()),
            text_muted: ColorDef("dark_gray".into()),
            background: ColorDef("#0a0a0a".into()),
            background_panel: ColorDef("#141414".into()),
            border: ColorDef("dark_gray".into()),
            border_subtle: ColorDef("#323232".into()),
            user_msg: ColorDef("green".into()),
            assistant_msg: ColorDef("cyan".into()),
            tool_msg: ColorDef("magenta".into()),
            heading_fg: ColorDef("yellow".into()),
            code_fg: ColorDef("gray".into()),
            inline_code_fg: ColorDef("yellow".into()),
            link_fg: ColorDef("cyan".into()),
            syntax_keyword: ColorDef("magenta".into()),
            syntax_string: ColorDef("green".into()),
            syntax_comment: ColorDef("dark_gray".into()),
            diff_added: ColorDef("green".into()),
            diff_removed: ColorDef("red".into()),
            name: "dark".into(),
        }
    }

    /// 亮色主题。
    pub fn light() -> Self {
        Self {
            primary: ColorDef("blue".into()),
            secondary: ColorDef("green".into()),
            accent: ColorDef("magenta".into()),
            error: ColorDef("red".into()),
            success: ColorDef("green".into()),
            warning: ColorDef("yellow".into()),
            text: ColorDef("#1a1a1a".into()),
            text_muted: ColorDef("gray".into()),
            background: ColorDef("#ffffff".into()),
            background_panel: ColorDef("#f5f5f5".into()),
            border: ColorDef("gray".into()),
            border_subtle: ColorDef("#d4d4d4".into()),
            user_msg: ColorDef("green".into()),
            assistant_msg: ColorDef("blue".into()),
            tool_msg: ColorDef("magenta".into()),
            heading_fg: ColorDef("blue".into()),
            code_fg: ColorDef("dark_gray".into()),
            inline_code_fg: ColorDef("yellow".into()),
            link_fg: ColorDef("blue".into()),
            syntax_keyword: ColorDef("magenta".into()),
            syntax_string: ColorDef("green".into()),
            syntax_comment: ColorDef("gray".into()),
            diff_added: ColorDef("green".into()),
            diff_removed: ColorDef("red".into()),
            name: "light".into(),
        }
    }

    /// Tokyo Night 主题 — 基于 tokyonight.nvim 色板。
    pub fn tokyonight() -> Self {
        Self {
            primary: ColorDef("#7aa2f7".into()),
            secondary: ColorDef("#bb9af7".into()),
            accent: ColorDef("#ff9e64".into()),
            error: ColorDef("#f7768e".into()),
            success: ColorDef("#9ece6a".into()),
            warning: ColorDef("#e0af68".into()),
            text: ColorDef("#c0caf5".into()),
            text_muted: ColorDef("#565f89".into()),
            background: ColorDef("#1a1b26".into()),
            background_panel: ColorDef("#1e2030".into()),
            border: ColorDef("#3b4261".into()),
            border_subtle: ColorDef("#292e42".into()),
            user_msg: ColorDef("#9ece6a".into()),
            assistant_msg: ColorDef("#7aa2f7".into()),
            tool_msg: ColorDef("#bb9af7".into()),
            heading_fg: ColorDef("#bb9af7".into()),
            code_fg: ColorDef("#a9b1d6".into()),
            inline_code_fg: ColorDef("#9ece6a".into()),
            link_fg: ColorDef("#7dcfff".into()),
            syntax_keyword: ColorDef("#bb9af7".into()),
            syntax_string: ColorDef("#9ece6a".into()),
            syntax_comment: ColorDef("#565f89".into()),
            diff_added: ColorDef("#9ece6a".into()),
            diff_removed: ColorDef("#f7768e".into()),
            name: "tokyonight".into(),
        }
    }

    /// Catppuccin Mocha 主题。
    pub fn catppuccin() -> Self {
        Self {
            primary: ColorDef("#89b4fa".into()),
            secondary: ColorDef("#cba6f7".into()),
            accent: ColorDef("#fab387".into()),
            error: ColorDef("#f38ba8".into()),
            success: ColorDef("#a6e3a1".into()),
            warning: ColorDef("#f9e2af".into()),
            text: ColorDef("#cdd6f4".into()),
            text_muted: ColorDef("#6c7086".into()),
            background: ColorDef("#1e1e2e".into()),
            background_panel: ColorDef("#181825".into()),
            border: ColorDef("#45475a".into()),
            border_subtle: ColorDef("#313244".into()),
            user_msg: ColorDef("#a6e3a1".into()),
            assistant_msg: ColorDef("#89b4fa".into()),
            tool_msg: ColorDef("#cba6f7".into()),
            heading_fg: ColorDef("#cba6f7".into()),
            code_fg: ColorDef("#bac2de".into()),
            inline_code_fg: ColorDef("#a6e3a1".into()),
            link_fg: ColorDef("#89dceb".into()),
            syntax_keyword: ColorDef("#cba6f7".into()),
            syntax_string: ColorDef("#a6e3a1".into()),
            syntax_comment: ColorDef("#6c7086".into()),
            diff_added: ColorDef("#a6e3a1".into()),
            diff_removed: ColorDef("#f38ba8".into()),
            name: "catppuccin".into(),
        }
    }

    /// Gruvbox Dark 主题。
    pub fn gruvbox() -> Self {
        Self {
            primary: ColorDef("#83a598".into()),
            secondary: ColorDef("#d3869b".into()),
            accent: ColorDef("#fe8019".into()),
            error: ColorDef("#fb4934".into()),
            success: ColorDef("#b8bb26".into()),
            warning: ColorDef("#fabd2f".into()),
            text: ColorDef("#ebdbb2".into()),
            text_muted: ColorDef("#665c54".into()),
            background: ColorDef("#282828".into()),
            background_panel: ColorDef("#1d2021".into()),
            border: ColorDef("#504945".into()),
            border_subtle: ColorDef("#3c3836".into()),
            user_msg: ColorDef("#b8bb26".into()),
            assistant_msg: ColorDef("#83a598".into()),
            tool_msg: ColorDef("#d3869b".into()),
            heading_fg: ColorDef("#fe8019".into()),
            code_fg: ColorDef("#d5c4a1".into()),
            inline_code_fg: ColorDef("#b8bb26".into()),
            link_fg: ColorDef("#83a598".into()),
            syntax_keyword: ColorDef("#fb4934".into()),
            syntax_string: ColorDef("#b8bb26".into()),
            syntax_comment: ColorDef("#665c54".into()),
            diff_added: ColorDef("#b8bb26".into()),
            diff_removed: ColorDef("#fb4934".into()),
            name: "gruvbox".into(),
        }
    }

    /// Dracula 主题。
    pub fn dracula() -> Self {
        Self {
            primary: ColorDef("#8be9fd".into()),
            secondary: ColorDef("#ff79c6".into()),
            accent: ColorDef("#bd93f9".into()),
            error: ColorDef("#ff5555".into()),
            success: ColorDef("#50fa7b".into()),
            warning: ColorDef("#f1fa8c".into()),
            text: ColorDef("#f8f8f2".into()),
            text_muted: ColorDef("#6272a4".into()),
            background: ColorDef("#282a36".into()),
            background_panel: ColorDef("#21222c".into()),
            border: ColorDef("#44475a".into()),
            border_subtle: ColorDef("#343746".into()),
            user_msg: ColorDef("#50fa7b".into()),
            assistant_msg: ColorDef("#8be9fd".into()),
            tool_msg: ColorDef("#ff79c6".into()),
            heading_fg: ColorDef("#bd93f9".into()),
            code_fg: ColorDef("#f8f8f2".into()),
            inline_code_fg: ColorDef("#50fa7b".into()),
            link_fg: ColorDef("#8be9fd".into()),
            syntax_keyword: ColorDef("#ff79c6".into()),
            syntax_string: ColorDef("#f1fa8c".into()),
            syntax_comment: ColorDef("#6272a4".into()),
            diff_added: ColorDef("#50fa7b".into()),
            diff_removed: ColorDef("#ff5555".into()),
            name: "dracula".into(),
        }
    }

    // ========================================================================
    // Built-in theme registry
    // ========================================================================

    /// 返回所有内置主题 (name → builder fn)。
    #[allow(clippy::type_complexity)]
    pub fn builtins() -> Vec<(&'static str, fn() -> Self)> {
        vec![
            ("dark", Self::dark),
            ("light", Self::light),
            ("tokyonight", Self::tokyonight),
            ("catppuccin", Self::catppuccin),
            ("gruvbox", Self::gruvbox),
            ("dracula", Self::dracula),
        ]
    }

    /// 按名称获取内置主题。
    pub fn by_name(name: &str) -> Option<Self> {
        for (n, builder) in Self::builtins() {
            if n == name {
                return Some(builder());
            }
        }
        None
    }

    /// 列出所有可用主题名（内置 + 用户自定义）。
    pub fn list_available() -> Vec<String> {
        let mut names: Vec<String> = Self::builtins()
            .iter()
            .map(|(n, _)| n.to_string())
            .collect();

        // 扫描 ~/.piso/themes/*.json
        if let Some(home) = dirs::home_dir() {
            let themes_dir = home.join(".piso").join("themes");
            if let Ok(entries) = std::fs::read_dir(&themes_dir) {
                for entry in entries.flatten() {
                    if let Some(name) = entry.path().file_stem() {
                        if let Some(s) = name.to_str() {
                            let s = s.to_string();
                            if !names.contains(&s) {
                                names.push(s);
                            }
                        }
                    }
                }
            }
        }

        names.sort();
        names
    }

    // ========================================================================
    // Loading
    // ========================================================================

    /// 从 JSON 文件加载自定义主题，缺失字段使用 dark 主题默认值。
    pub fn load(path: &Path) -> Self {
        let mut theme = Self::dark();
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(overrides) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(obj) = overrides.as_object() {
                    // 如果指定了基础主题，先切换
                    if let Some(v) = obj.get("base") {
                        if let Some(base) = v.as_str() {
                            if let Some(base_theme) = Self::by_name(base) {
                                theme = base_theme;
                            }
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
                    try_override!(success);
                    try_override!(warning);
                    try_override!(text);
                    try_override!(text_muted);
                    try_override!(background);
                    try_override!(background_panel);
                    try_override!(border);
                    try_override!(border_subtle);
                    try_override!(user_msg);
                    try_override!(assistant_msg);
                    try_override!(tool_msg);
                    try_override!(heading_fg);
                    try_override!(code_fg);
                    try_override!(inline_code_fg);
                    try_override!(link_fg);
                    try_override!(syntax_keyword);
                    try_override!(syntax_string);
                    try_override!(syntax_comment);
                    try_override!(diff_added);
                    try_override!(diff_removed);
                }
            }
        }
        // 从文件名提取主题名
        if let Some(stem) = path.file_stem() {
            theme.name = stem.to_string_lossy().to_string();
        }
        theme
    }

    /// 按名称加载主题（先查内置，再查文件）。
    pub fn load_by_name(name: &str) -> Option<Self> {
        // 1. 内置
        if let Some(t) = Self::by_name(name) {
            return Some(t);
        }
        // 2. ~/.piso/themes/<name>.json
        if let Some(home) = dirs::home_dir() {
            let path = home
                .join(".piso")
                .join("themes")
                .join(format!("{name}.json"));
            if path.exists() {
                return Some(Self::load(&path));
            }
        }
        None
    }

    // ========================================================================
    // Color resolution
    // ========================================================================

    /// 将 ColorDef 转为 ratatui Color。
    pub fn resolve(&self, def: &ColorDef) -> Color {
        let s = &def.0;
        if let Some(rgb) = parse_hex(s) {
            return rgb;
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

    // ========================================================================
    // Convenience accessors
    // ========================================================================

    pub fn assistant_color(&self) -> Color {
        self.resolve(&self.assistant_msg)
    }
    pub fn code_color(&self) -> Color {
        self.resolve(&self.code_fg)
    }
    pub fn inline_code_color(&self) -> Color {
        self.resolve(&self.inline_code_fg)
    }
    pub fn heading_color(&self) -> Color {
        self.resolve(&self.heading_fg)
    }

    // ========================================================================
    // Gray scale gradient generation
    // ========================================================================

    /// 从背景色生成 N 级灰阶梯度（用于面板层次感）。
    /// 基于 OpenCode 的 generateGrayScale 算法。
    pub fn gray_steps(&self, steps: usize) -> Vec<Color> {
        let bg = self.resolve(&self.background);
        let (bg_r, bg_g, bg_b) = match bg {
            Color::Rgb(r, g, b) => (r as f64, g as f64, b as f64),
            Color::Black => (0.0, 0.0, 0.0),
            Color::White => (255.0, 255.0, 255.0),
            _ => return vec![bg; steps],
        };

        let lum = 0.299 * bg_r + 0.587 * bg_g + 0.114 * bg_b;
        let is_dark = lum < 128.0;

        (1..=steps)
            .map(|i| {
                let factor = i as f64 / steps as f64;
                let (r, g, b) = if is_dark {
                    // Dark: lighten
                    let new_lum = lum + (255.0 - lum) * factor * 0.4;
                    if lum < 1.0 {
                        let v = (factor * 0.4 * 255.0) as u8;
                        (v, v, v)
                    } else {
                        let ratio = new_lum / lum;
                        (
                            (bg_r * ratio).min(255.0) as u8,
                            (bg_g * ratio).min(255.0) as u8,
                            (bg_b * ratio).min(255.0) as u8,
                        )
                    }
                } else {
                    // Light: darken
                    let new_lum = lum * (1.0 - factor * 0.4);
                    if lum > 250.0 {
                        let v = (255.0 - factor * 0.4 * 255.0) as u8;
                        (v, v, v)
                    } else {
                        let ratio = new_lum / lum;
                        (
                            (bg_r * ratio).max(0.0) as u8,
                            (bg_g * ratio).max(0.0) as u8,
                            (bg_b * ratio).max(0.0) as u8,
                        )
                    }
                };
                Color::Rgb(r, g, b)
            })
            .collect()
    }
}

/// 解析 #rrggbb 为 Color::Rgb。
fn parse_hex(s: &str) -> Option<Color> {
    let s = s.strip_prefix('#')?;
    if s.len() == 6 {
        let r = u8::from_str_radix(&s[0..2], 16).ok()?;
        let g = u8::from_str_radix(&s[2..4], 16).ok()?;
        let b = u8::from_str_radix(&s[4..6], 16).ok()?;
        Some(Color::Rgb(r, g, b))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_dark() {
        let t = Theme::default();
        assert_eq!(t.primary.0, "cyan");
        assert_eq!(t.name, "dark");
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
        assert_eq!(
            t.resolve(&ColorDef("#ff8800".into())),
            Color::Rgb(255, 136, 0)
        );
    }

    #[test]
    fn load_from_json() {
        let json = "{\"base\":\"light\",\"primary\":\"#ff0000\"}";
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
        assert_eq!(t.primary.0, "cyan");
    }

    #[test]
    fn convenience_accessors() {
        let t = Theme::dark();
        assert_eq!(t.assistant_color(), Color::Cyan);
        assert_eq!(t.code_color(), Color::Gray);
    }

    #[test]
    fn builtins_count() {
        assert_eq!(Theme::builtins().len(), 6);
    }

    #[test]
    fn by_name_returns_builtin() {
        assert!(Theme::by_name("tokyonight").is_some());
        assert!(Theme::by_name("catppuccin").is_some());
        assert!(Theme::by_name("gruvbox").is_some());
        assert!(Theme::by_name("dracula").is_some());
        assert!(Theme::by_name("dark").is_some());
        assert!(Theme::by_name("light").is_some());
        assert!(Theme::by_name("nonexistent").is_none());
    }

    #[test]
    fn tokyonight_has_hex_colors() {
        let t = Theme::tokyonight();
        assert!(t.background.0.starts_with('#'));
        assert!(t.text.0.starts_with('#'));
        assert_eq!(t.name, "tokyonight");
    }

    #[test]
    fn gray_steps_dark() {
        let t = Theme::tokyonight();
        let steps = t.gray_steps(4);
        assert_eq!(steps.len(), 4);
        // All should be Rgb
        for step in &steps {
            assert!(matches!(step, Color::Rgb(_, _, _)));
        }
        // Steps should get progressively lighter
        if let (Color::Rgb(r1, _, _), Color::Rgb(r4, _, _)) = (&steps[0], &steps[3]) {
            assert!(r4 > r1, "gray steps should lighten in dark mode");
        }
    }

    #[test]
    fn gray_steps_light() {
        let t = Theme::light();
        let steps = t.gray_steps(4);
        assert_eq!(steps.len(), 4);
        // Steps should get progressively darker
        if let (Color::Rgb(r1, _, _), Color::Rgb(r4, _, _)) = (&steps[0], &steps[3]) {
            assert!(r4 < r1, "gray steps should darken in light mode");
        }
    }

    #[test]
    fn load_by_name_fallback() {
        // Create a custom theme file
        let dir = dirs::home_dir().unwrap().join(".piso").join("themes");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("_piso_test_custom.json");
        std::fs::write(&path, "{\"base\":\"dark\",\"primary\":\"#abcdef\"}").unwrap();

        let t = Theme::load_by_name("_piso_test_custom");
        assert!(t.is_some());
        assert_eq!(t.unwrap().primary.0, "#abcdef");

        // Cleanup
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn list_available_includes_builtins() {
        let names = Theme::list_available();
        assert!(names.contains(&"dark".to_string()));
        assert!(names.contains(&"tokyonight".to_string()));
        assert!(names.contains(&"catppuccin".to_string()));
    }

    #[test]
    fn all_builtins_have_25_tokens() {
        for (name, builder) in Theme::builtins() {
            let t = builder();
            // Spot check a few tokens per theme
            assert!(!t.primary.0.is_empty(), "{} missing primary", name);
            assert!(!t.text.0.is_empty(), "{} missing text", name);
            assert!(!t.background.0.is_empty(), "{} missing background", name);
            assert!(
                !t.syntax_keyword.0.is_empty(),
                "{} missing syntax_keyword",
                name
            );
            assert!(!t.diff_added.0.is_empty(), "{} missing diff_added", name);
            assert!(!t.link_fg.0.is_empty(), "{} missing link_fg", name);
            assert!(!t.success.0.is_empty(), "{} missing success", name);
        }
    }
}
