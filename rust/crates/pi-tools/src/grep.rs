//! Grep 工具 — 在文件内容中搜索文本模式。
//!
//! 对应 `packages/coding-agent/src/core/tools/grep.ts`。

use async_trait::async_trait;
use regex::Regex;
use serde_json::Value;

use pi_types::error::PiError;
use pi_types::tool::{ExecutionMode, ToolDefinition, ToolExecutor, ToolResult};

use crate::truncate::truncate_output;

/// Grep 工具执行器。
pub struct GrepTool {
    cwd: String,
}

impl GrepTool {
    pub fn new(cwd: impl Into<String>) -> Self {
        Self { cwd: cwd.into() }
    }
}

#[async_trait]
impl ToolExecutor for GrepTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "grep".to_string(),
            description: "Search file contents for a text pattern. Supports regex.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "pattern": {
                        "type": "string",
                        "description": "Search pattern (regex supported)"
                    },
                    "path": {
                        "type": "string",
                        "description": "File or directory to search in (default: current working directory)"
                    },
                    "include": {
                        "type": "string",
                        "description": "File glob pattern to include (e.g. \"*.rs\", \"*.ts\")"
                    },
                    "case_insensitive": {
                        "type": "boolean",
                        "description": "Case-insensitive search (default: false)"
                    }
                },
                "required": ["pattern"]
            }),
            requires_approval: false,
            execution_mode: ExecutionMode::default(),
        }
    }

    async fn execute(&self, input: Value) -> Result<ToolResult, PiError> {
        let pattern = input["pattern"].as_str().ok_or_else(|| PiError::Tool {
            tool: "grep".to_string(),
            message: "missing 'pattern' parameter".to_string(),
        })?;

        let base_path = input["path"].as_str().unwrap_or(&self.cwd);

        let include = input["include"].as_str().unwrap_or("**/*");

        let case_insensitive = input["case_insensitive"].as_bool().unwrap_or(false);

        // 构建正则
        let re = if case_insensitive {
            Regex::new(&format!("(?i){}", pattern))
        } else {
            Regex::new(pattern)
        }
        .map_err(|e| PiError::Tool {
            tool: "grep".to_string(),
            message: format!("invalid regex '{}': {e}", pattern),
        })?;

        let mut results = Vec::new();
        let mut match_count = 0;
        let mut file_count = 0;

        let base = std::path::Path::new(base_path);
        if base.is_file() {
            // 搜索单个文件
            grep_file(&re, base, base_path, &mut results, &mut match_count)?;
            if match_count > 0 {
                file_count = 1;
            }
        } else {
            // 搜索目录
            let builder = globwalk::GlobWalkerBuilder::new(base_path, include);

            let walker = builder.build().map_err(|e| PiError::Tool {
                tool: "grep".to_string(),
                message: format!("invalid glob '{}': {e}", include),
            })?;

            for entry in walker.filter_map(|e| e.ok()) {
                let path = entry.path();
                if !path.is_file() {
                    continue;
                }

                // 跳过隐藏和非项目目录
                let components: Vec<_> = path.components().collect();
                let skip = components.iter().any(|c| {
                    let s = c.as_os_str().to_string_lossy();
                    s.starts_with('.') || s == "node_modules" || s == "target" || s == "dist"
                });
                if skip {
                    continue;
                }

                let rel = path.display().to_string();
                let before = match_count;
                grep_file(&re, path, &rel, &mut results, &mut match_count)?;
                if match_count > before {
                    file_count += 1;
                }
            }
        }

        let output = if results.is_empty() {
            format!("No matches for '{}' in {}", pattern, base_path)
        } else {
            results.join("\n")
        };

        let truncated = truncate_output(&output);
        let mut result = truncated.output;
        result.push_str(&format!(
            "\n[{} matches across {} files]",
            match_count, file_count
        ));

        Ok(ToolResult {
            tool_use_id: String::new(),
            output: result,
            is_error: false,
            duration_ms: None,
            terminate: false,
        })
    }
}

/// 搜索单个文件。
fn grep_file(
    re: &Regex,
    path: &std::path::Path,
    display_path: &str,
    results: &mut Vec<String>,
    match_count: &mut usize,
) -> Result<(), PiError> {
    let content = std::fs::read_to_string(path).map_err(|e| PiError::Tool {
        tool: "grep".to_string(),
        message: format!("failed to read '{}': {e}", display_path),
    })?;

    for (i, line) in content.lines().enumerate() {
        if re.is_match(line) {
            results.push(format!("{}:{}\t{}", display_path, i + 1, line.trim_end()));
            *match_count += 1;
            if *match_count > 500 {
                return Ok(());
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn grep_finds_match() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("test.txt"),
            "hello world\nfoo bar\nhello rust",
        )
        .unwrap();

        let tool = GrepTool::new(dir.path().to_string_lossy());
        let result = tool
            .execute(serde_json::json!({
                "pattern": "hello"
            }))
            .await
            .unwrap();

        assert!(!result.is_error);
        assert!(result.output.contains("hello"));
    }

    #[tokio::test]
    async fn grep_no_match() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("test.txt"), "no match here").unwrap();

        let tool = GrepTool::new(dir.path().to_string_lossy());
        let result = tool
            .execute(serde_json::json!({
                "pattern": "NONEXISTENT_PATTERN_XYZ"
            }))
            .await
            .unwrap();

        assert!(!result.is_error);
        assert!(
            result.output.contains("0 matches")
                || result.output.is_empty()
                || result.output.contains("0")
        );
    }

    #[test]
    fn tool_definition() {
        let tool = GrepTool::new("/tmp");
        let def = tool.definition();
        assert_eq!(def.name, "grep");
    }
}
