//! pi — AI coding agent for your terminal
//!
//! Rust reimplementation of pi-mono (TypeScript).

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let cli = pi_cli::args::Cli::parse();
    pi_cli::dispatch::run(cli)
}
