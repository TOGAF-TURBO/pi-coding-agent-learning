//! Find 工具 — 查找文件和目录。
//!
//! 对应 `packages/coding-agent/src/core/tools/find.ts`。

use async_trait::async_trait;
use serde_json::Value;

use pi_types::error::PiError;
use pi_types::tool::{ToolDefinition, ToolExecutor, ToolResult};

use crate::truncate::truncate_output;

/// Find 工具执行器。
pub struct FindTool {
    cwd: String,
}

impl FindTool {
    pub fn new(cwd: impl Into<String>) -> Self {
        Self { cwd: cwd.into() }
    }
}

#[async_trait]
impl ToolExecutor for FindTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "find".to_string(),
            description: "Find files and directories matching a pattern. Supports glob patterns.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "pattern": {
                        "type": "string",
                        "description": "Glob pattern to match (e.g. \"**/*.rs\", \"src/**/*.ts\")"
                    },
                    "path": {
                        "type": "string",
                        "description": "Base directory to search in (default: current working directory)"
                    },
                    "max_depth": {
                        "type": "number",
                        "description": "Maximum directory depth to search"
                    }
                },
                "required": ["pattern"]
            }),
            requires_approval: false,
        }
    }

    async fn execute(&self, input: Value) -> Result<ToolResult, PiError> {
        let pattern = input["pattern"]
            .as_str()
            .ok_or_else(|| PiError::Tool {
                tool: "find".to_string(),
                message: "missing 'pattern' parameter".to_string(),
            })?;

        let base_path = input["path"]
            .as_str()
            .unwrap_or(&self.cwd);

        let max_depth = input["max_depth"].as_u64().map(|d| d as usize);

        let mut builder = globwalk::GlobWalkerBuilder::new(base_path, pattern);
        if let Some(depth) = max_depth {
            builder = builder.max_depth(depth);
        }

        let walker = builder.build().map_err(|e| PiError::Tool {
            tool: "find".to_string(),
            message: format!("invalid pattern '{}': {e}", pattern),
        })?;

        let mut results = Vec::new();
        for entry in walker.filter_map(|e| e.ok()) {
            let path = entry.path();
            // 跳过隐藏和非项目目录
            let components: Vec<_> = path.components().collect();
            let skip = components.iter().any(|c| {
                let s = c.as_os_str().to_string_lossy();
                s.starts_with('.') || s == "node_modules" || s == "target" || s == "dist"
            });
            if skip {
                continue;
            }
            let path_str = path.display().to_string();
            let relative = path_str.strip_prefix(base_path)
                .unwrap_or(&path_str)
                .trim_start_matches('/');
            results.push(relative.to_string());
        }

        let output = if results.is_empty() {
            format!("No files matching '{}' found in {}", pattern, base_path)
        } else {
            results.sort();
            results.join("\n")
        };

        let truncated = truncate_output(&output);
        let mut result = truncated.output;
        if truncated.truncated {
            result.push_str(&format!(
                "\n[showing partial results, {} total matches]",
                results.len()
            ));
        }

        Ok(ToolResult {
            tool_use_id: String::new(),
            output: result,
            is_error: false,
            duration_ms: None,
        })
    }
}
