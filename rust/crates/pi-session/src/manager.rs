//! 会话管理器 — 创建、列出、恢复、分叉会话。
//!
//! 对应 `packages/coding-agent/src/core/session-manager.ts`。
//!
//! 会话存储在 `~/.pi/agent/sessions/<session-id>/session.jsonl`。

use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow, bail};
use chrono::DateTime;
use tokio::fs;

use crate::JsonlSession;

/// 会话管理器。
pub struct SessionManager {
    sessions_dir: PathBuf,
}

/// 会话列表条目（简要信息）。
#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub id: String,
    pub path: PathBuf,
    pub created_at: String,
    pub cwd: String,
    pub message_count: usize,
    pub is_today: bool,
}

impl SessionManager {
    /// 创建会话管理器。
    pub fn new(sessions_dir: impl Into<PathBuf>) -> Self {
        Self {
            sessions_dir: sessions_dir.into(),
        }
    }

    /// 确保会话目录存在。
    pub async fn ensure_dir(&self) -> Result<()> {
        fs::create_dir_all(&self.sessions_dir).await?;
        Ok(())
    }

    /// 创建新会话。
    pub async fn create(&self, cwd: &str) -> Result<JsonlSession> {
        self.ensure_dir().await?;
        let id = generate_session_id();
        let dir = self.sessions_dir.join(&id);
        fs::create_dir_all(&dir).await?;
        let path = dir.join("session.jsonl");
        JsonlSession::create_with_id(&path, cwd, &id).await
    }

    /// 打开已有会话（按 ID 或目录名）。
    pub async fn open(&self, session_id: &str) -> Result<JsonlSession> {
        // 1. 直接路径尝试
        let path = self.sessions_dir.join(session_id).join("session.jsonl");
        if path.exists() {
            return JsonlSession::open(&path).await;
        }

        // 2. 在所有会话中按 header ID 搜索
        let sessions = self.list().await?;
        for s in &sessions {
            if s.id == session_id {
                return JsonlSession::open(&s.path).await;
            }
        }

        bail!("Session '{}' not found", session_id)
    }

    /// 恢复最近的会话（`--continue`）。
    pub async fn continue_last(&self) -> Result<JsonlSession> {
        let sessions = self.list().await?;
        let latest = sessions.first().ok_or_else(|| anyhow!("No previous sessions found"))?;
        self.open(&latest.id).await
    }

    /// 列出所有会话（按时间倒序）。
    pub async fn list(&self) -> Result<Vec<SessionInfo>> {
        if !self.sessions_dir.exists() {
            return Ok(Vec::new());
        }

        let mut entries = fs::read_dir(&self.sessions_dir).await?;
        let mut sessions = Vec::new();

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let session_file = path.join("session.jsonl");
            if !session_file.exists() {
                continue;
            }

            match JsonlSession::open(&session_file).await {
                Ok(session) => {
                    let header = session.header();
                    let created = &header.timestamp;
                    let is_today = is_today(created);

                    sessions.push(SessionInfo {
                        id: header.id.clone(),
                        path: session_file,
                        created_at: created.clone(),
                        cwd: header.cwd.clone(),
                        message_count: session.len(),
                        is_today,
                    });
                }
                Err(_) => continue,
            }
        }

        // 按创建时间倒序（最新在前）
        sessions.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(sessions)
    }

    /// 删除会话。
    pub async fn delete(&self, session_id: &str) -> Result<()> {
        let dir = self.sessions_dir.join(session_id);
        if dir.exists() {
            fs::remove_dir_all(&dir).await?;
        }
        Ok(())
    }

    /// 按部分 ID 查找会话。
    pub async fn find_by_prefix(&self, prefix: &str) -> Result<Option<SessionInfo>> {
        let sessions = self.list().await?;
        let matches: Vec<_> = sessions.iter().filter(|s| s.id.starts_with(prefix)).collect();
        match matches.len() {
            0 => Ok(None),
            1 => Ok(Some(matches[0].clone())),
            _ => Err(anyhow!("Ambiguous session prefix '{}', matches {} sessions", prefix, matches.len())),
        }
    }

    /// 会话目录路径。
    pub fn sessions_dir(&self) -> &Path {
        &self.sessions_dir
    }
}

/// 生成会话 ID（时间戳 + 随机后缀，如 "20260511-1430-a3f2"）。
fn generate_session_id() -> String {
    let now = chrono::Local::now();
    let ts = now.format("%Y%m%d-%H%M").to_string();
    let rand_part = {
        use std::time::{SystemTime, UNIX_EPOCH};
        let t = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        format!("{:04x}", (t as u16) ^ ((t >> 16) as u16))
    };
    format!("{}-{}", ts, rand_part)
}

/// 检查时间戳是否是今天。
fn is_today(timestamp: &str) -> bool {
    let Ok(dt) = DateTime::parse_from_rfc3339(timestamp) else {
        return false;
    };
    let today = chrono::Local::now().date_naive();
    dt.date_naive() == today
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn create_and_list_sessions() {
        let dir = TempDir::new().unwrap();
        let mgr = SessionManager::new(dir.path());

        let mut s1 = mgr.create("/project/a").await.unwrap();
        let mut s2 = mgr.create("/project/b").await.unwrap();

        // 追加消息
        let entry = serde_json::from_str(
            r#"{"type":"message","id":"m1","parentId":null,"timestamp":"2026-05-11T12:00:00Z","role":"user","content":[{"type":"text","text":"hello"}]}"#,
        ).unwrap();
        s1.append(entry).await.unwrap();

        let sessions = mgr.list().await.unwrap();
        assert_eq!(sessions.len(), 2);
        // 最新在前
        assert_eq!(sessions[0].message_count, 0); // s2 没有 messages
        assert_eq!(sessions[1].message_count, 1); // s1 有 1 message
    }

    #[tokio::test]
    async fn continue_last_session() {
        let dir = TempDir::new().unwrap();
        let mgr = SessionManager::new(dir.path());

        let mut s1 = mgr.create("/project/a").await.unwrap();
        let entry = serde_json::from_str(
            r#"{"type":"message","id":"m1","parentId":null,"timestamp":"2026-05-11T12:00:00Z","role":"user","content":[{"type":"text","text":"hello"}]}"#,
        ).unwrap();
        s1.append(entry).await.unwrap();

        let s2 = mgr.continue_last().await.unwrap();
        assert_eq!(s2.len(), 1); // 应该恢复了 s1 的消息
    }

    #[tokio::test]
    async fn find_by_prefix() {
        let dir = TempDir::new().unwrap();
        let mgr = SessionManager::new(dir.path());

        let s1 = mgr.create("/test").await.unwrap();
        let id = s1.id();

        let found = mgr.find_by_prefix(&id[..4]).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, id);
    }
}
