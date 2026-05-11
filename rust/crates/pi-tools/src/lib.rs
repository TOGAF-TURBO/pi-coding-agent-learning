//! # pi-tools
//!
//! 内置工具实现 — bash, read, write, edit, find, grep。

pub mod bash;
pub mod read;
pub mod write;
pub mod edit;
pub mod find;
pub mod grep;
pub mod registry;
pub mod truncate;

pub use registry::ToolRegistry;
