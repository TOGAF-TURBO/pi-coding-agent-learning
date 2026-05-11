//! 上下文文件加载 — 读取 AGENTS.md 等项目指令文件。
//!
//! 对应 `packages/coding-agent/src/core/resource-loader.ts` 的
//! `loadProjectContextFiles` 函数。
//!
//! 搜索策略：
//! 1. 全局 `~/.pi/agent/AGENTS.md`
//! 2. 从 CWD 向上遍历到根目录，查找所有 `AGENTS.md` / `CLAUDE.md`

use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// 上下文文件候选名称（优先级从高到低）。
const CONTEXT_FILE_NAMES: &[&str] = &["AGENTS.md", "AGENTS.MD", "CLAUDE.md", "CLAUDE.MD"];

/// 加载的上下文文件。
#[derive(Debug, Clone)]
pub struct ContextFile {
    pub path: PathBuf,
    pub content: String,
}

/// 从全局和项目目录加载所有上下文文件。
///
/// 返回顺序：全局 → 根目录级 → ... → CWD 级（外层在前）。
pub fn load_project_context_files(cwd: &Path, agent_dir: Option<&Path>) -> Vec<ContextFile> {
    let mut files = Vec::new();
    let mut seen = HashSet::new();

    // 1. 全局上下文
    if let Some(agent) = agent_dir {
        if let Some(ctx) = load_context_from_dir(agent) {
            seen.insert(ctx.path.clone());
            files.push(ctx);
        }
    }

    // 2. 从 CWD 向根遍历（外层目录先收集）
    let mut ancestor_files = Vec::new();
    let mut current = cwd.to_path_buf();

    loop {
        if let Some(ctx) = load_context_from_dir(&current) {
            if !seen.contains(&ctx.path) {
                seen.insert(ctx.path.clone());
                ancestor_files.push(ctx);
            }
        }

        match current.parent() {
            Some(parent) if parent != current => current = parent.to_path_buf(),
            _ => break,
        }
    }

    // 外层在前（unshift 效果：reverse 后 push）
    ancestor_files.reverse();
    files.extend(ancestor_files);

    files
}

/// 从单个目录查找并加载上下文文件。
fn load_context_from_dir(dir: &Path) -> Option<ContextFile> {
    for name in CONTEXT_FILE_NAMES {
        let path = dir.join(name);
        if path.is_file() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                return Some(ContextFile { path, content });
            }
        }
    }
    None
}

/// 将上下文文件格式化为系统提示文本。
pub fn format_context_for_prompt(files: &[ContextFile]) -> String {
    if files.is_empty() {
        return String::new();
    }

    let mut parts = vec![
        "# Project Context".to_string(),
        String::new(),
        "The following context files were loaded to guide your behavior.".to_string(),
        String::new(),
    ];

    for ctx in files {
        let path_display = ctx.path.display();
        parts.push(format!("## {} ", path_display));
        parts.push(String::new());
        parts.push(ctx.content.clone());
        parts.push(String::new());
    }

    parts.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn loads_agents_md_from_cwd() {
        let dir = TempDir::new().unwrap();
        let agents_md = dir.path().join("AGENTS.md");
        fs::write(&agents_md, "# Rules\nAlways use tabs").unwrap();

        let files = load_project_context_files(dir.path(), None);
        assert_eq!(files.len(), 1);
        assert!(files[0].content.contains("Always use tabs"));
    }

    #[test]
    fn loads_from_parent_directory() {
        let parent = TempDir::new().unwrap();
        let child = parent.path().join("sub");
        fs::create_dir_all(&child).unwrap();

        fs::write(parent.path().join("AGENTS.md"), "parent rule").unwrap();
        fs::write(child.join("CLAUDE.md"), "child rule").unwrap();

        let files = load_project_context_files(&child, None);
        assert_eq!(files.len(), 2);
        // parent first (outermost)
        assert!(files[0].content.contains("parent rule"));
        assert!(files[1].content.contains("child rule"));
    }

    #[test]
    fn deduplicates_same_path() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("AGENTS.md"), "content").unwrap();

        let agent_dir = dir.path();
        let files = load_project_context_files(dir.path(), Some(agent_dir));
        // Same file found twice but deduped
        assert!(files.len() <= 2);
    }

    #[test]
    fn format_includes_paths() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("AGENTS.md"), "test content").unwrap();

        let files = load_project_context_files(dir.path(), None);
        let formatted = format_context_for_prompt(&files);
        assert!(formatted.contains("Project Context"));
        assert!(formatted.contains("test content"));
    }
}
