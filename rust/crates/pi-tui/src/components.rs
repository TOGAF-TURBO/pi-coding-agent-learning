//! 渲染组件 — 五个区域的 ratatui 渲染。

use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::{AgentState, AppState, ChatRole};
use crate::layout::LayoutRegions;
use crate::markdown::render_markdown;

/// 工具结果最大显示行数。
const MAX_TOOL_LINES: usize = 8;

/// 渲染 header 区域。
pub fn render_header(
    f: &mut ratatui::Frame,
    area: Rect,
    model: &str,
    provider: &str,
    session_id: &str,
    git: &str,
) {
    // 截断 session ID
    let short_id = if session_id.len() > 12 {
        &session_id[..12]
    } else {
        session_id
    };

    let mut spans = vec![
        Span::styled(
            " piso ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{} ({})", model, provider),
            Style::default().fg(Color::DarkGray),
        ),
    ];

    // Git 分支
    if !git.is_empty() {
        spans.push(Span::styled(
            format!(" {}", git),
            Style::default().fg(Color::Magenta),
        ));
    }

    spans.push(Span::styled(
        format!("  {}", short_id),
        Style::default().fg(Color::DarkGray),
    ));

    let line = Line::from(spans);
    let para = Paragraph::new(line);
    f.render_widget(para, area);
}

/// 将文本按宽度换行。
fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    for line in text.lines() {
        if line.is_empty() {
            lines.push(String::new());
            continue;
        }
        // 按 unicode 字符宽度换行
        let mut current = String::new();
        let mut current_width = 0;
        for ch in line.chars() {
            let ch_width = unicode_width(ch);
            if current_width + ch_width > max_width && !current.is_empty() {
                lines.push(current.clone());
                current.clear();
                current_width = 0;
            }
            current.push(ch);
            current_width += ch_width;
        }
        if !current.is_empty() {
            lines.push(current);
        }
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

/// Unicode 字符显示宽度（简化版：CJK = 2，其余 = 1）。
fn unicode_width(ch: char) -> usize {
    if ('\u{4e00}'..='\u{9fff}').contains(&ch)
        || ('\u{3000}'..='\u{303f}').contains(&ch)
        || ('\u{ff01}'..='\u{ff60}').contains(&ch)
    {
        2
    } else if ch == '\t' {
        4
    } else {
        1
    }
}

/// 渲染 chat 区域。
/// `scroll_offset` — 从底部向上的滚动偏移（0 = 底部）。
pub fn render_chat(f: &mut ratatui::Frame, area: Rect, state: &AppState, scroll_offset: usize) {
    let entries = state.entries.read();
    let content_width = area.width.saturating_sub(4) as usize; // "   " prefix
    let mut lines: Vec<Line> = Vec::new();

    for entry in entries.iter() {
        let (prefix, style) = match &entry.role {
            ChatRole::User => (
                "You",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            ChatRole::Assistant => (
                "Assistant",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            ChatRole::Thinking => (
                "Thinking",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            ),
            ChatRole::System => ("System", Style::default().fg(Color::Yellow)),
            ChatRole::Tool { name, is_error } => (
                name.as_str(),
                if *is_error {
                    Style::default().fg(Color::Red)
                } else {
                    Style::default().fg(Color::Magenta)
                },
            ),
        };

        // 头部行
        lines.push(Line::from(vec![Span::styled(
            format!(" {} ", prefix),
            style,
        )]));

        // 内容行 — 自动换行
        let content_style = match &entry.role {
            ChatRole::Tool { is_error, .. } if *is_error => Style::default().fg(Color::Red),
            ChatRole::Tool { .. } => Style::default().fg(Color::Gray),
            ChatRole::Thinking => Style::default().fg(Color::DarkGray),
            _ => Style::default(),
        };

        // Assistant 消息用 Markdown 渲染，其余用纯文本
        if matches!(entry.role, ChatRole::Assistant) {
            let md_lines = render_markdown(&entry.content, content_style);
            for md_line in md_lines {
                // 在每个 span 前添加缩进，保留 markdown 样式
                let mut spans: Vec<Span<'static>> = vec![Span::raw("   ")];
                for s in md_line.spans {
                    spans.push(Span::styled(s.content, s.style));
                }
                lines.push(Line::from(spans));
            }
        } else {
            let wrapped = wrap_text(&entry.content, content_width);
            let max_lines = match &entry.role {
                ChatRole::Tool { .. } => MAX_TOOL_LINES,
                _ => usize::MAX,
            };

            let display_lines: Vec<String> = if wrapped.len() > max_lines {
                let mut truncated: Vec<String> = wrapped[..max_lines].to_vec();
                truncated.push(format!("  ... ({} more lines)", wrapped.len() - max_lines));
                truncated
            } else {
                wrapped
            };

            for line in &display_lines {
                lines.push(Line::from(Span::styled(
                    format!("   {}", line),
                    content_style,
                )));
            }
        }

        lines.push(Line::from("")); // 空行分隔
    }

    // 思考/工具执行指示器
    let footer_state = state.footer.read();
    match &footer_state.state {
        AgentState::Thinking => {
            lines.push(Line::from(Span::styled(
                " ● Thinking...",
                Style::default().fg(Color::Yellow),
            )));
        }
        AgentState::ToolRunning { name } => {
            lines.push(Line::from(Span::styled(
                format!(" ● Running {}...", name),
                Style::default().fg(Color::Magenta),
            )));
        }
        _ => {}
    }

    // 滚动计算
    let visible = area.height as usize;
    let total = lines.len();
    let start = total.saturating_sub(visible).saturating_sub(scroll_offset);

    let para = Paragraph::new(lines.into_iter().skip(start).collect::<Vec<_>>());
    f.render_widget(para, area);
}

/// 渲染 status 区域。
pub fn render_status(f: &mut ratatui::Frame, area: Rect, state: &AppState) {
    let footer = state.footer.read();

    let (state_text, state_color) = match &footer.state {
        AgentState::Idle => ("READY", Color::Green),
        AgentState::Thinking => ("THINKING", Color::Yellow),
        AgentState::Streaming => ("STREAMING", Color::Cyan),
        AgentState::ToolRunning { name } => {
            // 截断长工具名
            let display = if name.len() > 16 {
                format!("{}...", &name[..13])
            } else {
                name.clone()
            };
            (leak_str(display), Color::Magenta)
        }
        AgentState::Error(e) => {
            let display = if e.len() > 30 {
                format!("ERR: {}...", &e[..24])
            } else {
                format!("ERR: {}", e)
            };
            (leak_str(display), Color::Red)
        }
    };

    let tokens = if footer.input_tokens + footer.output_tokens > 0 {
        format!(" | {}in/{}out", footer.input_tokens, footer.output_tokens)
    } else {
        String::new()
    };

    // 上下文窗口占用
    let ctx = if footer.context_tokens > 0 {
        // 大多数模型 context window ~128K
        let pct = (footer.context_tokens as f64 / 128_000.0 * 100.0) as u32;
        let ctx_color = if pct > 80 {
            Color::Red
        } else if pct > 50 {
            Color::Yellow
        } else {
            Color::DarkGray
        };
        (format!(" | ctx:{}%", pct.min(100)), Some(ctx_color))
    } else {
        (String::new(), None)
    };
    let elapsed = if let Some(start) = *state.turn_start.read() {
        let secs = start.elapsed().as_secs();
        if secs >= 60 {
            format!(" | {}m{}s", secs / 60, secs % 60)
        } else {
            format!(" | {}s", secs)
        }
    } else if footer.duration_secs > 0 {
        if footer.duration_secs >= 60 {
            format!(
                " | {}m{}s",
                footer.duration_secs / 60,
                footer.duration_secs % 60
            )
        } else {
            format!(" | {}s", footer.duration_secs)
        }
    } else {
        String::new()
    };

    let mut line_spans = vec![
        Span::styled(
            format!(" {}", state_text),
            Style::default()
                .fg(state_color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(tokens, Style::default().fg(Color::DarkGray)),
    ];

    if let Some(ctx_color) = ctx.1 {
        line_spans.push(Span::styled(ctx.0, Style::default().fg(ctx_color)));
    }

    line_spans.push(Span::styled(elapsed, Style::default().fg(Color::DarkGray)));

    let line = Line::from(line_spans);

    let para = Paragraph::new(line);
    f.render_widget(para, area);
}

/// 渲染 editor 区域 — 多行支持。
/// `cursor_pos` — 光标在文本中的字节偏移。
pub fn render_editor(
    f: &mut ratatui::Frame,
    area: Rect,
    input: &str,
    cursor_pos: usize,
    cursor: bool,
    is_running: bool,
) {
    if input.is_empty() {
        let hint = if is_running {
            "Waiting for agent..."
        } else {
            "Type a message... (Enter to send, Shift+Enter for newline)"
        };
        let para = Paragraph::new(hint)
            .style(Style::default().fg(Color::DarkGray))
            .block(
                Block::default()
                    .borders(Borders::TOP)
                    .border_style(Style::default().fg(Color::DarkGray)),
            );
        f.render_widget(para, area);
        if cursor && !is_running {
            f.set_cursor_position((area.x, area.y + 1));
        }
        return;
    }

    let para = Paragraph::new(input)
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: false })
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(if is_running {
                    Color::Yellow
                } else {
                    Color::Cyan
                })),
        );

    f.render_widget(para, area);

    if cursor && !is_running {
        // 计算光标位置（在 render_widget 之后，否则被覆盖）
        let before = &input[..cursor_pos];
        let area_width = area.width.saturating_sub(2) as usize;
        let area_width = area_width.max(1);

        // 逐字符计算光标的 (row, col)
        let mut row: usize = 0;
        let mut col: usize = 0;

        for ch in before.chars() {
            if ch == '\n' {
                row += 1;
                col = 0;
            } else {
                let w = unicode_width(ch);
                col += w;
                if col >= area_width {
                    row += 1;
                    col = w; // 宽字符换行后从自身宽度开始
                }
            }
        }

        f.set_cursor_position((
            area.x + col as u16,
            area.y + 1 + row.min(area.height.saturating_sub(2) as usize) as u16,
        ));
    }
}

/// 渲染 footer 区域。
pub fn render_footer(
    f: &mut ratatui::Frame,
    area: Rect,
    _is_running: bool,
    hints: &[(&'static str, String)],
) {
    let mut spans = Vec::new();
    for (tag, text) in hints {
        if *tag == "key" {
            spans.push(Span::styled(text.clone(), Style::default().fg(Color::Cyan)));
        } else {
            spans.push(Span::styled(
                text.clone(),
                Style::default().fg(Color::DarkGray),
            ));
        }
    }
    let para = Paragraph::new(Line::from(spans));
    f.render_widget(para, area);
}

/// 渲染全部五个区域。
pub fn render_all(
    f: &mut ratatui::Frame,
    regions: LayoutRegions,
    state: &AppState,
    input: &str,
    cursor_pos: usize,
    scroll_offset: usize,
    session_id: &str,
    footer_hints: &[(&'static str, String)],
    git: &str,
) {
    let footer = state.footer.read();
    let model = footer.model.clone();
    let provider = footer.provider.clone();
    let is_running = !matches!(&footer.state, AgentState::Idle);
    drop(footer);

    render_header(f, regions.header, &model, &provider, session_id, git);
    render_chat(f, regions.chat, state, scroll_offset);
    render_status(f, regions.status, state);
    render_editor(f, regions.editor, input, cursor_pos, true, is_running);
    render_footer(f, regions.footer, is_running, footer_hints);
}

/// Leaked string for static lifetime (used in status display).
/// Only used for short-lived display strings — acceptable leak.
fn leak_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

