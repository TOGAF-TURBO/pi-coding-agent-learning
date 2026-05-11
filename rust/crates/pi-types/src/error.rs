//! 统一错误类型。
//!
//! 设计原则（继承自 pi）：
//! - 使用 thiserror 派生，保留错误链
//! - 每个变体携带足够的上下文信息用于诊断
//! - 错误可序列化用于 RPC 传输

use thiserror::Error;

#[derive(Debug, Error)]
pub enum PiError {
    #[error("LLM provider error: {0}")]
    Llm(String),

    #[error("API key not found for provider: {0}")]
    MissingApiKey(String),

    #[error("Session error: {0}")]
    Session(String),

    #[error("Tool execution error: {tool} — {message}")]
    Tool { tool: String, message: String },

    #[error("Extension error: {0}")]
    Extension(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialize(#[from] serde_json::Error),

    #[error("{0}")]
    Other(String),
}
