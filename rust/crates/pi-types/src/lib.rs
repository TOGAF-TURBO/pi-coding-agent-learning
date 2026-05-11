//! # pi-types
//!
//! pi 所有 crate 共享的核心类型定义。
//!
//! 对应 TypeScript 源码：
//! - `packages/ai/src/types.ts` — 消息、工具、流事件
//! - `packages/agent/src/harness/types.ts` — 会话、技能、执行环境
//! - `packages/coding-agent/src/core/settings-manager.ts` — 设置
//!
//! 本 crate 零依赖外部 IO，只定义数据结构和 trait。

pub mod message;
pub mod tool;
pub mod event;
pub mod session;
pub mod config;
pub mod skill;
pub mod model;
pub mod error;
