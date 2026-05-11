//! Git 集成 — 检测 Git 仓库状态。
//!
//! 轻量级 Git 状态检测（无 libgit2 依赖）。
//! 通过 `git` CLI 命令获取分支名和脏状态。

use std::path::Path;
use std::process::Command;

/// Git 仓库状态。
#[derive(Debug, Clone, Default)]
pub struct GitStatus {
    /// 当前分支名。
    pub branch: String,
    /// 是否有未提交的修改。
    pub dirty: bool,
    /// 是否在 Git 仓库内。
    pub in_repo: bool,
}

/// 检测指定目录的 Git 状态。
///
/// 使用 `git rev-parse` 和 `git status` 命令。
/// 如果 `git` 不可用或不在仓库中，返回默认值。
pub fn detect(cwd: &Path) -> GitStatus {
    // 检查是否在 Git 仓库中
    let output = Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .current_dir(cwd)
        .output();

    let output = match output {
        Ok(o) => o,
        Err(_) => return GitStatus::default(),
    };

    if !output.status.success() {
        return GitStatus::default();
    }

    // 获取分支名
    let branch = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(cwd)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();

    // 检查是否有未提交的修改
    let dirty = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(cwd)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false);

    GitStatus {
        branch,
        dirty,
        in_repo: true,
    }
}

/// 格式化 Git 状态为 header 显示文本。
pub fn format_status(status: &GitStatus) -> String {
    if !status.in_repo {
        return String::new();
    }

    let dirty_marker = if status.dirty { "*" } else { "" };
    format!("{}{}", status.branch, dirty_marker)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_clean() {
        let status = GitStatus {
            branch: "main".to_string(),
            dirty: false,
            in_repo: true,
        };
        assert_eq!(format_status(&status), "main");
    }

    #[test]
    fn format_dirty() {
        let status = GitStatus {
            branch: "feature/test".to_string(),
            dirty: true,
            in_repo: true,
        };
        assert_eq!(format_status(&status), "feature/test*");
    }

    #[test]
    fn format_not_in_repo() {
        let status = GitStatus::default();
        assert!(format_status(&status).is_empty());
    }

    #[test]
    fn detect_in_git_repo() {
        // 当前 piso 项目目录
        let status = detect(Path::new(env!("CARGO_MANIFEST_DIR")));
        // 这个项目应该是一个 Git 仓库
        assert!(status.in_repo);
        assert!(!status.branch.is_empty());
    }

    #[test]
    fn detect_outside_repo() {
        let status = detect(Path::new("/tmp"));
        // /tmp 通常不在 Git 仓库中
        assert!(!status.in_repo);
    }
}
