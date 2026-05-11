//! LSP 客户端管理器 — 启动和管理语言服务器子进程。
//!
//! 通过 stdio JSON-RPC 与 LSP 服务器通信。
//! 支持 initialize → initialized 握手 + textDocument/completion 请求。
//! 未来可扩展 diagnostic 发布等。

use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;

use anyhow::{Context, Result};
use serde_json::{json, Value};

use super::lsp::{self, CompletionItem, CompletionList};

/// LSP 客户端 — 管理一个语言服务器子进程。
pub struct LspClient {
    /// 子进程。
    process: Child,
    /// 请求 ID 计数器。
    next_id: Mutex<u64>,
    /// 是否已初始化。
    initialized: bool,
}

impl LspClient {
    /// 启动 LSP 服务器子进程。
    ///
    /// `command` 是启动语言服务器的命令（如 "rust-analyzer", "clangd"）。
    /// `args` 是命令参数。
    pub fn start(command: &str, args: &[&str]) -> Result<Self> {
        let child = Command::new(command)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .with_context(|| format!("Failed to start LSP server: {}", command))?;

        Ok(Self {
            process: child,
            next_id: Mutex::new(1),
            initialized: false,
        })
    }

    /// 分配下一个请求 ID。
    fn next_request_id(&self) -> u64 {
        let mut id = self.next_id.lock().unwrap();
        let current = *id;
        *id += 1;
        current
    }

    /// 发送 JSON-RPC 请求并读取响应。
    ///
    /// 使用 Content-Length 头部协议（LSP 规范）。
    fn send_request(&mut self, method: &str, params: Value) -> Result<Value> {
        let id = self.next_request_id();
        let request = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        });

        let body = serde_json::to_string(&request)?;
        let header = format!("Content-Length: {}\r\n\r\n", body.len());

        let stdin = self
            .process
            .stdin
            .as_mut()
            .context("LSP process stdin not available")?;
        use std::io::Write;
        stdin.write_all(header.as_bytes())?;
        stdin.write_all(body.as_bytes())?;
        stdin.flush()?;

        // 读取响应
        self.read_response()
    }

    /// 发送 JSON-RPC 通知（无响应）。
    fn send_notification(&mut self, method: &str, params: Value) -> Result<()> {
        let notification = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
        });

        let body = serde_json::to_string(&notification)?;
        let header = format!("Content-Length: {}\r\n\r\n", body.len());

        let stdin = self
            .process
            .stdin
            .as_mut()
            .context("LSP process stdin not available")?;
        use std::io::Write;
        stdin.write_all(header.as_bytes())?;
        stdin.write_all(body.as_bytes())?;
        stdin.flush()?;
        Ok(())
    }

    /// 读取 JSON-RPC 响应。
    fn read_response(&mut self) -> Result<Value> {
        let stdout = self
            .process
            .stdout
            .as_mut()
            .context("LSP process stdout not available")?;
        use std::io::Read;

        // 读取 Content-Length 头
        let mut header_buf = String::new();
        let mut byte: [u8; 1] = [0];
        let mut newlines = 0;
        loop {
            stdout.read_exact(&mut byte)?;
            header_buf.push(byte[0] as char);
            if byte[0] == b'\n' {
                newlines += 1;
                if newlines >= 2 {
                    break;
                }
            } else if byte[0] != b'\r' {
                newlines = 0;
            }
        }

        // 解析 Content-Length
        let content_len: usize = header_buf
            .lines()
            .find_map(|line| {
                line.strip_prefix("Content-Length: ")
                    .and_then(|v| v.trim().parse().ok())
            })
            .context("No Content-Length header in LSP response")?;

        // 读取 body
        let mut body_buf = vec![0u8; content_len];
        stdout.read_exact(&mut body_buf)?;
        let body_str = String::from_utf8(body_buf)?;

        let response: Value = serde_json::from_str(&body_str)?;
        Ok(response)
    }

    /// 执行 LSP initialize 握手。
    pub fn initialize(&mut self, root_uri: &str) -> Result<Value> {
        let params = lsp::default_initialize_params(root_uri);
        let params_json = serde_json::to_value(&params)?;

        let result = self.send_request("initialize", params_json)?;

        // 发送 initialized 通知
        self.send_notification("initialized", json!({}))?;
        self.initialized = true;

        Ok(result)
    }

    /// 请求 textDocument/completion。
    pub fn completion(
        &mut self,
        file_uri: &str,
        line: u32,
        character: u32,
    ) -> Result<Vec<CompletionItem>> {
        if !self.initialized {
            anyhow::bail!("LSP client not initialized");
        }

        let params = json!({
            "textDocument": { "uri": file_uri },
            "position": { "line": line, "character": character }
        });

        let response = self.send_request("textDocument/completion", params)?;

        // 响应可能是 CompletionList 或 CompletionItem[]
        let items = if let Some(result) = response.get("result") {
            if let Ok(list) = serde_json::from_value::<CompletionList>(result.clone()) {
                list.items
            } else if let Some(arr) = result.as_array() {
                arr.iter()
                    .filter_map(|v| serde_json::from_value(v.clone()).ok())
                    .collect()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        Ok(items)
    }

    /// 发送 shutdown + exit 关闭 LSP 服务器。
    pub fn shutdown(&mut self) -> Result<()> {
        if self.initialized {
            let _ = self.send_request("shutdown", json!(null));
            let _ = self.send_notification("exit", json!({}));
        }
        Ok(())
    }
}

impl Drop for LspClient {
    fn drop(&mut self) {
        let _ = self.shutdown();
        let _ = self.process.kill();
    }
}

/// 已知语言服务器配置。
pub struct LanguageServer {
    pub command: &'static str,
    pub args: &'static [&'static str],
    pub languages: &'static [&'static str],
}

/// 已知语言服务器列表。
pub static KNOWN_SERVERS: &[LanguageServer] = &[
    LanguageServer {
        command: "rust-analyzer",
        args: &[],
        languages: &["rust"],
    },
    LanguageServer {
        command: "clangd",
        args: &[],
        languages: &["c", "cpp", "objc", "objcpp"],
    },
    LanguageServer {
        command: "pylsp",
        args: &[],
        languages: &["python"],
    },
    LanguageServer {
        command: "gopls",
        args: &[],
        languages: &["go"],
    },
    LanguageServer {
        command: "typescript-language-server",
        args: &["--stdio"],
        languages: &["typescript", "javascript"],
    },
];

/// 已知语言服务器列表。
pub fn known_servers() -> &'static [LanguageServer] {
    KNOWN_SERVERS
}

/// 根据文件扩展名查找合适的语言服务器。
pub fn find_server_for_file(path: &Path) -> Option<&'static LanguageServer> {
    let ext = path.extension()?.to_str()?.to_lowercase();
    let lang = match ext.as_str() {
        "rs" => "rust",
        "c" => "c",
        "cpp" | "cc" | "cxx" | "h" | "hpp" => "cpp",
        "py" => "python",
        "go" => "go",
        "ts" | "tsx" => "typescript",
        "js" | "jsx" => "javascript",
        _ => return None,
    };

    known_servers().iter().find(|s| s.languages.contains(&lang))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_server_for_rust() {
        let server = find_server_for_file(Path::new("src/main.rs"));
        assert!(server.is_some());
        assert_eq!(server.unwrap().command, "rust-analyzer");
    }

    #[test]
    fn find_server_for_python() {
        let server = find_server_for_file(Path::new("app.py"));
        assert!(server.is_some());
        assert_eq!(server.unwrap().command, "pylsp");
    }

    #[test]
    fn find_server_for_unknown() {
        let server = find_server_for_file(Path::new("data.csv"));
        assert!(server.is_none());
    }

    #[test]
    fn known_servers_not_empty() {
        let servers = known_servers();
        assert!(!servers.is_empty());
        assert!(servers.iter().any(|s| s.command == "rust-analyzer"));
    }
}
