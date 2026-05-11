//! 配置类型 — 设置、keybinding、主题。
//!
//! 对应 `packages/coding-agent/src/core/settings-manager.ts`。

use serde::{Deserialize, Serialize};

/// 思考级别映射到模型的内部参数。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingBudgets {
    pub minimal: Option<String>,
    pub low: Option<String>,
    pub medium: Option<String>,
    pub high: Option<String>,
    pub xhigh: Option<String>,
}

/// 压缩策略设置。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionSettings {
    /// 是否启用自动压缩。
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// 触发压缩的消息数阈值。
    #[serde(default = "default_compaction_threshold")]
    pub threshold: u32,
}

fn default_true() -> bool {
    true
}

fn default_compaction_threshold() -> u32 {
    100
}

/// 终端设置。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalSettings {
    #[serde(default)]
    pub mouse_capture: bool,
    #[serde(default = "default_true")]
    pub bracketed_paste: bool,
}

/// 全局设置。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
#[derive(Default)]
pub struct Settings {
    pub model: Option<String>,
    pub provider: Option<String>,
    pub thinking: Option<String>,
    pub compaction: CompactionSettings,
    pub terminal: TerminalSettings,
    pub thinking_budgets: Option<ThinkingBudgets>,
}


impl Default for CompactionSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            threshold: default_compaction_threshold(),
        }
    }
}

impl Default for TerminalSettings {
    fn default() -> Self {
        Self {
            mouse_capture: false,
            bracketed_paste: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_settings() {
        let s = Settings::default();
        assert!(s.model.is_none());
        assert!(s.provider.is_none());
        assert!(s.compaction.enabled);
    }

    #[test]
    fn settings_serialization_roundtrip() {
        let s = Settings {
            model: Some("claude-sonnet-4".to_string()),
            provider: Some("anthropic".to_string()),
            thinking: Some("high".to_string()),
            ..Settings::default()
        };
        let json = serde_json::to_string(&s).unwrap();
        let parsed: Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.model, s.model);
        assert_eq!(parsed.provider, s.provider);
    }

    #[test]
    fn compaction_defaults() {
        let c = CompactionSettings::default();
        assert!(c.enabled);
        assert_eq!(c.threshold, 100);
    }

    #[test]
    fn terminal_defaults() {
        let t = TerminalSettings::default();
        assert!(!t.mouse_capture);
        assert!(t.bracketed_paste);
    }
}
