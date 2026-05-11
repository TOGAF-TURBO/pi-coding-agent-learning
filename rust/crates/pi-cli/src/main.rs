//! pi — AI coding agent for your terminal
//!
//! Rust reimplementation of pi-mono (TypeScript).

use anyhow::Result;
use clap::{CommandFactory, Parser};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    // 拦截 "completions" 子命令
    if let Some(pos) = args.iter().position(|a| a == "completions") {
        let shell_name = args.get(pos + 1).map(|s| s.as_str()).unwrap_or("bash");
        let shell = match shell_name {
            "zsh" => clap_complete::Shell::Zsh,
            "fish" => clap_complete::Shell::Fish,
            "elvish" => clap_complete::Shell::Elvish,
            "powershell" => clap_complete::Shell::PowerShell,
            _ => clap_complete::Shell::Bash,
        };
        let mut app = pi_cli::args::Cli::command();
        clap_complete::generate(shell, &mut app, "piso", &mut std::io::stdout());
        return Ok(());
    }

    let cli = pi_cli::args::Cli::parse();
    pi_cli::dispatch::run(cli)
}
