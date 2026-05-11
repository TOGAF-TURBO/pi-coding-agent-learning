//! CLI 参数定义。
//!
//! 环境变量：
//! - `PISO_PROVIDER` / `PI_PROVIDER` — 默认 provider
//! - `PISO_MODEL` / `PI_MODEL` — 默认 model
//! - `PISO_BASE_URL` — 默认 base URL
//! - `<PROVIDER>_API_KEY` — API key（如 `GLM_API_KEY`）

use clap::Parser;

/// piso — AI coding agent for your terminal.
#[derive(Debug, Parser)]
#[command(
    name = "piso",
    version,
    about = "AI coding agent for your terminal",
    long_about = None,
    override_usage = "piso [OPTIONS] [PROMPT]\n       piso [OPTIONS] <COMMAND>"
)]
pub struct Cli {
    /// LLM provider (anthropic, openai, google, etc.)
    #[arg(long)]
    pub provider: Option<String>,

    /// Model to use. Supports compact syntax: provider/id:thinking
    #[arg(long, short = 'm')]
    pub model: Option<String>,

    /// API key for the provider.
    #[arg(long)]
    pub api_key: Option<String>,

    /// Base URL for the provider API.
    #[arg(long)]
    pub base_url: Option<String>,

    /// Thinking level (off, minimal, low, medium, high, xhigh).
    #[arg(long)]
    pub thinking: Option<String>,

    /// System prompt override.
    #[arg(long)]
    pub system_prompt: Option<String>,

    /// Append to system prompt (repeatable).
    #[arg(long)]
    pub append_system_prompt: Option<Vec<String>>,

    /// Continue the most recent session.
    #[arg(long, short = 'c')]
    pub r#continue: bool,

    /// Resume a previous session (interactive picker).
    #[arg(long, short = 'r')]
    pub resume: bool,

    /// Fork a session at a specific point.
    #[arg(long)]
    pub fork: Option<String>,

    /// Session ID to use.
    #[arg(long)]
    pub session: Option<String>,

    /// Run mode: interactive, text, json, rpc.
    #[arg(long)]
    pub mode: Option<String>,

    /// Disable all tools.
    #[arg(long, short = 'n')]
    pub no_tools: bool,

    /// Disable built-in tools only.
    #[arg(long)]
    pub no_builtin_tools: bool,

    /// Disable extensions.
    #[arg(long)]
    pub no_extensions: bool,

    /// Disable skills.
    #[arg(long)]
    pub no_skills: bool,

    /// Disable prompt templates.
    #[arg(long)]
    pub no_prompt_templates: bool,

    /// Disable context files (AGENTS.md etc).
    #[arg(long)]
    pub no_context_files: bool,

    /// Load a skill by name (can be repeated).
    #[arg(long)]
    pub skill: Option<Vec<String>>,

    /// Load a theme by name (can be repeated).
    #[arg(long)]
    pub theme: Option<Vec<String>>,

    /// Use a prompt template by name (can be repeated).
    #[arg(long)]
    pub prompt_template: Option<Vec<String>>,

    /// List available models.
    #[arg(long)]
    pub list_models: Option<Option<String>>,

    /// List previous sessions.
    #[arg(long)]
    pub list_sessions: bool,

    /// Run offline (skip version check).
    #[arg(long)]
    pub offline: bool,

    /// Verbose logging.
    #[arg(long, short = 'v')]
    pub verbose: bool,

    /// Generate template config files in ~/.piso/.
    #[arg(long)]
    pub init: bool,

    /// Print mode: non-interactive, output to stdout.
    #[arg(long, short = 'p')]
    pub print: bool,

    /// Export session to HTML.
    #[arg(long)]
    pub export: Option<String>,

    /// Load extensions from paths.
    #[arg(long, short = 'e')]
    pub extension: Option<Vec<String>>,

    /// Initial prompt message (positional args).
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub messages: Vec<String>,

    /// Comma-separated model patterns for Ctrl+P cycling.
    #[arg(long)]
    pub models: Option<String>,

    /// Comma-separated allowlist of tool names to enable.
    #[arg(long, short = 't')]
    pub tools: Option<String>,

    /// Directory for session storage.
    #[arg(long)]
    pub session_dir: Option<String>,

    /// Don't persist session (ephemeral).
    #[arg(long)]
    pub no_session: bool,
}

/// 子命令。
#[derive(Debug, Parser)]
pub enum Commands {
    /// Generate shell completion scripts.
    #[command(name = "completions")]
    Completions {
        /// Shell type (bash, zsh, fish, elvish).
        shell: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn parse_basic_prompt() {
        let cli = Cli::try_parse_from(["piso", "hello", "world"]);
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        assert_eq!(cli.messages, vec!["hello", "world"]);
    }

    #[test]
    fn parse_provider_and_model() {
        let cli = Cli::try_parse_from([
            "piso",
            "--provider",
            "glm",
            "--model",
            "glm-5.1",
            "-p",
            "test",
        ]);
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        assert_eq!(cli.provider.as_deref(), Some("glm"));
        assert_eq!(cli.model.as_deref(), Some("glm-5.1"));
        assert!(cli.print);
    }

    #[test]
    fn parse_flags() {
        let cli = Cli::try_parse_from([
            "piso",
            "--no-tools",
            "--no-session",
            "--offline",
            "--verbose",
        ]);
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        assert!(cli.no_tools);
        assert!(cli.no_session);
        assert!(cli.offline);
        assert!(cli.verbose);
    }

    #[test]
    fn parse_tools_whitelist() {
        let cli = Cli::try_parse_from(["piso", "--tools", "read,grep,find"]);
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        assert_eq!(cli.tools.as_deref(), Some("read,grep,find"));
    }

    #[test]
    fn parse_models_filter() {
        let cli = Cli::try_parse_from(["piso", "--models", "claude-*,gpt-*"]);
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        assert_eq!(cli.models.as_deref(), Some("claude-*,gpt-*"));
    }

    #[test]
    fn parse_session_dir() {
        let cli = Cli::try_parse_from(["piso", "--session-dir", "/tmp/my-sessions"]);
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        assert_eq!(cli.session_dir.as_deref(), Some("/tmp/my-sessions"));
    }

    #[test]
    fn parse_continue_and_resume() {
        let cli = Cli::try_parse_from(["piso", "--continue"]);
        assert!(cli.is_ok());
        assert!(cli.unwrap().r#continue);

        let cli = Cli::try_parse_from(["piso", "--resume"]);
        assert!(cli.is_ok());
        assert!(cli.unwrap().resume);
    }

    #[test]
    fn parse_completions_subcommand() {
        let cli = Cli::try_parse_from(["piso", "completions", "bash"]);
        assert!(cli.is_ok());
        // subcommands are not directly in Cli, they need a top-level parser
    }

    #[test]
    fn parse_skill_flag() {
        let cli = Cli::try_parse_from(["piso", "--skill", "code-review"]);
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        assert_eq!(cli.skill.unwrap(), vec!["code-review"]);
    }

    #[test]
    fn parse_skill_multiple() {
        let cli = Cli::try_parse_from(["piso", "--skill", "a", "--skill", "b"]);
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        assert_eq!(cli.skill.unwrap(), vec!["a", "b"]);
    }

    #[test]
    fn parse_theme_flag() {
        let cli = Cli::try_parse_from(["piso", "--theme", "dracula"]);
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        assert_eq!(cli.theme.unwrap(), vec!["dracula"]);
    }

    #[test]
    fn parse_theme_multiple() {
        let cli = Cli::try_parse_from(["piso", "--theme", "dark", "--theme", "custom"]);
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        assert_eq!(cli.theme.unwrap(), vec!["dark", "custom"]);
    }

    #[test]
    fn parse_prompt_template_flag() {
        let cli = Cli::try_parse_from(["piso", "--prompt-template", "review"]);
        assert!(cli.is_ok());
        let cli = cli.unwrap();
        assert_eq!(cli.prompt_template.unwrap(), vec!["review"]);
    }
}
