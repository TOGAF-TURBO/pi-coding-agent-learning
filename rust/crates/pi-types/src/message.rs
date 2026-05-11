//! 消息类型 — LLM 对话的核心数据结构。
//!
//! 对应 `packages/ai/src/types.ts`。
//!
//! 设计原则：
//! - 所有消息都是不可变的（Clone + Serialize）
//! - ToolCall / ToolResult 通过 ID 关联
//! - 支持 thinking content（扩展思考）和 image content
//! - JSON 序列化格式与 TS 版本兼容（serde rename_all = "camelCase"）

use serde::{Deserialize, Serialize};

// ─── 思考级别 ──────────────────────────────────────────────────

/// 思考级别，控制模型推理深度。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum ThinkingLevel {
    #[default]
    Off,
    Minimal,
    Low,
    Medium,
    High,
    Xhigh,
}

impl ThinkingLevel {
    /// 所有可能的级别（不含 Off）。
    pub const ALL: [ThinkingLevel; 5] = [
        ThinkingLevel::Minimal,
        ThinkingLevel::Low,
        ThinkingLevel::Medium,
        ThinkingLevel::High,
        ThinkingLevel::Xhigh,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Minimal => "minimal",
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Xhigh => "xhigh",
        }
    }

    /// 循环到下一个级别。
    pub fn cycle_next(&self) -> ThinkingLevel {
        match self {
            Self::Off => Self::High,
            Self::Minimal => Self::Low,
            Self::Low => Self::Medium,
            Self::Medium => Self::High,
            Self::High => Self::Xhigh,
            Self::Xhigh => Self::Off,
        }
    }
}

// ─── 停止原因 ──────────────────────────────────────────────────

/// LLM 停止生成的原因。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    Stop,
    Length,
    ToolUse,
    Error,
    Aborted,
}

// ─── 内容块 ────────────────────────────────────────────────────

/// 文本内容块。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextContent {
    pub text: String,
}

/// 扩展思考内容块。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingContent {
    pub thinking: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
}

/// 图片内容块（base64 内联）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageContent {
    #[serde(rename = "type")]
    pub source_type: String,
    pub media_type: String,
    pub data: String,
}

/// 工具调用内容块。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub input: serde_json::Value,
}

/// 工具结果内容块。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_use_id: String,
    pub content: String,
    #[serde(default)]
    pub is_error: bool,
}

/// 内容块枚举 — 单条消息内的一个内容片段。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlock {
    #[serde(rename = "text")]
    Text(TextContent),
    #[serde(rename = "thinking")]
    Thinking(ThinkingContent),
    #[serde(rename = "image")]
    Image(ImageContent),
    #[serde(rename = "tool_use")]
    ToolUse(ToolCall),
    #[serde(rename = "tool_result")]
    ToolResult(ToolResult),
}

impl ContentBlock {
    pub fn text(s: impl Into<String>) -> Self {
        Self::Text(TextContent { text: s.into() })
    }

    pub fn tool_call(
        id: impl Into<String>,
        name: impl Into<String>,
        input: serde_json::Value,
    ) -> Self {
        Self::ToolUse(ToolCall {
            id: id.into(),
            name: name.into(),
            input,
        })
    }

    pub fn tool_result(
        tool_use_id: impl Into<String>,
        content: impl Into<String>,
        is_error: bool,
    ) -> Self {
        Self::ToolResult(ToolResult {
            tool_use_id: tool_use_id.into(),
            content: content.into(),
            is_error,
        })
    }

    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text(t) => Some(&t.text),
            _ => None,
        }
    }

    pub fn as_tool_call(&self) -> Option<&ToolCall> {
        match self {
            Self::ToolUse(tc) => Some(tc),
            _ => None,
        }
    }
}

// ─── 用量 ──────────────────────────────────────────────────────

/// Token 用量统计。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_creation_input_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_read_input_tokens: Option<u32>,
}

// ─── 消息角色 ──────────────────────────────────────────────────

/// 用户消息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMessage {
    pub role: String, // "user"
    pub content: Vec<ContentBlock>,
}

/// 助手消息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantMessage {
    pub role: String, // "assistant"
    pub content: Vec<ContentBlock>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<StopReason>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}

/// 工具结果消息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResultMessage {
    pub role: String, // varies
    pub content: Vec<ContentBlock>,
}

// ─── 统一消息 ──────────────────────────────────────────────────

/// 统一消息枚举。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Message {
    User(UserMessage),
    Assistant(AssistantMessage),
    ToolResult(ToolResultMessage),
}

impl Message {
    pub fn user_text(text: impl Into<String>) -> Self {
        Self::User(UserMessage {
            role: "user".to_string(),
            content: vec![ContentBlock::text(text)],
        })
    }

    pub fn assistant_text(text: impl Into<String>) -> Self {
        Self::Assistant(AssistantMessage {
            role: "assistant".to_string(),
            content: vec![ContentBlock::text(text)],
            model: None,
            stop_reason: None,
            usage: None,
            timestamp: Some(chrono::Utc::now().to_rfc3339()),
        })
    }

    pub fn tool_result(
        tool_use_id: impl Into<String>,
        output: impl Into<String>,
        is_error: bool,
    ) -> Self {
        Self::ToolResult(ToolResultMessage {
            role: "tool_result".to_string(),
            content: vec![ContentBlock::tool_result(tool_use_id, output, is_error)],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_message_serializes() {
        let msg = Message::user_text("hello");
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"role\":\"user\""));
        assert!(json.contains("\"text\":\"hello\""));
    }

    #[test]
    fn assistant_serializes_with_tool_call() {
        let msg = Message::Assistant(AssistantMessage {
            role: "assistant".to_string(),
            content: vec![
                ContentBlock::text("I'll run ls"),
                ContentBlock::tool_call("tc_1", "bash", serde_json::json!({"command": "ls"})),
            ],
            model: Some("claude-sonnet-4".to_string()),
            stop_reason: Some(StopReason::ToolUse),
            usage: Some(Usage {
                input_tokens: 100,
                output_tokens: 50,
                ..Default::default()
            }),
            timestamp: None,
        });
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"role\":\"assistant\""));
        assert!(json.contains("\"tool_use\""));
        assert!(json.contains("\"claude-sonnet-4\""));
    }

    #[test]
    fn thinking_level_cycle() {
        assert_eq!(ThinkingLevel::Off.cycle_next(), ThinkingLevel::High);
        assert_eq!(ThinkingLevel::High.cycle_next(), ThinkingLevel::Xhigh);
        assert_eq!(ThinkingLevel::Xhigh.cycle_next(), ThinkingLevel::Off);
    }
}
