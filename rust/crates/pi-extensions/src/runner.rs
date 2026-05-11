//! 扩展运行器 — 在 agent 生命周期中调用已注册的钩子。
//!
//! 运行器持有 HookRegistry，在合适的时机分发事件。

use crate::api::HookRegistry;

/// 扩展运行时 — 调用已注册的钩子处理器。
pub struct ExtensionRunner {
    hooks: HookRegistry,
}

impl ExtensionRunner {
    pub fn new(hooks: HookRegistry) -> Self {
        Self { hooks }
    }

    /// 创建空运行时（无扩展）。
    pub fn empty() -> Self {
        Self {
            hooks: HookRegistry::default(),
        }
    }

    /// 通知：agent 开始处理。
    pub fn fire_agent_start(&self, prompt: &str) {
        for handler in &self.hooks.on_agent_start {
            handler(prompt);
        }
    }

    /// 通知：agent 完成。
    pub fn fire_agent_done(&self, output: &str) {
        for handler in &self.hooks.on_agent_done {
            handler(output);
        }
    }

    /// 通知：工具调用开始。
    pub fn fire_tool_call_start(&self, name: &str, call_id: &str) {
        for handler in &self.hooks.on_tool_call_start {
            handler(name, call_id);
        }
    }

    /// 通知：工具调用完成。
    pub fn fire_tool_call_end(&self, name: &str, output: &str, is_error: bool) {
        for handler in &self.hooks.on_tool_call_end {
            handler(name, output, is_error);
        }
    }

    /// 通知：模型切换。
    pub fn fire_model_switched(&self, provider: &str, model: &str) {
        for handler in &self.hooks.on_model_switched {
            handler(provider, model);
        }
    }

    /// 查找已注册的自定义工具。
    pub fn find_tool(&self, name: &str) -> Option<&crate::api::ToolEntry> {
        self.hooks.tools.iter().find(|t| t.name == name)
    }

    /// 查找已注册的命令。
    pub fn find_command(&self, name: &str) -> Option<&crate::api::CommandEntry> {
        self.hooks.commands.iter().find(|c| c.name == name)
    }

    /// 获取所有已注册的工具名。
    pub fn tool_names(&self) -> Vec<&str> {
        self.hooks.tools.iter().map(|t| t.name.as_str()).collect()
    }

    /// 获取所有已注册的命令名。
    pub fn command_names(&self) -> Vec<&str> {
        self.hooks.commands.iter().map(|c| c.name.as_str()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::ToolEntry;
    use std::sync::{Arc, Mutex};

    #[test]
    fn fire_lifecycle_hooks() {
        let log = Arc::new(Mutex::new(Vec::new()));

        let mut hooks = HookRegistry::default();

        let log_start = log.clone();
        hooks.on_agent_start.push(Box::new(move |prompt| {
            log_start.lock().unwrap().push(format!("start:{}", prompt));
        }));

        let log_done = log.clone();
        hooks.on_agent_done.push(Box::new(move |output| {
            log_done.lock().unwrap().push(format!("done:{}", output));
        }));

        let runner = ExtensionRunner::new(hooks);
        runner.fire_agent_start("hello");
        runner.fire_agent_done("world");

        let entries = log.lock().unwrap();
        assert_eq!(&*entries, &vec!["start:hello", "done:world"]);
    }

    #[test]
    fn find_tool_by_name() {
        let mut hooks = HookRegistry::default();
        hooks.tools.push(ToolEntry {
            name: "grep".into(),
            description: "Search".into(),
            handler: Box::new(|_| Ok("found".into())),
        });

        let runner = ExtensionRunner::new(hooks);
        assert!(runner.find_tool("grep").is_some());
        assert!(runner.find_tool("missing").is_none());
        assert_eq!(runner.tool_names(), vec!["grep"]);
    }
}
