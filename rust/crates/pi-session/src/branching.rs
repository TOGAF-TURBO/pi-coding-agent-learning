//! 会话分支 — 从已有条目创建分支。
//!
//! 对应 TS 版本的 `branch()` 功能。
//! 会话是 append-only 的，分支通过设置 `parent_id` 指向
//! 更早的条目来实现。这样可以在不修改已有数据的情况下
//! 创建新的对话路径。

use anyhow::Result;

use pi_types::session::SessionEntry;

/// 分支创建结果。
#[derive(Debug)]
pub struct BranchResult {
    /// 新分支的起始条目 ID。
    pub branch_entry_id: String,
    /// 从分支点截断的消息数量。
    pub pruned_count: usize,
}

/// 从指定条目创建分支。
///
/// `entries` 是当前会话的所有条目。
/// `from_entry_id` 是要分支出去的条目 ID。
///
/// 返回分支点之后被截断的条目数量。
/// 调用方应在追加新消息时设置 parent_id 指向 `from_entry_id`。
pub fn create_branch(
    entries: &[SessionEntry],
    from_entry_id: &str,
) -> Result<BranchResult> {
    // 找到分支点在 entries 中的索引
    let branch_idx = entries.iter().position(|e| match e {
        SessionEntry::Message(m) => m.id == from_entry_id,
        _ => false,
    }).ok_or_else(|| anyhow::anyhow!("Entry {} not found", from_entry_id))?;

    // 计算分支点之后的条目数
    let pruned_count = entries.len() - branch_idx - 1;

    Ok(BranchResult {
        branch_entry_id: from_entry_id.to_string(),
        pruned_count,
    })
}

/// 获取从根到指定条目的线性路径（忽略其他分支）。
pub fn lineage_to(
    entries: &[SessionEntry],
    target_id: &str,
) -> Vec<SessionEntry> {
    // 建立 id → entry 映射
    let mut id_map: std::collections::HashMap<String, &SessionEntry> =
        std::collections::HashMap::new();
    for e in entries {
        if let SessionEntry::Message(m) = e {
            id_map.insert(m.id.clone(), e);
        }
    }

    // 从 target 回溯到根
    let mut path = Vec::new();
    let mut current_id = Some(target_id.to_string());

    while let Some(id) = current_id {
        if let Some(entry) = id_map.get(&id) {
            if let SessionEntry::Message(m) = entry {
                current_id = m.parent_id.clone();
                path.push((*entry).clone());
            } else {
                break;
            }
        } else {
            break;
        }
    }

    path.reverse();
    path
}

#[cfg(test)]
mod tests {
    use super::*;
    use pi_types::session::{SessionEntry, MessageEntry};

    fn make_entry(id: &str, parent_id: Option<&str>, role: &str) -> SessionEntry {
        SessionEntry::Message(MessageEntry {
            entry_type: "message".to_string(),
            id: id.to_string(),
            parent_id: parent_id.map(|s| s.to_string()),
            timestamp: String::new(),
            role: role.to_string(),
            content: serde_json::json!([{"type": "text", "text": format!("msg {}", id)}]),
            model: None,
            stop_reason: None,
            usage: None,
        })
    }

    #[test]
    fn create_branch_prunes_after_point() {
        let entries = vec![
            make_entry("a", None, "user"),
            make_entry("b", Some("a"), "assistant"),
            make_entry("c", Some("b"), "user"),
            make_entry("d", Some("c"), "assistant"),
            make_entry("e", Some("d"), "user"),
        ];

        let result = create_branch(&entries, "c").unwrap();
        assert_eq!(result.pruned_count, 2); // d, e
        assert_eq!(result.branch_entry_id, "c");
    }

    #[test]
    fn branch_at_last_entry() {
        let entries = vec![
            make_entry("a", None, "user"),
            make_entry("b", Some("a"), "assistant"),
        ];

        let result = create_branch(&entries, "b").unwrap();
        assert_eq!(result.pruned_count, 0);
    }

    #[test]
    fn branch_at_unknown_entry_fails() {
        let entries = vec![make_entry("a", None, "user")];
        assert!(create_branch(&entries, "z").is_err());
    }

    #[test]
    fn lineage_traces_parent_chain() {
        let entries = vec![
            make_entry("a", None, "user"),
            make_entry("b", Some("a"), "assistant"),
            make_entry("c", Some("b"), "user"),
        ];

        let lineage = lineage_to(&entries, "c");
        assert_eq!(lineage.len(), 3);

        let lineage = lineage_to(&entries, "b");
        assert_eq!(lineage.len(), 2);
    }
}
