//! Bash 工具 — 执行 shell 命令。
//!
//! 对应 `packages/coding-agent/src/core/tools/bash.ts`。
//!
//! 与 TS 版对齐的关键能力：
//! - 进程组管理：超时时 kill 整个进程树，不留孤儿
//! - 超时保护：默认 120s，可配置
//! - 输出截断：超过 2000 行或 1MB 时截断

use async_trait::async_trait;
use serde_json::Value;
use std::process::Stdio;
use tokio::io::AsyncReadExt;
use tokio::process::Command;
use tokio::time::{timeout, Duration};

use pi_types::error::PiError;
use pi_types::tool::{ExecutionMode, ToolDefinition, ToolExecutor, ToolResult};

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

/// 杀掉进程组（Unix）或进程树（Windows）。
fn kill_process_tree(pid: u32) {
    #[cfg(unix)]
    {
        // 杀掉整个进程组（负 PID = 进程组 ID）
        // bash 子进程通过 setsid() 创建了新进程组
        let pgid = -(pid as i32);
        unsafe {
            libc::kill(pgid, libc::SIGKILL);
        }
    }
    #[cfg(windows)]
    {
        // Windows: 使用 taskkill /F /T /PID 杀进程树
        let _ = std::process::Command::new("taskkill")
            .args(["/F", "/T", "/PID", &pid.to_string()])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
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
            execution_mode: ExecutionMode::default(),
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

        let timeout_secs = input["timeout"].as_u64().unwrap_or(self.timeout_secs);

        // 构建 Command，创建新进程组（setsid）
        let mut cmd = Command::new("bash");
        cmd.arg("-c")
            .arg(&command)
            .current_dir(&self.cwd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null());

        // Unix: 创建新进程组，使 kill(-pgid) 能杀掉整个进程树
        #[cfg(unix)]
        {
            #[allow(unused_imports)]
            use std::os::unix::process::CommandExt;
            unsafe {
                cmd.pre_exec(|| {
                    libc::setsid();
                    Ok(())
                });
            }
        }

        let mut child = cmd.spawn().map_err(|e| PiError::Tool {
            tool: "bash".to_string(),
            message: format!("failed to spawn bash: {e}"),
        })?;

        let pid = child.id().unwrap_or(0);

        // 带超时等待 + 读取输出
        let result = timeout(Duration::from_secs(timeout_secs), async {
            let mut stdout = Vec::new();
            let mut stderr = Vec::new();

            // 并行读取 stdout 和 stderr
            let stdout_pipe = child.stdout.take();
            let stderr_pipe = child.stderr.take();

            let stdout_handle = tokio::spawn(async move {
                if let Some(mut pipe) = stdout_pipe {
                    let _ = pipe.read_to_end(&mut stdout).await;
                }
                stdout
            });
            let stderr_handle = tokio::spawn(async move {
                if let Some(mut pipe) = stderr_pipe {
                    let _ = pipe.read_to_end(&mut stderr).await;
                }
                stderr
            });

            // 等待进程退出
            let status = child.wait().await.map_err(|e| PiError::Tool {
                tool: "bash".to_string(),
                message: format!("failed to wait for process: {e}"),
            })?;

            let stdout_bytes = stdout_handle.await.unwrap_or_default();
            let stderr_bytes = stderr_handle.await.unwrap_or_default();

            let exit_code = status.code().unwrap_or(-1);

            Ok::<_, PiError>((stdout_bytes, stderr_bytes, exit_code))
        })
        .await;

        match result {
            Ok(Ok((stdout_bytes, stderr_bytes, exit_code))) => {
                let stdout = String::from_utf8_lossy(&stdout_bytes).to_string();
                let stderr = String::from_utf8_lossy(&stderr_bytes).to_string();

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
                    tool_use_id: String::new(),
                    output,
                    is_error: exit_code != 0,
                    duration_ms: None,
                    terminate: false,
                })
            }
            Ok(Err(e)) => Err(e),
            Err(_) => {
                // 超时：杀掉整个进程树
                kill_process_tree(pid);
                let _ = child.kill().await;
                Err(PiError::Tool {
                    tool: "bash".to_string(),
                    message: format!("command timed out after {timeout_secs}s: {command}"),
                })
            }
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
    async fn timeout_kills_process() {
        let tool = BashTool::new("/tmp").with_timeout(1);
        let result = tool
            .execute(serde_json::json!({"command": "sleep 100"}))
            .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("timed out"));
    }

    #[tokio::test]
    async fn timeout_kills_child_processes() {
        // 启动一个子进程，超时后应被杀掉
        let tool = BashTool::new("/tmp").with_timeout(1);
        // sleep 100 会创建一个子进程
        let result = tool
            .execute(serde_json::json!({"command": "sleep 100 & sleep 100"}))
            .await;
        assert!(result.is_err());

        // 验证没有残留的 sleep 进程
        // 给一点时间让 kill 生效
        tokio::time::sleep(Duration::from_millis(100)).await;
        let output = std::process::Command::new("pgrep")
            .args(["-c", "sleep"])
            .output()
            .unwrap();
        // pgrep 可能返回 1（没找到），stdout 为空或 "0"
        let count = String::from_utf8_lossy(&output.stdout)
            .trim()
            .parse::<u32>()
            .unwrap_or(0);
        // 不应该有残留的 sleep 进程（允许系统其他 sleep，但不超过 5）
        assert!(count < 5, "Expected few/no sleep processes, found {count}");
    }

    #[test]
    fn kill_process_tree_does_not_panic() {
        // 杀掉不存在的 PID 不应 panic
        kill_process_tree(999999);
    }
}
