//! Tool Provider API — 扩展注册的自定义工具适配为 ToolExecutor。
//!
//! 扩展通过闭包注册工具，通过 ExtensionTool 适配到统一的
//! ToolExecutor trait，可被 ToolRegistry 发现和调用。

use std::sync::Arc;

use pi_types::error::PiError;
use pi_types::tool::{ToolDefinition, ToolExecutor, ToolResult};

/// 扩展工具 — 从闭包适配到 ToolExecutor。
#[allow(clippy::type_complexity)]
pub struct ExtensionTool {
    def: ToolDefinition,
    handler: Arc<dyn Fn(&str) -> Result<String, String> + Send + Sync>,
}

impl ExtensionTool {
    /// 从名称、描述和处理器创建扩展工具。
    pub fn new(
        name: &str,
        description: &str,
        handler: impl Fn(&str) -> Result<String, String> + Send + Sync + 'static,
    ) -> Self {
        Self {
            def: ToolDefinition {
                name: name.to_string(),
                description: description.to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "input": { "type": "string", "description": "JSON input" }
                    },
                    "required": []
                }),
                requires_approval: false,
            },
            handler: Arc::new(handler),
        }
    }
}

#[async_trait::async_trait]
impl ToolExecutor for ExtensionTool {
    fn definition(&self) -> ToolDefinition {
        self.def.clone()
    }

    async fn execute(&self, input: serde_json::Value) -> Result<ToolResult, PiError> {
        let input_str = input.to_string();
        match (self.handler)(&input_str) {
            Ok(output) => Ok(ToolResult {
                tool_use_id: String::new(),
                output,
                is_error: false,
                duration_ms: None,
            }),
            Err(e) => Ok(ToolResult {
                tool_use_id: String::new(),
                output: e,
                is_error: true,
                duration_ms: None,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn extension_tool_execute() {
        let tool = ExtensionTool::new("echo", "Echo back the input", |input| {
            Ok(format!("echo: {}", input))
        });

        assert_eq!(tool.definition().name, "echo");

        let result = tool.execute(serde_json::json!("hello")).await.unwrap();
        assert!(!result.is_error);
        assert!(result.output.contains("hello"));
    }

    #[tokio::test]
    async fn extension_tool_error() {
        let tool = ExtensionTool::new("fail", "Always fails", |_| {
            Err("intentional error".to_string())
        });

        let result = tool.execute(serde_json::json!({})).await.unwrap();
        assert!(result.is_error);
        assert_eq!(result.output, "intentional error");
    }

    #[test]
    fn definition_has_schema() {
        let tool = ExtensionTool::new("test", "desc", |_| Ok("ok".into()));
        let def = tool.definition();
        assert_eq!(def.name, "test");
        assert!(def.parameters.is_object());
    }
}
