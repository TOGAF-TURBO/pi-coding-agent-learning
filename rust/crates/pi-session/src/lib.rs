//! # pi-session
//!
//! 会话管理 — JSONL 持久化、分支、压缩。

pub mod jsonl;
pub mod manager;
pub mod compaction;
pub mod branching;

pub use jsonl::JsonlSession;
