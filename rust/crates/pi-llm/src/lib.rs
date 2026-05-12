//! # pi-llm
//!
//! LLM provider 抽象层 — 流式响应、工具调用、多 provider 路由。

pub mod azure;
pub mod bedrock;
pub mod cloudflare;
pub mod driver;
pub mod gemini;
pub mod openai;
pub mod openai_responses;
pub mod providers;
pub mod registry;
pub mod stream;
pub mod transform;
pub mod vertex;

pub use driver::{CompletionRequest, LlmDriver, StreamEvent};
pub use registry::ProviderRegistry;
