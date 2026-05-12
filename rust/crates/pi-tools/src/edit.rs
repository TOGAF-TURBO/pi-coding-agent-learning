//! Edit 工具 — 精确文件编辑（字符串替换）。
//!
//! 对应 `packages/coding-agent/src/core/tools/edit.ts`。
//!
//! 使用精确文本替换，oldText 必须匹配文件中的唯一位置。

use async_trait::async_trait;
use serde_json::Value;
use tokio::fs;

use pi_types::error::PiError;
use pi_types::tool::{ExecutionMode, ToolDefinition, ToolExecutor, ToolResult};

/// Edit 工具执行器。
pub struct EditTool;

impl Default for EditTool {
    fn default() -> Self {
        Self::new()
    }
}

impl EditTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ToolExecutor for EditTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "edit".to_string(),
            description: "Make precise edits to a file using exact text replacement. The oldText must match exactly in the file.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the file to edit"
                    },
                    "oldText": {
                        "type": "string",
                        "description": "Exact text to find and replace"
                    },
                    "newText": {
                        "type": "string",
                        "description": "Replacement text"
                    }
                },
                "required": ["path", "oldText", "newText"]
            }),
            requires_approval: false,
            execution_mode: ExecutionMode::default(),
        }
    }

    async fn execute(&self, input: Value) -> Result<ToolResult, PiError> {
        let path = input["path"].as_str().ok_or_else(|| PiError::Tool {
            tool: "edit".to_string(),
            message: "missing 'path' parameter".to_string(),
        })?;

        let old_text = input["oldText"].as_str().ok_or_else(|| PiError::Tool {
            tool: "edit".to_string(),
            message: "missing 'oldText' parameter".to_string(),
        })?;

        let new_text = input["newText"].as_str().ok_or_else(|| PiError::Tool {
            tool: "edit".to_string(),
            message: "missing 'newText' parameter".to_string(),
        })?;

        let content = fs::read_to_string(path).await.map_err(|e| PiError::Tool {
            tool: "edit".to_string(),
            message: format!("failed to read '{}': {e}", path),
        })?;

        // 检查 oldText 是否存在
        let count = content.matches(old_text).count();
        if count == 0 {
            return Err(PiError::Tool {
                tool: "edit".to_string(),
                message: format!(
                    "oldText not found in '{}'. The exact text must exist in the file.",
                    path
                ),
            });
        }
        if count > 1 {
            return Err(PiError::Tool {
                tool: "edit".to_string(),
                message: format!(
                    "oldText found {} times in '{}'. It must match exactly once to avoid ambiguous edits.",
                    count, path
                ),
            });
        }

        let new_content = content.replacen(old_text, new_text, 1);

        // 生成 diff
        let diff = crate::diff::format_diff(&content, &new_content, path);
        let diff_display = if diff.is_empty() {
            String::new()
        } else {
            format!("\n```diff\n{}\n```", diff.trim())
        };

        fs::write(path, &new_content)
            .await
            .map_err(|e| PiError::Tool {
                tool: "edit".to_string(),
                message: format!("failed to write '{}': {e}", path),
            })?;

        // 报告变更位置
        let old_line = content
            .lines()
            .position(|l| l.contains(old_text))
            .map(|l| l + 1);

        Ok(ToolResult {
            tool_use_id: String::new(),
            output: match old_line {
                Some(line) => format!("Edited {} at line {}{}", path, line, diff_display),
                None => format!("Edited {}{}", path, diff_display),
            },
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

    #[tokio::test]
    async fn edit_replaces_text() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.txt");
        tokio::fs::write(&path, "hello world").await.unwrap();

        let tool = EditTool::new();
        let result = tool
            .execute(serde_json::json!({
                "path": path.to_string_lossy(),
                "oldText": "world",
                "newText": "rust"
            }))
            .await
            .unwrap();

        assert!(!result.is_error);
        let content = tokio::fs::read_to_string(&path).await.unwrap();
        assert_eq!(content, "hello rust");
    }

    #[tokio::test]
    async fn edit_rejects_multiple_matches() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.txt");
        tokio::fs::write(&path, "abc abc").await.unwrap();

        let tool = EditTool::new();
        let result = tool
            .execute(serde_json::json!({
                "path": path.to_string_lossy(),
                "oldText": "abc",
                "newText": "xyz"
            }))
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn edit_rejects_no_match() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.txt");
        tokio::fs::write(&path, "hello").await.unwrap();

        let tool = EditTool::new();
        let result = tool
            .execute(serde_json::json!({
                "path": path.to_string_lossy(),
                "oldText": "notfound",
                "newText": "xyz"
            }))
            .await;

        assert!(result.is_err());
    }
}
