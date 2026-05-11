//! Tab 补全 — 文件路径补全。
//!
//! 检测输入中类似路径的片段（以 /, ./, ~, 或 ../ 开头），
//! 扫描文件系统匹配，返回补全候选。

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

/// 从输入文本和光标位置尝试补全。
///
/// 返回 (候选列表, 触发文本范围)。
/// `trigger_start` 是触发文本在 input 中的起始字节偏移。
pub fn complete(input: &str, cursor: usize, cwd: &Path) -> Option<(Vec<Completion>, usize)> {
    // 找到光标前的最后一个路径片段
    let before_cursor = &input[..cursor.min(input.len())];

    // 查找路径触发点：最后一个空格或行首之后的内容
    let trigger_start = before_cursor.rfind(|c: char| c.is_whitespace() || c == '"' || c == '\'')
        .map(|i| i + 1)
        .unwrap_or(0);

    let fragment = &before_cursor[trigger_start..];

    // 必须看起来像路径
    if fragment.is_empty() {
        return None;
    }

    // 检查是否包含路径分隔符
    let has_path_sep = fragment.contains('/') || fragment.starts_with('~');
    if !has_path_sep && !fragment.starts_with('.') {
        return None;
    }

    // 展开路径
    let expanded = shellexpand(fragment, cwd);
    let dir = expanded.parent().unwrap_or(Path::new(".")).to_path_buf();
    let prefix = expanded.file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_default();

    // 读取目录
    let entries = std::fs::read_dir(&dir).ok()?;
    let mut candidates = Vec::new();

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') && !prefix.starts_with('.') && !prefix.is_empty() {
            continue; // 跳过隐藏文件（除非明确请求）
        }
        if !name.to_lowercase().starts_with(&prefix.to_lowercase()) {
            continue;
        }

        let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
        let full_path = dir.join(&name);

        // 生成显示文本（相对于输入的格式）
        let completion_text = if fragment.starts_with('~') {
            format!("~/{}", full_path.strip_prefix(dir_home()).unwrap_or(&full_path).to_string_lossy())
        } else if fragment.starts_with('/') {
            full_path.to_string_lossy().to_string()
        } else {
            // 相对路径
            let rel = pathdiff(&full_path, cwd);
            rel.unwrap_or_else(|| full_path.to_string_lossy().to_string())
        };

        let display = if is_dir {
            format!("{}/", name)
        } else {
            name.clone()
        };

        candidates.push(Completion {
            text: if is_dir { format!("{}/", completion_text.trim_end_matches('/')) } else { completion_text },
            display,
            is_dir,
        });
    }

    if candidates.is_empty() {
        return None;
    }

    // 按 display 排序
    candidates.sort_by(|a, b| a.display.cmp(&b.display));

    Some((candidates, trigger_start))
}

/// 展开 ~/ 和相对路径。
fn shellexpand(path: &str, cwd: &Path) -> PathBuf {
    if path.starts_with('~') {
        let home = dir_home();
        let rest = path.strip_prefix('~').unwrap_or(path);
        let rest = rest.strip_prefix('/').unwrap_or(rest);
        home.join(rest)
    } else if path.starts_with('/') {
        PathBuf::from(path)
    } else {
        cwd.join(path)
    }
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
    fn complete_home_tilde() {
        let cwd = PathBuf::from("/tmp");
        let home = dirs::home_dir().unwrap();
        // 创建一个已知存在的目录来测试
        let result = complete("~/Documents ", 11, &cwd);
        // 可能存在也可能不存在
        if home.join("Documents").exists() {
            assert!(result.is_some());
        }
    }

    #[test]
    fn no_complete_empty() {
        let cwd = PathBuf::from("/tmp");
        assert!(complete("", 0, &cwd).is_none());
    }

    #[test]
    fn no_complete_plain_text() {
        let cwd = PathBuf::from("/tmp");
        assert!(complete("hello world", 11, &cwd).is_none());
    }

    #[test]
    fn complete_tmp_dir() {
        let cwd = PathBuf::from("/");
        // /tmp/ exists on all Unix systems
        let result = complete("/tmp/", 5, &cwd);
        // 应该能列出 /tmp 下的文件
        if Path::new("/tmp").exists() {
            assert!(result.is_some() || Path::new("/tmp").read_dir().unwrap().next().is_none());
        }
    }

    #[test]
    fn complete_dot_slash() {
        // 创建临时目录结构
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join("src")).unwrap();
        fs::write(dir.path().join("Cargo.toml"), "").unwrap();

        let result = complete("./", 2, dir.path());
        assert!(result.is_some());
        let (candidates, start) = result.unwrap();
        assert_eq!(start, 0);
        assert!(candidates.len() >= 2); // at least src/ and Cargo.toml
    }

    #[test]
    fn complete_after_space() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("test.rs"), "").unwrap();

        let result = complete("read ./test", 11, dir.path());
        assert!(result.is_some());
        let (candidates, start) = result.unwrap();
        assert_eq!(start, 5); // after "read "
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
