//! 轻量 Markdown 渲染 — 将 Markdown 文本转为 ratatui Line。
//!
//! 支持：代码块、行内代码、粗体、标题。
//! 不支持（暂不需要）：链接、图片、表格、列表。

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

/// 将 Markdown 文本渲染为 ratatui Line 列表。
pub fn render_markdown(text: &str, base_style: Style) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let mut in_code_block = false;

    for raw_line in text.lines() {
        // 代码块边界
        if raw_line.starts_with("```") {
            if in_code_block {
                in_code_block = false;
            } else {
                in_code_block = true;
            }
            continue;
        }

        if in_code_block {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {}", raw_line),
                    Style::default().fg(Color::Gray),
                ),
            ]));
            continue;
        }

        // 标题
        if raw_line.starts_with("### ") {
            lines.push(Line::from(vec![
                Span::styled(
                    raw_line[4..].to_string(),
                    base_style.add_modifier(Modifier::BOLD),
                ),
            ]));
            continue;
        }
        if raw_line.starts_with("## ") {
            lines.push(Line::from(vec![
                Span::styled(
                    raw_line[3..].to_string(),
                    base_style.add_modifier(Modifier::BOLD),
                ),
            ]));
            continue;
        }
        if raw_line.starts_with("# ") {
            lines.push(Line::from(vec![
                Span::styled(
                    raw_line[2..].to_string(),
                    base_style.add_modifier(Modifier::BOLD).fg(Color::Yellow),
                ),
            ]));
            continue;
        }

        // 普通行 — 解析行内格式
        lines.push(render_inline(raw_line, base_style));
    }

    lines
}

/// 渲染单行内联格式（粗体、行内代码）。
fn render_inline(line: &str, base_style: Style) -> Line<'static> {
    let mut spans = Vec::new();
    let mut chars = line.chars().peekable();
    let mut current = String::new();

    while let Some(ch) = chars.next() {
        match ch {
            '`' => {
                // flush current
                if !current.is_empty() {
                    spans.push(Span::styled(
                        std::mem::take(&mut current),
                        base_style,
                    ));
                }
                // 收集到下一个 `
                let mut code = String::new();
                while let Some(&c) = chars.peek() {
                    if c == '`' {
                        chars.next();
                        break;
                    }
                    code.push(chars.next().unwrap());
                }
                if !code.is_empty() {
                    spans.push(Span::styled(
                        code,
                        Style::default().fg(Color::Yellow),
                    ));
                }
            }
            '*' if chars.peek() == Some(&'*') => {
                chars.next(); // consume second *
                if !current.is_empty() {
                    spans.push(Span::styled(
                        std::mem::take(&mut current),
                        base_style,
                    ));
                }
                // 收集到 **
                let mut bold = String::new();
                while let Some(&c) = chars.peek() {
                    if c == '*' && bold.len() > 0 {
                        chars.next(); // first *
                        if chars.peek() == Some(&'*') {
                            chars.next(); // second *
                            break;
                        } else {
                            bold.push('*');
                            continue;
                        }
                    }
                    bold.push(chars.next().unwrap());
                }
                if !bold.is_empty() {
                    spans.push(Span::styled(
                        bold,
                        base_style.add_modifier(Modifier::BOLD),
                    ));
                }
            }
            _ => current.push(ch),
        }
    }

    if !current.is_empty() {
        spans.push(Span::styled(current, base_style));
    }

    if spans.is_empty() {
        Line::from(Span::styled(String::new(), base_style))
    } else {
        Line::from(spans)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heading_levels() {
        let lines = render_markdown("# Title\n## Sub\n### Small", Style::default());
        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn code_block() {
        let md = "before\n```rust\nfn main() {}\n```\nafter";
        let lines = render_markdown(md, Style::default());
        assert_eq!(lines.len(), 3); // before, code, after (fence lines skipped)
        assert_eq!(lines[1].spans[0].content, "  fn main() {}");
    }

    #[test]
    fn inline_code() {
        let line = render_inline("use `cargo` to build", Style::default());
        assert_eq!(line.spans.len(), 3);
        assert_eq!(line.spans[1].content, "cargo");
    }

    #[test]
    fn bold_text() {
        let line = render_inline("this is **bold** text", Style::default());
        assert_eq!(line.spans.len(), 3);
        assert_eq!(line.spans[1].content, "bold");
    }
}
