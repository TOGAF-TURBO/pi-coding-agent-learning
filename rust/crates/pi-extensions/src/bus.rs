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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_runner_from_hooks() {
        let mut hooks = HookRegistry::default();
        hooks.on_agent_start.push(Box::new(|_| {}));
        let runner = create_runner(hooks);
        assert_eq!(runner.tool_names().len(), 0);
    }
}
