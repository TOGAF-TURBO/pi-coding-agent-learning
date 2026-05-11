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

    // 后台版本检查（非阻塞）
    if !cli.offline {
        let check_rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        if let Some(latest) = check_rt.block_on(pi_cli::version::check_for_update()) {
            eprintln!("{}", pi_cli::version::format_update_hint(&latest));
        }
        drop(check_rt);
    }

    pi_cli::dispatch::run(cli)
}
