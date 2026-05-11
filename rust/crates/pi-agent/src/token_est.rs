//! Token 估算 — 基于 BPE 规则的消息 token 数估算。
//!
//! 不需要真实 tokenizer，使用启发式方法：
//! - 英文: ~4 chars/token
//! - 中文/CJK: ~1.5 chars/token
//! - 代码: ~3.5 chars/token
//!
//! 用于 context window 预算，不是精确值。

/// Token 估算结果。
#[derive(Debug, Clone)]
pub struct TokenEstimate {
    /// 估算的 token 数。
    pub tokens: u32,
    /// 输入字符数。
    pub chars: usize,
}

/// 估算一段文本的 token 数。
pub fn estimate(text: &str) -> TokenEstimate {
    let chars = text.chars().count();
    if chars == 0 {
        return TokenEstimate {
            tokens: 0,
            chars: 0,
        };
    }

    // 计算各类字符比例
    let mut cjk = 0usize;
    for ch in text.chars() {
        if is_cjk(ch) {
            cjk += 1;
        }
    }

    let non_cjk = chars - cjk;
    // CJK: ~1.5 chars/token, 其他: ~4 chars/token
    let cjk_tokens = (cjk as f32 / 1.5).ceil() as u32;
    let other_tokens = (non_cjk as f32 / 4.0).ceil() as u32;

    TokenEstimate {
        tokens: cjk_tokens + other_tokens,
        chars,
    }
}

/// 估算消息历史的总 token 数。
pub fn estimate_messages(messages: &[serde_json::Value]) -> u32 {
    let mut total = 0u32;
    for msg in messages {
        if let Some(content) = msg.get("content") {
            if let Some(arr) = content.as_array() {
                for block in arr {
                    if block.get("type").and_then(|v| v.as_str()) == Some("text") {
                        if let Some(text) = block.get("text").and_then(|v| v.as_str()) {
                            total += estimate(text).tokens;
                        }
                    }
                }
            }
        }
        // 每条消息的开销（role, metadata）
        total += 4;
    }
    total
}

/// 判断字符是否为 CJK 统一汉字。
fn is_cjk(ch: char) -> bool {
    matches!(ch,
        '\u{4E00}'..='\u{9FFF}'   // CJK Unified
        | '\u{3400}'..='\u{4DBF}' // CJK Extension A
        | '\u{3000}'..='\u{303F}' // CJK Symbols
        | '\u{FF00}'..='\u{FFEF}' // Half/Fullwidth
        | '\u{2E80}'..='\u{2EFF}' // CJK Radicals
        | '\u{AC00}'..='\u{D7AF}' // Hangul
        | '\u{3040}'..='\u{309F}' // Hiragana
        | '\u{30A0}'..='\u{30FF}' // Katakana
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_text() {
        let est = estimate("");
        assert_eq!(est.tokens, 0);
    }

    #[test]
    fn english_text() {
        let est = estimate("Hello world, this is a test.");
        // ~32 chars / 4 = 8 tokens
        assert!(est.tokens > 0 && est.tokens < 15);
    }

    #[test]
    fn cjk_text() {
        let est = estimate("你好世界测试");
        // 6 CJK chars / 1.5 = 4 tokens
        assert_eq!(est.chars, 6);
        assert!(est.tokens >= 4);
    }

    #[test]
    fn mixed_text() {
        let est = estimate("Hello 你好 world 世界");
        // 11 english chars (~3 tokens) + 4 CJK chars (~3 tokens) = ~6
        assert!(est.tokens >= 5 && est.tokens <= 10);
    }

    #[test]
    fn long_text() {
        let text = "a ".repeat(1000);
        let est = estimate(&text);
        assert!(est.tokens > 400 && est.tokens < 600);
    }

    #[test]
    fn estimate_messages_empty() {
        let msgs: Vec<serde_json::Value> = vec![];
        assert_eq!(estimate_messages(&msgs), 0);
    }

    #[test]
    fn estimate_messages_with_text() {
        let msgs = vec![serde_json::json!({
            "role": "user",
            "content": [{"type": "text", "text": "Hello"}]
        })];
        let tokens = estimate_messages(&msgs);
        assert!(tokens > 0);
    }
}
