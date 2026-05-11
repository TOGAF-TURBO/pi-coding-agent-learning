//! # pi-cli
//!
//! pi 命令行入口 — 参数解析、模式分发、启动流水线。
//!
//! 对应 TypeScript 源码：
//! - `packages/coding-agent/src/cli.ts` — 入口
//! - `packages/coding-agent/src/main.ts` — 主逻辑
//! - `packages/coding-agent/src/cli/args.ts` — 参数解析
//!
//! 启动流水线（与 TS 版本完全一致）：
//! ```text
//! parse args → load config → resolve auth → register models
//!     → create/restore session → load resources → dispatch mode
//! ```
//!
//! 模式分发：
//! - interactive: 接管终端，启动 TUI
//! - print: 一次性输出（管道友好）
//! - rpc: JSON-over-stdio 机器接口

pub mod args;
pub mod config;
pub mod auth;
pub mod dispatch;
