//! 输入处理 — 多行文本编辑 + 历史记录。
//!
//! 支持基本编辑操作：输入字符、退格、删除、光标移动、回车换行。
//! 支持上下箭头浏览已发送的消息历史。

/// 最大历史记录条数。
const MAX_HISTORY: usize = 100;

/// 输入编辑器状态。
pub struct InputEditor {
    /// 输入文本。
    text: String,
    /// 光标位置（字节偏移）。
    cursor: usize,
    /// 历史记录（最新在末尾）。
    history: Vec<String>,
    /// 当前历史浏览位置（None = 不在浏览历史）。
    history_index: Option<usize>,
    /// 进入历史前的当前文本（用于恢复）。
    saved_text: String,
}

impl InputEditor {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            cursor: 0,
            history: Vec::new(),
            history_index: None,
            saved_text: String::new(),
        }
    }

    /// 获取当前文本。
    pub fn text(&self) -> &str {
        &self.text
    }

    /// 获取光标位置。
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// 文本是否为空。
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// 在光标位置插入字符。
    pub fn insert(&mut self, c: char) {
        self.text.insert(self.cursor, c);
        self.cursor += c.len_utf8();
    }

    /// 删除光标前的字符（Backspace）。
    pub fn backspace(&mut self) {
        if self.cursor > 0 {
            let prev = self.text[..self.cursor]
                .char_indices()
                .next_back()
                .map(|(i, _)| i)
                .unwrap_or(0);
            self.text.drain(prev..self.cursor);
            self.cursor = prev;
        }
    }

    /// 删除光标后的字符（Delete）。
    pub fn delete(&mut self) {
        if self.cursor < self.text.len() {
            let next = self.text[self.cursor..]
                .char_indices()
                .nth(1)
                .map(|(i, _)| self.cursor + i)
                .unwrap_or(self.text.len());
            self.text.drain(self.cursor..next);
        }
    }

    /// 光标左移。
    pub fn move_left(&mut self) {
        if self.cursor > 0 {
            self.cursor = self.text[..self.cursor]
                .char_indices()
                .next_back()
                .map(|(i, _)| i)
                .unwrap_or(0);
        }
    }

    /// 光标右移。
    pub fn move_right(&mut self) {
        if self.cursor < self.text.len() {
            self.cursor = self.text[self.cursor..]
                .char_indices()
                .nth(1)
                .map(|(i, _)| self.cursor + i)
                .unwrap_or(self.text.len());
        }
    }

    /// 光标移到开头。
    pub fn move_home(&mut self) {
        self.cursor = 0;
    }

    /// 光标移到末尾。
    pub fn move_end(&mut self) {
        self.cursor = self.text.len();
    }

    /// 清空输入。
    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor = 0;
        self.history_index = None;
    }

    /// 替换从 start 字节到光标位置的文本。
    /// 用于 Tab 补全。
    pub fn replace_range(&mut self, start: usize, replacement: &str) {
        if start > self.text.len() || start > self.cursor {
            return;
        }
        self.text.drain(start..self.cursor);
        self.text.insert_str(start, replacement);
        self.cursor = start + replacement.len();
    }

    /// 取出文本并清空（同时记录到历史）。
    pub fn take(&mut self) -> String {
        let text = self.text.clone();

        // 记录到历史（非空且与上一条不同）
        if !text.is_empty() && self.history.last().map(|s| s.as_str()) != Some(&text) {
            self.history.push(text.clone());
            if self.history.len() > MAX_HISTORY {
                self.history.remove(0);
            }
        }

        self.text.clear();
        self.cursor = 0;
        self.history_index = None;
        text
    }

    /// 浏览历史——上一条（上箭头）。
    pub fn history_up(&mut self) {
        if self.history.is_empty() {
            return;
        }

        match self.history_index {
            None => {
                // 第一次按上：保存当前文本，跳到最新历史
                self.saved_text = self.text.clone();
                let idx = self.history.len() - 1;
                self.history_index = Some(idx);
                self.text = self.history[idx].clone();
                self.cursor = self.text.len();
            }
            Some(idx) => {
                if idx > 0 {
                    let new_idx = idx - 1;
                    self.history_index = Some(new_idx);
                    self.text = self.history[new_idx].clone();
                    self.cursor = self.text.len();
                }
            }
        }
    }

    /// 浏览历史——下一条（下箭头）。
    pub fn history_down(&mut self) {
        match self.history_index {
            None => (),
            Some(idx) => {
                if idx + 1 >= self.history.len() {
                    // 回到当前输入
                    self.history_index = None;
                    self.text = self.saved_text.clone();
                    self.cursor = self.text.len();
                } else {
                    let new_idx = idx + 1;
                    self.history_index = Some(new_idx);
                    self.text = self.history[new_idx].clone();
                    self.cursor = self.text.len();
                }
            }
        }
    }
}

impl Default for InputEditor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_backspace() {
        let mut ed = InputEditor::new();
        ed.insert('h');
        ed.insert('i');
        assert_eq!(ed.text(), "hi");
        ed.backspace();
        assert_eq!(ed.text(), "h");
    }

    #[test]
    fn cursor_movement() {
        let mut ed = InputEditor::new();
        ed.insert('a');
        ed.insert('b');
        ed.insert('c');
        assert_eq!(ed.cursor(), 3);
        ed.move_left();
        assert_eq!(ed.cursor(), 2);
        ed.backspace();
        assert_eq!(ed.text(), "ac");
        ed.move_end();
        assert_eq!(ed.cursor(), 2);
        ed.move_home();
        assert_eq!(ed.cursor(), 0);
    }

    #[test]
    fn take_clears() {
        let mut ed = InputEditor::new();
        ed.insert('x');
        let text = ed.take();
        assert_eq!(text, "x");
        assert!(ed.is_empty());
    }

    #[test]
    fn multiline() {
        let mut ed = InputEditor::new();
        ed.insert('a');
        ed.insert('\n');
        ed.insert('b');
        assert_eq!(ed.text(), "a\nb");
    }

    #[test]
    fn history_navigation() {
        let mut ed = InputEditor::new();

        // 发送 3 条消息
        ed.insert('a');
        ed.take();
        ed.insert('b');
        ed.take();
        ed.insert('c');
        ed.take();

        assert!(ed.is_empty());

        // 上箭头：c → b → a
        ed.history_up();
        assert_eq!(ed.text(), "c");
        ed.history_up();
        assert_eq!(ed.text(), "b");
        ed.history_up();
        assert_eq!(ed.text(), "a");

        // 再上：到底了，不变
        ed.history_up();
        assert_eq!(ed.text(), "a");

        // 下箭头：a → b → c → 恢复
        ed.history_down();
        assert_eq!(ed.text(), "b");
        ed.history_down();
        assert_eq!(ed.text(), "c");
        ed.history_down();
        assert!(ed.is_empty()); // 恢复空输入
    }

    #[test]
    fn history_saves_current_on_up() {
        let mut ed = InputEditor::new();
        ed.take(); // 空，不记录
        ed.insert('x');
        ed.take();

        // 当前输入 "hello"，按上
        for c in "hello".chars() {
            ed.insert(c);
        }
        ed.history_up();
        assert_eq!(ed.text(), "x");

        // 按下恢复 "hello"
        ed.history_down();
        assert_eq!(ed.text(), "hello");
    }
}
