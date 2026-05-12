//! 消息转换 — 将 pi 消息转换为各 provider 的 API 格式。
//!
//! 三种主要格式：
//! - Anthropic: content blocks 数组，tool_result 在 user role 内
//! - OpenAI: tool_result 作为独立 role: "tool" 消息
//! - Gemini: role 为 "user"/"model"，工具为 functionCall/functionResponse

use pi_types::message::{ContentBlock, Message};
use serde_json::{json, Value};

/// 转换为 Anthropic Messages API 格式。
///
/// 特点：
/// - content 为 block 数组
/// - tool_result 在 user role 的 content 内
/// - thinking 保留为 thinking block
/// - image 为 base64 source
pub fn to_anthropic_messages(messages: &[Message]) -> Vec<Value> {
    let mut result = Vec::new();

    for msg in messages {
        match msg {
            Message::User(u) => {
                let content: Vec<Value> = u.content.iter().map(block_to_anthropic).collect();
                result.push(json!({"role": "user", "content": content}));
            }
            Message::Assistant(a) => {
                let content: Vec<Value> = a.content.iter().map(block_to_anthropic).collect();
                result.push(json!({"role": "assistant", "content": content}));
            }
            Message::ToolResult(tr) => {
                let content: Vec<Value> = tr.content.iter().map(block_to_anthropic).collect();
                result.push(json!({"role": "user", "content": content}));
            }
        }
    }

    result
}

/// 单个 ContentBlock → Anthropic 格式。
fn block_to_anthropic(block: &ContentBlock) -> Value {
    match block {
        ContentBlock::Text(t) => json!({"type": "text", "text": t.text}),
        ContentBlock::Thinking(t) => json!({"type": "thinking", "thinking": t.thinking}),
        ContentBlock::Image(img) => json!({
            "type": "image",
            "source": {
                "type": "base64",
                "media_type": img.media_type,
                "data": img.data,
            }
        }),
        ContentBlock::ToolUse(tc) => json!({
            "type": "tool_use",
            "id": tc.id,
            "name": tc.name,
            "input": tc.input,
        }),
        ContentBlock::ToolResult(r) => json!({
            "type": "tool_result",
            "tool_use_id": r.tool_use_id,
            "content": r.content,
            "is_error": r.is_error,
        }),
    }
}

/// 转换为 OpenAI Chat Completions API 格式。
///
/// 特点：
/// - tool_result 作为独立 role: "tool" 消息（不在 user content 内）
/// - assistant 的 tool_calls 为数组
/// - image 为 data URL
pub fn to_openai_messages(messages: &[Message]) -> Vec<Value> {
    let mut result = Vec::new();

    for msg in messages {
        match msg {
            Message::User(u) => {
                // OpenAI 要求 tool_result 作为独立的 role: "tool" 消息
                let mut texts: Vec<Value> = Vec::new();
                for block in &u.content {
                    match block {
                        ContentBlock::Text(t) => {
                            texts.push(json!(t.text));
                        }
                        ContentBlock::Image(img) => {
                            texts.push(json!(format!(
                                "data:{};base64,{}",
                                img.media_type, img.data
                            )));
                        }
                        ContentBlock::ToolResult(r) => {
                            // flush 之前的文本
                            flush_openai_texts(&mut texts, &mut result);
                            result.push(json!({
                                "role": "tool",
                                "tool_call_id": r.tool_use_id,
                                "content": r.content,
                            }));
                        }
                        _ => {}
                    }
                }
                flush_openai_texts(&mut texts, &mut result);
            }
            Message::Assistant(a) => {
                let mut content_parts: Vec<Value> = Vec::new();
                let mut tool_calls: Vec<Value> = Vec::new();

                for block in &a.content {
                    match block {
                        ContentBlock::Text(t) => {
                            content_parts.push(json!(t.text));
                        }
                        ContentBlock::ToolUse(tc) => {
                            tool_calls.push(json!({
                                "id": tc.id,
                                "type": "function",
                                "function": {
                                    "name": tc.name,
                                    "arguments": tc.input,
                                }
                            }));
                        }
                        _ => {}
                    }
                }

                let mut msg = json!({"role": "assistant"});
                if !content_parts.is_empty() {
                    msg["content"] = json!(content_parts
                        .iter()
                        .filter_map(|v| v.as_str())
                        .collect::<Vec<_>>()
                        .join(""));
                }
                if !tool_calls.is_empty() {
                    msg["tool_calls"] = json!(tool_calls);
                }
                result.push(msg);
            }
            Message::ToolResult(tr) => {
                for block in &tr.content {
                    if let ContentBlock::ToolResult(r) = block {
                        result.push(json!({
                            "role": "tool",
                            "tool_call_id": r.tool_use_id,
                            "content": r.content,
                        }));
                    }
                }
            }
        }
    }

    result
}

/// flush OpenAI 文本缓冲区为一条 user 消息。
fn flush_openai_texts(texts: &mut Vec<Value>, result: &mut Vec<Value>) {
    if texts.is_empty() {
        return;
    }
    if texts.len() == 1 {
        result.push(json!({"role": "user", "content": texts[0]}));
    } else {
        result.push(json!({"role": "user", "content": texts.clone()}));
    }
    texts.clear();
}

/// 转换为 Gemini API 格式。
///
/// 特点：
/// - role 为 "user" / "model"
/// - 工具为 functionCall / functionResponse
/// - thinking 为 thought: true
pub fn to_gemini_contents(messages: &[Message]) -> Vec<Value> {
    let mut result = Vec::new();

    for msg in messages {
        match msg {
            Message::User(u) => {
                let parts: Vec<Value> = u.content.iter().map(block_to_gemini_part).collect();
                result.push(json!({"role": "user", "parts": parts}));
            }
            Message::Assistant(a) => {
                let parts: Vec<Value> = a.content.iter().map(block_to_gemini_part).collect();
                result.push(json!({"role": "model", "parts": parts}));
            }
            Message::ToolResult(tr) => {
                let parts: Vec<Value> = tr.content.iter().map(block_to_gemini_part).collect();
                result.push(json!({"role": "user", "parts": parts}));
            }
        }
    }

    result
}

/// 单个 ContentBlock → Gemini part 格式。
fn block_to_gemini_part(block: &ContentBlock) -> Value {
    match block {
        ContentBlock::Text(t) => json!({"text": t.text}),
        ContentBlock::Thinking(t) => json!({"thought": true, "text": t.thinking}),
        ContentBlock::Image(img) => json!({
            "inlineData": {
                "mimeType": img.media_type,
                "data": img.data,
            }
        }),
        ContentBlock::ToolUse(tc) => json!({
            "functionCall": {
                "name": tc.name,
                "args": tc.input,
            }
        }),
        ContentBlock::ToolResult(r) => json!({
            "functionResponse": {
                "name": r.tool_use_id,
                "response": {"content": r.content},
            }
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pi_types::message::{
        AssistantMessage, ToolCall, ToolResult as ToolResultContent, ToolResultMessage, UserMessage,
    };

    fn text_user(text: &str) -> Message {
        Message::User(UserMessage {
            role: "user".to_string(),
            content: vec![ContentBlock::text(text)],
        })
    }

    fn text_assistant(text: &str) -> Message {
        Message::Assistant(AssistantMessage {
            role: "assistant".to_string(),
            content: vec![ContentBlock::text(text)],
            model: None,
            stop_reason: None,
            usage: None,
            timestamp: None,
        })
    }

    fn tool_call_assistant(id: &str, name: &str) -> Message {
        Message::Assistant(AssistantMessage {
            role: "assistant".to_string(),
            content: vec![ContentBlock::tool_call(id, name, json!({}))],
            model: None,
            stop_reason: None,
            usage: None,
            timestamp: None,
        })
    }

    fn tool_result_user(id: &str, output: &str) -> Message {
        Message::User(UserMessage {
            role: "user".to_string(),
            content: vec![ContentBlock::tool_result(id, output, false)],
        })
    }

    // ── Anthropic ──────────────────────────────────────────────

    #[test]
    fn anthropic_simple_conversation() {
        let msgs = vec![text_user("hello"), text_assistant("hi there")];
        let result = to_anthropic_messages(&msgs);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0]["role"], "user");
        assert_eq!(result[0]["content"][0]["type"], "text");
        assert_eq!(result[0]["content"][0]["text"], "hello");
        assert_eq!(result[1]["role"], "assistant");
    }

    #[test]
    fn anthropic_tool_result_in_user_content() {
        let msgs = vec![
            tool_call_assistant("tc1", "bash"),
            tool_result_user("tc1", "output"),
        ];
        let result = to_anthropic_messages(&msgs);

        // tool_result 在 user 的 content 内
        assert_eq!(result[1]["role"], "user");
        assert_eq!(result[1]["content"][0]["type"], "tool_result");
        assert_eq!(result[1]["content"][0]["tool_use_id"], "tc1");
    }

    #[test]
    fn anthropic_thinking_block() {
        let msgs = vec![Message::Assistant(AssistantMessage {
            role: "assistant".to_string(),
            content: vec![
                ContentBlock::Thinking(pi_types::message::ThinkingContent {
                    thinking: "hmm".to_string(),
                    signature: None,
                }),
                ContentBlock::text("answer"),
            ],
            model: None,
            stop_reason: None,
            usage: None,
            timestamp: None,
        })];
        let result = to_anthropic_messages(&msgs);

        assert_eq!(result[0]["content"][0]["type"], "thinking");
        assert_eq!(result[0]["content"][1]["type"], "text");
    }

    // ── OpenAI ────────────────────────────────────────────────

    #[test]
    fn openai_simple_conversation() {
        let msgs = vec![text_user("hello"), text_assistant("hi")];
        let result = to_openai_messages(&msgs);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0]["role"], "user");
        assert_eq!(result[1]["role"], "assistant");
    }

    #[test]
    fn openai_tool_result_as_separate_message() {
        let msgs = vec![
            tool_call_assistant("tc1", "bash"),
            tool_result_user("tc1", "output"),
        ];
        let result = to_openai_messages(&msgs);

        // tool_result 应该是独立的 role: "tool" 消息
        assert_eq!(result[1]["role"], "tool");
        assert_eq!(result[1]["tool_call_id"], "tc1");
    }

    #[test]
    fn openai_tool_calls_in_assistant() {
        let msgs = vec![tool_call_assistant("tc1", "read")];
        let result = to_openai_messages(&msgs);

        assert_eq!(result[0]["role"], "assistant");
        assert!(result[0]["tool_calls"].is_array());
        assert_eq!(result[0]["tool_calls"][0]["function"]["name"], "read");
    }

    // ── Gemini ────────────────────────────────────────────────

    #[test]
    fn gemini_simple_conversation() {
        let msgs = vec![text_user("hello"), text_assistant("hi")];
        let result = to_gemini_contents(&msgs);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0]["role"], "user");
        assert_eq!(result[1]["role"], "model");
    }

    #[test]
    fn gemini_tool_calls() {
        let msgs = vec![
            tool_call_assistant("tc1", "bash"),
            tool_result_user("tc1", "output"),
        ];
        let result = to_gemini_contents(&msgs);

        assert_eq!(result[0]["role"], "model");
        assert_eq!(result[0]["parts"][0]["functionCall"]["name"], "bash");
        assert_eq!(result[1]["role"], "user");
        assert_eq!(result[1]["parts"][0]["functionResponse"]["name"], "tc1");
    }

    #[test]
    fn gemini_thinking_block() {
        let msgs = vec![Message::Assistant(AssistantMessage {
            role: "assistant".to_string(),
            content: vec![
                ContentBlock::Thinking(pi_types::message::ThinkingContent {
                    thinking: "pondering".to_string(),
                    signature: None,
                }),
                ContentBlock::text("answer"),
            ],
            model: None,
            stop_reason: None,
            usage: None,
            timestamp: None,
        })];
        let result = to_gemini_contents(&msgs);

        assert_eq!(result[0]["parts"][0]["thought"], true);
        assert_eq!(result[0]["parts"][1]["text"], "answer");
    }
}
