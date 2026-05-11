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
pub fn render_header(f: &mut ratatui::Frame, area: Rect, model: &str, provider: &str, session_id: &str) {
    // 截断 session ID
    let short_id = if session_id.len() > 12 {
        &session_id[..12]
    } else {
        session_id
    };
    let line = Line::from(vec![
        Span::styled(
            " piso ",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{} ({})", model, provider),
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(
            format!("  {}", short_id),
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(
            " [Ctrl+O submit, Ctrl+C quit]",
            Style::default().fg(Color::DarkGray),
        ),
    ]);
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
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            ),
            ChatRole::Assistant => (
                "Assistant",
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
            ChatRole::System => (
                "System",
                Style::default().fg(Color::Yellow),
            ),
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
        lines.push(Line::from(vec![
            Span::styled(format!(" {} ", prefix), style),
        ]));

        // 内容行 — 自动换行
        let content_style = match &entry.role {
            ChatRole::Tool { is_error, .. } if *is_error => {
                Style::default().fg(Color::Red)
            }
            ChatRole::Tool { .. } => {
                Style::default().fg(Color::Gray)
            }
            _ => Style::default(),
        };

        // Assistant 消息用 Markdown 渲染，其余用纯文本
        if matches!(entry.role, ChatRole::Assistant) {
            let md_lines = render_markdown(&entry.content, content_style);
            for md_line in md_lines {
                // 在每个 span 前添加缩进，保留 markdown 样式
                let mut spans: Vec<Span<'static>> = vec![
                    Span::raw("   ")
                ];
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
            lines.push(Line::from(
                Span::styled(" ● Thinking...", Style::default().fg(Color::Yellow)),
            ));
        }
        AgentState::ToolRunning { name } => {
            lines.push(Line::from(
                Span::styled(
                    format!(" ● Running {}...", name),
                    Style::default().fg(Color::Magenta),
                ),
            ));
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

    let line = Line::from(vec![
        Span::styled(
            format!(" {}", state_text),
            Style::default().fg(state_color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            tokens,
            Style::default().fg(Color::DarkGray),
        ),
    ]);

    let para = Paragraph::new(line);
    f.render_widget(para, area);
}

/// 渲染 editor 区域 — 多行支持。
pub fn render_editor(f: &mut ratatui::Frame, area: Rect, input: &str, cursor: bool, is_running: bool) {
    if input.is_empty() {
        let hint = if is_running {
            "Waiting for agent..."
        } else {
            "Type a message... (Ctrl+O to send, Enter for newline)"
        };
        let para = Paragraph::new(hint)
            .style(Style::default().fg(Color::DarkGray))
            .block(
                Block::default()
                    .borders(Borders::TOP)
                    .border_style(Style::default().fg(Color::DarkGray)),
            );
        f.render_widget(para, area);
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

    if cursor && !is_running {
        // 计算光标在多行文本中的位置
        let text_before_cursor = input;
        let mut row: u16 = 0;
        let area_width = area.width.saturating_sub(2); // borders
        for line in text_before_cursor.lines() {
            row += (unicode_width_str(line) as u16 + area_width - 1) / area_width.max(1);
        }
        let last_line = text_before_cursor.lines().last().unwrap_or("");
        let col = (unicode_width_str(last_line) as u16) % area_width.max(1);
        f.set_cursor_position((area.x + 1 + col, area.y + 1 + row.min(area.height.saturating_sub(2))));
    }

    f.render_widget(para, area);
}

/// 渲染 footer 区域。
pub fn render_footer(f: &mut ratatui::Frame, area: Rect, is_running: bool) {
    let spans = if is_running {
        vec![
            Span::styled(" Esc", Style::default().fg(Color::Yellow)),
            Span::styled(" Cancel  ", Style::default().fg(Color::DarkGray)),
            Span::styled(" Ctrl+C", Style::default().fg(Color::Cyan)),
            Span::styled(" Quit", Style::default().fg(Color::DarkGray)),
        ]
    } else {
        vec![
            Span::styled(" Ctrl+O", Style::default().fg(Color::Cyan)),
            Span::styled(" Send  ", Style::default().fg(Color::DarkGray)),
            Span::styled(" Ctrl+C", Style::default().fg(Color::Cyan)),
            Span::styled(" Quit  ", Style::default().fg(Color::DarkGray)),
            Span::styled(" PgUp/PgDn", Style::default().fg(Color::Cyan)),
            Span::styled(" Scroll  ", Style::default().fg(Color::DarkGray)),
            Span::styled(" Ctrl+S", Style::default().fg(Color::Cyan)),
            Span::styled(" Sessions", Style::default().fg(Color::DarkGray)),
        ]
    };
    let para = Paragraph::new(Line::from(spans));
    f.render_widget(para, area);
}

/// 渲染全部五个区域。
pub fn render_all(f: &mut ratatui::Frame, regions: LayoutRegions, state: &AppState, input: &str, scroll_offset: usize, session_id: &str) {
    let footer = state.footer.read();
    let model = footer.model.clone();
    let provider = footer.provider.clone();
    let is_running = !matches!(&footer.state, AgentState::Idle);
    drop(footer);

    render_header(f, regions.header, &model, &provider, session_id);
    render_chat(f, regions.chat, state, scroll_offset);
    render_status(f, regions.status, state);
    render_editor(f, regions.editor, input, true, is_running);
    render_footer(f, regions.footer, is_running);
}

/// Leaked string for static lifetime (used in status display).
/// Only used for short-lived display strings — acceptable leak.
fn leak_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

/// 计算 unicode 显示宽度。
fn unicode_width_str(s: &str) -> usize {
    s.chars().map(unicode_width).sum()
}
