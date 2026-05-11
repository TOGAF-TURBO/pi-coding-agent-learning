//! 事件总线桥接 — 连接 TUI EventBus 与扩展系统。
//!
//! 当前实现：将 EventBus event 转发到 HookRegistry 回调。
//! 未来可支持异步订阅。

use crate::api::HookRegistry;
use crate::runner::ExtensionRunner;

/// 将 ExtensionRunner 连接到事件流。
///
/// 此函数从 HookRegistry 创建 ExtensionRunner，
/// 供 agent loop 在合适时机调用。
pub fn create_runner(hooks: HookRegistry) -> ExtensionRunner {
    ExtensionRunner::new(hooks)
}
