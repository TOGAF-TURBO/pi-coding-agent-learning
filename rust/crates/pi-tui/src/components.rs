//! 渲染组件 — 五个区域的 ratatui 渲染。
//!
//! 对齐 TS 版本的视觉风格：
//! - Header: 品牌 logo + 模型信息
//! - Chat: 用户消息带背景色、工具执行带状态色、thinking 暗灰
//! - Status: 左侧 token/ctx，右侧 模型名
//! - Editor: 主题色边框
//! - Footer: 快捷键提示

use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::{AgentState, AppState, ChatRole};
use crate::layout::LayoutRegions;
use crate::markdown::render_markdown;

/// 工具结果最大显示行数。
const MAX_TOOL_LINES: usize = 8;

/// 主题色彩常量（与 TS dark.json 对齐）。
pub(crate) mod colors {
    use ratatui::style::Color;

    // 精确色值来自 TS dark.json vars
    pub const ACCENT: Color = Color::Rgb(138, 190, 183);       // accent
    pub const BORDER: Color = Color::Rgb(95, 135, 255);        // blue
    pub const BORDER_ACCENT: Color = Color::Rgb(0, 215, 255);  // cyan
    pub const SUCCESS: Color = Color::Rgb(181, 189, 104);      // green
    pub const ERROR: Color = Color::Rgb(204, 102, 102);        // red
    pub const WARNING: Color = Color::Rgb(255, 255, 0);        // yellow
    pub const MUTED: Color = Color::Rgb(128, 128, 128);        // gray
    pub const DIM: Color = Color::Rgb(102, 102, 102);          // dimGray
    pub const DARK_GRAY: Color = Color::Rgb(80, 80, 80);       // darkGray
    #[allow(dead_code)]
    pub const MD_HEADING: Color = Color::Rgb(240, 198, 116);   // #f0c674
    #[allow(dead_code)]
    pub const MD_LINK: Color = Color::Rgb(129, 162, 190);      // #81a2be
    #[allow(dead_code)]
    pub const MD_CODE: Color = Color::Rgb(138, 190, 183);      // accent
    pub const USER_MSG_BG: Color = Color::Rgb(52, 53, 65);     // #343541
    #[allow(dead_code)]
    pub const TOOL_PENDING_BG: Color = Color::Rgb(40, 40, 50); // #282832
    pub const TOOL_SUCCESS_BG: Color = Color::Rgb(40, 50, 40); // #283228
    pub const TOOL_ERROR_BG: Color = Color::Rgb(60, 40, 40);   // #3c2828
    pub const THINKING: Color = Color::Rgb(128, 128, 128);     // gray
    pub const SELECTED_BG: Color = Color::Rgb(58, 58, 74);     // #3a3a4a
}

/// 格式化 token 数量（与 TS 版一致）。
fn format_tokens(count: u32) -> String {
    if count < 1000 {
        count.to_string()
    } else if count < 10_000 {
        format!("{:.1}k", count as f64 / 1000.0)
    } else if count < 1_000_000 {
        format!("{}k", count / 1000)
    } else {
        format!("{:.1}M", count as f64 / 1_000_000.0)
    }
}

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
    let short_id = if session_id.len() > 8 {
        &session_id[..8]
    } else {
        session_id
    };

    let mut spans = vec![
        // 品牌名
        Span::styled(
            " piso",
            Style::default()
                .fg(colors::BORDER_ACCENT)
                .add_modifier(Modifier::BOLD),
        ),
        // 分隔符
        Span::styled(
            " │ ",
            Style::default().fg(colors::DARK_GRAY),
        ),
        // 模型信息
        Span::styled(
            format!("{} ({})", model, provider),
            Style::default().fg(colors::DIM),
        ),
    ];

    // Git 分支
    if !git.is_empty() {
        spans.push(Span::styled(
            " ─ ".to_string(),
            Style::default().fg(colors::DARK_GRAY),
        ));
        spans.push(Span::styled(
            format!("{}", git),
            Style::default().fg(colors::ACCENT),
        ));
    }

    // Session ID 右对齐（用空格填充）
    let right_text = format!("{} ", short_id);
    spans.push(Span::raw(right_text));

    let line = Line::from(spans);
    let para = Paragraph::new(line).style(Style::default().bg(colors::SELECTED_BG));
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

/// Unicode 字符显示宽度。
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
pub fn render_chat(f: &mut ratatui::Frame, area: Rect, state: &AppState, scroll_offset: usize) {
    let entries = state.entries.read();
    let content_width = area.width.saturating_sub(4) as usize;
    let mut lines: Vec<Line> = Vec::new();

    for entry in entries.iter() {
        match &entry.role {
            ChatRole::User => {
                // 用户消息：带背景色的块
                lines.push(Line::from(Span::styled(
                    " ── You ──────────────────────────────────",
                    Style::default()
                        .fg(colors::ACCENT)
                        .add_modifier(Modifier::BOLD),
                )));
                let wrapped = wrap_text(&entry.content, content_width);
                for line in &wrapped {
                    lines.push(Line::from(Span::styled(
                        format!(" {}", line),
                        Style::default().bg(colors::USER_MSG_BG),
                    )));
                }
                // 填充背景色到宽度（确保整行有背景色）
                lines.push(Line::from(Span::styled(
                    " ".repeat(content_width),
                    Style::default().bg(colors::USER_MSG_BG),
                )));
            }
            ChatRole::Assistant => {
                // 助手消息：无背景色，Markdown 渲染
                let content_style = Style::default();
                let md_lines = render_markdown(&entry.content, content_style);
                for md_line in md_lines {
                    let mut spans: Vec<Span<'static>> = Vec::new();
                    for s in md_line.spans {
                        spans.push(Span::styled(s.content, s.style));
                    }
                    if spans.is_empty() {
                        spans.push(Span::raw(""));
                    }
                    lines.push(Line::from(spans));
                }
            }
            ChatRole::Thinking => {
                // 思考内容：暗灰 + 斜体
                lines.push(Line::from(Span::styled(
                    " ┊ Thinking",
                    Style::default().fg(colors::DARK_GRAY),
                )));
                let wrapped = wrap_text(&entry.content, content_width);
                for line in &wrapped {
                    lines.push(Line::from(Span::styled(
                        format!(" ┊ {}", line),
                        Style::default().fg(colors::THINKING),
                    )));
                }
            }
            ChatRole::System => {
                lines.push(Line::from(Span::styled(
                    format!(" ℹ {}", &entry.content),
                    Style::default().fg(colors::WARNING),
                )));
            }
            ChatRole::Tool { name, is_error } => {
                let bg = if *is_error {
                    colors::TOOL_ERROR_BG
                } else {
                    colors::TOOL_SUCCESS_BG
                };
                let icon = if *is_error { "✗" } else { "✔" };
                let name_color = if *is_error {
                    colors::ERROR
                } else {
                    colors::SUCCESS
                };

                // 工具头部
                lines.push(Line::from(vec![
                    Span::styled(
                        format!(" {} {} ", icon, name),
                        Style::default()
                            .fg(name_color)
                            .add_modifier(Modifier::BOLD)
                            .bg(bg),
                    ),
                ]));

                // 工具输出
                let wrapped = wrap_text(&entry.content, content_width);
                let display_lines: Vec<String> = if wrapped.len() > MAX_TOOL_LINES {
                    let mut truncated: Vec<String> = wrapped[..MAX_TOOL_LINES].to_vec();
                    truncated.push(format!(
                        "  ... ({} more lines)",
                        wrapped.len() - MAX_TOOL_LINES
                    ));
                    truncated
                } else {
                    wrapped
                };

                let output_style = if *is_error {
                    Style::default().fg(colors::ERROR)
                } else {
                    Style::default().fg(colors::MUTED)
                };
                for line in &display_lines {
                    lines.push(Line::from(Span::styled(
                        format!(" {}", line),
                        output_style,
                    )));
                }
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
                Style::default().fg(colors::WARNING),
            )));
        }
        AgentState::Streaming => {
            lines.push(Line::from(Span::styled(
                " ● Streaming...",
                Style::default().fg(colors::BORDER_ACCENT),
            )));
        }
        AgentState::ToolRunning { name } => {
            lines.push(Line::from(Span::styled(
                format!(" ● Running {}...", name),
                Style::default().fg(colors::ACCENT),
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

    // 左侧：状态 + token + context
    let (state_text, state_color) = match &footer.state {
        AgentState::Idle => ("●", colors::SUCCESS),
        AgentState::Thinking => ("◌", colors::WARNING),
        AgentState::Streaming => ("●", colors::BORDER_ACCENT),
        AgentState::ToolRunning { name } => {
            let display = if name.len() > 16 {
                format!("{}…", &name[..14])
            } else {
                name.clone()
            };
            (leak_str(format!("▶ {}", display)), colors::ACCENT)
        }
        AgentState::Error(e) => {
            let display = if e.len() > 30 {
                format!("✗ {}…", &e[..24])
            } else {
                format!("✗ {}", e)
            };
            (leak_str(display), colors::ERROR)
        }
    };

    // token 统计
    let mut left_parts = vec![Span::styled(
        format!(" {} ", state_text),
        Style::default()
            .fg(state_color)
            .add_modifier(Modifier::BOLD),
    )];

    if footer.input_tokens + footer.output_tokens > 0 {
        left_parts.push(Span::styled(
            format!(
                "↑{} ↓{}",
                format_tokens(footer.input_tokens),
                format_tokens(footer.output_tokens)
            ),
            Style::default().fg(colors::DIM),
        ));
    }

    // context 占用
    if footer.context_tokens > 0 {
        let pct = (footer.context_tokens as f64 / 128_000.0 * 100.0) as u32;
        let pct = pct.min(100);
        let ctx_color = if pct > 80 {
            colors::ERROR
        } else if pct > 50 {
            colors::WARNING
        } else {
            colors::DIM
        };
        left_parts.push(Span::styled(
            format!("  ctx:{}%", pct),
            Style::default().fg(ctx_color),
        ));
    }

    // 耗时
    let elapsed = if let Some(start) = *state.turn_start.read() {
        let secs = start.elapsed().as_secs();
        if secs >= 60 {
            format!("  {}m{}s", secs / 60, secs % 60)
        } else {
            format!("  {}s", secs)
        }
    } else if footer.duration_secs > 0 {
        if footer.duration_secs >= 60 {
            format!(
                "  {}m{}s",
                footer.duration_secs / 60,
                footer.duration_secs % 60
            )
        } else {
            format!("  {}s", footer.duration_secs)
        }
    } else {
        String::new()
    };
    if !elapsed.is_empty() {
        left_parts.push(Span::styled(
            elapsed,
            Style::default().fg(colors::DIM),
        ));
    }

    // 右侧：模型名
    let model_display = format!("{} ", footer.model);
    let right_width = unicode_width_str(&model_display);
    let left_width: usize = left_parts.iter().map(|s| unicode_width_str(&s.content)).sum();
    let area_width = area.width as usize;
    let padding = area_width.saturating_sub(left_width).saturating_sub(right_width);

    let mut all_spans = left_parts;
    if padding > 0 {
        all_spans.push(Span::raw(" ".repeat(padding)));
    }
    all_spans.push(Span::styled(
        model_display,
        Style::default().fg(colors::DIM),
    ));

    let line = Line::from(all_spans);
    let para = Paragraph::new(line);
    f.render_widget(para, area);
}

/// 渲染 editor 区域。
pub fn render_editor(
    f: &mut ratatui::Frame,
    area: Rect,
    input: &str,
    cursor_pos: usize,
    cursor: bool,
    is_running: bool,
) {
    let border_color = if is_running {
        colors::WARNING
    } else {
        colors::BORDER
    };

    if input.is_empty() {
        let hint = if is_running {
            "Waiting for agent..."
        } else {
            "Type a message... (Enter to send, Shift+Enter for newline)"
        };
        let para = Paragraph::new(hint)
            .style(Style::default().fg(colors::DARK_GRAY))
            .block(
                Block::default()
                    .borders(Borders::TOP)
                    .border_style(Style::default().fg(border_color)),
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
                .border_style(Style::default().fg(border_color)),
        );

    f.render_widget(para, area);

    if cursor && !is_running {
        let before = &input[..cursor_pos];
        let area_width = area.width.saturating_sub(2) as usize;
        let area_width = area_width.max(1);

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
                    col = w;
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
            spans.push(Span::styled(
                text.clone(),
                Style::default()
                    .fg(colors::BORDER_ACCENT)
                    .add_modifier(Modifier::BOLD),
            ));
        } else {
            spans.push(Span::styled(
                text.clone(),
                Style::default().fg(colors::DIM),
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

/// Leaked string for static lifetime.
fn leak_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

/// Unicode 字符串显示宽度。
fn unicode_width_str(s: &str) -> usize {
    s.chars().map(unicode_width).sum()
}
