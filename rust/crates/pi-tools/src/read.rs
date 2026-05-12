//! Read 工具 — 读取文件内容。
//!
//! 对应 `packages/coding-agent/src/core/tools/read.ts`。

use async_trait::async_trait;
use serde_json::Value;
use tokio::fs;

use pi_types::error::PiError;
use pi_types::tool::{ExecutionMode, ToolDefinition, ToolExecutor, ToolResult};

use crate::truncate::truncate_output;

/// Read 工具执行器。
pub struct ReadTool;

impl Default for ReadTool {
    fn default() -> Self {
        Self::new()
    }
}

impl ReadTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ToolExecutor for ReadTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "read".to_string(),
            description: "Read the contents of a file. Returns the file content with line numbers."
                .to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the file to read (relative or absolute)"
                    },
                    "offset": {
                        "type": "number",
                        "description": "Line number to start reading from (1-indexed)"
                    },
                    "limit": {
                        "type": "number",
                        "description": "Maximum number of lines to read"
                    }
                },
                "required": ["path"]
            }),
            requires_approval: false,
            execution_mode: ExecutionMode::default(),
        }
    }

    async fn execute(&self, input: Value) -> Result<ToolResult, PiError> {
        let path = input["path"].as_str().ok_or_else(|| PiError::Tool {
            tool: "read".to_string(),
            message: "missing 'path' parameter".to_string(),
        })?;

        let offset = input["offset"].as_u64().unwrap_or(0) as usize;
        let limit = input["limit"].as_u64().map(|l| l as usize);

        let content = fs::read_to_string(path).await.map_err(|e| PiError::Tool {
            tool: "read".to_string(),
            message: format!("failed to read '{}': {e}", path),
        })?;

        let lines: Vec<&str> = content.lines().collect();
        let start = if offset > 0 { offset - 1 } else { 0 };
        let end = limit
            .map(|l| (start + l).min(lines.len()))
            .unwrap_or(lines.len());

        let selected: Vec<String> = lines[start..end]
            .iter()
            .enumerate()
            .map(|(i, line)| format!("{:>6}\t{}", start + i + 1, line))
            .collect();

        let output = selected.join("\n");
        let truncated = truncate_output(&output);

        let mut result = truncated.output;
        if truncated.truncated {
            result.push_str(&format!(
                "\n[showing {}-{} of {} lines]",
                start + 1,
                end,
                lines.len()
            ));
        }

        Ok(ToolResult {
            tool_use_id: String::new(),
            output: result,
            is_error: false,
            duration_ms: None,
            terminate: false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs;

    #[tokio::test]
    async fn read_file_with_line_numbers() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.txt");
        fs::write(&path, "line1\nline2\nline3\n").await.unwrap();

        let tool = ReadTool::new();
        let result = tool
            .execute(serde_json::json!({
                "path": path.to_string_lossy()
            }))
            .await
            .unwrap();

        assert!(!result.is_error);
        assert!(result.output.contains("line1"));
        assert!(result.output.contains("line3"));
    }

    #[tokio::test]
    async fn read_file_with_offset_and_limit() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.txt");
        fs::write(&path, "line1\nline2\nline3\nline4\nline5\n")
            .await
            .unwrap();

        let tool = ReadTool::new();
        let result = tool
            .execute(serde_json::json!({
                "path": path.to_string_lossy(),
                "offset": 2,
                "limit": 2
            }))
            .await
            .unwrap();

        assert!(!result.is_error);
        assert!(result.output.contains("line2"));
        assert!(result.output.contains("line3"));
        assert!(!result.output.contains("line1"));
        assert!(!result.output.contains("line5"));
    }

    #[tokio::test]
    async fn read_nonexistent_file() {
        let tool = ReadTool::new();
        let result = tool
            .execute(serde_json::json!({
                "path": "/nonexistent/file.txt"
            }))
            .await;
        assert!(result.is_err());
    }
}
