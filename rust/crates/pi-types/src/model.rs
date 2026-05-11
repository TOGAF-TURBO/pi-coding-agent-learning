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
