//! pi — AI coding agent for your terminal
//!
//! Rust reimplementation of pi-mono (TypeScript).

use anyhow::Result;
use clap::{CommandFactory, Parser};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    // 拦截子命令（在 clap 解析前处理）
    if let Some(subcmd) = args.get(1).map(|s| s.as_str()) {
        match subcmd {
            "completions" => {
                let shell_name = args.get(2).map(|s| s.as_str()).unwrap_or("bash");
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
            "install" => {
                let source = args.get(2).map(|s| s.as_str()).unwrap_or("");
                let local = args.contains(&"--local".to_string())
                    || args.contains(&"-l".to_string());
                if source.is_empty() {
                    eprintln!("Usage: piso install <source> [-l]");
                    std::process::exit(1);
                }
                return pi_cli::subcommands::install(source, local);
            }
            "remove" | "uninstall" => {
                let source = args.get(2).map(|s| s.as_str()).unwrap_or("");
                if source.is_empty() {
                    eprintln!("Usage: piso remove <source>");
                    std::process::exit(1);
                }
                return pi_cli::subcommands::remove(source);
            }
            "update" => {
                let target = args.get(2).map(|s| s.as_str());
                return pi_cli::subcommands::update(target);
            }
            "list" => {
                return pi_cli::subcommands::list();
            }
            _ => {}
        }
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
