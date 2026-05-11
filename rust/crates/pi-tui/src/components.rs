//! 渲染组件 — 五个区域的 ratatui 渲染。

use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::{AgentState, AppState, ChatRole};
use crate::layout::LayoutRegions;

/// 渲染 header 区域。
pub fn render_header(f: &mut ratatui::Frame, area: Rect, model: &str, provider: &str) {
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
            " [Ctrl+O submit, Ctrl+C quit]",
            Style::default().fg(Color::DarkGray),
        ),
    ]);
    let para = Paragraph::new(line);
    f.render_widget(para, area);
}

/// 渲染 chat 区域。
pub fn render_chat(f: &mut ratatui::Frame, area: Rect, state: &AppState) {
    let entries = state.entries.read();
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

        // 内容行（截断到区域宽度）
        let content_width = area.width.saturating_sub(2) as usize;
        for content_line in entry.content.lines() {
            let truncated = if content_line.len() > content_width {
                format!("{}...", &content_line[..content_width.saturating_sub(3)])
            } else {
                content_line.to_string()
            };
            lines.push(Line::from(format!("   {}", truncated)));
        }

        lines.push(Line::from("")); // 空行分隔
    }

    // 如果正在思考，显示指示器
    let footer_state = state.footer.read();
    if matches!(footer_state.state, AgentState::Thinking) {
        lines.push(Line::from(
            Span::styled(" ● Thinking...", Style::default().fg(Color::Yellow)),
        ));
    } else if let AgentState::ToolRunning { name } = &footer_state.state {
        lines.push(Line::from(
            Span::styled(
                format!(" ● Running {}...", name),
                Style::default().fg(Color::Magenta),
            ),
        ));
    }

    // 滚动到底部
    let visible = area.height as usize;
    let total = lines.len();
    let start = total.saturating_sub(visible);

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
        AgentState::ToolRunning { name } => (name.as_str(), Color::Magenta),
        AgentState::Error(e) => (e.as_str(), Color::Red),
    };

    let tokens = if footer.input_tokens + footer.output_tokens > 0 {
        format!(" │ {}in/{}out", footer.input_tokens, footer.output_tokens)
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

/// 渲染 editor 区域。
pub fn render_editor(f: &mut ratatui::Frame, area: Rect, input: &str, cursor: bool) {
    let display = if input.is_empty() {
        "Type a message...".to_string()
    } else {
        input.to_string()
    };

    let style = if input.is_empty() {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default().fg(Color::White)
    };

    let para = Paragraph::new(display).style(style).block(
        Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    if cursor && !input.is_empty() {
        f.set_cursor(
            area.x + (input.len() as u16).min(area.width.saturating_sub(2)),
            area.y + 1,
        );
    }

    f.render_widget(para, area);
}

/// 渲染 footer 区域。
pub fn render_footer(f: &mut ratatui::Frame, area: Rect) {
    let line = Line::from(vec![
        Span::styled(" Ctrl+O", Style::default().fg(Color::Cyan)),
        Span::styled(" Send  ", Style::default().fg(Color::DarkGray)),
        Span::styled(" Ctrl+C", Style::default().fg(Color::Cyan)),
        Span::styled(" Quit  ", Style::default().fg(Color::DarkGray)),
        Span::styled(" Esc", Style::default().fg(Color::Cyan)),
        Span::styled(" Cancel", Style::default().fg(Color::DarkGray)),
    ]);
    let para = Paragraph::new(line);
    f.render_widget(para, area);
}

/// 渲染全部五个区域。
pub fn render_all(f: &mut ratatui::Frame, regions: LayoutRegions, state: &AppState, input: &str) {
    let footer = state.footer.read();
    let model = footer.model.clone();
    let provider = footer.provider.clone();
    drop(footer);

    render_header(f, regions.header, &model, &provider);
    render_chat(f, regions.chat, state);
    render_status(f, regions.status, state);
    render_editor(f, regions.editor, input, true);
    render_footer(f, regions.footer);
}
