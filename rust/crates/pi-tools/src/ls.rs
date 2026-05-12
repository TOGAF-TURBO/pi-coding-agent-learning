//! Ls 工具 — 列出目录内容，支持过滤和排序。
//!
//! 对应 `packages/coding-agent/src/core/tools/ls.ts`。

use async_trait::async_trait;
use serde_json::Value;
use tokio::fs;

use pi_types::error::PiError;
use pi_types::tool::{ToolDefinition, ToolExecutor, ToolResult};

/// Ls 工具执行器。
pub struct LsTool;

impl Default for LsTool {
    fn default() -> Self {
        Self::new()
    }
}

impl LsTool {
    pub fn new() -> Self {
        Self
    }
}

/// 目录条目信息。
struct EntryInfo {
    name: String,
    is_dir: bool,
    size: u64,
    modified: String,
}

#[async_trait]
impl ToolExecutor for LsTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "ls".to_string(),
            description: "List directory contents with file metadata. Shows name, type, size, and modified time.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Directory path to list (default: current directory)"
                    },
                    "all": {
                        "type": "boolean",
                        "description": "Show hidden files (default: false)"
                    },
                    "sort_by": {
                        "type": "string",
                        "enum": ["name", "size", "modified"],
                        "description": "Sort entries by name, size, or modified time (default: name)"
                    }
                },
                "required": []
            }),
            requires_approval: false,
        }
    }

    async fn execute(&self, input: Value) -> Result<ToolResult, PiError> {
        let path = input["path"].as_str().unwrap_or(".");
        let show_all = input["all"].as_bool().unwrap_or(false);
        let sort_by = input["sort_by"].as_str().unwrap_or("name");

        let dir_path = std::path::Path::new(path);
        if !dir_path.exists() {
            return Ok(ToolResult {
                tool_use_id: String::new(),
                output: format!("Directory not found: {}", path),
                is_error: true,
                duration_ms: None,
            terminate: false,
            });
        }

        if !dir_path.is_dir() {
            return Ok(ToolResult {
                tool_use_id: String::new(),
                output: format!("Not a directory: {}", path),
                is_error: true,
                duration_ms: None,
            terminate: false,
            });
        }

        let mut entries = Vec::new();
        let mut read_dir = fs::read_dir(dir_path).await.map_err(|e| PiError::Tool {
            tool: "ls".to_string(),
            message: format!("failed to read directory '{}': {e}", path),
        })?;

        while let Some(entry) = read_dir.next_entry().await.map_err(|e| PiError::Tool {
            tool: "ls".to_string(),
            message: format!("failed to read entry: {e}"),
        })? {
            let name = entry.file_name().to_string_lossy().to_string();

            // 跳过隐藏文件
            if !show_all && name.starts_with('.') {
                continue;
            }

            let metadata = match entry.metadata().await {
                Ok(m) => m,
                Err(_) => match std::fs::metadata(entry.path()) {
                    Ok(m) => m,
                    Err(_) => {
                        // 无法获取 metadata，跳过此条目
                        entries.push(EntryInfo {
                            name,
                            is_dir: false,
                            size: 0,
                            modified: "-".to_string(),
                        });
                        continue;
                    }
                },
            };

            let modified = metadata
                .modified()
                .map(|t| {
                    let datetime: chrono::DateTime<chrono::Local> = t.into();
                    datetime.format("%Y-%m-%d %H:%M").to_string()
                })
                .unwrap_or_else(|_| "-".to_string());

            entries.push(EntryInfo {
                name,
                is_dir: metadata.is_dir(),
                size: if metadata.is_file() {
                    metadata.len()
                } else {
                    0
                },
                modified,
            });
        }

        // 排序
        match sort_by {
            "size" => entries.sort_by(|a, b| b.size.cmp(&a.size).then(a.name.cmp(&b.name))),
            "modified" => {
                entries.sort_by(|a, b| b.modified.cmp(&a.modified).then(a.name.cmp(&b.name)))
            }
            _ => entries.sort_by(|a, b| a.name.cmp(&b.name)),
        }

        // 格式化输出
        let mut lines = Vec::new();
        lines.push(format!(
            "{} ({} entries)",
            dir_path.display(),
            entries.len()
        ));
        lines.push(String::new());

        let max_name_len = entries.iter().map(|e| e.name.len()).max().unwrap_or(20);
        let name_col = max_name_len.max(20) + 1;

        for entry in &entries {
            let type_marker = if entry.is_dir { "/" } else { " " };
            let size_str = if entry.size > 0 {
                format_human_size(entry.size)
            } else {
                "-".to_string()
            };
            lines.push(format!(
                "  {:<name_col$}{} {:>8}  {}",
                entry.name,
                type_marker,
                size_str,
                entry.modified,
                name_col = name_col,
            ));
        }

        Ok(ToolResult {
            tool_use_id: String::new(),
            output: lines.join("\n"),
            is_error: false,
            duration_ms: None,
            terminate: false,
        })
    }
}

/// 格式化文件大小为人类可读格式。
fn format_human_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{}B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.0}K", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1}M", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1}G", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn list_directory() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("hello.txt"), "hi").unwrap();
        fs::create_dir(dir.path().join("subdir")).unwrap();
        fs::write(dir.path().join("big.dat"), "x".repeat(2048)).unwrap();

        let tool = LsTool::new();
        let result = tool
            .execute(serde_json::json!({
                "path": dir.path().to_string_lossy()
            }))
            .await
            .unwrap();

        assert!(!result.is_error, "Error: {}", result.output);
        assert!(
            result.output.contains("hello.txt"),
            "Output: {}",
            result.output
        );
        assert!(
            result.output.contains("subdir"),
            "Output: {}",
            result.output
        );
        assert!(
            result.output.contains("2.0K") || result.output.contains("2K"),
            "Output: {}",
            result.output
        );
        assert!(result.output.contains("3 entries"));
    }

    #[tokio::test]
    async fn hidden_files() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join(".hidden"), "secret").unwrap();
        fs::write(dir.path().join("visible.txt"), "ok").unwrap();

        let tool = LsTool::new();

        // 默认不显示隐藏文件
        let result = tool
            .execute(serde_json::json!({
                "path": dir.path().to_string_lossy()
            }))
            .await
            .unwrap();
        assert!(!result.output.contains(".hidden"));
        assert!(result.output.contains("visible.txt"));

        // 显示隐藏文件
        let result = tool
            .execute(serde_json::json!({
                "path": dir.path().to_string_lossy(),
                "all": true
            }))
            .await
            .unwrap();
        assert!(result.output.contains(".hidden"));
    }

    #[tokio::test]
    async fn sort_by_size() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("small.txt"), "a").unwrap();
        fs::write(dir.path().join("big.txt"), "a".repeat(1000)).unwrap();

        let tool = LsTool::new();
        let result = tool
            .execute(serde_json::json!({
                "path": dir.path().to_string_lossy(),
                "sort_by": "size"
            }))
            .await
            .unwrap();

        // big.txt 应排在前面
        let big_pos = result.output.find("big.txt").unwrap();
        let small_pos = result.output.find("small.txt").unwrap();
        assert!(big_pos < small_pos);
    }

    #[tokio::test]
    async fn nonexistent_directory() {
        let tool = LsTool::new();
        let result = tool
            .execute(serde_json::json!({
                "path": "/nonexistent/path/xyz"
            }))
            .await
            .unwrap();
        assert!(result.is_error);
        assert!(result.output.contains("not found"));
    }

    #[test]
    fn human_size_format() {
        assert_eq!(format_human_size(0), "0B");
        assert_eq!(format_human_size(512), "512B");
        assert_eq!(format_human_size(1536), "2K");
        assert_eq!(format_human_size(2_000_000), "1.9M");
        assert_eq!(format_human_size(3_000_000_000), "2.8G");
    }
}
