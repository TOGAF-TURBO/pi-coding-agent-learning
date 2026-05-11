//! Write 工具 — 创建或覆盖文件。
//!
//! 对应 `packages/coding-agent/src/core/tools/write.ts`。

use async_trait::async_trait;
use serde_json::Value;
use tokio::fs;

use pi_types::error::PiError;
use pi_types::tool::{ToolDefinition, ToolExecutor, ToolResult};

/// Write 工具执行器。
pub struct WriteTool;

impl Default for WriteTool {
    fn default() -> Self {
        Self::new()
    }
}

impl WriteTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ToolExecutor for WriteTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "write".to_string(),
            description: "Write content to a file. Creates the file if it doesn't exist, overwrites if it does. Automatically creates parent directories.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the file to write (relative or absolute)"
                    },
                    "content": {
                        "type": "string",
                        "description": "Content to write to the file"
                    }
                },
                "required": ["path", "content"]
            }),
            requires_approval: false,
        }
    }

    async fn execute(&self, input: Value) -> Result<ToolResult, PiError> {
        let path = input["path"]
            .as_str()
            .ok_or_else(|| PiError::Tool {
                tool: "write".to_string(),
                message: "missing 'path' parameter".to_string(),
            })?;

        let content = input["content"]
            .as_str()
            .ok_or_else(|| PiError::Tool {
                tool: "write".to_string(),
                message: "missing 'content' parameter".to_string(),
            })?;

        // 读取旧内容（如果存在）用于 diff
        let old_content = fs::read_to_string(path).await.ok().unwrap_or_default();

        // 自动创建父目录
        if let Some(parent) = std::path::Path::new(path).parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).await.map_err(|e| PiError::Tool {
                    tool: "write".to_string(),
                    message: format!("failed to create parent directory: {e}"),
                })?;
            }
        }

        fs::write(path, content).await.map_err(|e| PiError::Tool {
            tool: "write".to_string(),
            message: format!("failed to write '{}': {e}", path),
        })?;

        let line_count = content.lines().count();
        let byte_count = content.len();

        // 生成 diff（仅对已有文件）
        let diff_display = if !old_content.is_empty() && old_content != content {
            let diff = crate::diff::format_diff(&old_content, content, path);
            if diff.is_empty() {
                String::new()
            } else {
                format!("\n```diff\n{}\n```", diff.trim())
            }
        } else if old_content.is_empty() {
            " (new file)".to_string()
        } else {
            String::new() // 内容未变
        };

        Ok(ToolResult {
            tool_use_id: String::new(),
            output: format!("Wrote {} bytes, {} lines to {}{}", byte_count, line_count, path, diff_display),
            is_error: false,
            duration_ms: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn write_creates_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("output.txt");

        let tool = WriteTool::new();
        let result = tool.execute(serde_json::json!({
            "path": path.to_string_lossy(),
            "content": "hello world"
        })).await.unwrap();

        assert!(!result.is_error);
        let content = tokio::fs::read_to_string(&path).await.unwrap();
        assert_eq!(content, "hello world");
    }

    #[tokio::test]
    async fn write_creates_parent_dirs() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("a").join("b").join("file.txt");

        let tool = WriteTool::new();
        let result = tool.execute(serde_json::json!({
            "path": path.to_string_lossy(),
            "content": "nested"
        })).await.unwrap();

        assert!(!result.is_error);
        let content = tokio::fs::read_to_string(&path).await.unwrap();
        assert_eq!(content, "nested");
    }
}
