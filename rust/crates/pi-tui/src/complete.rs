//! Tab 补全 — 混合补全系统。
//!
//! 支持多种来源：
//! - `/` 开头 → slash 命令补全
//! - `@` 开头 → 文件引用补全
//! - 路径片段 (`./`, `~/`, `/`) → 文件路径补全

use std::path::{Path, PathBuf};

/// 补全候选。
#[derive(Debug, Clone)]
pub struct Completion {
    /// 补全文本（替换触发文本的完整路径）。
    pub text: String,
    /// 显示标签。
    pub display: String,
    /// 是否是目录。
    pub is_dir: bool,
}

/// 所有已知的 slash 命令。
const SLASH_COMMANDS: &[(&str, &str)] = &[
    ("/help", "Show available commands"),
    ("/clear", "Clear chat history"),
    ("/compact", "Compact context window"),
    ("/cost", "Show estimated cost"),
    ("/usage", "Show token usage and cost"),
    ("/sessions", "List recent sessions"),
    ("/diff", "Open diff viewer"),
    ("/login", "GitHub Copilot login"),
    ("/logout", "Clear OAuth token"),
    ("/find", "Search across sessions"),
    ("/grep", "Search current session"),
    ("/new", "Start new session"),
    ("/reload", "Reload config files"),
    ("/copy", "Copy last assistant message"),
    ("/fork", "Fork session at message"),
    ("/session", "Show session info"),
    ("/name", "Set session name"),
    ("/export", "Export session (HTML or JSONL)"),
];

/// 从输入文本和光标位置尝试补全。
///
/// 返回 (候选列表, 触发文本范围)。
/// `trigger_start` 是触发文本在 input 中的起始字节偏移。
pub fn complete(input: &str, cursor: usize, cwd: &Path) -> Option<(Vec<Completion>, usize)> {
    let before_cursor = &input[..cursor.min(input.len())];

    // 查找触发点
    let trigger_start = before_cursor
        .rfind(|c: char| c.is_whitespace() || c == '"' || c == '\'')
        .map(|i| i + 1)
        .unwrap_or(0);

    let fragment = &before_cursor[trigger_start..];

    if fragment.is_empty() {
        return None;
    }

    // 1. Slash 命令补全
    if fragment.starts_with('/') {
        return complete_slash_command(fragment, trigger_start);
    }

    // 2. @file 补全
    if fragment.starts_with('@') {
        return complete_at_file(fragment, trigger_start, cwd);
    }

    // 3. 文件路径补全
    complete_file_path(fragment, trigger_start, cwd)
}

/// Slash 命令补全。
fn complete_slash_command(
    fragment: &str,
    trigger_start: usize,
) -> Option<(Vec<Completion>, usize)> {
    let mut candidates = Vec::new();
    for (cmd, desc) in SLASH_COMMANDS {
        if cmd.starts_with(fragment) {
            candidates.push(Completion {
                text: cmd.to_string(),
                display: format!("{:<12} {}", cmd, desc),
                is_dir: false,
            });
        }
    }
    if candidates.is_empty() {
        None
    } else {
        Some((candidates, trigger_start))
    }
}

/// @file 补全 — 以 @ 开头时补全文件路径。
fn complete_at_file(
    fragment: &str,
    trigger_start: usize,
    cwd: &Path,
) -> Option<(Vec<Completion>, usize)> {
    let path_fragment = &fragment[1..]; // 去掉 @

    // 至少需要一个字符或有路径分隔符
    if path_fragment.is_empty() {
        // 列出 cwd 内容
        let entries = std::fs::read_dir(cwd).ok()?;
        let mut candidates = Vec::new();
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') {
                continue;
            }
            let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
            candidates.push(Completion {
                text: format!("@{}", name),
                display: if is_dir {
                    format!("{}/", name)
                } else {
                    name.clone()
                },
                is_dir,
            });
        }
        candidates.sort_by(|a, b| a.display.cmp(&b.display));
        return Some((candidates, trigger_start));
    }

    // 路径补全
    let has_path_sep = path_fragment.contains('/') || path_fragment.starts_with('~');
    if !has_path_sep && !path_fragment.starts_with('.') {
        // 简单文件名，在 cwd 中搜索
        let entries = std::fs::read_dir(cwd).ok()?;
        let mut candidates = Vec::new();
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') && !path_fragment.starts_with('.') {
                continue;
            }
            if !name
                .to_lowercase()
                .starts_with(&path_fragment.to_lowercase())
            {
                continue;
            }
            let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
            candidates.push(Completion {
                text: format!("@{}", name),
                display: if is_dir {
                    format!("{}/", name)
                } else {
                    name.clone()
                },
                is_dir,
            });
        }
        candidates.sort_by(|a, b| a.display.cmp(&b.display));
        return if candidates.is_empty() {
            None
        } else {
            Some((candidates, trigger_start))
        };
    }

    // 展开路径
    let expanded = shellexpand(path_fragment, cwd);
    let trailing_slash = path_fragment.ends_with('/');

    let (dir, prefix) = if trailing_slash && expanded.is_dir() {
        (expanded, String::new())
    } else {
        let dir = expanded.parent().unwrap_or(Path::new(".")).to_path_buf();
        let prefix = expanded
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_default();
        (dir, prefix)
    };

    let entries = std::fs::read_dir(&dir).ok()?;
    let mut candidates = Vec::new();

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') && !prefix.starts_with('.') && !prefix.is_empty() {
            continue;
        }
        if !name.to_lowercase().starts_with(&prefix.to_lowercase()) {
            continue;
        }

        let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
        let display = if is_dir {
            format!("{}/", name)
        } else {
            name.clone()
        };

        // 生成 @ 路径
        let full = dir.join(&name);
        let at_path = if path_fragment.starts_with('~') {
            let home = dir_home();
            format!(
                "@~/{}",
                full.strip_prefix(&home).unwrap_or(&full).to_string_lossy()
            )
        } else if path_fragment.starts_with('/') {
            format!("@{}", full.to_string_lossy())
        } else {
            let rel = pathdiff(&full, cwd);
            format!(
                "@{}",
                rel.unwrap_or_else(|| full.to_string_lossy().to_string())
            )
        };

        candidates.push(Completion {
            text: at_path,
            display,
            is_dir,
        });
    }

    if candidates.is_empty() {
        return None;
    }
    candidates.sort_by(|a, b| a.display.cmp(&b.display));
    Some((candidates, trigger_start))
}

/// 文件路径补全。
fn complete_file_path(
    fragment: &str,
    trigger_start: usize,
    cwd: &Path,
) -> Option<(Vec<Completion>, usize)> {
    let has_path_sep = fragment.contains('/') || fragment.starts_with('~');
    if !has_path_sep && !fragment.starts_with('.') {
        return None;
    }

    let expanded = shellexpand(fragment, cwd);
    let trailing_slash = fragment.ends_with('/') || fragment.ends_with("/.");

    let (dir, prefix) = if trailing_slash && expanded.is_dir() {
        (expanded.clone(), String::new())
    } else {
        let dir = expanded.parent().unwrap_or(Path::new(".")).to_path_buf();
        let prefix = expanded
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_default();
        (dir, prefix)
    };

    let entries = std::fs::read_dir(&dir).ok()?;
    let mut candidates = Vec::new();

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') && !prefix.starts_with('.') && !prefix.is_empty() {
            continue;
        }
        if !name.to_lowercase().starts_with(&prefix.to_lowercase()) {
            continue;
        }

        let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
        let full_path = dir.join(&name);

        let completion_text = if fragment.starts_with('~') {
            format!(
                "~/{}",
                full_path
                    .strip_prefix(dir_home())
                    .unwrap_or(&full_path)
                    .to_string_lossy()
            )
        } else if fragment.starts_with('/') {
            full_path.to_string_lossy().to_string()
        } else {
            let rel = pathdiff(&full_path, cwd);
            rel.unwrap_or_else(|| full_path.to_string_lossy().to_string())
        };

        let display = if is_dir {
            format!("{}/", name)
        } else {
            name.clone()
        };

        candidates.push(Completion {
            text: if is_dir {
                format!("{}/", completion_text.trim_end_matches('/'))
            } else {
                completion_text
            },
            display,
            is_dir,
        });
    }

    if candidates.is_empty() {
        return None;
    }
    candidates.sort_by(|a, b| a.display.cmp(&b.display));
    Some((candidates, trigger_start))
}

/// 展开 ~/ 和相对路径。
fn shellexpand(path: &str, cwd: &Path) -> PathBuf {
    let raw = if path.starts_with('~') {
        let home = dir_home();
        let rest = path.strip_prefix('~').unwrap_or(path);
        let rest = rest.strip_prefix('/').unwrap_or(rest);
        home.join(rest)
    } else if path.starts_with('/') {
        PathBuf::from(path)
    } else {
        cwd.join(path)
    };

    let mut normalized = PathBuf::new();
    for comp in raw.components() {
        match comp {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other),
        }
    }
    normalized
}

/// 获取 home 目录。
fn dir_home() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"))
}

/// 简易相对路径计算。
fn pathdiff(path: &Path, base: &Path) -> Option<String> {
    let path_str = path.to_string_lossy();
    let base_str = base.to_string_lossy();
    if path_str.starts_with(base_str.as_ref()) {
        let rel = &path_str[base_str.len()..];
        let rel = rel.strip_prefix('/').unwrap_or(rel);
        if rel.is_empty() {
            return None;
        }
        Some(rel.to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn slash_command_completion() {
        let result = complete("/co", 3, Path::new("/tmp"));
        assert!(result.is_some());
        let (candidates, start) = result.unwrap();
        assert_eq!(start, 0);
        assert!(candidates.iter().any(|c| c.text == "/compact"));
        assert!(candidates.iter().any(|c| c.text == "/cost"));
    }

    #[test]
    fn slash_command_help() {
        let result = complete("/h", 2, Path::new("/tmp"));
        assert!(result.is_some());
        let (candidates, _) = result.unwrap();
        assert!(candidates.iter().any(|c| c.text == "/help"));
    }

    #[test]
    fn at_file_completion() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("hello.rs"), "fn main() {}").unwrap();
        fs::write(dir.path().join("world.txt"), "content").unwrap();

        let result = complete("@", 1, dir.path());
        assert!(result.is_some());
        let (candidates, _) = result.unwrap();
        assert!(candidates.iter().any(|c| c.text == "@hello.rs"));
        assert!(candidates.iter().any(|c| c.text == "@world.txt"));
    }

    #[test]
    fn at_file_with_prefix() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("hello.rs"), "").unwrap();
        fs::write(dir.path().join("hello_test.rs"), "").unwrap();
        fs::write(dir.path().join("other.txt"), "").unwrap();

        let result = complete("@hello", 6, dir.path());
        assert!(result.is_some());
        let (candidates, _) = result.unwrap();
        assert_eq!(candidates.len(), 2);
    }

    #[test]
    fn no_complete_plain_text() {
        let cwd = PathBuf::from("/tmp");
        assert!(complete("hello world", 11, &cwd).is_none());
    }

    #[test]
    fn complete_dot_slash() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join("src_unique_a7")).unwrap();
        fs::write(dir.path().join("Cargo_unique_b3.toml"), "").unwrap();

        let result = complete("./", 2, dir.path());
        assert!(result.is_some());
        let (candidates, _) = result.unwrap();
        assert!(candidates.len() >= 2);
    }

    #[test]
    fn complete_after_space() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("test.rs"), "").unwrap();

        let result = complete("read ./test", 11, dir.path());
        assert!(result.is_some());
        let (candidates, start) = result.unwrap();
        assert_eq!(start, 5);
        assert!(candidates.iter().any(|c| c.display.contains("test.rs")));
    }

    #[test]
    fn shellexpand_tilde() {
        let cwd = PathBuf::from("/tmp");
        let expanded = shellexpand("~/foo", &cwd);
        let home = dirs::home_dir().unwrap();
        assert_eq!(expanded, home.join("foo"));
    }

    #[test]
    fn shellexpand_relative() {
        let cwd = PathBuf::from("/project");
        let expanded = shellexpand("src/main.rs", &cwd);
        assert_eq!(expanded, PathBuf::from("/project/src/main.rs"));
    }
}
