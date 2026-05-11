//! 轻量 Markdown 渲染 — 将 Markdown 文本转为 ratatui Line。
//!
//! 支持：代码块（带语言标签）、行内代码、粗体、标题。
//! 颜色由 Theme 配置驱动。

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

/// Markdown 渲染配色。
#[derive(Debug, Clone)]
pub struct MdColors {
    /// 代码块前景。
    pub code_fg: Color,
    /// 行内代码前景。
    pub inline_code_fg: Color,
    /// 标题颜色。
    pub heading_fg: Color,
    /// 粗体加色。
    pub bold_modifier: bool,
}

impl Default for MdColors {
    fn default() -> Self {
        Self::dark()
    }
}

impl MdColors {
    pub fn dark() -> Self {
        Self {
            code_fg: crate::components::colors::MD_CODE,
            inline_code_fg: crate::components::colors::MD_CODE,
            heading_fg: crate::components::colors::MD_HEADING,
            bold_modifier: true,
        }
    }

    /// 从 Theme 构建。
    pub fn from_theme(theme: &crate::theme::Theme) -> Self {
        Self {
            code_fg: theme.code_color(),
            inline_code_fg: theme.inline_code_color(),
            heading_fg: theme.heading_color(),
            bold_modifier: true,
        }
    }
}

/// 将 Markdown 文本渲染为 ratatui Line 列表。
pub fn render_markdown(text: &str, base_style: Style) -> Vec<Line<'static>> {
    render_markdown_with_colors(text, base_style, &MdColors::default())
}

/// 将 Markdown 文本渲染为 ratatui Line 列表（带自定义颜色）。
pub fn render_markdown_with_colors(
    text: &str,
    base_style: Style,
    colors: &MdColors,
) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let mut in_code_block = false;
    let mut code_lang = String::new();

    for raw_line in text.lines() {
        // 代码块边界
        if raw_line.starts_with("```") {
            in_code_block = !in_code_block;
            // 如果是开头的 ```，提取语言标签显示
            if in_code_block {
                code_lang = raw_line.trim_start_matches('`').trim().to_string();
                if !code_lang.is_empty() {
                    lines.push(Line::from(vec![Span::styled(
                        format!("  {}", code_lang),
                        Style::default()
                            .fg(colors.inline_code_fg)
                            .add_modifier(Modifier::ITALIC),
                    )]));
                }
            } else {
                code_lang.clear();
            }
            continue;
        }

        if in_code_block {
            // diff 语法高亮
            if code_lang == "diff" || raw_line.starts_with("---") || raw_line.starts_with("+++") {
                let (_prefix, fg) = if raw_line.starts_with('+') && !raw_line.starts_with("+++") {
                    ('+', Color::Green)
                } else if raw_line.starts_with('-') && !raw_line.starts_with("---") {
                    ('-', Color::Red)
                } else if raw_line.starts_with("@@") {
                    ('@', Color::Cyan)
                } else {
                    (' ', colors.code_fg)
                };
                lines.push(Line::from(vec![Span::styled(
                    format!("  {}", raw_line),
                    Style::default().fg(fg),
                )]));
            } else {
                lines.push(Line::from(vec![Span::styled(
                    format!("  {}", raw_line),
                    Style::default().fg(colors.code_fg),
                )]));
            }
            continue;
        }

        // 标题
        if raw_line.starts_with("### ") {
            lines.push(Line::from(vec![Span::styled(
                raw_line
                    .strip_prefix("### ")
                    .unwrap_or(raw_line)
                    .to_string(),
                base_style
                    .add_modifier(Modifier::BOLD)
                    .fg(colors.heading_fg),
            )]));
            continue;
        }
        if raw_line.starts_with("## ") {
            lines.push(Line::from(vec![Span::styled(
                raw_line.strip_prefix("## ").unwrap_or(raw_line).to_string(),
                base_style
                    .add_modifier(Modifier::BOLD)
                    .fg(colors.heading_fg),
            )]));
            continue;
        }
        if raw_line.starts_with("# ") {
            lines.push(Line::from(vec![Span::styled(
                raw_line.strip_prefix("# ").unwrap_or(raw_line).to_string(),
                base_style
                    .add_modifier(Modifier::BOLD)
                    .fg(colors.heading_fg),
            )]));
            continue;
        }

        // 列表项（- / * / 1.）
        let trimmed = raw_line.trim_start();
        if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
            let content = &trimmed[2..];
            let mut md_line = render_inline_with_colors(content, base_style, colors);
            // 在开头加 bullet span
            let mut new_spans = vec![Span::styled("  • ".to_string(), base_style)];
            new_spans.append(&mut md_line.spans);
            md_line.spans = new_spans;
            lines.push(md_line);
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix(|c: char| c.is_ascii_digit()) {
            if let Some(content) = rest.strip_prefix(". ") {
                let prefix = &trimmed[..trimmed.len() - rest.len()];
                let mut md_line = render_inline_with_colors(content, base_style, colors);
                let mut new_spans = vec![Span::styled(format!("  {}. ", prefix), base_style)];
                new_spans.append(&mut md_line.spans);
                md_line.spans = new_spans;
                lines.push(md_line);
                continue;
            }
        }

        // Markdown 表格
        let trimmed = raw_line.trim();
        if trimmed.starts_with('|') && trimmed.ends_with('|') {
            // 分隔行 (|---|---|) 跳过
            let stripped: String = trimmed
                .chars()
                .filter(|c| !matches!(c, '|' | '-' | ':' | ' '))
                .collect();
            if stripped.is_empty() {
                // 绘制水平线替代
                let col_count = trimmed.split('|').filter(|s| !s.is_empty()).count();
                let separator: String = std::iter::repeat_n("─", col_count * 16).collect();
                lines.push(Line::from(vec![Span::styled(
                    separator,
                    Style::default().fg(Color::DarkGray),
                )]));
                continue;
            }
            // 数据行 — 解析单元格
            let cells: Vec<&str> = trimmed
                .trim_start_matches('|')
                .trim_end_matches('|')
                .split('|')
                .map(|c| c.trim())
                .collect();
            let mut spans = Vec::new();
            spans.push(Span::styled("  ".to_string(), base_style));
            for (i, cell) in cells.iter().enumerate() {
                if i > 0 {
                    spans.push(Span::styled(
                        " │ ".to_string(),
                        Style::default().fg(Color::DarkGray),
                    ));
                }
                spans.push(Span::styled(cell.to_string(), base_style));
            }
            lines.push(Line::from(spans));
            continue;
        }

        // 普通行 — 解析行内格式
        lines.push(render_inline_with_colors(raw_line, base_style, colors));
    }

    lines
}

/// 渲染单行内联格式（粗体、行内代码）。
pub fn render_inline(line: &str, base_style: Style) -> Line<'static> {
    render_inline_with_colors(line, base_style, &MdColors::default())
}

/// 渲染单行内联格式（带自定义颜色）。
fn render_inline_with_colors(line: &str, base_style: Style, colors: &MdColors) -> Line<'static> {
    let mut spans = Vec::new();
    let mut chars = line.chars().peekable();
    let mut current = String::new();

    while let Some(ch) = chars.next() {
        match ch {
            '`' => {
                // flush current
                if !current.is_empty() {
                    spans.push(Span::styled(std::mem::take(&mut current), base_style));
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
                        Style::default().fg(colors.inline_code_fg),
                    ));
                }
            }
            '*' if chars.peek() == Some(&'*') => {
                chars.next(); // consume second *
                if !current.is_empty() {
                    spans.push(Span::styled(std::mem::take(&mut current), base_style));
                }
                // 收集到 **
                let mut bold = String::new();
                while let Some(&c) = chars.peek() {
                    if c == '*' && !bold.is_empty() {
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
                    spans.push(Span::styled(bold, base_style.add_modifier(Modifier::BOLD)));
                }
            }
            '[' => {
                // 尝试匹配 [text](url)
                if !current.is_empty() {
                    spans.push(Span::styled(std::mem::take(&mut current), base_style));
                }
                // 收集 [text]
                let mut link_text = String::new();
                let mut found_bracket = false;
                while let Some(&c) = chars.peek() {
                    chars.next();
                    if c == ']' {
                        found_bracket = true;
                        break;
                    }
                    link_text.push(c);
                }
                if found_bracket && chars.peek() == Some(&'(') {
                    chars.next(); // consume (
                    let mut url = String::new();
                    while let Some(&c) = chars.peek() {
                        chars.next();
                        if c == ')' {
                            break;
                        }
                        url.push(c);
                    }
                    // 渲染为带下划线的链接文本
                    spans.push(Span::styled(
                        link_text,
                        base_style
                            .fg(colors.inline_code_fg)
                            .add_modifier(Modifier::UNDERLINED),
                    ));
                } else {
                    // 不是链接，还原 [text
                    spans.push(Span::styled(format!("[{}", link_text), base_style));
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
        // before, rust lang, code, after (fence lines skipped)
        assert_eq!(lines.len(), 4);
        assert_eq!(lines[2].spans[0].content, "  fn main() {}");
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

    #[test]
    fn list_items() {
        let colors = MdColors::default();
        let md = "- item one\n- item two\n  normal text";
        let lines = render_markdown_with_colors(md, Style::default(), &colors);
        // list items + normal line
        assert!(lines.len() >= 2);
    }

    #[test]
    fn code_block_with_language() {
        let md = "before\n```rust\nfn main() {}\n```
after";
        let lines = render_markdown(md, Style::default());
        // before, rust lang, code line, after = 4
        assert_eq!(lines.len(), 4);
        assert_eq!(lines[1].spans[0].content, "  rust");
    }

    #[test]
    fn custom_colors() {
        let colors = MdColors {
            code_fg: Color::Red,
            inline_code_fg: Color::Blue,
            heading_fg: Color::Green,
            bold_modifier: true,
        };
        let md = "# Title\n`code`";
        let lines = render_markdown_with_colors(md, Style::default(), &colors);
        // heading color = green
        assert_eq!(lines[0].spans[0].style.fg, Some(Color::Green));
    }

    #[test]
    fn table_rendering() {
        let md = "| Name | Value |\n|------|-------|\n| foo  | bar   |";
        let lines = render_markdown(md, Style::default());
        assert_eq!(lines.len(), 3); // header + separator + data
    }

    #[test]
    fn table_separator() {
        let md = "|------|-------|";
        let lines = render_markdown(md, Style::default());
        // separator line becomes horizontal rule
        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn link_rendering() {
        let line = render_inline("click [here](https://example.com) now", Style::default());
        assert!(line.spans.len() >= 3);
        assert_eq!(line.spans[1].content, "here");
    }
}
