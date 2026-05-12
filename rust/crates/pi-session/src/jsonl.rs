//! JSONL 会话存储 — 读写与 TS 版本兼容的 JSONL 会话文件。
//!
//! 对应 `packages/agent/src/harness/session/storage/jsonl.ts`。
//!
//! 格式：
//! ```text
//! 第 1 行: {"type":"session","version":3,"id":"...","timestamp":"...","cwd":"..."}
//! 第 2+ 行: {"type":"message","id":"...","parentId":"...","role":"...","content":[...]}
//! ```

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use tokio::fs;
use tokio::io::AsyncWriteExt;

use pi_types::session::{SessionEntry, SessionHeader, SessionMetadata};

/// JSONL 会话存储。
pub struct JsonlSession {
    path: PathBuf,
    header: SessionHeader,
    entries: Vec<SessionEntry>,
    by_id: HashMap<String, usize>,
    current_leaf_id: Option<String>,
}

impl JsonlSession {
    /// 打开已有的 JSONL 会话文件。
    pub async fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let content = fs::read_to_string(&path)
            .await
            .with_context(|| format!("Failed to read session file: {}", path.display()))?;

        let mut lines = content.lines().filter(|l| !l.trim().is_empty());
        let header_line = lines.next().context("Empty session file")?;

        let header: SessionHeader = serde_json::from_str(header_line)
            .with_context(|| format!("Invalid session header in {}", path.display()))?;

        if header.entry_type != "session" {
            bail!("First line is not a session header in {}", path.display());
        }

        let mut entries = Vec::new();
        let mut by_id = HashMap::new();
        let mut current_leaf_id = None;
        let mut corrupted_lines = 0usize;

        for line in lines {
            match serde_json::from_str::<SessionEntry>(line) {
                Ok(entry) => {
                    current_leaf_id = Some(entry.id().to_string());
                    by_id.insert(entry.id().to_string(), entries.len());
                    entries.push(entry);
                }
                Err(_) => {
                    // TS version 跳过格式错误的行（crash 恢复）
                    corrupted_lines += 1;
                }
            }
        }

        if corrupted_lines > 0 {
            eprintln!(
                "Warning: skipped {} corrupted line(s) in session {}",
                corrupted_lines,
                path.display()
            );
        }

        Ok(Self {
            path,
            header,
            entries,
            by_id,
            current_leaf_id,
        })
    }

    /// 创建新的 JSONL 会话文件。
    pub async fn create(path: impl AsRef<Path>, cwd: impl Into<String>) -> Result<Self> {
        let id = Self::generate_id();
        Self::create_with_id(path, cwd, &id).await
    }

    /// 创建新会话文件，使用指定的 ID。
    pub async fn create_with_id(
        path: impl AsRef<Path>,
        cwd: impl Into<String>,
        id: &str,
    ) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let header = SessionHeader::new(id, cwd);

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let header_json = serde_json::to_string(&header)?;
        let mut file = fs::File::create(&path).await?;
        file.write_all(header_json.as_bytes()).await?;
        file.write_all(b"\n").await?;

        Ok(Self {
            path,
            header,
            entries: Vec::new(),
            by_id: HashMap::new(),
            current_leaf_id: None,
        })
    }

    /// 追加条目到会话文件。
    pub async fn append(&mut self, entry: SessionEntry) -> Result<()> {
        let json = serde_json::to_string(&entry)?;
        let mut file = fs::OpenOptions::new().append(true).open(&self.path).await?;
        file.write_all(json.as_bytes()).await?;
        file.write_all(b"\n").await?;

        self.current_leaf_id = Some(entry.id().to_string());
        self.by_id
            .insert(entry.id().to_string(), self.entries.len());
        self.entries.push(entry);
        Ok(())
    }

    /// 获取会话头。
    pub fn header(&self) -> &SessionHeader {
        &self.header
    }

    /// 获取所有条目。
    pub fn entries(&self) -> &[SessionEntry] {
        &self.entries
    }

    /// 获取所有未被归档的条目（跳过 compaction archived_range 覆盖的旧条目）。
    pub fn active_entries(&self) -> Vec<&SessionEntry> {
        // 收集所有被归档的条目 ID
        let mut archived_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
        for entry in &self.entries {
            if let SessionEntry::Compaction(c) = entry {
                if let Some(ref range) = c.archived_range {
                    if range.len() == 2 {
                        let start_idx = self.by_id.get(&range[0]).copied();
                        let end_idx = self.by_id.get(&range[1]).copied();
                        if let (Some(si), Some(ei)) = (start_idx, end_idx) {
                            for i in si..=ei {
                                if let Some(e) = self.entries.get(i) {
                                    archived_ids.insert(e.id().to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        self.entries
            .iter()
            .filter(|e| !archived_ids.contains(e.id()))
            .collect()
    }

    /// 获取当前叶子节点 ID（最后追加的条目）。
    pub fn leaf_id(&self) -> Option<&str> {
        self.current_leaf_id.as_deref()
    }

    /// 按 ID 查找条目。
    pub fn get(&self, id: &str) -> Option<&SessionEntry> {
        self.by_id.get(id).map(|&idx| &self.entries[idx])
    }

    /// 获取从根到指定条目的路径。
    pub fn path_to_root(&self, leaf_id: &str) -> Vec<&SessionEntry> {
        let mut path = Vec::new();
        let mut current = self.get(leaf_id);
        while let Some(entry) = current {
            path.push(entry);
            current = entry.parent_id().and_then(|pid| self.get(pid));
        }
        path.reverse();
        path
    }

    /// 获取会话元数据。
    pub fn metadata(&self) -> SessionMetadata {
        SessionMetadata {
            id: self.header.id.clone(),
            name: None,
            created_at: self.header.timestamp.clone(),
            cwd: self.header.cwd.clone(),
            path: self.path.display().to_string(),
            parent_session_path: self.header.parent_session.clone(),
        }
    }

    /// 会话 ID。
    pub fn id(&self) -> &str {
        &self.header.id
    }

    /// 会话 CWD。
    pub fn cwd(&self) -> &str {
        &self.header.cwd
    }

    /// 条目数。
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// 压缩会话，只保留最后 N 条消息。
    pub async fn compact_keep_last(&mut self, keep: usize) -> anyhow::Result<usize> {
        if self.entries.len() <= keep {
            return Ok(0);
        }
        let removed = self.entries.len() - keep;
        self.entries = self.entries.split_off(self.entries.len() - keep);
        self.rewrite_file().await?;
        Ok(removed)
    }

    /// 重写整个 JSONL 文件。
    async fn rewrite_file(&self) -> anyhow::Result<()> {
        use tokio::io::AsyncWriteExt;
        let tmp_path = self.path.with_extension("jsonl.tmp");
        let mut f = tokio::fs::File::create(&tmp_path).await?;
        for entry in &self.entries {
            let line = serde_json::to_string(entry)?;
            f.write_all(line.as_bytes()).await?;
            f.write_all(b"\n").await?;
        }
        f.flush().await?;
        tokio::fs::rename(&tmp_path, &self.path).await?;
        Ok(())
    }

    fn generate_id() -> String {
        // 短 ID，匹配 TS 版本的 randomUUID().slice(0, 8)
        use std::time::{SystemTime, UNIX_EPOCH};
        let t = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        format!("{:08x}", (t as u32) ^ ((t >> 32) as u32))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn create_and_reopen() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.jsonl");

        let mut session = JsonlSession::create(&path, "/test/cwd").await.unwrap();
        assert_eq!(session.len(), 0);

        let entry = serde_json::from_str(
            r#"{"type":"message","id":"m1","parentId":null,"timestamp":"2026-05-10T12:00:00Z","role":"user","content":[{"type":"text","text":"hello"}]}"#,
        ).unwrap();
        session.append(entry).await.unwrap();
        assert_eq!(session.len(), 1);

        // Reopen
        let session2 = JsonlSession::open(&path).await.unwrap();
        assert_eq!(session2.len(), 1);
        assert_eq!(session2.cwd(), "/test/cwd");
    }

    #[tokio::test]
    async fn path_to_root() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.jsonl");

        let mut session = JsonlSession::create(&path, "/test").await.unwrap();

        // m1 → m2 → m3
        for i in 1..=3 {
            let parent = if i == 1 {
                "null".to_string()
            } else {
                format!(r#""m{}""#, i - 1)
            };
            let json = format!(
                r#"{{"type":"message","id":"m{i}","parentId":{parent},"timestamp":"2026-05-10T12:00:0{i}Z","role":"user","content":[]}}"#
            );
            let entry: SessionEntry = serde_json::from_str(&json).unwrap();
            session.append(entry).await.unwrap();
        }

        let path = session.path_to_root("m3");
        assert_eq!(path.len(), 3);
        assert_eq!(path[0].id(), "m1");
        assert_eq!(path[2].id(), "m3");
    }

    #[tokio::test]
    async fn active_entries_skips_archived() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.jsonl");
        let mut session = JsonlSession::create(&path, "/test").await.unwrap();

        // 追加 m1, m2, m3
        for i in 1..=3 {
            let entry = SessionEntry::Message(pi_types::session::MessageEntry {
                entry_type: "message".to_string(),
                id: format!("m{}", i),
                parent_id: if i == 1 { None } else { Some(format!("m{}", i - 1)) },
                timestamp: format!("2025-01-01T00:00:0{}Z", i),
                role: "user".to_string(),
                content: serde_json::json!([{"type": "text", "text": format!("msg {}", i)}]),
                model: None,
                stop_reason: None,
                usage: None,
            });
            session.append(entry).await.unwrap();
        }

        assert_eq!(session.entries().len(), 3);
        assert_eq!(session.active_entries().len(), 3);

        // 追加 compaction，归档 m1..m2
        let compaction = SessionEntry::Compaction(pi_types::session::CompactionEntry {
            entry_type: "compaction".to_string(),
            id: "c1".to_string(),
            parent_id: Some("m3".to_string()),
            timestamp: "2025-01-01T00:00:04Z".to_string(),
            summary: serde_json::json!({"text": "summary of m1-m2"}),
            archived_range: Some(vec!["m1".to_string(), "m2".to_string()]),
        });
        session.append(compaction).await.unwrap();

        // active_entries 应该只返回 m3 + compaction entry (不归档自身)
        let active = session.active_entries();
        assert_eq!(active.len(), 2); // m3 + compaction
        assert_eq!(active[0].id(), "m3");
        assert_eq!(active[1].id(), "c1");
    }
}
