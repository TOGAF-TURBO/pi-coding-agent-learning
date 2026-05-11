//! Diff 引擎 — 统一差异计算，纯 Rust 无外部依赖。
//!
//! 用于 edit 工具的差异预览和渲染。

/// 差异块类型。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffLineType {
    Context,
    Added,
    Removed,
}

/// 一行差异。
#[derive(Debug, Clone)]
pub struct DiffLine {
    pub line_type: DiffLineType,
    pub old_line: Option<usize>,
    pub new_line: Option<usize>,
    pub content: String,
}

/// 完整的 diff 结果。
#[derive(Debug, Clone)]
pub struct DiffResult {
    pub lines: Vec<DiffLine>,
    pub added_count: usize,
    pub removed_count: usize,
}

/// 计算两段文本的 unified diff。
pub fn compute_diff(old_text: &str, new_text: &str) -> DiffResult {
    let old_lines: Vec<&str> = old_text.lines().collect();
    let new_lines: Vec<&str> = new_text.lines().collect();

    // 使用最长公共子序列 (LCS) 算法计算 diff
    let lcs = compute_lcs(&old_lines, &new_lines);
    let mut lines = Vec::new();
    let mut added = 0;
    let mut removed = 0;

    let mut oi = 0; // old index
    let mut ni = 0; // new index
    let mut li = 0; // lcs index

    while oi < old_lines.len() || ni < new_lines.len() {
        if li < lcs.len() {
            let lcs_line = lcs[li];

            // 输出 old 中在 LCS 行之前的行（被删除）
            while oi < old_lines.len() && old_lines[oi] != lcs_line {
                lines.push(DiffLine {
                    line_type: DiffLineType::Removed,
                    old_line: Some(oi + 1),
                    new_line: None,
                    content: old_lines[oi].to_string(),
                });
                removed += 1;
                oi += 1;
            }

            // 输出 new 中在 LCS 行之前的行（被添加）
            while ni < new_lines.len() && new_lines[ni] != lcs_line {
                lines.push(DiffLine {
                    line_type: DiffLineType::Added,
                    old_line: None,
                    new_line: Some(ni + 1),
                    content: new_lines[ni].to_string(),
                });
                added += 1;
                ni += 1;
            }

            // 输出 LCS 行（上下文）
            if oi < old_lines.len() && ni < new_lines.len() {
                lines.push(DiffLine {
                    line_type: DiffLineType::Context,
                    old_line: Some(oi + 1),
                    new_line: Some(ni + 1),
                    content: old_lines[oi].to_string(),
                });
                oi += 1;
                ni += 1;
                li += 1;
            }
        } else {
            // LCS 结束后，剩余的行
            while oi < old_lines.len() {
                lines.push(DiffLine {
                    line_type: DiffLineType::Removed,
                    old_line: Some(oi + 1),
                    new_line: None,
                    content: old_lines[oi].to_string(),
                });
                removed += 1;
                oi += 1;
            }
            while ni < new_lines.len() {
                lines.push(DiffLine {
                    line_type: DiffLineType::Added,
                    old_line: None,
                    new_line: Some(ni + 1),
                    content: new_lines[ni].to_string(),
                });
                added += 1;
                ni += 1;
            }
        }
    }

    DiffResult {
        lines,
        added_count: added,
        removed_count: removed,
    }
}

/// 生成 unified diff 字符串。
pub fn format_diff(old_text: &str, new_text: &str, file_path: &str) -> String {
    let diff = compute_diff(old_text, new_text);
    if diff.lines.is_empty() {
        return String::new();
    }

    let max_line = diff
        .lines
        .iter()
        .filter_map(|l| l.old_line.or(l.new_line))
        .max()
        .unwrap_or(1);
    let width = format!("{}", max_line).len();

    let mut output = format!("--- a/{}\n+++ b/{}\n", file_path, file_path);

    for line in &diff.lines {
        let (prefix, old_n, new_n) = match line.line_type {
            DiffLineType::Context => (
                ' ',
                format!("{:>width$}", line.old_line.unwrap(), width = width),
                format!("{:>width$}", line.new_line.unwrap(), width = width),
            ),
            DiffLineType::Added => (
                '+',
                " ".repeat(width),
                format!("{:>width$}", line.new_line.unwrap(), width = width),
            ),
            DiffLineType::Removed => (
                '-',
                format!("{:>width$}", line.old_line.unwrap(), width = width),
                " ".repeat(width),
            ),
        };
        output.push_str(&format!("{}{} {} {}\n", prefix, old_n, new_n, line.content));
    }

    output
}

/// 计算 LCS（最长公共子序列）— 返回公共行的列表。
fn compute_lcs<'a>(old: &[&'a str], new: &[&'a str]) -> Vec<&'a str> {
    let m = old.len();
    let n = new.len();

    // DP table
    let mut dp = vec![vec![0usize; n + 1]; m + 1];

    for i in 1..=m {
        for j in 1..=n {
            if old[i - 1] == new[j - 1] {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
            }
        }
    }

    // 回溯
    let mut result = Vec::new();
    let mut i = m;
    let mut j = n;

    while i > 0 && j > 0 {
        if old[i - 1] == new[j - 1] {
            result.push(old[i - 1]);
            i -= 1;
            j -= 1;
        } else if dp[i - 1][j] > dp[i][j - 1] {
            i -= 1;
        } else {
            j -= 1;
        }
    }

    result.reverse();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_text_no_diff() {
        let diff = compute_diff("hello\nworld", "hello\nworld");
        assert_eq!(diff.added_count, 0);
        assert_eq!(diff.removed_count, 0);
        assert_eq!(diff.lines.len(), 2);
        assert!(diff
            .lines
            .iter()
            .all(|l| l.line_type == DiffLineType::Context));
    }

    #[test]
    fn simple_addition() {
        let diff = compute_diff("a\nb", "a\nb\nc");
        assert_eq!(diff.added_count, 1);
        assert_eq!(diff.removed_count, 0);
        let added: Vec<&str> = diff
            .lines
            .iter()
            .filter(|l| l.line_type == DiffLineType::Added)
            .map(|l| l.content.as_str())
            .collect();
        assert_eq!(added, vec!["c"]);
    }

    #[test]
    fn simple_removal() {
        let diff = compute_diff("a\nb\nc", "a\nc");
        assert_eq!(diff.added_count, 0);
        assert_eq!(diff.removed_count, 1);
        let removed: Vec<&str> = diff
            .lines
            .iter()
            .filter(|l| l.line_type == DiffLineType::Removed)
            .map(|l| l.content.as_str())
            .collect();
        assert_eq!(removed, vec!["b"]);
    }

    #[test]
    fn replacement() {
        let diff = compute_diff("hello\nworld", "hello\nrust");
        assert_eq!(diff.added_count, 1);
        assert_eq!(diff.removed_count, 1);
    }

    #[test]
    fn empty_to_content() {
        let diff = compute_diff("", "new\ncontent");
        assert_eq!(diff.added_count, 2);
        assert_eq!(diff.removed_count, 0);
    }

    #[test]
    fn content_to_empty() {
        let diff = compute_diff("old\ncontent", "");
        assert_eq!(diff.added_count, 0);
        assert_eq!(diff.removed_count, 2);
    }

    #[test]
    fn format_diff_output() {
        let output = format_diff("a\nb\nc", "a\nx\nc", "test.txt");
        assert!(output.starts_with("--- a/test.txt"));
        assert!(output.contains("+++ b/test.txt"));
        assert!(output.contains("-2"), "removed line marker: {}", output);
        assert!(output.contains("+ "), "added line marker: {}", output);
        assert!(output.contains("b"));
        assert!(output.contains("x"));
    }

    #[test]
    fn lcs_correctness() {
        let old = vec!["a", "b", "c", "d"];
        let new = vec!["a", "x", "c", "d"];
        let lcs = compute_lcs(&old, &new);
        assert_eq!(lcs, vec!["a", "c", "d"]);
    }
}
