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
use crate::markdown::render_markdown_truncated;

/// 工具结果最大显示行数。
const MAX_TOOL_LINES: usize = 8;

/// 主题驱动的颜色解析器 — 从 Theme 对象提取 ratatui Color。
///
/// 提供零参数构造函数（使用默认 dark 主题），
/// 以及 from_theme() 构造函数（从活跃主题解析所有颜色）。
pub struct ThemeColors {
    pub accent: Color,
    pub border: Color,
    pub border_accent: Color,
    pub success: Color,
    pub error: Color,
    pub warning: Color,
    pub muted: Color,
    pub dim: Color,
    pub dark_gray: Color,
    pub text: Color,
    pub text_muted: Color,
    pub background: Color,
    pub background_panel: Color,
    pub border_subtle: Color,
    pub user_msg_fg: Color,
    pub assistant_fg: Color,
    pub tool_fg: Color,
    pub heading: Color,
    pub code: Color,
    pub inline_code: Color,
    pub link: Color,
    pub syntax_keyword: Color,
    pub syntax_string: Color,
    pub syntax_comment: Color,
    pub diff_added: Color,
    pub diff_removed: Color,
    /// 灰阶梯度（4 级）。
    pub grays: [Color; 4],
    /// 用户消息背景色。
    pub user_msg_bg: Color,
    /// 工具成功背景色。
    pub tool_success_bg: Color,
    /// 工具错误背景色。
    pub tool_error_bg: Color,
    /// 思考文字色。
    pub thinking: Color,
}

impl ThemeColors {
    /// 从 Theme 对象解析所有颜色。
    pub fn from_theme(t: &crate::theme::Theme) -> Self {
        let grays_vec = t.gray_steps(4);
        let grays = [grays_vec[0], grays_vec[1], grays_vec[2], grays_vec[3]];

        let bg = t.resolve(&t.background);
        let success = t.resolve(&t.success);
        let error = t.resolve(&t.error);

        // 计算用户消息背景色 = 背景色 + success 色微混
        let user_msg_bg = tint(bg, success, 0.08);
        let tool_success_bg = tint(bg, success, 0.12);
        let tool_error_bg = tint(bg, error, 0.12);

        Self {
            accent: t.resolve(&t.accent),
            border: t.resolve(&t.border),
            border_accent: t.resolve(&t.primary),
            success,
            error,
            warning: t.resolve(&t.warning),
            muted: t.resolve(&t.text_muted),
            dim: t.resolve(&t.text_muted),
            dark_gray: grays[1],
            text: t.resolve(&t.text),
            text_muted: t.resolve(&t.text_muted),
            background: bg,
            background_panel: t.resolve(&t.background_panel),
            border_subtle: t.resolve(&t.border_subtle),
            user_msg_fg: t.resolve(&t.user_msg),
            assistant_fg: t.resolve(&t.assistant_msg),
            tool_fg: t.resolve(&t.tool_msg),
            heading: t.resolve(&t.heading_fg),
            code: t.resolve(&t.code_fg),
            inline_code: t.resolve(&t.inline_code_fg),
            link: t.resolve(&t.link_fg),
            syntax_keyword: t.resolve(&t.syntax_keyword),
            syntax_string: t.resolve(&t.syntax_string),
            syntax_comment: t.resolve(&t.syntax_comment),
            diff_added: t.resolve(&t.diff_added),
            diff_removed: t.resolve(&t.diff_removed),
            grays,
            user_msg_bg,
            tool_success_bg,
            tool_error_bg,
            thinking: t.resolve(&t.text_muted),
        }
    }
}

/// 颜色混合：base + overlay * alpha。
fn tint(base: Color, overlay: Color, alpha: f64) -> Color {
    let (br, bg, bb) = to_rgb(base);
    let (or, og, ob) = to_rgb(overlay);
    let r = (br + (or - br) * alpha).round().clamp(0.0, 255.0) as u8;
    let g = (bg + (og - bg) * alpha).round().clamp(0.0, 255.0) as u8;
    let b = (bb + (ob - bb) * alpha).round().clamp(0.0, 255.0) as u8;
    Color::Rgb(r, g, b)
}

/// 将 ratatui Color 提取为 (r, g, b) f64。
fn to_rgb(c: Color) -> (f64, f64, f64) {
    match c {
        Color::Rgb(r, g, b) => (r as f64, g as f64, b as f64),
        Color::Black => (0.0, 0.0, 0.0),
        Color::White => (255.0, 255.0, 255.0),
        Color::Red => (255.0, 0.0, 0.0),
        Color::Green => (0.0, 255.0, 0.0),
        Color::Blue => (0.0, 0.0, 255.0),
        Color::Cyan => (0.0, 255.0, 255.0),
        Color::Magenta => (255.0, 0.0, 255.0),
        Color::Yellow => (255.0, 255.0, 0.0),
        Color::DarkGray => (80.0, 80.0, 80.0),
        Color::Gray => (128.0, 128.0, 128.0),
        _ => (128.0, 128.0, 128.0),
    }
}

/// Braille spinner 帧序列（与 TS 版 DEFAULT_FRAMES 一致）。
const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// 获取当前 spinner 帧。
fn spinner_frame(tick: usize) -> &'static str {
    SPINNER_FRAMES[tick % SPINNER_FRAMES.len()]
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
pub fn render_chat(
    f: &mut ratatui::Frame,
    area: Rect,
    state: &AppState,
    scroll_offset: usize,
    tick: usize,
    colors: &ThemeColors,
) {
    let entries = state.entries.read();
    let content_width = area.width.saturating_sub(4) as usize;
    let mut lines: Vec<Line> = Vec::new();

    for entry in entries.iter() {
        match &entry.role {
            ChatRole::User => {
                // 用户消息：带背景色的块
                let wrapped = wrap_text(&entry.content, content_width);
                for line in &wrapped {
                    lines.push(Line::from(Span::styled(
                        format!(" {}", line),
                        Style::default()
                            .bg(colors.user_msg_bg)
                            .fg(colors.user_msg_fg),
                    )));
                }
                // 填充背景色到宽度
                lines.push(Line::from(Span::styled(
                    " ".repeat(content_width),
                    Style::default().bg(colors.user_msg_bg),
                )));
            }
            ChatRole::Assistant => {
                // 助手消息：无背景色，Markdown 渲染
                let content_style = Style::default();
                let (md_lines, _truncated) =
                    render_markdown_truncated(&entry.content, content_style);
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
                    Style::default().fg(colors.dark_gray),
                )));
                let wrapped = wrap_text(&entry.content, content_width);
                for line in &wrapped {
                    lines.push(Line::from(Span::styled(
                        format!(" ┊ {}", line),
                        Style::default().fg(colors.thinking),
                    )));
                }
            }
            ChatRole::System => {
                lines.push(Line::from(Span::styled(
                    format!(" ℹ {}", &entry.content),
                    Style::default().fg(colors.warning),
                )));
            }
            ChatRole::Tool { name, is_error } => {
                let bg = if *is_error {
                    colors.tool_error_bg
                } else {
                    colors.tool_success_bg
                };
                let icon = if *is_error { "✗" } else { "✔" };
                let name_color = if *is_error {
                    colors.error
                } else {
                    colors.success
                };

                // 工具头部
                lines.push(Line::from(vec![Span::styled(
                    format!(" {} {} ", icon, name),
                    Style::default()
                        .fg(name_color)
                        .add_modifier(Modifier::BOLD)
                        .bg(bg),
                )]));

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
                    Style::default().fg(colors.error)
                } else {
                    Style::default().fg(colors.muted)
                };
                for line in &display_lines {
                    lines.push(Line::from(Span::styled(format!(" {}", line), output_style)));
                }
            }
        }

        lines.push(Line::from("")); // 空行分隔
    }

    // 思考/工具执行指示器（带 spinner 动画）
    let footer_state = state.footer.read();
    let frame = spinner_frame(tick);
    match &footer_state.state {
        AgentState::Thinking => {
            lines.push(Line::from(Span::styled(
                format!(" {} Thinking...", frame),
                Style::default().fg(colors.warning),
            )));
        }
        AgentState::Streaming => {
            lines.push(Line::from(Span::styled(
                format!(" {} Streaming...", frame),
                Style::default().fg(colors.border_accent),
            )));
        }
        AgentState::ToolRunning { name } => {
            lines.push(Line::from(Span::styled(
                format!(" {} Running {}...", frame, name),
                Style::default().fg(colors.accent),
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
pub fn render_status(f: &mut ratatui::Frame, area: Rect, state: &AppState, colors: &ThemeColors) {
    let footer = state.footer.read();

    // 左侧：状态 + token + context
    let (state_text, state_color) = match &footer.state {
        AgentState::Idle => ("●", colors.success),
        AgentState::Thinking => ("◌", colors.warning),
        AgentState::Streaming => ("●", colors.border_accent),
        AgentState::ToolRunning { name } => {
            let display = if name.len() > 16 {
                format!("{}…", &name[..14])
            } else {
                name.clone()
            };
            (leak_str(format!("▶ {}", display)), colors.accent)
        }
        AgentState::Error(e) => {
            let display = if e.len() > 30 {
                format!("✗ {}…", &e[..24])
            } else {
                format!("✗ {}", e)
            };
            (leak_str(display), colors.error)
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
            Style::default().fg(colors.dim),
        ));
    }

    // context 占用
    if footer.context_tokens > 0 {
        let pct = (footer.context_tokens as f64 / 128_000.0 * 100.0) as u32;
        let pct = pct.min(100);
        let ctx_color = if pct > 80 {
            colors.error
        } else if pct > 50 {
            colors.warning
        } else {
            colors.dim
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
        left_parts.push(Span::styled(elapsed, Style::default().fg(colors.dim)));
    }

    let line = Line::from(left_parts);
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
    colors: &ThemeColors,
) {
    let border_color = if is_running {
        colors.warning
    } else {
        colors.border
    };

    if input.is_empty() {
        let hint = if is_running {
            "Waiting for agent..."
        } else {
            "Type a message... (Enter to send, Shift+Enter for newline)"
        };
        let para = Paragraph::new(hint)
            .style(Style::default().fg(colors.dark_gray))
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

/// 渲染 footer 区域 — 模型信息 + 快捷键提示。
#[allow(clippy::too_many_arguments)]
pub fn render_footer(
    f: &mut ratatui::Frame,
    area: Rect,
    _is_running: bool,
    model: &str,
    provider: &str,
    git: &str,
    hints: &[(&'static str, String)],
    colors: &ThemeColors,
) {
    let mut spans = Vec::new();

    // 左侧：模型信息
    spans.push(Span::styled(
        format!(" {} ({})", model, provider),
        Style::default().fg(colors.dim),
    ));

    // Git 分支
    if !git.is_empty() {
        spans.push(Span::styled(
            format!(" {}", git),
            Style::default().fg(colors.accent),
        ));
    }

    // 分隔
    spans.push(Span::styled("  ".to_string(), Style::default()));

    // 快捷键提示
    for (tag, text) in hints {
        if *tag == "key" {
            spans.push(Span::styled(
                text.clone(),
                Style::default()
                    .fg(colors.border_accent)
                    .add_modifier(Modifier::BOLD),
            ));
        } else {
            spans.push(Span::styled(text.clone(), Style::default().fg(colors.dim)));
        }
    }

    let para = Paragraph::new(Line::from(spans));
    f.render_widget(para, area);
}

/// 渲染全部五个区域。
/// 渲染全部四个区域。
#[allow(clippy::too_many_arguments)]
pub fn render_all(
    f: &mut ratatui::Frame,
    regions: LayoutRegions,
    state: &AppState,
    input: &str,
    cursor_pos: usize,
    scroll_offset: usize,
    git: &str,
    footer_hints: &[(&'static str, String)],
    tick: usize,
    theme: &crate::theme::Theme,
) {
    let colors = ThemeColors::from_theme(theme);
    let footer = state.footer.read();
    let model = footer.model.clone();
    let provider = footer.provider.clone();
    let is_running = !matches!(&footer.state, AgentState::Idle);
    drop(footer);

    render_chat(f, regions.chat, state, scroll_offset, tick, &colors);
    render_status(f, regions.status, state, &colors);
    render_editor(
        f,
        regions.editor,
        input,
        cursor_pos,
        true,
        is_running,
        &colors,
    );
    render_footer(
        f,
        regions.footer,
        is_running,
        &model,
        &provider,
        git,
        footer_hints,
        &colors,
    );
}

/// Leaked string for static lifetime.
fn leak_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}
