//! @file 引用解析 — 在用户消息中检测 @path 模式并内联文件内容。
//!
//! 对应 `packages/coding-agent/src/utils/paths.ts`。

use std::path::{Path, PathBuf};

/// 解析消息中的 @file 引用，返回文件内容和清理后的消息。
///
/// 支持格式：
/// - `@file.txt` — 单个文件
/// - `@dir/file.rs` — 带路径的文件
/// - `@./relative/path` — 相对路径
///
/// 如果文件存在，将其内容作为上下文注入到消息前面。
pub fn resolve_file_refs(message: &str, cwd: &Path) -> (String, Vec<FileRef>) {
    let mut refs = Vec::new();
    let mut cleaned = message.to_string();

    // 查找所有 @xxx 模式
    let re = regex::Regex::new(r"@(\.{0,2}/[^\s,;)]+|@[^\s,;)]+|[a-zA-Z0-9_./-]+\.[a-zA-Z0-9]+)").unwrap();

    for cap in re.captures_iter(message) {
        let full = &cap[0];
        let file_path = &full[1..]; // 去掉 @ 前缀

        // 跳过明显不是文件的模式
        if file_path.starts_with('@') // @@ 转义
            || file_path.contains("://") // URL
            || file_path.starts_with('{') // JSON
        {
            continue;
        }

        let resolved = if file_path.starts_with('/') {
            PathBuf::from(file_path)
        } else {
            cwd.join(file_path)
        };

        if resolved.exists() && resolved.is_file() {
            if let Ok(content) = std::fs::read_to_string(&resolved) {
                let rel = pathdiff::diff_paths(&resolved, cwd)
                    .unwrap_or_else(|| resolved.clone());
                let display = rel.to_string_lossy();

                // 检测是否为二进制文件
                if is_binary_content(&content) {
                    refs.push(FileRef {
                        path: display.to_string(),
                        content: format!("[binary file: {} ({} bytes)]", display, content.len()),
                        is_binary: true,
                    });
                } else {
                    refs.push(FileRef {
                        path: display.to_string(),
                        content,
                        is_binary: false,
                    });
                }

                // 从消息中移除 @file 引用
                cleaned = cleaned.replace(full, "");
            }
        }
    }

    // 构建最终消息
    let final_message = if refs.is_empty() {
        cleaned.trim().to_string()
    } else {
        let mut parts = Vec::new();
        for f in &refs {
            parts.push(format!("--- {} ---\n{}\n--- end of {} ---", f.path, f.content, f.path));
        }
        let file_context = parts.join("\n\n");
        let user_text = cleaned.trim();
        if user_text.is_empty() {
            file_context
        } else {
            format!("{}\n\n{}", file_context, user_text)
        }
    };

    (final_message, refs)
}

/// 检测内容是否为二进制。
fn is_binary_content(content: &str) -> bool {
    // 简单启发式：如果前 8KB 中有 NUL 字节则视为二进制
    let check_len = content.len().min(8192);
    content.as_bytes()[..check_len].contains(&0)
}

/// 文件引用。
#[derive(Debug, Clone)]
pub struct FileRef {
    pub path: String,
    pub content: String,
    pub is_binary: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn resolve_existing_file() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("hello.txt"), "hello world").unwrap();

        let (msg, refs) = resolve_file_refs(
            "Please review @hello.txt and fix bugs",
            dir.path(),
        );

        assert!(msg.contains("hello world"));
        assert!(msg.contains("Please review"));
        assert!(msg.contains("fix bugs"));
        assert!(!msg.contains("@hello.txt"));
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].path, "hello.txt");
    }

    #[test]
    fn no_file_ref() {
        let (msg, refs) = resolve_file_refs("Just a normal message", Path::new("."));
        assert_eq!(msg, "Just a normal message");
        assert!(refs.is_empty());
    }

    #[test]
    fn non_existent_file_ignored() {
        let (msg, refs) = resolve_file_refs("Review @nonexistent.txt please", Path::new("."));
        assert!(refs.is_empty());
        assert!(msg.contains("@nonexistent.txt"));
    }

    #[test]
    fn relative_path() {
        let dir = TempDir::new().unwrap();
        let sub = dir.path().join("src");
        fs::create_dir_all(&sub).unwrap();
        fs::write(sub.join("main.rs"), "fn main() {}").unwrap();

        let (msg, refs) = resolve_file_refs(
            "Check @src/main.rs",
            dir.path(),
        );

        assert_eq!(refs.len(), 1);
        assert!(refs[0].path.ends_with("main.rs"));
        assert!(msg.contains("fn main()"));
    }

    #[test]
    fn absolute_path() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("test.txt");
        fs::write(&file, "content").unwrap();

        let abs = file.to_string_lossy().to_string();
        let (msg, refs) = resolve_file_refs(
            &format!("@{}", abs),
            Path::new("/tmp"),
        );

        assert_eq!(refs.len(), 1);
        assert!(msg.contains("content"));
    }

    #[test]
    fn binary_detection() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("binary.dat"), b"hello\x00world").unwrap();

        let (_, refs) = resolve_file_refs("@binary.dat", dir.path());
        assert_eq!(refs.len(), 1);
        assert!(refs[0].is_binary);
        assert!(refs[0].content.contains("binary file"));
    }

    #[test]
    fn multiple_refs() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("a.txt"), "aaa").unwrap();
        fs::write(dir.path().join("b.txt"), "bbb").unwrap();

        let (msg, refs) = resolve_file_refs("@a.txt and @b.txt", dir.path());
        assert_eq!(refs.len(), 2);
        assert!(msg.contains("aaa"));
        assert!(msg.contains("bbb"));
    }
}
