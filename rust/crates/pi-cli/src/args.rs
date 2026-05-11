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

    /// Model to use.
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
