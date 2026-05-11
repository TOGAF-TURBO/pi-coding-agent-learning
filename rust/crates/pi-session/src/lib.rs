//! # pi-session
//!
//! 会话管理 — JSONL 持久化、分支、压缩。

pub mod branching;
pub mod compaction;
pub mod jsonl;
pub mod manager;

pub use jsonl::JsonlSession;
