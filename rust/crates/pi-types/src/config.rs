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
pub struct Settings {
    pub model: Option<String>,
    pub provider: Option<String>,
    pub thinking: Option<String>,
    pub compaction: CompactionSettings,
    pub terminal: TerminalSettings,
    pub thinking_budgets: Option<ThinkingBudgets>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            model: None,
            provider: None,
            thinking: None,
            compaction: CompactionSettings::default(),
            terminal: TerminalSettings::default(),
            thinking_budgets: None,
        }
    }
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
