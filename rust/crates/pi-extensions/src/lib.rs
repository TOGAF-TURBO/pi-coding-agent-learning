//! # pi-extensions
//!
//! 扩展系统 — 生命周期钩子、工具注册、事件总线。
//!
//! 对应 TypeScript 源码：
//! - `packages/coding-agent/src/core/extensions/types.ts` — ExtensionAPI 全量接口
//! - `packages/coding-agent/src/core/extensions/loader.ts` — 扩展加载
//! - `packages/coding-agent/src/core/extensions/runner.ts` — 扩展执行
//! - `packages/coding-agent/src/core/event-bus.ts` — 事件总线
//!
//! 架构：
//! - `ExtensionApi` trait — 扩展可用的 API
//! - `ExtensionLoader` — 发现和注册扩展
//! - `ExtensionRunner` — 在 agent 生命周期中调用钩子
//! - `HookRegistry` — 所有已注册回调的存储
//!
//! 使用方式：
//! ```ignore
//! let mut loader = ExtensionLoader::new();
//! loader.register("my-ext", |api| {
//!     api.on_agent_start(Box::new(|prompt| {
//!         eprintln!("Agent starting: {}", prompt);
//!     }));
//!     api.register_tool("my_tool", "Does something", Box::new(|input| {
//!         Ok(format!("Result: {}", input))
//!     }));
//! });
//! let hooks = loader.load_all();
//! let runner = ExtensionRunner::new(hooks);
//!
//! // 在 agent loop 中：
//! runner.fire_agent_start("hello");
//! ```

#![allow(clippy::type_complexity)]

pub mod api;
pub mod bus;
pub mod loader;
pub mod runner;

pub use api::{BasicExtensionApi, ExtensionApi, ExtensionFactory, ExtensionStore, HookRegistry};
pub use loader::ExtensionLoader;
pub use runner::ExtensionRunner;
