//! 输出截断 — 将工具输出截断到安全大小。
//!
//! 对应 `packages/coding-agent/src/core/tools/truncate.ts`。
//! 规则：2000 行 / 1MB，保留头尾。

/// 截断阈值。
pub const MAX_LINES: usize = 2000;
pub const MAX_BYTES: usize = 1024 * 1024; // 1 MB

/// 截断结果。
#[derive(Debug, Clone)]
pub struct TruncatedOutput {
    pub output: String,
    pub truncated: bool,
    pub original_lines: usize,
    pub original_bytes: usize,
}

/// 按行数和字节数截断输出。
pub fn truncate_output(output: &str) -> TruncatedOutput {
    let original_bytes = output.len();
    let original_lines = output.lines().count();

    // 先按字节截断
    if original_bytes <= MAX_BYTES && original_lines <= MAX_LINES {
        return TruncatedOutput {
            output: output.to_string(),
            truncated: false,
            original_lines,
            original_bytes,
        };
    }

    // 按行截断
    let lines: Vec<&str> = output.lines().collect();
    if lines.len() <= MAX_LINES && original_bytes <= MAX_BYTES {
        return TruncatedOutput {
            output: output.to_string(),
            truncated: false,
            original_lines,
            original_bytes,
        };
    }

    // 截断：保留前半 + 标记 + 后半
    let half = MAX_LINES / 2;
    let mut result = String::with_capacity(MAX_BYTES);

    for line in lines.iter().take(half) {
        result.push_str(line);
        result.push('\n');
    }

    result.push_str(&format!(
        "\n<output truncated: {} lines, {} bytes — showing first {} and last {} lines>\n\n",
        original_lines, original_bytes, half, half,
    ));

    for line in lines.iter().rev().take(half).rev() {
        result.push_str(line);
        result.push('\n');
    }

    // 最终字节截断
    if result.len() > MAX_BYTES {
        result.truncate(MAX_BYTES);
        result.push_str("\n<output truncated at byte limit>\n");
    }

    TruncatedOutput {
        output: result,
        truncated: true,
        original_lines,
        original_bytes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_truncation_under_limit() {
        let output = "line 1\nline 2\nline 3";
        let result = truncate_output(output);
        assert!(!result.truncated);
        assert_eq!(result.output, output);
    }

    #[test]
    fn truncates_many_lines() {
        let input: String = (0..5000).map(|i| format!("line {i}\n")).collect();
        let result = truncate_output(&input);
        assert!(result.truncated);
        assert!(result.output.contains("<output truncated"));
        assert!(result.original_lines >= 5000);
    }
}
