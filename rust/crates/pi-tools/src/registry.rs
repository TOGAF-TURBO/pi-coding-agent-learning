//! 工具注册表 — 管理所有已注册的工具。
//!
//! 对应 `packages/coding-agent/src/core/tools/index.ts`。

use std::collections::HashMap;
use std::sync::Arc;

use parking_lot::RwLock;
use pi_types::tool::{ToolDefinition, ToolExecutor};

/// 工具注册表。
pub struct ToolRegistry {
    tools: RwLock<HashMap<String, Arc<dyn ToolExecutor>>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: RwLock::new(HashMap::new()),
        }
    }

    /// 注册一个工具。
    pub fn register(&self, tool: impl ToolExecutor + 'static) {
        let def = tool.definition();
        let name = def.name.clone();
        self.tools.write().insert(name, Arc::new(tool));
    }

    /// 按 name 查找工具。
    pub fn get(&self, name: &str) -> Option<Arc<dyn ToolExecutor>> {
        self.tools.read().get(name).cloned()
    }

    /// 获取工具定义（用于参数校验）。
    pub fn get_definition(&self, name: &str) -> Option<ToolDefinition> {
        self.tools.read().get(name).map(|t| t.definition())
    }

    /// 获取所有工具定义（用于发送给 LLM）。
    pub fn definitions(&self) -> Vec<ToolDefinition> {
        self.tools.read().values().map(|t| t.definition()).collect()
    }

    /// 列出所有已注册工具名称。
    pub fn names(&self) -> Vec<String> {
        self.tools.read().keys().cloned().collect()
    }

    /// 保留指定名称的工具（白名单过滤）。
    pub fn retain<F>(&self, f: F)
    where
        F: Fn(&str) -> bool,
    {
        self.tools.write().retain(|name, _| f(name));
    }

    /// Clone registry for a new agent instance.
    /// Since tools are Arc<dyn ToolExecutor>, cloning is cheap.
    pub fn clone_for_agent(&self) -> Self {
        let tools = self.tools.read();
        Self {
            tools: parking_lot::RwLock::new(tools.clone()),
        }
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use pi_types::error::PiError;
    use pi_types::tool::{ToolExecutor, ToolResult};
    use serde_json::Value;

    struct DummyTool;

    #[async_trait]
    impl ToolExecutor for DummyTool {
        fn definition(&self) -> ToolDefinition {
            ToolDefinition {
                name: "dummy".to_string(),
                description: "A test tool".to_string(),
                parameters: serde_json::json!({}),
                requires_approval: false,
            execution_mode: Default::default(),
            }
        }

        async fn execute(&self, _input: Value) -> Result<ToolResult, PiError> {
            Ok(ToolResult {
                tool_use_id: String::new(),
                output: "dummy result".to_string(),
                is_error: false,
                duration_ms: None,
            terminate: false,
            })
        }
    }

    #[test]
    fn register_and_get() {
        let reg = ToolRegistry::new();
        reg.register(DummyTool);

        assert!(reg.get("dummy").is_some());
        assert!(reg.get("nonexistent").is_none());
    }

    #[test]
    fn list_names() {
        let reg = ToolRegistry::new();
        reg.register(DummyTool);

        let names = reg.names();
        assert!(names.contains(&"dummy".to_string()));
    }

    #[test]
    fn definitions() {
        let reg = ToolRegistry::new();
        reg.register(DummyTool);

        let defs = reg.definitions();
        assert_eq!(defs.len(), 1);
        assert_eq!(defs[0].name, "dummy");
    }

    #[test]
    fn retain_whitelist() {
        let reg = ToolRegistry::new();
        reg.register(DummyTool);

        // Retain only nonexistent — removes all
        reg.retain(|name| name == "nonexistent");
        assert!(reg.get("dummy").is_none());
        assert!(reg.names().is_empty());
    }

    #[test]
    fn retain_keeps_matching() {
        let reg = ToolRegistry::new();
        reg.register(DummyTool);

        reg.retain(|name| name == "dummy");
        assert!(reg.get("dummy").is_some());
    }

    #[test]
    fn clone_for_agent() {
        let reg = ToolRegistry::new();
        reg.register(DummyTool);

        let clone = reg.clone_for_agent();
        assert!(clone.get("dummy").is_some());
        // Both independent
        assert!(reg.get("dummy").is_some());
    }
}
