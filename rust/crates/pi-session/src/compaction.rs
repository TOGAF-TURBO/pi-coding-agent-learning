//! 会话级压缩 — 将旧消息摘要为单条 summary 条目。
//!
//! 对应 `packages/coding-agent/src/core/compaction/`。
//! 当消息数超过阈值时，将早期消息压缩为一条摘要，
//! 只保留最近的消息以节省 token。

use anyhow::Result;

use pi_types::session::{MessageEntry, SessionEntry};

/// 压缩阈值：超过此消息数触发压缩。
const COMPACTION_THRESHOLD: usize = 40;

/// 压缩结果。
#[derive(Debug)]
pub struct CompactionResult {
    /// 原始消息数。
    pub original_count: usize,
    /// 压缩后消息数。
    pub compacted_count: usize,
    /// 被摘要的消息数。
    pub summarized_count: usize,
}

/// 检查是否需要压缩。
pub fn should_compact(entries: &[SessionEntry]) -> bool {
    let msg_count = entries
        .iter()
        .filter(|e| matches!(e, SessionEntry::Message(_)))
        .count();
    msg_count > COMPACTION_THRESHOLD
}

/// 创建压缩条目 — 将早期消息替换为摘要。
///
/// 保留最后 `keep_recent` 条消息，之前的全部合并为
/// 一条 "system" 角色的 summary 条目。
pub fn create_summary(
    entries: &[SessionEntry],
    summary_text: &str,
    keep_recent: usize,
) -> Result<(MessageEntry, CompactionResult)> {
    let messages: Vec<&SessionEntry> = entries
        .iter()
        .filter(|e| matches!(e, SessionEntry::Message(_)))
        .collect();

    let original_count = messages.len();

    if original_count <= keep_recent {
        return Err(anyhow::anyhow!(
            "Not enough messages to compact ({} <= {})",
            original_count,
            keep_recent
        ));
    }

    let summarized_count = original_count - keep_recent;

    let summary_entry = MessageEntry {
        entry_type: "message".to_string(),
        id: format!("summary-{}", chrono::Utc::now().timestamp_millis()),
        parent_id: None,
        timestamp: chrono::Utc::now().to_rfc3339(),
        role: "system".to_string(),
        content: serde_json::json!([{
            "type": "text",
            "text": format!("[Context Summary]\n{}", summary_text)
        }]),
        model: None,
        stop_reason: None,
        usage: None,
    };

    Ok((
        summary_entry,
        CompactionResult {
            original_count,
            compacted_count: 1 + keep_recent,
            summarized_count,
        },
    ))
}

/// 构建压缩后的条目列表：summary + 最近的 keep_recent 条消息。
pub fn build_compacted_entries(
    entries: &[SessionEntry],
    summary_entry: MessageEntry,
    keep_recent: usize,
) -> Vec<SessionEntry> {
    let messages: Vec<SessionEntry> = entries
        .iter()
        .filter_map(|e| match e {
            SessionEntry::Message(m) if m.role != "system" => Some(e.clone()),
            _ => None,
        })
        .collect();

    let recent: Vec<SessionEntry> = messages.into_iter().rev().take(keep_recent).rev().collect();

    let mut result = vec![SessionEntry::Message(summary_entry)];
    result.extend(recent);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(id: &str, role: &str) -> SessionEntry {
        SessionEntry::Message(MessageEntry {
            entry_type: "message".to_string(),
            id: id.to_string(),
            parent_id: None,
            timestamp: String::new(),
            role: role.to_string(),
            content: serde_json::json!([{"type": "text", "text": format!("msg {}", id)}]),
            model: None,
            stop_reason: None,
            usage: None,
        })
    }

    #[test]
    fn should_compact_below_threshold() {
        let entries: Vec<SessionEntry> = (0..10)
            .map(|i| make_entry(&format!("{}", i), "user"))
            .collect();
        assert!(!should_compact(&entries));
    }

    #[test]
    fn should_compact_above_threshold() {
        let entries: Vec<SessionEntry> = (0..50)
            .map(|i| make_entry(&format!("{}", i), "user"))
            .collect();
        assert!(should_compact(&entries));
    }

    #[test]
    fn create_summary_works() {
        let entries: Vec<SessionEntry> = (0..20)
            .map(|i| {
                make_entry(
                    &format!("{}", i),
                    if i % 2 == 0 { "user" } else { "assistant" },
                )
            })
            .collect();

        let (summary, result) = create_summary(&entries, "Summary text", 5).unwrap();
        assert_eq!(result.original_count, 20);
        assert_eq!(result.summarized_count, 15);
        assert_eq!(result.compacted_count, 6); // 1 summary + 5 recent
        assert_eq!(summary.role, "system");
    }

    #[test]
    fn build_compacted_keeps_recent() {
        let entries: Vec<SessionEntry> = (0..10)
            .map(|i| make_entry(&format!("{}", i), "user"))
            .collect();

        let (summary, _) = create_summary(&entries, "Summary", 3).unwrap();
        let compacted = build_compacted_entries(&entries, summary, 3);

        assert_eq!(compacted.len(), 4); // 1 summary + 3 recent
                                        // 最后 3 条应该保留
        if let SessionEntry::Message(m) = &compacted[3] {
            assert_eq!(m.id, "9"); // 最后一条
        } else {
            panic!("Expected Message entry");
        }
    }

    #[test]
    fn not_enough_messages_fails() {
        let entries = vec![make_entry("1", "user"), make_entry("2", "assistant")];
        assert!(create_summary(&entries, "Summary", 5).is_err());
    }
}
