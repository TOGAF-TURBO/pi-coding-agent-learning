//! LSP 客户端框架 — 与语言服务器通信的基础设施。
//!
//! 当前为框架代码，支持未来接入 tower-lsp 或直接 JSON-RPC over stdio。
//! 设计目标：
//! - 启动 LSP 服务器子进程
//! - 发送 initialize → initialized 握手
//! - 请求 textDocument/completion
//! - 解析响应
//!
//! 暂不依赖 tower-lsp，仅定义类型和协议常量。

use serde::{Deserialize, Serialize};

/// LSP 请求 ID。
pub type RequestId = i64;

/// LSP 位置。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

/// LSP 文本文档标识符。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextDocumentIdentifier {
    pub uri: String,
}

/// LSP 文本文档位置。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextDocumentPositionParams {
    pub text_document: TextDocumentIdentifier,
    pub position: Position,
}

/// LSP 补全项。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
#[derive(Default)]
pub struct CompletionItem {
    pub label: String,
    #[serde(default)]
    pub kind: Option<i32>,
    pub detail: Option<String>,
    pub insert_text: Option<String>,
}

/// LSP 补全列表。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompletionList {
    pub is_incomplete: bool,
    pub items: Vec<CompletionItem>,
}

/// LSP 初始化参数。
#[derive(Debug, Serialize)]
pub struct InitializeParams {
    pub process_id: Option<i32>,
    pub root_uri: Option<String>,
    pub capabilities: ClientCapabilities,
}

/// 客户端能力声明。
#[derive(Debug, Serialize)]
pub struct ClientCapabilities {
    pub text_document: TextDocumentClientCapabilities,
}

/// 文本文档客户端能力。
#[derive(Debug, Serialize)]
pub struct TextDocumentClientCapabilities {
    pub completion: CompletionClientCapabilities,
}

/// 补全客户端能力。
#[derive(Debug, Serialize)]
pub struct CompletionClientCapabilities {
    pub completion_item: CompletionItemClientCapabilities,
}

/// 补全项客户端能力。
#[derive(Debug, Serialize)]
pub struct CompletionItemClientCapabilities {
    pub snippet_support: bool,
}

/// 补全项类型。
pub mod completion_item_kind {
    pub const TEXT: i32 = 1;
    pub const METHOD: i32 = 2;
    pub const FUNCTION: i32 = 3;
    pub const FIELD: i32 = 5;
    pub const VARIABLE: i32 = 6;
    pub const CLASS: i32 = 7;
    pub const INTERFACE: i32 = 8;
    pub const MODULE: i32 = 9;
    pub const PROPERTY: i32 = 10;
    pub const FILE: i32 = 17;
    pub const FOLDER: i32 = 19;
}

/// 构建默认的 InitializeParams。
pub fn default_initialize_params(root_uri: &str) -> InitializeParams {
    InitializeParams {
        process_id: Some(std::process::id() as i32),
        root_uri: Some(root_uri.to_string()),
        capabilities: ClientCapabilities {
            text_document: TextDocumentClientCapabilities {
                completion: CompletionClientCapabilities {
                    completion_item: CompletionItemClientCapabilities {
                        snippet_support: false,
                    },
                },
            },
        },
    }
}

/// 文件 URI 转换。
pub fn file_uri(path: &str) -> String {
    format!("file://{}", path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_uri_format() {
        assert_eq!(file_uri("/home/user/project"), "file:///home/user/project");
    }

    #[test]
    fn default_init_params() {
        let params = default_initialize_params("file:///project");
        assert_eq!(params.root_uri, Some("file:///project".to_string()));
        assert!(params.process_id.is_some());
    }

    #[test]
    fn completion_item_deserialize() {
        let json = r#"{"label":"fn main","kind":3,"detail":"fn item","insertText":"fn main()"}"#;
        let item: CompletionItem = serde_json::from_str(json).unwrap();
        assert_eq!(item.label, "fn main");
        assert_eq!(item.kind, Some(3));
        assert_eq!(item.insert_text, Some("fn main()".to_string()));
    }

    #[test]
    fn completion_list_deserialize() {
        let json = r#"{"isIncomplete":false,"items":[{"label":"foo"},{"label":"bar"}]}"#;
        let list: CompletionList = serde_json::from_str(json).unwrap();
        assert!(!list.is_incomplete);
        assert_eq!(list.items.len(), 2);
    }

    #[test]
    fn completion_item_kinds() {
        assert_eq!(completion_item_kind::FUNCTION, 3);
        assert_eq!(completion_item_kind::FILE, 17);
        assert_eq!(completion_item_kind::MODULE, 9);
    }
}
