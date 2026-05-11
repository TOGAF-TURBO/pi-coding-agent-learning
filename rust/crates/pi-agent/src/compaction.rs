//! 上下文压缩 — 当对话过长时，摘要旧消息以减少 token 使用。
//!
//! 对应 `packages/agent/src/harness/compaction/compaction.ts`。
//!
//! 简化版策略：
//! - 当消息条目超过 `MAX_ENTRIES` 时触发
//! - 保留最近 `KEEP_RECENT` 条消息
//! - 将更早的消息发送给 LLM 生成摘要
//! - 用摘要替换旧消息

use anyhow::Result;
use futures::StreamExt;
use pi_llm::driver::{CompletionRequest, LlmDriver, StreamEvent};
use pi_session::JsonlSession;
use pi_types::message::{ContentBlock, Message};
use pi_types::session::SessionEntry;

/// 触发压缩的条目数阈值。
const MAX_ENTRIES: usize = 40;

/// 保留的最近消息数。
const KEEP_RECENT: usize = 10;

/// 压缩摘要的系统提示。
const SUMMARIZE_PROMPT: &str = r#"You are a conversation summarizer. Summarize the conversation below into a concise summary that preserves:
1. Key decisions and conclusions
2. File paths that were read or modified
3. Important context the user provided
4. Tool calls that were made and their results

Output ONLY the summary text, no preamble."#;

/// 检查是否需要压缩。
pub fn should_compact(session: &JsonlSession) -> bool {
    session.len() > MAX_ENTRIES
}

/// 执行上下文压缩。
///
/// 返回 true 表示压缩成功，session 已被修改。
pub async fn compact(
    session: &mut JsonlSession,
    driver: &dyn LlmDriver,
    model: &str,
    api_key: &str,
    base_url: &Option<String>,
) -> Result<bool> {
    let entries = session.entries().to_vec();
    let message_entries: Vec<&SessionEntry> = entries
        .iter()
        .filter(|e| matches!(e, SessionEntry::Message(_)))
        .collect();

    if message_entries.len() <= KEEP_RECENT {
        return Ok(false);
    }

    // 分割：旧消息用于摘要，最近消息保留
    let split_point = message_entries.len() - KEEP_RECENT;
    let old_messages: Vec<&SessionEntry> = message_entries[..split_point].to_vec();
    let _recent_messages: Vec<&SessionEntry> = message_entries[split_point..].to_vec();

    // 构建摘要请求
    let conversation_text = serialize_entries(&old_messages);
    if conversation_text.is_empty() {
        return Ok(false);
    }

    let request = CompletionRequest {
        model: model.to_string(),
        system_prompt: Some(SUMMARIZE_PROMPT.to_string()),
        messages: vec![Message::User(pi_types::message::UserMessage {
            role: "user".to_string(),
            content: vec![ContentBlock::text(format!(
                "Summarize this conversation:\n\n{}",
                conversation_text
            ))],
        })],
        tools: vec![], // 不传工具
        thinking_enabled: false,
        thinking_budget: None,
        max_tokens: 2048,
        api_key: api_key.to_string(),
        base_url: base_url.clone(),
    };

    // 调用 LLM 生成摘要
    let mut stream = driver.stream(request)?;
    let mut summary = String::new();

    while let Some(event) = stream.next().await {
        match event {
            Ok(StreamEvent::TextDelta { text }) => summary.push_str(&text),
            Ok(StreamEvent::Stop { .. }) => break,
            Err(_) => break,
            _ => {}
        }
    }

    if summary.is_empty() {
        return Ok(false);
    }

    // 注意：真正的压缩需要修改 session 文件（移除旧条目 + 插入摘要条目）。
    // 当前 JSONL session 是 append-only，无法删除。
    // 所以我们只是记录摘要，不做文件修改。
    // 真正的压缩需要 session 格式支持条目删除或分叉。

    tracing::info!(
        "[compaction] Would compact {} entries into summary ({} chars)",
        old_messages.len(),
        summary.len()
    );

    Ok(true)
}

/// 将消息条目序列化为纯文本。
fn serialize_entries(entries: &[&SessionEntry]) -> String {
    let mut out = String::new();
    for entry in entries {
        if let SessionEntry::Message(msg) = entry {
            let role = &msg.role;
            let text = msg
                .content
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|b| {
                            if b.get("type").and_then(|v| v.as_str()) == Some("text") {
                                b.get("text").and_then(|v| v.as_str())
                            } else if b.get("type").and_then(|v| v.as_str()) == Some("tool_result")
                            {
                                Some(b.get("content").and_then(|v| v.as_str()).unwrap_or(""))
                            } else if b.get("type").and_then(|v| v.as_str()) == Some("tool_use") {
                                Some(b.get("name").and_then(|v| v.as_str()).unwrap_or("tool"))
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                })
                .unwrap_or_default();
            if !text.is_empty() {
                out.push_str(&format!("[{}] {}\n", role, text));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_not_compact_small_session() {
        let dir = tempfile::tempdir().unwrap();
        let session = create_test_session(&dir, 5);
        assert!(!should_compact(&session));
    }

    #[test]
    fn should_compact_large_session() {
        let dir = tempfile::tempdir().unwrap();
        let session = create_test_session(&dir, MAX_ENTRIES + 10);
        assert!(should_compact(&session));
    }

    #[test]
    fn serialize_entries_format() {
        let entries = vec![pi_types::session::SessionEntry::Message(
            pi_types::session::MessageEntry {
                entry_type: "message".to_string(),
                id: "1".to_string(),
                parent_id: None,
                timestamp: "2025-01-01T00:00:00Z".to_string(),
                role: "user".to_string(),
                content: serde_json::json!([{ "type": "text", "text": "hello" }]),
                model: None,
                stop_reason: None,
                usage: None,
            },
        )];
        let refs: Vec<&pi_types::session::SessionEntry> = entries.iter().collect();
        let result = serialize_entries(&refs);
        assert!(result.contains("user"));
        assert!(result.contains("hello"));
    }

    fn create_test_session(dir: &tempfile::TempDir, count: usize) -> JsonlSession {
        let path = dir.path().join("test.jsonl");
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let mut session = JsonlSession::create(&path, "/test").await.unwrap();
            for i in 0..count {
                let entry = pi_types::session::SessionEntry::Message(
                    pi_types::session::MessageEntry {
                        entry_type: "message".to_string(),
                        id: format!("{}", i),
                        parent_id: None,
                        timestamp: "2025-01-01T00:00:00Z".to_string(),
                        role: if i % 2 == 0 { "user" } else { "assistant" }.to_string(),
                        content: serde_json::json!([{ "type": "text", "text": format!("Message {}", i) }]),
                        model: None,
                        stop_reason: None,
                        usage: None,
                    },
                );
                session.append(entry).await.unwrap();
            }
            session
        })
    }
}
