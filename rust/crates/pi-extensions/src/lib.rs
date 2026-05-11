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
//! 这是 pi 哲学的核心体现：
//! - 一切皆可扩展（工具、命令、快捷键、渲染器、编辑器）
//! - 扩展通过事件驱动，非侵入式
//! - 扩展可以是 JS/WASM（未来），当前先支持 Rust 原生扩展
//!
//! 架构决策：
//! - `ExtensionApi` trait 对应 TS 的 `ExtensionAPI` interface
//! - `EventBus` 使用 tokio broadcast channel
//! - 扩展注册是声明式的（类似 TS 的 `export default (pi) => { ... }`）

pub mod api;
pub mod bus;
pub mod loader;
pub mod runner;
