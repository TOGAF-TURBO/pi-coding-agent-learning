//! 输入处理 — 多行文本编辑。
//!
//! 支持基本编辑操作：输入字符、退格、删除、光标移动、回车换行。

/// 输入编辑器状态。
#[derive(Debug, Clone)]
pub struct InputEditor {
    /// 输入文本。
    text: String,
    /// 光标位置（字节偏移）。
    cursor: usize,
}

impl InputEditor {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            cursor: 0,
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
            // 找到前一个字符边界
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
    }

    /// 取出文本并清空。
    pub fn take(&mut self) -> String {
        let text = self.text.clone();
        self.clear();
        text
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
}
