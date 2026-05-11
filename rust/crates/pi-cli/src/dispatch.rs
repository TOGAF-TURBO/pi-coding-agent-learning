//! 模式分发 — 根据 CLI 参数启动对应运行模式。

use std::env;

use anyhow::{Context, Result, anyhow, bail};

use crate::args::Cli;
use crate::auth::AuthStorage;
use crate::config;

use pi_agent::loop_engine::AgentLoop;
use pi_llm::driver::LlmDriver;
use pi_llm::openai::OpenAiDriver;
use pi_llm::providers::AnthropicDriver;
use pi_session::JsonlSession;
use pi_tools::bash::BashTool;
use pi_tools::read::ReadTool;
use pi_tools::write::WriteTool;
use pi_tools::edit::EditTool;
use pi_tools::find::FindTool;
use pi_tools::grep::GrepTool;
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

    // 解析认证 + provider 配置
    let config_dir = config::config_dir();
    let auth = AuthStorage::load(config_dir.as_deref())?;

    // 确定 provider
    let provider = cli.provider.as_deref()
        .or(cfg.provider.as_deref())
        .unwrap_or("anthropic");

    let api_key = cli.api_key.as_deref()
        .or_else(|| auth.get_key(provider))
        .ok_or_else(|| anyhow!(
            "No API key found for provider '{}'. Set {}_API_KEY or configure ~/.pi/agent/models.json",
            provider,
            provider.to_uppercase().replace('-', "_"),
        ))?;

    // 确定 model
    let model = cli.model.as_deref()
        .or(cfg.model.as_deref())
        .unwrap_or("claude-sonnet-4-20250514");

    // 从 models.json 获取 provider 配置（baseUrl、api 类型）
    let provider_config = auth.get_provider(provider);
    let api_type = provider_config.map(|c| c.api.as_str()).unwrap_or("anthropic-messages");
    let base_url = provider_config.and_then(|c| c.base_url.clone());

    // 根据 API 类型创建对应的 LLM driver
    let driver: Box<dyn LlmDriver> = match api_type {
        "openai-completions" | "openai-responses" => Box::new(OpenAiDriver::new()),
        _ => Box::new(AnthropicDriver::new()),
    };

    // OpenAI-completions 的 base URL 需要指向 /chat/completions
    let effective_base_url = if api_type == "openai-completions" || api_type == "openai-responses" {
        base_url.map(|url| {
            if url.ends_with("/chat/completions") {
                url
            } else if url.ends_with('/') {
                format!("{}chat/completions", url)
            } else {
                format!("{}/chat/completions", url)
            }
        })
    } else {
        // Anthropic 风格的 base URL
        base_url
    };

    // 创建工具注册表
    let tools = ToolRegistry::new();
    if !cli.no_tools {
        tools.register(BashTool::new(&cwd_str));
        tools.register(ReadTool::new());
        tools.register(WriteTool::new());
        tools.register(EditTool::new());
        tools.register(FindTool::new(&cwd_str));
        tools.register(GrepTool::new(&cwd_str));
    }

    // 创建或恢复会话
    let session = if cli.no_session {
        let tmp = tempfile::tempdir()?;
        let path = tmp.path().join("session.jsonl");
        // 使用 into_path 防止 TempDir drop 时删除目录
        let _tmp_persist = tmp.into_path();
        JsonlSession::create(&path, &cwd_str).await?
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
        read_piped_stdin().unwrap_or_default()
    } else {
        user_message
    };

    if user_message.is_empty() {
        bail!("No prompt provided. Usage: piso -p \"your prompt\"");
    }

    // 创建 Agent 循环
    let mut agent = AgentLoop::new(session, driver, tools, model)
        .with_api_key(api_key);

    if let Some(url) = effective_base_url {
        agent = agent.with_base_url(url);
    }

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

    println!("Configured providers:");
    for name in auth.configured_providers() {
        let key_status = if auth.get_key(&name).is_some() { "API key found" } else { "no API key" };
        let config = auth.get_provider(&name);
        let api = config.map(|c| c.api.as_str()).unwrap_or("unknown");
        println!("  {name} ({api}, {key_status})");
    }

    if auth.configured_providers().is_empty() {
        let available = auth.available_providers();
        if available.is_empty() {
            println!("  No providers configured. Set ANTHROPIC_API_KEY or configure ~/.pi/agent/models.json");
        } else {
            println!("Providers with API keys from environment:");
            for name in available {
                println!("  {name}");
            }
        }
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
