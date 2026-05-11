//! Slash 命令 — 在输入框中以 / 开头的快捷命令。
//!
//! 命令列表：
//! - /help         显示帮助
//! - /clear        清空聊天
//! - /compact      手动触发上下文压缩
//! - /model [name] 切换模型
//! - /branch       显示分支信息
//! - /export       导出会话为 HTML
//! - /usage        显示 token 用量
//! - /sessions     打开会话选择器
//! - /quit         退出

/// Slash 命令解析结果。
#[derive(Debug, Clone)]
pub enum SlashCommand {
    /// 显示帮助。
    Help,
    /// 清空聊天记录（不清除会话）。
    Clear,
    /// 手动触发上下文压缩。
    Compact,
    /// 切换模型（可选模型名）。
    Model(Option<String>),
    /// 显示分支信息。
    Branch,
    /// 导出会话。
    Export(String),
    /// 显示 token 用量。
    Usage,
    /// 显示费用估算。
    Cost,
    /// 打开会话选择器。
    Sessions,
    /// 退出。
    Quit,
    /// 未知命令。
    Unknown(String),
}

/// 解析输入文本为 slash 命令。
/// 如果文本以 / 开头，解析为 SlashCommand。
/// 否则返回 None。
pub fn parse(input: &str) -> Option<SlashCommand> {
    let trimmed = input.trim();
    if !trimmed.starts_with('/') {
        return None;
    }

    let parts: Vec<&str> = trimmed[1..].splitn(2, ' ').collect();
    let cmd = parts.first().unwrap_or(&"");
    let arg = parts.get(1).map(|s| s.trim().to_string());

    Some(match *cmd {
        "help" | "h" | "?" => SlashCommand::Help,
        "clear" | "cls" => SlashCommand::Clear,
        "compact" | "compress" => SlashCommand::Compact,
        "model" | "m" => SlashCommand::Model(arg),
        "branch" | "b" => SlashCommand::Branch,
        "export" | "e" => SlashCommand::Export(arg.unwrap_or_else(|| "session.html".to_string())),
        "usage" | "tokens" | "u" => SlashCommand::Usage,
        "cost" | "c" => SlashCommand::Cost,
        "sessions" | "s" => SlashCommand::Sessions,
        "quit" | "q" | "exit" => SlashCommand::Quit,
        _ => SlashCommand::Unknown(cmd.to_string()),
    })
}

/// 生成帮助文本。
pub fn help_text() -> String {
    [
        "Slash commands:",
        "  /help, /h, /?       Show this help",
        "  /clear, /cls        Clear chat display",
        "  /compact, /compress Compact context (summarize old messages)",
        "  /model, /m [name]   Switch model (no arg = open picker)",
        "  /branch, /b         Show branch info",
        "  /export, /e [path]  Export session to HTML",
        "  /usage, /u          Show token usage stats",
        "  /cost, /c           Show estimated API cost",
        "  /sessions, /s       Open session picker",
        "  /quit, /q           Quit piso",
    ].join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_help() {
        assert!(matches!(parse("/help"), Some(SlashCommand::Help)));
        assert!(matches!(parse("/h"), Some(SlashCommand::Help)));
        assert!(matches!(parse("/?"), Some(SlashCommand::Help)));
    }

    #[test]
    fn parse_model_with_arg() {
        assert!(matches!(parse("/model gpt-4"), Some(SlashCommand::Model(Some(_)))));
    }

    #[test]
    fn parse_model_without_arg() {
        assert!(matches!(parse("/model"), Some(SlashCommand::Model(None))));
    }

    #[test]
    fn parse_clear() {
        assert!(matches!(parse("/clear"), Some(SlashCommand::Clear)));
        assert!(matches!(parse("/cls"), Some(SlashCommand::Clear)));
    }

    #[test]
    fn parse_export() {
        match parse("/export out.html") {
            Some(SlashCommand::Export(path)) => assert_eq!(path, "out.html"),
            _ => panic!("Expected Export"),
        }
    }

    #[test]
    fn parse_unknown() {
        match parse("/xyz") {
            Some(SlashCommand::Unknown(cmd)) => assert_eq!(cmd, "xyz"),
            _ => panic!("Expected Unknown"),
        }
    }

    #[test]
    fn not_a_command() {
        assert!(parse("hello world").is_none());
        assert!(parse("  /help").is_some()); // trimmed
    }

    #[test]
    fn help_text_not_empty() {
        let text = help_text();
        assert!(text.contains("/help"));
        assert!(text.contains("/quit"));
    }
}
