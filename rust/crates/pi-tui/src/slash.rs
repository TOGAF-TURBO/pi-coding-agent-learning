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
    /// 搜索会话内容。
    Find(String),
    /// 在当前会话中搜索消息。
    Grep(String),
    /// 创建新会话。
    NewSession,
    /// 重载配置。
    Reload,
    /// 复制最后一条助手消息。
    Copy,
    /// 从指定消息分叉。
    Fork(String),
    /// 显示会话信息。
    SessionInfo,
    /// 设置会话名称。
    Name(String),
    /// 导入 JSONL 会话文件。
    Import(String),
    /// 克隆当前会话。
    Clone,
    /// 展开 skill: /skill:<name> 读取 SKILL.md 并注入。
    Skill(String),
    /// 打开会话选择器。
    Sessions,
    /// 打开 diff 查看器。
    Diff,
    /// GitHub Copilot OAuth 登录。
    Login,
    /// 清除缓存的 OAuth token。
    Logout,
    /// 切换主题。
    Theme(Option<String>),
    /// 退出。
    Quit,
    /// 未知命令。
    Unknown(String),
}

/// 从文件系统解析 skill 的 SKILL.md 内容。
/// 搜索路径：
/// 1. <project>/.piso/skills/<name>/SKILL.md
/// 2. ~/.piso/skills/<name>/SKILL.md
///
/// 返回 SKILL.md 的文本内容，或错误信息。
pub fn resolve_skill(name: &str, cwd: &std::path::Path) -> Result<String, String> {
    let skill_file = std::path::PathBuf::from(format!("{}/SKILL.md", name));

    // 1. Project-local skills
    let project_path = cwd.join(".piso").join("skills").join(&skill_file);
    if project_path.exists() {
        return std::fs::read_to_string(&project_path)
            .map_err(|e| format!("Failed to read {}: {e}", project_path.display()));
    }

    // 2. User-global skills
    if let Some(home) = dirs::home_dir() {
        let global_path = home.join(".piso").join("skills").join(&skill_file);
        if global_path.exists() {
            return std::fs::read_to_string(&global_path)
                .map_err(|e| format!("Failed to read {}: {e}", global_path.display()));
        }
    }

    Err(format!(
        "Skill '{}' not found. Searched .piso/skills/ and ~/.piso/skills/",
        name
    ))
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
        "find" | "f" | "search" => SlashCommand::Find(arg.unwrap_or_default()),
        "grep" | "g" => SlashCommand::Grep(arg.unwrap_or_default()),
        "new" => SlashCommand::NewSession,
        "reload" => SlashCommand::Reload,
        "copy" => SlashCommand::Copy,
        "fork" => SlashCommand::Fork(arg.unwrap_or_default()),
        "session" | "info" => SlashCommand::SessionInfo,
        "name" => SlashCommand::Name(arg.unwrap_or_default()),
        "import" | "i" => SlashCommand::Import(arg.unwrap_or_default()),
        "clone" => SlashCommand::Clone,
        "sessions" | "s" => SlashCommand::Sessions,
        "diff" | "d" => SlashCommand::Diff,
        "login" => SlashCommand::Login,
        "logout" => SlashCommand::Logout,
        "theme" | "t" => SlashCommand::Theme(arg),
        "quit" | "q" | "exit" => SlashCommand::Quit,
        other if other.starts_with("skill:") => {
            let skill_name = other.strip_prefix("skill:").unwrap_or("").to_string();
            if skill_name.is_empty() {
                SlashCommand::Unknown(other.to_string())
            } else {
                SlashCommand::Skill(skill_name)
            }
        }
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
        "  /export, /e [path]  Export session (HTML or JSONL based on extension)",
        "  /usage, /u          Show token usage stats",
        "  /cost, /c           Show estimated API cost",
        "  /find, /f <term>    Search across sessions",
        "  /grep, /g <term>   Search current session messages",
        "  /new                Start a new session",
        "  /reload             Reload keybindings/themes/skills",
        "  /copy               Copy last assistant message",
        "  /fork [at]          Fork session at message",
        "  /session, /info     Show session info",
        "  /name <name>        Set session display name",
        "  /import, /i <path>  Import JSONL session file",
        "  /clone              Duplicate current session",
        "  /sessions, /s       Open session picker",
        "  /skill:<name>       Load skill (e.g. /skill:debug)",
        "  /diff, /d           Open diff viewer",
        "  /login              GitHub Copilot OAuth login",
        "  /logout             Clear cached OAuth token",
        "  /theme, /t [name]   Switch theme (no arg = list available)",
        "  /quit, /q           Quit piso",
    ]
    .join("\n")
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
        assert!(matches!(
            parse("/model gpt-4"),
            Some(SlashCommand::Model(Some(_)))
        ));
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
        assert!(text.contains("/import"));
        assert!(text.contains("/clone"));
    }

    #[test]
    fn parse_import() {
        match parse("/import /tmp/session.jsonl") {
            Some(SlashCommand::Import(path)) => assert_eq!(path, "/tmp/session.jsonl"),
            _ => panic!("Expected Import"),
        }
    }

    #[test]
    fn parse_import_short() {
        match parse("/i /path/to/file") {
            Some(SlashCommand::Import(path)) => assert_eq!(path, "/path/to/file"),
            _ => panic!("Expected Import"),
        }
    }

    #[test]
    fn parse_clone() {
        match parse("/clone") {
            Some(SlashCommand::Clone) => {}
            _ => panic!("Expected Clone"),
        }
    }

    #[test]
    fn parse_skill() {
        match parse("/skill:debug") {
            Some(SlashCommand::Skill(name)) => assert_eq!(name, "debug"),
            _ => panic!("Expected Skill"),
        }
    }

    #[test]
    fn parse_skill_empty_is_unknown() {
        match parse("/skill:") {
            Some(SlashCommand::Unknown(cmd)) => assert_eq!(cmd, "skill:"),
            _ => panic!("Expected Unknown"),
        }
    }

    #[test]
    fn resolve_skill_reads_project_local() {
        let dir = tempfile::tempdir().unwrap();
        let skills_dir = dir.path().join(".piso").join("skills").join("debug");
        std::fs::create_dir_all(&skills_dir).unwrap();
        std::fs::write(
            skills_dir.join("SKILL.md"),
            "# Debug Skill\nUse for debugging.",
        )
        .unwrap();

        let result = resolve_skill("debug", dir.path());
        assert!(result.is_ok());
        assert!(result.unwrap().contains("Debug Skill"));
    }

    #[test]
    fn resolve_skill_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let result = resolve_skill("nonexistent", dir.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }
}
