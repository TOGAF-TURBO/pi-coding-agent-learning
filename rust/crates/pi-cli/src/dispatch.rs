//! 模式分发 — 根据 CLI 参数启动对应运行模式。

use std::env;
use std::path::PathBuf;

use anyhow::{Context, Result, anyhow, bail};

use crate::args::Cli;
use crate::auth::AuthStorage;
use crate::config;

use pi_agent::loop_engine::{AgentLoop, AgentOutput};
use pi_llm::providers::AnthropicDriver;
use pi_llm::registry::ProviderRegistry;
use pi_session::JsonlSession;
use pi_tools::bash::BashTool;
use pi_tools::registry::ToolRegistry;

/// 启动流水线。
pub fn run(cli: Cli) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async_main(cli))
}

async fn async_main(cli: Cli) -> Result<()> {
    let mode = resolve_mode(&cli);

    match mode {
        AppMode::Interactive => run_interactive(cli).await,
        AppMode::Print => run_print(cli).await,
        AppMode::Rpc => run_rpc(cli).await,
        AppMode::ListModels => run_list_models(cli).await,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppMode {
    Interactive,
    Print,
    Rpc,
    ListModels,
}

fn resolve_mode(cli: &Cli) -> AppMode {
    if !cli.messages.is_empty() || cli.print {
        return AppMode::Print;
    }
    if cli.list_models.is_some() {
        return AppMode::ListModels;
    }
    match cli.mode.as_deref() {
        Some("rpc") => AppMode::Rpc,
        Some("json") | Some("text") => AppMode::Print,
        _ => AppMode::Interactive,
    }
}

async fn run_print(cli: Cli) -> Result<()> {
    let cwd = env::current_dir().context("Failed to get current directory")?;
    let cwd_str = cwd.to_string_lossy().to_string();

    // 加载配置
    let cfg = config::load_config(Some(&cwd));

    // 解析认证
    let config_dir = config::config_dir();
    let auth = AuthStorage::load(config_dir.as_deref())?;

    // 确定 provider
    let provider = cli.provider.as_deref()
        .or(cfg.provider.as_deref())
        .unwrap_or("anthropic");

    let api_key = cli.api_key.as_deref()
        .or_else(|| auth.get_key(provider))
        .ok_or_else(|| anyhow!(
            "No API key found for provider '{}'. Set {}_API_KEY or run 'piso auth set --provider {}'",
            provider,
            provider.to_uppercase().replace('-', "_"),
            provider
        ))?;

    // 确定 model
    let model = cli.model.as_deref()
        .or(cfg.model.as_deref())
        .unwrap_or("claude-sonnet-4-20250514");

    // 创建 LLM driver
    let driver = ProviderRegistry::new();
    let llm = driver.get(provider)
        .ok_or_else(|| anyhow!("Unknown provider: {}", provider))?;

    // 创建工具注册表
    let tools = ToolRegistry::new();
    if !cli.no_tools {
        tools.register(BashTool::new(&cwd_str));
    }

    // 创建或恢复会话
    let session = if cli.no_session {
        // 使用临时文件
        let tmp = tempfile::tempdir()?;
        JsonlSession::create(tmp.path().join("session.jsonl"), &cwd_str).await?
    } else {
        let session_dir = config::session_dir(&cfg)
            .unwrap_or_else(|| cwd.join(".pi").join("sessions"));
        let session_id = format!("{:08x}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos() as u32);
        let session_path = session_dir.join(&session_id).join("session.jsonl");
        JsonlSession::create(&session_path, &cwd_str).await?
    };

    // 构建用户消息
    let user_message = cli.messages.join(" ");
    let user_message = if user_message.is_empty() {
        // 尝试从 stdin 读取
        read_piped_stdin().unwrap_or_default()
    } else {
        user_message
    };

    if user_message.is_empty() {
        bail!("No prompt provided. Usage: piso -p \"your prompt\"");
    }

    // 创建 Agent 循环
    let mut agent = AgentLoop::new(session, Box::new(AnthropicDriver::new()), tools, model);

    // 注入 API key 到 driver（通过环境变量传递给 reqwest）
    // 注意：当前简化版直接在 driver 中使用 CompletionRequest.api_key
    // TODO: 重构 LlmDriver trait 使其持有 api_key

    // 运行 agent
    let output = agent.run(&user_message).await?;

    // 输出结果
    print!("{}", output.text);

    Ok(())
}

async fn run_interactive(_cli: Cli) -> Result<()> {
    eprintln!("piso — interactive mode not yet implemented");
    eprintln!("Use 'piso -p \"your prompt\"' for print mode");
    Ok(())
}

async fn run_rpc(_cli: Cli) -> Result<()> {
    eprintln!("piso — rpc mode not yet implemented");
    Ok(())
}

async fn run_list_models(_cli: Cli) -> Result<()> {
    let config_dir = config::config_dir();
    let auth = AuthStorage::load(config_dir.as_deref())?;

    println!("Available providers:");
    for provider in auth.available_providers() {
        println!("  {provider} (API key found)");
    }
    Ok(())
}

fn read_piped_stdin() -> Option<String> {
    if atty::is(atty::Stream::Stdin) {
        return None;
    }
    use std::io::Read;
    let mut buf = String::new();
    std::io::stdin().read_to_string(&mut buf).ok()?;
    let trimmed = buf.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}
