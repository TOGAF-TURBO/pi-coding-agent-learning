//! 模型类型 — LLM 模型的元数据和注册表。
//!
//! 对应 `packages/ai/src/models.generated.ts` 和 `packages/coding-agent/src/core/model-registry.ts`。

use serde::{Deserialize, Serialize};

/// 模型能力描述。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub api: String,
    #[serde(default)]
    pub reasoning: bool,
    #[serde(default)]
    pub context_window: u32,
    #[serde(default)]
    pub max_tokens: u32,
    pub cost: Option<ModelCost>,
}

/// 模型的 token 计费信息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCost {
    pub input: f64,
    pub output: f64,
    #[serde(default)]
    pub cache_write: Option<f64>,
    #[serde(default)]
    pub cache_read: Option<f64>,
}

/// 模型注册表条目（含运行时状态）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelEntry {
    pub info: ModelInfo,
    /// 是否有可用的 API key。
    pub available: bool,
    /// 支持的思考级别。
    pub thinking_levels: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_info_serialization() {
        let info = ModelInfo {
            id: "claude-sonnet-4".to_string(),
            name: "Claude Sonnet 4".to_string(),
            provider: "anthropic".to_string(),
            api: "anthropic-messages".to_string(),
            reasoning: true,
            context_window: 200_000,
            max_tokens: 16_384,
            cost: None,
        };
        let json = serde_json::to_string(&info).unwrap();
        let parsed: ModelInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, info.id);
        assert_eq!(parsed.reasoning, true);
        assert_eq!(parsed.context_window, 200_000);
    }

    #[test]
    fn model_cost_serialization() {
        let cost = ModelCost {
            input: 3.0,
            output: 15.0,
            cache_write: Some(3.75),
            cache_read: Some(0.3),
        };
        let json = serde_json::to_string(&cost).unwrap();
        let parsed: ModelCost = serde_json::from_str(&json).unwrap();
        assert!((parsed.input - 3.0).abs() < 0.01);
    }
}
