//! # pi-llm
//!
//! LLM provider 抽象层 — 流式响应、工具调用、多 provider 路由。

pub mod driver;
pub mod stream;
pub mod transform;
pub mod registry;
pub mod providers;
pub mod openai;
pub mod gemini;
pub mod azure;
pub mod bedrock;
pub mod openai_responses;
pub mod vertex;

pub use driver::{CompletionRequest, LlmDriver, StreamEvent};
pub use registry::ProviderRegistry;
