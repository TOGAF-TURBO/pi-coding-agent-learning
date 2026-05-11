//! Bash 工具 — 执行 shell 命令。
//!
//! 对应 `packages/coding-agent/src/core/tools/bash.ts`。

use async_trait::async_trait;
use serde_json::Value;
use tokio::process::Command;
use tokio::time::{Duration, timeout};

use pi_types::error::PiError;
use pi_types::tool::{ToolDefinition, ToolExecutor, ToolResult};

use crate::truncate::truncate_output;

const DEFAULT_TIMEOUT_SECS: u64 = 120;

/// Bash 工具执行器。
pub struct BashTool {
    cwd: String,
    timeout_secs: u64,
}

impl BashTool {
    pub fn new(cwd: impl Into<String>) -> Self {
        Self {
            cwd: cwd.into(),
            timeout_secs: DEFAULT_TIMEOUT_SECS,
        }
    }

    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }
}

#[async_trait]
impl ToolExecutor for BashTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "bash".to_string(),
            description: "Execute a shell command and return the output.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "The shell command to execute"
                    },
                    "timeout": {
                        "type": "number",
                        "description": "Timeout in seconds (default 120)"
                    }
                },
                "required": ["command"]
            }),
            requires_approval: false,
        }
    }

    async fn execute(&self, input: Value) -> Result<ToolResult, PiError> {
        let command = input["command"]
            .as_str()
            .ok_or_else(|| PiError::Tool {
                tool: "bash".to_string(),
                message: "missing 'command' parameter".to_string(),
            })?
            .to_string();

        let timeout_secs = input["timeout"]
            .as_u64()
            .unwrap_or(self.timeout_secs);

        let result = timeout(Duration::from_secs(timeout_secs), async {
            let output = Command::new("bash")
                .arg("-c")
                .arg(&command)
                .current_dir(&self.cwd)
                .output()
                .await
                .map_err(|e| PiError::Tool {
                    tool: "bash".to_string(),
                    message: format!("failed to spawn bash: {e}"),
                })?;

            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let exit_code = output.status.code().unwrap_or(-1);

            Ok::<_, PiError>((stdout, stderr, exit_code))
        })
        .await;

        match result {
            Ok(Ok((stdout, stderr, exit_code))) => {
                let combined = if stderr.is_empty() {
                    stdout
                } else {
                    format!("{stdout}\n[stderr]\n{stderr}")
                };
                let truncated = truncate_output(&combined);
                let mut output = truncated.output;
                if exit_code != 0 {
                    output.push_str(&format!("\n[exit code: {exit_code}]"));
                }
                if truncated.truncated {
                    output.push_str(&format!(
                        "\n[original: {} lines, {} bytes]",
                        truncated.original_lines, truncated.original_bytes
                    ));
                }
                Ok(ToolResult {
                    tool_use_id: String::new(), // filled in by agent loop
                    output,
                    is_error: exit_code != 0,
                    duration_ms: None,
                })
            }
            Ok(Err(e)) => Err(e),
            Err(_) => Err(PiError::Tool {
                tool: "bash".to_string(),
                message: format!("command timed out after {timeout_secs}s: {command}"),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn echo_command() {
        let tool = BashTool::new("/tmp");
        let result = tool
            .execute(serde_json::json!({"command": "echo hello"}))
            .await
            .unwrap();
        assert!(!result.is_error);
        assert!(result.output.contains("hello"));
    }

    #[tokio::test]
    async fn failing_command() {
        let tool = BashTool::new("/tmp");
        let result = tool
            .execute(serde_json::json!({"command": "exit 1"}))
            .await
            .unwrap();
        assert!(result.is_error);
        assert!(result.output.contains("[exit code: 1]"));
    }

    #[tokio::test]
    async fn timeout_command() {
        let tool = BashTool::new("/tmp").with_timeout(1);
        let result = tool
            .execute(serde_json::json!({"command": "sleep 100"}))
            .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("timed out"));
    }
}
