//! 模式分发 — 根据 CLI 参数启动对应运行模式。

use std::env;

use anyhow::{Context, Result, anyhow, bail};

use crate::args::Cli;
use crate::auth::AuthStorage;
use crate::config;

use pi_agent::loop_engine::AgentLoop;
use pi_agent::context;
use pi_agent::skills;
use pi_agent::system_prompt::SystemPromptBuilder;
use pi_llm::driver::LlmDriver;
use pi_llm::openai::OpenAiDriver;
use pi_llm::gemini::GeminiDriver;
use pi_llm::providers::AnthropicDriver;
use pi_session::manager::SessionManager;
use pi_session::JsonlSession;
use pi_tools::bash::BashTool;
use pi_tools::read::ReadTool;
use pi_tools::write::WriteTool;
use pi_tools::edit::EditTool;
use pi_tools::find::FindTool;
use pi_tools::grep::GrepTool;
use pi_tools::registry::ToolRegistry;
use pi_tui;
use pi_tui::app::AgentState;

/// 启动流水线。
pub fn run(cli: Cli) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async_main(cli))
}

async fn async_main(cli: Cli) -> Result<()> {
    let mode = resolve_mode(&cli);

    match mode {
        AppMode::Interactive => run_interactive_mode(cli).await,
        AppMode::Print => run_print(cli).await,
        AppMode::Rpc => run_rpc(cli).await,
        AppMode::ListModels => run_list_models(cli).await,
        AppMode::ListSessions => run_list_sessions().await,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppMode {
    Interactive,
    Print,
    Rpc,
    ListModels,
    ListSessions,
}

fn resolve_mode(cli: &Cli) -> AppMode {
    if !cli.messages.is_empty() || cli.print {
        return AppMode::Print;
    }
    if cli.list_models.is_some() {
        return AppMode::ListModels;
    }
    if cli.list_sessions {
        return AppMode::ListSessions;
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
            "No API key found for provider '{}'. Set {}_API_KEY or configure ~/.piso/models.json",
            provider,
            provider.to_uppercase().replace('-', "_"),
        ))?;

    // 确定 model
    let model = cli.model.as_deref()
        .or(cfg.model.as_deref())
        .unwrap_or("claude-sonnet-4-20250514");

    // 从 models.json 获取 provider 配置
    let provider_config = auth.get_provider(provider);
    let api_type = provider_config.map(|c| c.api.as_str()).unwrap_or("anthropic-messages");
    let base_url = provider_config.and_then(|c| c.base_url.clone());

    // 根据 API 类型创建对应的 LLM driver
    let driver: Box<dyn LlmDriver> = match api_type {
        "openai-completions" | "openai-responses" => Box::new(OpenAiDriver::new()),
        "google-gemini" | "gemini" => Box::new(GeminiDriver::new()),
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
    let session = resolve_session(&cli, &cfg, &cwd, &cwd_str).await?;

    // 构建系统提示
    let mut prompt_builder = SystemPromptBuilder::new(&cwd_str);
    if !cli.no_tools {
        prompt_builder = prompt_builder.with_tool_guides();
    }
    if let Some(sp) = &cli.system_prompt {
        prompt_builder = prompt_builder.with_custom_prompt(sp);
    }
    if let Some(append) = &cli.append_system_prompt {
        for extra in append {
            prompt_builder = prompt_builder.append(extra);
        }
    }

    // 加载上下文文件（AGENTS.md 等）
    if !cli.no_context_files {
        let agent_dir = config_dir.as_deref();
        let ctx_files = context::load_project_context_files(
            &cwd,
            agent_dir.as_deref(),
        );
        if !ctx_files.is_empty() {
            for ctx in &ctx_files {
                eprintln!("[context] {}", ctx.path.display());
            }
            prompt_builder = prompt_builder.append(
                context::format_context_for_prompt(&ctx_files)
            );
        }
    }

    // 加载技能（~/.piso/skills/ + .piso/skills/）
    if !cli.no_skills {
        let global_skills = config::skills_dir();
        let loaded_skills = skills::load_skills(&cwd, global_skills.as_deref());
        if !loaded_skills.is_empty() {
            for s in &loaded_skills {
                eprintln!("[skill] {} ({})", s.name, s.source);
            }
            prompt_builder = prompt_builder.append(
                skills::format_skills_for_prompt(&loaded_skills)
            );
        }
    }

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
        .with_api_key(api_key)
        .with_system_prompt(prompt_builder.build());

    if let Some(url) = effective_base_url {
        agent = agent.with_base_url(url);
    }

    // 运行 agent
    let output = agent.run(&user_message).await?;

    // 输出结果
    print!("{}", output.text);

    Ok(())
}

/// 解析会话：--continue / --session / 新建。
async fn resolve_session(
    cli: &Cli,
    cfg: &crate::config::Config,
    cwd: &std::path::Path,
    cwd_str: &str,
) -> Result<JsonlSession> {
    let session_dir = config::session_dir(cfg)
        .unwrap_or_else(|| cwd.join(".pi").join("sessions"));

    if cli.no_session {
        let tmp = tempfile::tempdir()?;
        let path = tmp.path().join("session.jsonl");
        let _tmp_persist = tmp.keep();
        return JsonlSession::create(&path, cwd_str).await;
    }

    let mgr = SessionManager::new(&session_dir);
    mgr.ensure_dir().await?;

    // --session <id>: 打开指定会话
    if let Some(session_id) = &cli.session {
        return mgr.open(session_id).await;
    }

    // --continue: 恢复最近的会话
    if cli.r#continue {
        return mgr.continue_last().await;
    }

    // --resume: 列出会话供选择
    if cli.resume {
        let sessions = mgr.list().await?;
        if sessions.is_empty() {
            bail!("No previous sessions found");
        }
        eprintln!("Recent sessions:");
        for s in sessions.iter().take(5) {
            let indicator = if s.is_today { "*" } else { " " };
            eprintln!("  {} {} ({} messages, cwd: {})", indicator, s.id, s.message_count, s.cwd);
        }
        return mgr.open(&sessions[0].id).await;
    }

    // --fork <message-id>: 从指定消息分叉新会话
    if let Some(fork_at) = &cli.fork {
        let source = mgr.continue_last().await?;
        let entries = source.entries();

        // 查找分叉点
        let fork_idx = entries.iter().position(|e| e.id() == fork_at)
            .ok_or_else(|| anyhow!("Message '{}' not found in session", fork_at))?;

        // 创建新会话，复制分叉点之前的条目
        let mut new_session = mgr.create(cwd_str).await?;
        for entry in &entries[..=fork_idx] {
            new_session.append(entry.clone()).await?;
        }

        eprintln!("Forked session at message '{}' ({} entries copied)", fork_at, fork_idx + 1);
        return Ok(new_session);
    }

    // 默认：创建新会话
    mgr.create(cwd_str).await
}

async fn run_interactive_mode(cli: Cli) -> Result<()> {
    let cwd = env::current_dir().context("Failed to get current directory")?;
    let cwd_str = cwd.to_string_lossy().to_string();
    let cfg = config::load_config(Some(&cwd));
    let config_dir = config::config_dir();
    let auth = AuthStorage::load(config_dir.as_deref())?;

    let provider = cli.provider.clone()
        .or(cfg.provider.clone())
        .unwrap_or_else(|| "anthropic".to_string());

    let api_key = cli.api_key.clone()
        .or_else(|| auth.get_key(&provider).map(|s| s.to_string()))
        .ok_or_else(|| anyhow!(
            "No API key found for provider '{}'. Set {}_API_KEY or configure ~/.piso/models.json",
            provider,
            provider.to_uppercase().replace('-', "_"),
        ))?;

    let model = cli.model.clone()
        .or(cfg.model.clone())
        .unwrap_or_else(|| "claude-sonnet-4-20250514".to_string());

    let provider_config = auth.get_provider(&provider);
    let api_type = provider_config.map(|c| c.api.as_str()).unwrap_or("anthropic-messages");
    let base_url = provider_config.and_then(|c| c.base_url.clone());

    let effective_base_url = if api_type == "openai-completions" || api_type == "openai-responses" {
        base_url.map(|url| {
            if url.ends_with("/chat/completions") { url }
            else if url.ends_with('/') { format!("{}chat/completions", url) }
            else { format!("{}/chat/completions", url) }
        })
    } else {
        base_url
    };

    // 创建工具
    let tools = ToolRegistry::new();
    tools.register(BashTool::new(&cwd_str));
    tools.register(ReadTool::new());
    tools.register(WriteTool::new());
    tools.register(EditTool::new());
    tools.register(FindTool::new(&cwd_str));
    tools.register(GrepTool::new(&cwd_str));

    // 创建会话
    let session_dir = config::session_dir(&cfg)
        .unwrap_or_else(|| cwd.join(".piso").join("sessions"));
    let mgr = SessionManager::new(&session_dir);
    mgr.ensure_dir().await?;
    let _session = mgr.create(&cwd_str).await?;

    // 系统提示
    let mut prompt_builder = SystemPromptBuilder::new(&cwd_str).with_tool_guides();
    if !cli.no_context_files {
        let ctx_files = context::load_project_context_files(&cwd, config_dir.as_deref());
        if !ctx_files.is_empty() {
            prompt_builder = prompt_builder.append(
                context::format_context_for_prompt(&ctx_files)
            );
        }
    }

    let system_prompt = prompt_builder.build();
    let model_clone = model.clone();
    let api_key_clone = api_key.clone();
    let base_url_clone = effective_base_url.clone();

    // 创建 driver（每次 submit 新建，因为 AgentLoop takes ownership）
    let api_type_clone = api_type.to_string();
    let agent_runner = Box::new(move |text: String, state: std::sync::Arc<pi_tui::AppState>| {
        state.push_user(&text);
        state.set_state(AgentState::Thinking);

        let driver: Box<dyn LlmDriver> = match api_type_clone.as_str() {
            "openai-completions" | "openai-responses" => Box::new(OpenAiDriver::new()),
            "google-gemini" | "gemini" => Box::new(GeminiDriver::new()),
            _ => Box::new(AnthropicDriver::new()),
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            // 新建 session（简化版）
            let tmp = tempfile::tempdir().unwrap();
            let path = tmp.path().join("session.jsonl");
            let _keep = tmp.keep();
            let s = JsonlSession::create(&path, &cwd_str).await.unwrap();

            let mut agent = AgentLoop::new(s, driver, tools.clone_for_agent(), model_clone.clone())
                .with_api_key(api_key_clone.clone())
                .with_system_prompt(system_prompt.clone());

            if let Some(url) = base_url_clone.clone() {
                agent = agent.with_base_url(url);
            }

            state.set_state(AgentState::Streaming);

            match agent.run(&text).await {
                Ok(output) => {
                    state.finish_assistant();
                    state.set_state(AgentState::Idle);
                    let _ = output;
                }
                Err(e) => {
                    state.set_state(AgentState::Error(format!("{e}")));
                }
            }
        });
    });

    pi_tui::run_interactive(model.to_string(), provider.to_string(), agent_runner).await
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
            println!("  No providers configured. Set ANTHROPIC_API_KEY or configure ~/.piso/models.json");
        } else {
            println!("Providers with API keys from environment:");
            for name in available {
                println!("  {name}");
            }
        }
    }

    Ok(())
}

async fn run_list_sessions() -> Result<()> {
    let cwd = env::current_dir()?;
    let cfg = config::load_config(Some(&cwd));
    let session_dir = config::session_dir(&cfg)
        .unwrap_or_else(|| cwd.join(".pi").join("sessions"));

    let mgr = SessionManager::new(&session_dir);
    let sessions = mgr.list().await?;

    if sessions.is_empty() {
        println!("No sessions found.");
        return Ok(());
    }

    println!("Sessions ({} total):", sessions.len());
    println!("{:<25} {:<6} {}", "ID", "Msgs", "CWD");
    println!("{}", "-".repeat(60));
    for s in &sessions {
        let tag = if s.is_today { "*" } else { " " };
        println!("{} {:<24} {:<6} {}", tag, s.id, s.message_count, s.cwd);
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
