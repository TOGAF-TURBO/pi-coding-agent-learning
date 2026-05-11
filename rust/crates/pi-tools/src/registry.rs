//! 工具注册表 — 管理所有已注册的工具。
//!
//! 对应 `packages/coding-agent/src/core/tools/index.ts`。

use std::collections::HashMap;
use std::sync::Arc;

use parking_lot::RwLock;
use pi_types::tool::{ToolDefinition, ToolExecutor, ToolResult};

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

    /// 获取所有工具定义（用于发送给 LLM）。
    pub fn definitions(&self) -> Vec<ToolDefinition> {
        self.tools
            .read()
            .values()
            .map(|t| t.definition())
            .collect()
    }

    /// 列出所有已注册工具名称。
    pub fn names(&self) -> Vec<String> {
        self.tools.read().keys().cloned().collect()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
