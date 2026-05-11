//! # pi-agent
//!
//! Agent 循环 — 编排 LLM 调用、工具执行、会话状态。

pub mod compaction;
pub mod context;
pub mod loop_engine;
pub mod session;
pub mod skills;
pub mod system_prompt;
pub mod token_est;
pub mod runtime;
