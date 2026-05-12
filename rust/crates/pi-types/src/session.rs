//! 会话类型 — 会话树条目、JSONL 格式、分支。
//!
//! 对应 `packages/agent/src/harness/types.ts` 的 SessionTreeEntry 体系和
//! `packages/agent/src/harness/session/storage/jsonl.ts` 的 JSONL 格式。
//!
//! JSONL 格式与 TS 版本双向兼容：
//! - 第一行是 session header（type: "session", version: 3）
//! - 后续每行是一个 SessionEntry
//! - Rust 写入的 JSONL 文件必须能被 TS 版本读取，反之亦然

use serde::{Deserialize, Serialize};

// ─── 会话头 ────────────────────────────────────────────────────

/// JSONL 会话文件的第一行。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionHeader {
    #[serde(rename = "type")]
    pub entry_type: String, // always "session"
    pub version: u32, // always 3
    pub id: String,
    pub timestamp: String,
    pub cwd: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_session: Option<String>,
}

impl SessionHeader {
    pub fn new(id: impl Into<String>, cwd: impl Into<String>) -> Self {
        Self {
            entry_type: "session".to_string(),
            version: 3,
            id: id.into(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            cwd: cwd.into(),
            parent_session: None,
        }
    }
}

// ─── 会话树条目 ────────────────────────────────────────────────

/// 会话树条目的基础字段。
/// 对应 TS 的 SessionTreeEntryBase。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionEntryBase {
    #[serde(rename = "type")]
    pub entry_type: String,
    pub id: String,
    #[serde(rename = "parentId")]
    pub parent_id: Option<String>,
    pub timestamp: String,
}

/// 消息条目 — 存储一条 Agent 消息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEntry {
    #[serde(rename = "type")]
    pub entry_type: String, // "message"
    pub id: String,
    #[serde(rename = "parentId")]
    pub parent_id: Option<String>,
    pub timestamp: String,
    pub role: String,
    pub content: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(rename = "stopReason", skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<serde_json::Value>,
}

/// 压缩条目 — 存储上下文压缩的摘要。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionEntry {
    #[serde(rename = "type")]
    pub entry_type: String, // "compaction"
    pub id: String,
    #[serde(rename = "parentId")]
    pub parent_id: Option<String>,
    pub timestamp: String,
    pub summary: serde_json::Value,
}

/// 标签条目 — 用户在条目上打的标签。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelEntry {
    #[serde(rename = "type")]
    pub entry_type: String, // "label"
    pub id: String,
    #[serde(rename = "parentId")]
    pub parent_id: Option<String>,
    pub timestamp: String,
    #[serde(rename = "targetId")]
    pub target_id: String,
    pub label: String,
}

/// 自定义条目 — 扩展注入的任意数据。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomEntry {
    #[serde(rename = "type")]
    pub entry_type: String, // "custom"
    pub id: String,
    #[serde(rename = "parentId")]
    pub parent_id: Option<String>,
    pub timestamp: String,
    #[serde(rename = "customType")]
    pub custom_type: String,
    pub content: serde_json::Value,
}

/// 模型切换事件 — 记录用户切换 provider/model 的时刻。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelChangeEntry {
    #[serde(rename = "type")]
    pub entry_type: String, // "model_change"
    pub id: String,
    #[serde(rename = "parentId")]
    pub parent_id: Option<String>,
    pub timestamp: String,
    /// 新的 provider 名称。
    pub provider: Option<String>,
    /// 新的模型 ID。
    #[serde(rename = "modelId")]
    pub model_id: String,
}

/// 统一的会话条目枚举。
/// 使用 untagged 以匹配 TS 版本的 flat JSON 格式。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SessionEntry {
    Message(MessageEntry),
    Compaction(CompactionEntry),
    Label(LabelEntry),
    Custom(CustomEntry),
    ModelChange(ModelChangeEntry),
    /// 未知类型 — 保留原始 JSON 以实现前向兼容。
    Other(serde_json::Value),
}

impl SessionEntry {
    pub fn id(&self) -> &str {
        match self {
            Self::Message(e) => &e.id,
            Self::Compaction(e) => &e.id,
            Self::Label(e) => &e.id,
            Self::Custom(e) => &e.id,
            Self::ModelChange(e) => &e.id,
            Self::Other(v) => v.get("id").and_then(|v| v.as_str()).unwrap_or(""),
        }
    }

    pub fn parent_id(&self) -> Option<&str> {
        match self {
            Self::Message(e) => e.parent_id.as_deref(),
            Self::Compaction(e) => e.parent_id.as_deref(),
            Self::Label(e) => e.parent_id.as_deref(),
            Self::Custom(e) => e.parent_id.as_deref(),
            Self::ModelChange(e) => e.parent_id.as_deref(),
            Self::Other(v) => v.get("parentId").and_then(|v| v.as_str()),
        }
    }

    pub fn entry_type(&self) -> &str {
        match self {
            Self::Message(_) => "message",
            Self::Compaction(_) => "compaction",
            Self::Label(_) => "label",
            Self::Custom(e) => &e.custom_type,
            Self::ModelChange(_) => "model_change",
            Self::Other(v) => v.get("type").and_then(|v| v.as_str()).unwrap_or("unknown"),
        }
    }
}

// ─── 会话元数据 ────────────────────────────────────────────────

/// 会话列表中的元数据摘要。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub created_at: String,
    pub cwd: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_session_path: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_header_json() {
        let header = SessionHeader::new("test-123", "/home/user/project");
        let json = serde_json::to_string(&header).unwrap();
        assert!(json.contains(r#""type":"session""#));
        assert!(json.contains(r#""version":3"#));

        let parsed: SessionHeader = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, "test-123");
        assert_eq!(parsed.cwd, "/home/user/project");
    }

    #[test]
    fn message_entry_json_compatible_with_ts() {
        // This is what TS version writes
        let ts_json = r#"{"type":"message","id":"abc","parentId":null,"timestamp":"2026-05-10T12:00:00Z","role":"user","content":[{"type":"text","text":"hello"}]}"#;
        let entry: SessionEntry = serde_json::from_str(ts_json).unwrap();
        assert_eq!(entry.entry_type(), "message");
        assert_eq!(entry.id(), "abc");
    }

    #[test]
    fn unknown_entry_preserved() {
        let json = r#"{"type":"future_type","id":"xyz","parentId":"abc","timestamp":"2026-05-10T12:00:00Z","customField":42}"#;
        let entry: SessionEntry = serde_json::from_str(json).unwrap();
        assert_eq!(entry.entry_type(), "future_type");
        // Round-trip preserves unknown fields
        let back = serde_json::to_string(&entry).unwrap();
        assert!(back.contains(r#""customField":42"#));
    }

    #[test]
    fn model_change_entry_round_trip() {
        let json = r#"{"type":"model_change","id":"abc123","parentId":"parent","timestamp":"2025-01-01T00:00:00Z","provider":"anthropic","modelId":"claude-3.5-sonnet"}"#;
        let entry: SessionEntry = serde_json::from_str(json).unwrap();
        match entry {
            SessionEntry::ModelChange(e) => {
                assert_eq!(e.model_id, "claude-3.5-sonnet");
                assert_eq!(e.provider, Some("anthropic".to_string()));
            }
            _ => panic!("Expected ModelChange, got {:?}", entry.entry_type()),
        }
    }
}
