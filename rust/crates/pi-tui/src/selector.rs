//! 选择器组件 — 通用的交互式列表选择 overlay。
//!
//! 用于会话选择、模型选择等场景。
//! 显示一个浮动列表，用户通过上下箭头和 Enter 选择。

use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

use crate::input::InputEditor;

/// 选择器条目。
#[derive(Debug, Clone)]
pub struct SelectItem {
    pub id: String,
    pub label: String,
    pub detail: String,
}

/// 选择器状态。
pub struct Selector {
    pub title: String,
    pub items: Vec<SelectItem>,
    /// 当前选中索引。
    pub selected: usize,
    /// 滚动偏移。
    pub scroll: usize,
}

impl Selector {
    pub fn new(title: &str, items: Vec<SelectItem>) -> Self {
        Self {
            title: title.to_string(),
            items,
            selected: 0,
            scroll: 0,
        }
    }

    /// 上移。
    pub fn up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            self.adjust_scroll();
        }
    }

    /// 下移。
    pub fn down(&mut self) {
        if self.selected + 1 < self.items.len() {
            self.selected += 1;
            self.adjust_scroll();
        }
    }

    /// 获取当前选中项的 ID。
    pub fn selected_id(&self) -> Option<&str> {
        self.items.get(self.selected).map(|i| i.id.as_str())
    }

    /// 根据可见行数调整滚动。
    fn adjust_scroll(&mut self) {
        // 由 render 时计算
    }

    /// 渲染选择器。
    pub fn render(f: &mut ratatui::Frame, area: Rect, selector: &mut Self) {
        let visible = area.height.saturating_sub(2) as usize; // borders

        // 调整滚动
        if selector.selected < selector.scroll {
            selector.scroll = selector.selected;
        }
        if selector.selected >= selector.scroll + visible {
            selector.scroll = selector.selected - visible + 1;
        }

        let lines: Vec<Line> = selector.items.iter()
            .skip(selector.scroll)
            .take(visible)
            .enumerate()
            .map(|(i, item)| {
                let idx = selector.scroll + i;
                let is_selected = idx == selector.selected;
                let prefix = if is_selected { " > " } else { "   " };
                let style = if is_selected {
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                let detail_style = Style::default().fg(Color::DarkGray);

                Line::from(vec![
                    Span::styled(prefix.to_string(), style),
                    Span::styled(item.label.clone(), style),
                    Span::styled(format!("  {}", item.detail), detail_style),
                ])
            })
            .collect();

        let para = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan))
                    .title(format!(" {} ", selector.title))
            );

        // 清除区域后渲染
        f.render_widget(Clear, area);
        f.render_widget(para, area);
    }
}
