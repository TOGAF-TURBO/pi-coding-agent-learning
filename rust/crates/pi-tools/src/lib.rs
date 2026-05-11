//! # pi-tools
//!
//! 内置工具实现 — bash, read, write, edit, find, grep。

pub mod bash;
pub mod diff;
pub mod edit;
pub mod extension_tool;
pub mod find;
pub mod grep;
pub mod lsp;
pub mod lsp_client;
pub mod fileref;
pub mod ls;
pub mod read;
pub mod registry;
pub mod truncate;
pub mod write;

pub use registry::ToolRegistry;
