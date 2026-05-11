//! # pi-tui
//!
//! 终端 UI 引擎 — 基于 ratatui 的布局、组件、渲染。
//!
//! 对应 TypeScript 源码：
//! - `packages/tui/src/tui.ts` — TUI 引擎核心
//! - `packages/tui/src/components/` — 所有 TUI 组件
//! - `packages/coding-agent/src/modes/interactive/` — 交互模式
//!
//! 架构决策：
//! - ratatui 而非自建渲染（pi 的 TS 版本自建了轻量 TUI 框架，
//!   但 Rust 生态中 ratatui 已是事实标准，没必要重复造轮子）
//! - 组件模型：实现 `Component` trait，对应 pi 的 `Component` interface
//! - 事件循环：crossterm 事件 → TUI 调度 → 组件处理
//! - 布局系统：保留 pi 的 header/chat/status/editor/footer 五区布局

pub mod engine;
pub mod layout;
pub mod components;
pub mod theme;
pub mod keybinding;
pub mod input;
