//! @file 引用解析 — 在用户消息中检测 @path 模式并内联文件内容。
//!
//! 支持文本文件内联和图片文件 base64 编码。

use std::path::{Path, PathBuf};

/// 支持的图片扩展名 → MIME 类型。
const IMAGE_TYPES: &[(&str, &str)] = &[
    (".png", "image/png"),
    (".jpg", "image/jpeg"),
    (".jpeg", "image/jpeg"),
    (".gif", "image/gif"),
    (".webp", "image/webp"),
];

/// 根据扩展名检测图片 MIME 类型。
pub fn detect_image_mime(path: &Path) -> Option<&'static str> {
    let ext = path.extension()?.to_str()?.to_lowercase();
    IMAGE_TYPES
        .iter()
        .find(|(e, _)| e.strip_prefix('.').unwrap_or(e) == ext)
        .map(|(_, mime)| *mime)
}

/// 文件引用结果。
#[derive(Debug, Clone)]
pub struct FileRef {
    pub path: String,
    pub content: String,
    pub is_binary: bool,
    /// 如果是图片文件，包含 (mime_type, base64_data)。
    pub image: Option<ImageData>,
}

/// 图片数据。
#[derive(Debug, Clone)]
pub struct ImageData {
    pub mime_type: String,
    pub base64: String,
}

/// 解析消息中的 @file 引用，返回文件内容和清理后的消息。
///
/// 支持格式：
/// - `@file.txt` — 单个文件（文本内联）
/// - `@photo.png` — 图片文件（base64 编码）
/// - `@dir/file.rs` — 带路径的文件
/// - `@./relative/path` — 相对路径
pub fn resolve_file_refs(message: &str, cwd: &Path) -> (String, Vec<FileRef>) {
    let mut refs = Vec::new();
    let mut cleaned = message.to_string();

    let re = regex::Regex::new(r"@(\.{0,2}/[^\s,;)]+|@[^\s,;)]+|[a-zA-Z0-9_./-]+\.[a-zA-Z0-9]+)")
        .unwrap();

    for cap in re.captures_iter(message) {
        let full = &cap[0];
        let file_path = &full[1..];

        if file_path.starts_with('@') || file_path.contains("://") || file_path.starts_with('{') {
            continue;
        }

        let resolved = if file_path.starts_with('/') {
            PathBuf::from(file_path)
        } else {
            cwd.join(file_path)
        };

        if !resolved.exists() || !resolved.is_file() {
            continue;
        }

        let rel = pathdiff::diff_paths(&resolved, cwd).unwrap_or_else(|| resolved.clone());
        let display = rel.to_string_lossy().to_string();

        // 图片文件：base64 编码（大小限制 1MB）
        if let Some(mime) = detect_image_mime(&resolved) {
            if let Ok(bytes) = std::fs::read(&resolved) {
                const MAX_IMAGE_BYTES: usize = 1024 * 1024;
                if bytes.len() > MAX_IMAGE_BYTES {
                    refs.push(FileRef {
                        path: display,
                        content: format!(
                            "[image too large: {} ({} bytes, max {} bytes)]",
                            file_path,
                            bytes.len(),
                            MAX_IMAGE_BYTES
                        ),
                        is_binary: true,
                        image: None,
                    });
                } else {
                    refs.push(FileRef {
                        path: display,
                        content: format!("[image: {} ({} bytes)]", file_path, bytes.len()),
                        is_binary: true,
                        image: Some(ImageData {
                            mime_type: mime.to_string(),
                            base64: base64_encode(&bytes),
                        }),
                    });
                }
                cleaned = cleaned.replace(full, "");
                continue;
            }
        }

        // 文本文件
        if let Ok(content) = std::fs::read_to_string(&resolved) {
            if is_binary_content(&content) {
                refs.push(FileRef {
                    path: display,
                    content: format!("[binary file: {} ({} bytes)]", file_path, content.len()),
                    is_binary: true,
                    image: None,
                });
            } else {
                refs.push(FileRef {
                    path: display,
                    content,
                    is_binary: false,
                    image: None,
                });
            }
            cleaned = cleaned.replace(full, "");
        }
    }

    let final_message = if refs.is_empty() {
        cleaned.trim().to_string()
    } else {
        let mut parts = Vec::new();
        for f in &refs {
            if f.image.is_some() {
                // 图片已在 image 字段中，不需要文本内联
                continue;
            }
            parts.push(format!(
                "--- {} ---\n{}\n--- end of {} ---",
                f.path, f.content, f.path
            ));
        }
        let file_context = parts.join("\n\n");
        let user_text = cleaned.trim();
        if file_context.is_empty() {
            user_text.to_string()
        } else if user_text.is_empty() {
            file_context
        } else {
            format!("{}\n\n{}", file_context, user_text)
        }
    };

    (final_message, refs)
}

/// 检测内容是否为二进制。
fn is_binary_content(content: &str) -> bool {
    let check_len = content.len().min(8192);
    content.as_bytes()[..check_len].contains(&0)
}

/// Base64 编码（使用标准库之外的简易实现）。
fn base64_encode(data: &[u8]) -> String {
    use base64::{engine::general_purpose::STANDARD, Engine};
    STANDARD.encode(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn resolve_existing_file() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("hello.txt"), "hello world").unwrap();

        let (msg, refs) = resolve_file_refs("Please review @hello.txt and fix bugs", dir.path());

        assert!(msg.contains("hello world"));
        assert!(msg.contains("Please review"));
        assert!(!msg.contains("@hello.txt"));
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].path, "hello.txt");
        assert!(refs[0].image.is_none());
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

        let (msg, refs) = resolve_file_refs("Check @src/main.rs", dir.path());

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
        let (msg, refs) = resolve_file_refs(&format!("@{}", abs), Path::new("/tmp"));

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
        assert!(refs[0].image.is_none());
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

    #[test]
    fn detect_image_mime_png() {
        assert_eq!(detect_image_mime(Path::new("photo.png")), Some("image/png"));
        assert_eq!(detect_image_mime(Path::new("photo.PNG")), Some("image/png"));
    }

    #[test]
    fn detect_image_mime_jpg() {
        assert_eq!(detect_image_mime(Path::new("img.jpg")), Some("image/jpeg"));
        assert_eq!(detect_image_mime(Path::new("img.jpeg")), Some("image/jpeg"));
    }

    #[test]
    fn detect_image_mime_unknown() {
        assert_eq!(detect_image_mime(Path::new("doc.pdf")), None);
        assert_eq!(detect_image_mime(Path::new("noext")), None);
    }

    #[test]
    fn image_file_base64_encoded() {
        let dir = TempDir::new().unwrap();
        // 写入一个小 PNG header（非真实 PNG，但足够触发图片检测）
        let png_bytes = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00];
        fs::write(dir.path().join("test.png"), &png_bytes).unwrap();

        let (_, refs) = resolve_file_refs("@test.png", dir.path());
        assert_eq!(refs.len(), 1);
        assert!(refs[0].image.is_some());
        let img = refs[0].image.clone().unwrap();
        assert_eq!(img.mime_type, "image/png");
        assert!(!img.base64.is_empty());
    }

    #[test]
    fn large_image_skipped() {
        let dir = tempfile::tempdir().unwrap();
        // Create a "fake" png larger than 1MB
        let big = dir.path().join("big.png");
        let data = vec![0u8; 1024 * 1024 + 1]; // just over 1MB
        std::fs::write(&big, &data).unwrap();

        let (msg, refs) = resolve_file_refs(&format!("@{}", big.display()), dir.path());
        assert!(refs[0].image.is_none());
        assert!(refs[0].content.contains("image too large"));
    }
}
