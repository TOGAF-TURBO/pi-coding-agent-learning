//! 交互式 diff 查看器组件。
//!
//! 在 TUI 中渲染 unified diff，支持多文件切换和键盘导航：
//! - j/k: 上下滚动
//! - n/p: 切换文件
//! - q/Esc: 退出查看器
//! - 绿色 + added 行，红色 - removed 行，灰色 context 行

use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

/// diff 中的一行。
#[derive(Debug, Clone)]
pub enum DiffLine {
    /// 上下文行（不变）。
    Context(String),
    /// 添加行。
    Added(String),
    /// 删除行。
    Removed(String),
    /// 文件头（--- a/foo.rs）。
    FileHeader(String),
    /// Hunk 头（@@ -1,3 +1,4 @@）。
    HunkHeader(String),
}

/// 一个文件的 diff。
#[derive(Debug, Clone)]
pub struct FileDiff {
    pub path: String,
    pub lines: Vec<DiffLine>,
}

/// Diff 查看器状态。
#[derive(Debug, Clone)]
pub struct DiffViewer {
    /// 所有文件的 diff。
    files: Vec<FileDiff>,
    /// 当前文件索引。
    current_file: usize,
    /// 滚动偏移。
    scroll_offset: u16,
    /// 是否可见。
    visible: bool,
}

impl DiffViewer {
    /// 创建空的 diff 查看器。
    pub fn new() -> Self {
        Self {
            files: Vec::new(),
            current_file: 0,
            scroll_offset: 0,
            visible: false,
        }
    }

    /// 从 unified diff 文本解析。
    pub fn from_unified_diff(text: &str) -> Self {
        let mut files = Vec::new();
        let mut current_file: Option<FileDiff> = None;

        for line in text.lines() {
            if line.starts_with("diff --git") {
                // 新文件开始
                if let Some(f) = current_file.take() {
                    files.push(f);
                }
                let path = line
                    .split_whitespace()
                    .nth(3)
                    .unwrap_or("unknown")
                    .trim_start_matches("b/")
                    .to_string();
                current_file = Some(FileDiff {
                    path,
                    lines: vec![DiffLine::FileHeader(line.to_string())],
                });
                continue;
            }

            if line.starts_with("--- a/") || line.starts_with("--- ") {
                if current_file.is_none() {
                    // 没有 diff --git 前缀的独立 diff
                    let path = line
                        .trim_start_matches("--- a/")
                        .trim_start_matches("--- ")
                        .to_string();
                    current_file = Some(FileDiff {
                        path,
                        lines: vec![DiffLine::FileHeader(line.to_string())],
                    });
                } else if let Some(ref mut f) = current_file {
                    f.lines.push(DiffLine::FileHeader(line.to_string()));
                }
                continue;
            }

            if line.starts_with("+++ b/") {
                // 通常紧跟 --- a/ 后面，跳过或添加
                if let Some(ref mut f) = current_file {
                    f.lines.push(DiffLine::FileHeader(line.to_string()));
                }
                continue;
            }

            if line.starts_with("@@") {
                if let Some(ref mut f) = current_file {
                    f.lines.push(DiffLine::HunkHeader(line.to_string()));
                }
                continue;
            }

            if let Some(ref mut f) = current_file {
                if line.starts_with('+') {
                    f.lines.push(DiffLine::Added(line.to_string()));
                } else if line.starts_with('-') {
                    f.lines.push(DiffLine::Removed(line.to_string()));
                } else if line.starts_with(' ') {
                    f.lines.push(DiffLine::Context(
                        line.strip_prefix(' ').unwrap_or(line).to_string(),
                    ));
                } else {
                    f.lines.push(DiffLine::Context(line.to_string()));
                }
            }
        }

        if let Some(f) = current_file.take() {
            files.push(f);
        }

        Self {
            files,
            current_file: 0,
            scroll_offset: 0,
            visible: false,
        }
    }

    /// 显示查看器。
    pub fn show(&mut self) {
        self.visible = true;
        self.current_file = 0;
        self.scroll_offset = 0;
    }

    /// 隐藏查看器。
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// 是否可见。
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// 是否有内容。
    pub fn has_files(&self) -> bool {
        !self.files.is_empty()
    }

    /// 获取当前文件。
    pub fn current_file(&self) -> Option<&FileDiff> {
        self.files.get(self.current_file)
    }

    /// 文件总数。
    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    /// 下一个文件。
    pub fn next_file(&mut self) {
        if !self.files.is_empty() {
            self.current_file = (self.current_file + 1) % self.files.len();
            self.scroll_offset = 0;
        }
    }

    /// 上一个文件。
    pub fn prev_file(&mut self) {
        if !self.files.is_empty() {
            self.current_file = if self.current_file == 0 {
                self.files.len() - 1
            } else {
                self.current_file - 1
            };
            self.scroll_offset = 0;
        }
    }

    /// 向下滚动。
    pub fn scroll_down(&mut self, amount: u16) {
        self.scroll_offset = self.scroll_offset.saturating_add(amount);
    }

    /// 向上滚动。
    pub fn scroll_up(&mut self, amount: u16) {
        self.scroll_offset = self.scroll_offset.saturating_sub(amount);
    }

    /// 渲染到 frame。
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let file = match self.current_file() {
            Some(f) => f,
            None => {
                let text = Paragraph::new("No diff to display")
                    .block(Block::default().borders(Borders::ALL).title("Diff Viewer"));
                frame.render_widget(text, area);
                return;
            }
        };

        let title = format!(
            "Diff [{}/{}] {} (j/k:scroll n/p:file q:quit)",
            self.current_file + 1,
            self.files.len(),
            file.path
        );

        let lines: Vec<Line> = file
            .lines
            .iter()
            .map(|dl| match dl {
                DiffLine::FileHeader(text) => Line::from(Span::styled(
                    text.clone(),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )),
                DiffLine::HunkHeader(text) => Line::from(Span::styled(
                    text.clone(),
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::BOLD),
                )),
                DiffLine::Added(text) => Line::from(vec![
                    Span::styled("+", Style::default().fg(Color::Green)),
                    Span::styled(
                        text.clone(),
                        Style::default().fg(Color::Green),
                    ),
                ]),
                DiffLine::Removed(text) => Line::from(vec![
                    Span::styled("-", Style::default().fg(Color::Red)),
                    Span::styled(
                        text.clone(),
                        Style::default().fg(Color::Red),
                    ),
                ]),
                DiffLine::Context(text) => {
                    Line::from(Span::styled(
                        format!(" {}", text),
                        Style::default().fg(Color::DarkGray),
                    ))
                }
            })
            .collect();

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title),
            )
            .scroll((self.scroll_offset, 0))
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_diff() {
        let viewer = DiffViewer::from_unified_diff("");
        assert_eq!(viewer.file_count(), 0);
    }

    #[test]
    fn parse_single_file_diff() {
        let diff = r#"diff --git a/foo.rs b/foo.rs
--- a/foo.rs
+++ b/foo.rs
@@ -1,3 +1,4 @@
 old line 1
-old line 2
+new line 2
+new line 3
 old line 4"#;

        let viewer = DiffViewer::from_unified_diff(diff);
        assert_eq!(viewer.file_count(), 1);

        let file = viewer.current_file().unwrap();
        assert_eq!(file.path, "foo.rs");

        // FileHeader, FileHeader, FileHeader, HunkHeader, Context, Removed, Added, Added, Context
        assert_eq!(file.lines.len(), 9);
    }

    #[test]
    fn parse_multi_file_diff() {
        let diff = r#"diff --git a/a.rs b/a.rs
--- a/a.rs
+++ b/a.rs
@@ -1 +1 @@
-old
+new
diff --git b/b.rs c/b.rs
--- a/b.rs
+++ b/b.rs
@@ -1 +1 @@
-foo
+bar"#;

        let viewer = DiffViewer::from_unified_diff(diff);
        assert_eq!(viewer.file_count(), 2);
        assert_eq!(viewer.files[0].path, "a.rs");
        assert_eq!(viewer.files[1].path, "c/b.rs");
    }

    #[test]
    fn navigation() {
        let diff = r#"diff --git a/a.rs b/a.rs
--- a/a.rs
@@ -1 +1 @@
-a
+b
diff --git b/b.rs c/b.rs
--- a/b.rs
@@ -1 +1 @@
-c
+d"#;

        let mut viewer = DiffViewer::from_unified_diff(diff);
        assert_eq!(viewer.current_file, 0);

        viewer.next_file();
        assert_eq!(viewer.current_file, 1);

        viewer.next_file();
        assert_eq!(viewer.current_file, 0); // wraps

        viewer.prev_file();
        assert_eq!(viewer.current_file, 1); // wraps
    }

    #[test]
    fn scroll_offset() {
        let mut viewer = DiffViewer::new();
        viewer.scroll_down(5);
        assert_eq!(viewer.scroll_offset, 5);
        viewer.scroll_up(3);
        assert_eq!(viewer.scroll_offset, 2);
        viewer.scroll_up(10);
        assert_eq!(viewer.scroll_offset, 0);
    }

    #[test]
    fn show_hide() {
        let mut viewer = DiffViewer::new();
        assert!(!viewer.is_visible());
        viewer.show();
        assert!(viewer.is_visible());
        viewer.hide();
        assert!(!viewer.is_visible());
    }
}
