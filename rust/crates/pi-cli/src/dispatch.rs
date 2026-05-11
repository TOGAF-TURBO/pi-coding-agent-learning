//! 模式分发 — 根据 CLI 参数启动对应运行模式。

use std::env;
use std::sync::Arc;

use anyhow::{anyhow, bail, Context, Result};

use crate::args::Cli;
use crate::auth::AuthStorage;
use crate::config;

use pi_agent::context;
use pi_agent::loop_engine::{AgentLoop, StreamSink};
use pi_agent::skills;
use pi_agent::system_prompt::SystemPromptBuilder;
use pi_llm::driver::LlmDriver;
use pi_llm::gemini::GeminiDriver;
use pi_llm::openai::OpenAiDriver;
use pi_llm::providers::AnthropicDriver;
use pi_session::manager::SessionManager;
use pi_session::JsonlSession;
use pi_tools::bash::BashTool;
use pi_tools::edit::EditTool;
use pi_tools::find::FindTool;
use pi_tools::grep::GrepTool;
use pi_tools::ls::LsTool;
use pi_tools::read::ReadTool;
use pi_tools::registry::ToolRegistry;
use pi_tools::write::WriteTool;
use pi_tui;

/// 启动流水线。
pub fn run(cli: Cli) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async_main(cli))
}

async fn async_main(cli: Cli) -> Result<()> {
    // 处理 --export（可与任何模式组合）
    if let Some(ref path) = cli.export {
        return run_export(&cli, path).await;
    }

    let mode = resolve_mode(&cli);

    match mode {
        AppMode::Interactive => run_interactive_mode(cli).await,
        AppMode::Print => run_print(cli).await,
        AppMode::Rpc => run_rpc(cli).await,
        AppMode::ListModels => run_list_models(cli).await,
        AppMode::ListSessions => run_list_sessions().await,
        AppMode::Init => run_init().await,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppMode {
    Interactive,
    Print,
    Rpc,
    ListModels,
    ListSessions,
    Init,
}

fn resolve_mode(cli: &Cli) -> AppMode {
    if !cli.messages.is_empty() || cli.print {
        return AppMode::Print;
    }
    // 信息命令优先于 stdin 检测
    if cli.list_models.is_some() {
        return AppMode::ListModels;
    }
    if cli.list_sessions {
        return AppMode::ListSessions;
    }
    if cli.init {
        return AppMode::Init;
    }
    // stdin 是 pipe 时自动进入 print 模式
    if atty::isnt(atty::Stream::Stdin) {
        return AppMode::Print;
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

    // 解析 --model 紧凑语法：provider/id:thinking
    // 如果 model 值包含 / 或 :，从中提取 provider 和 thinking
    let (model_override, provider_override, _thinking_override) = cli
        .model
        .as_deref()
        .map(parse_model_pattern)
        .unwrap_or((None, None, None));

    // 确定 provider（优先级：CLI --provider > --model 中的 /provider > config > env > default）
    let provider_env = std::env::var("PISO_PROVIDER")
        .ok()
        .or_else(|| std::env::var("PI_PROVIDER").ok());
    let provider = cli
        .provider
        .as_deref()
        .or(provider_override)
        .or(cfg.provider.as_deref())
        .or(provider_env.as_deref())
        .unwrap_or("anthropic");

    let api_key = cli
        .api_key
        .as_deref()
        .or_else(|| auth.get_key(provider))
        .ok_or_else(|| {
            anyhow!(
            "No API key found for provider '{}'. Set {}_API_KEY or configure ~/.piso/models.json",
            provider,
            provider.to_uppercase().replace('-', "_"),
        )
        })?;

    // 确定 model（优先级：--model 解析后的 ID > config > env > default）
    let model_env = std::env::var("PISO_MODEL")
        .ok()
        .or_else(|| std::env::var("PI_MODEL").ok());
    let model = model_override
        .or(cfg.model.as_deref())
        .or(model_env.as_deref())
        .unwrap_or("claude-sonnet-4-20250514");

    // 从 models.json 获取 provider 配置
    let provider_config = auth.get_provider(provider);
    let api_type = provider_config
        .map(|c| c.api.as_str())
        .unwrap_or_else(|| default_api_type(provider));
    let base_url = cli
        .base_url
        .clone()
        .or_else(|| provider_config.and_then(|c| c.base_url.clone()));

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
        tools.register(LsTool::new());
    }

    // --tools 白名单过滤
    if let Some(tool_list) = &cli.tools {
        let allowed: Vec<&str> = tool_list
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();
        tools.retain(|name| allowed.contains(&name));
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
        let ctx_files = context::load_project_context_files(&cwd, agent_dir);
        if !ctx_files.is_empty() {
            for ctx in &ctx_files {
                eprintln!("[context] {}", ctx.path.display());
            }
            prompt_builder = prompt_builder.append(context::format_context_for_prompt(&ctx_files));
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
            prompt_builder =
                prompt_builder.append(skills::format_skills_for_prompt(&loaded_skills));
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
    let provider_display = provider_display_name(&provider);
    let model_display = format!("{} ({})", &model, &provider_display);
    let mut agent = AgentLoop::new(session, driver, tools, model)
        .with_api_key(api_key)
        .with_system_prompt(
            prompt_builder
                .with_model_info(&model_display, &provider_display)
                .build(),
        );

    if let Some(url) = effective_base_url {
        agent = agent.with_base_url(url);
    }

    // print 模式：流式输出到 stdout（不输出到 stderr）
    let print_sink: Arc<StreamSink> = Arc::new(Box::new(|event| {
        use pi_llm::driver::StreamEvent;
        use std::io::Write;
        if let StreamEvent::TextDelta { text } = event {
            print!("{}", text);
            let _ = std::io::stdout().flush();
        }
    }));
    agent = agent.with_stream_sink(print_sink);

    // 解析 @file 引用
    let user_message = pi_tools::fileref::resolve_file_refs(&user_message, &cwd).0;

    // 运行 agent
    let output = agent.run(&user_message).await?;

    // 最终换行
    println!();

    let _ = output;
    Ok(())
}

/// 解析会话：--continue / --session / 新建。
async fn resolve_session(
    cli: &Cli,
    cfg: &crate::config::Config,
    cwd: &std::path::Path,
    cwd_str: &str,
) -> Result<JsonlSession> {
    let session_dir = if let Some(dir) = &cli.session_dir {
        std::path::PathBuf::from(dir)
    } else {
        config::session_dir_for_cwd(cfg, cwd)
    };

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
            eprintln!(
                "  {} {} ({} messages, cwd: {})",
                indicator, s.id, s.message_count, s.cwd
            );
        }
        return mgr.open(&sessions[0].id).await;
    }

    // --fork <message-id>: 从指定消息分叉新会话
    if let Some(fork_at) = &cli.fork {
        let source = mgr.continue_last().await?;
        let entries = source.entries();

        // 查找分叉点
        let fork_idx = entries
            .iter()
            .position(|e| e.id() == fork_at)
            .ok_or_else(|| anyhow!("Message '{}' not found in session", fork_at))?;

        // 创建新会话，复制分叉点之前的条目
        let mut new_session = mgr.create(cwd_str).await?;
        for entry in &entries[..=fork_idx] {
            new_session.append(entry.clone()).await?;
        }

        eprintln!(
            "Forked session at message '{}' ({} entries copied)",
            fork_at,
            fork_idx + 1
        );
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

    // 解析 --model 紧凑语法
    let (model_override_int, provider_override_int, _thinking_int) =
        cli.model.as_deref().map(parse_model_pattern).unwrap_or((None, None, None));

    let provider_env = std::env::var("PISO_PROVIDER")
        .ok()
        .or_else(|| std::env::var("PI_PROVIDER").ok());
    let provider = cli
        .provider
        .clone()
        .or(provider_override_int.map(|s| s.to_string()))
        .or(cfg.provider.clone())
        .or(provider_env)
        .unwrap_or_else(|| "anthropic".to_string());

    let api_key = cli
        .api_key
        .clone()
        .or_else(|| auth.get_key(&provider).map(|s| s.to_string()))
        .ok_or_else(|| {
            anyhow!(
            "No API key found for provider '{}'. Set {}_API_KEY or configure ~/.piso/models.json",
            provider,
            provider.to_uppercase().replace('-', "_"),
        )
        })?;

    let model_env = std::env::var("PISO_MODEL")
        .ok()
        .or_else(|| std::env::var("PI_MODEL").ok());
    let model = model_override_int
        .map(|s| s.to_string())
        .or(cfg.model.clone())
        .or(model_env)
        .unwrap_or_else(|| "claude-sonnet-4-20250514".to_string());

    let provider_config = auth.get_provider(&provider);
    let api_type = provider_config
        .map(|c| c.api.clone())
        .unwrap_or_else(|| default_api_type(&provider).to_string());
    let base_url = provider_config.and_then(|c| c.base_url.clone());

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

    // 系统提示
    let mut prompt_builder = SystemPromptBuilder::new(&cwd_str).with_tool_guides();
    if !cli.no_context_files {
        let ctx_files = context::load_project_context_files(&cwd, config_dir.as_deref());
        if !ctx_files.is_empty() {
            for cf in &ctx_files {
                eprintln!("[context] {}", cf.path.display());
            }
            prompt_builder = prompt_builder.append(context::format_context_for_prompt(&ctx_files));
        }
    }

    // 加载技能
    if !cli.no_skills {
        let global_skills = config::skills_dir();
        let loaded_skills = skills::load_skills(&cwd, global_skills.as_deref());
        if !loaded_skills.is_empty() {
            for s in &loaded_skills {
                eprintln!("[skill] {}", s.name);
            }
            prompt_builder =
                prompt_builder.append(skills::format_skills_for_prompt(&loaded_skills));
        }
    }

    // 解析会话（--continue / --session / --resume / 新建）
    let session = resolve_session(&cli, &cfg, &cwd, &cwd_str).await?;
    let session_dir = config::session_dir_for_cwd(&cfg, &cwd);

    // 收集可用模型列表（用于模型选择器）
    let mut available_models = collect_available_models(&auth);

    // --models 过滤
    if let Some(patterns) = &cli.models {
        let patterns: Vec<&str> = patterns.split(',').map(|s| s.trim()).collect();
        available_models.retain(|(prov, _id, _name)| {
            patterns
                .iter()
                .any(|p| prov.contains(p) || _id.contains(p) || _name.contains(p))
        });
    }

    // 加载快捷键
    let kb_path = config::config_dir()
        .map(|d| d.join("keybindings.json"))
        .unwrap_or_else(|| cwd.join("keybindings.json"));
    let keybindings = pi_tui::KeyBindings::load(&kb_path);

    let tui_system_prompt = {
        let provider_display = provider_display_name(&provider);
        let md = format!("{} ({})", &model, &provider_display);
        prompt_builder
            .with_model_info(&md, &provider_display)
            .build()
    };
    let tui_cfg = pi_tui::InteractiveConfig {
        model,
        provider,
        api_key,
        api_type,
        base_url: effective_base_url,
        cwd,
        session_dir,
        system_prompt: tui_system_prompt,
        session: Some(session),
        available_models,
        keybindings,
        extension_runner: None,
    };

    pi_tui::run_interactive(tui_cfg).await
}

async fn run_rpc(cli: Cli) -> Result<()> {
    let cwd = env::current_dir().context("Failed to get current directory")?;
    let cwd_str = cwd.to_string_lossy().to_string();
    let cfg = config::load_config(Some(&cwd));
    let config_dir = config::config_dir();
    let auth = AuthStorage::load(config_dir.as_deref())?;

    let provider_env = std::env::var("PISO_PROVIDER")
        .ok()
        .or_else(|| std::env::var("PI_PROVIDER").ok());
    let provider = cli
        .provider
        .clone()
        .or(cfg.provider.clone())
        .or(provider_env)
        .unwrap_or_else(|| "anthropic".to_string());

    let api_key = cli
        .api_key
        .clone()
        .or_else(|| auth.get_key(&provider).map(|s| s.to_string()))
        .ok_or_else(|| anyhow!("No API key found for provider '{}'", provider,))?;

    let model_env = std::env::var("PISO_MODEL")
        .ok()
        .or_else(|| std::env::var("PI_MODEL").ok());
    let model = cli
        .model
        .clone()
        .or(cfg.model.clone())
        .or(model_env)
        .unwrap_or_else(|| "claude-sonnet-4-20250514".to_string());

    let provider_config = auth.get_provider(&provider);
    let api_type = provider_config
        .map(|c| c.api.clone())
        .unwrap_or_else(|| default_api_type(&provider).to_string());
    let base_url = provider_config.and_then(|c| c.base_url.clone());

    // OpenAI-compat 需要追加 /chat/completions
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

    let mut prompt_builder = SystemPromptBuilder::new(&cwd_str).with_tool_guides();
    if !cli.no_context_files {
        let ctx_files = context::load_project_context_files(&cwd, config_dir.as_deref());
        if !ctx_files.is_empty() {
            prompt_builder = prompt_builder.append(context::format_context_for_prompt(&ctx_files));
        }
    }

    crate::rpc::run_rpc(
        model,
        api_key,
        api_type,
        effective_base_url,
        prompt_builder.build(),
        cwd,
    )
    .await
}

/// Provider 内部 ID → 人类可读名称。
/// 解析 --model 紧凑语法：`provider/modelId:thinkingLevel`
/// 返回 (model_id, provider, thinking_level)
fn parse_model_pattern(pattern: &str) -> (Option<&str>, Option<&str>, Option<&str>) {
    let (prefix, thinking) = match pattern.find(':') {
        Some(idx) => (&pattern[..idx], Some(&pattern[idx + 1..])),
        None => (pattern, None),
    };

    let (model_id, provider) = match prefix.find('/') {
        Some(idx) => (&prefix[idx + 1..], Some(&prefix[..idx])),
        None => (prefix, None),
    };

    // model_id 为空时返回 None
    let model_id = if model_id.is_empty() {
        None
    } else {
        Some(model_id)
    };

    (model_id, provider, thinking)
}

fn provider_display_name(provider: &str) -> String {
    match provider {
        "anthropic" => "Anthropic".to_string(),
        "amazon-bedrock" | "bedrock" => "Amazon Bedrock".to_string(),
        "azure" | "azure-openai" => "Azure OpenAI".to_string(),
        "cerebras" => "Cerebras".to_string(),
        "cloudflare" | "cloudflare-workers" => "Cloudflare Workers AI".to_string(),
        "deepseek" => "DeepSeek".to_string(),
        "fireworks" => "Fireworks".to_string(),
        "google" | "gemini" => "Google Gemini".to_string(),
        "google-vertex" | "vertex" => "Google Vertex AI".to_string(),
        "groq" => "Groq".to_string(),
        "huggingface" => "Hugging Face".to_string(),
        "mistral" => "Mistral".to_string(),
        "openai" => "OpenAI".to_string(),
        "openrouter" => "OpenRouter".to_string(),
        "together" => "Together AI".to_string(),
        "xai" => "xAI".to_string(),
        "xiaomi" => "Xiaomi MiMo".to_string(),
        "glm" | "zhipu" => "Zhipu AI".to_string(),
        // 未知 provider：首字母大写返回
        other => {
            let mut c = other.chars();
            match c.next() {
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                None => other.to_string(),
            }
        }
    }
}

/// 根据 provider 名推断默认 API type（无 models.json 配置时的 fallback）。
fn default_api_type(provider: &str) -> &'static str {
    match provider {
        "openai" | "deepseek" | "groq" | "openrouter" | "together" | "fireworks" | "glm"
        | "zhipu" => "openai-completions",
        "google" | "gemini" => "google-gemini",
        _ => "anthropic-messages",
    }
}

async fn run_list_models(_cli: Cli) -> Result<()> {
    let config_dir = config::config_dir();
    let auth = AuthStorage::load(config_dir.as_deref())?;

    println!("Configured providers:");
    for name in auth.configured_providers() {
        let key_status = if auth.get_key(&name).is_some() {
            "API key found"
        } else {
            "no API key"
        };
        let config = auth.get_provider(&name);
        let api = config.map(|c| c.api.as_str()).unwrap_or("unknown");
        println!("  {name} ({api}, {key_status})");
    }

    if auth.configured_providers().is_empty() {
        let available = auth.available_providers();
        if available.is_empty() {
            println!(
                "  No providers configured. Set ANTHROPIC_API_KEY or configure ~/.piso/models.json"
            );
        } else {
            println!("Providers with API keys from environment:");
            for name in available {
                println!("  {name}");
            }
        }
    }

    Ok(())
}

/// 收集所有可用模型（用于模型选择器）。
pub fn collect_available_models(auth: &AuthStorage) -> Vec<(String, String, String)> {
    let mut models = Vec::new();
    for prov_name in auth.configured_providers() {
        if let Some(pc) = auth.get_provider(&prov_name) {
            for m in &pc.models {
                models.push((prov_name.clone(), m.id.clone(), m.name.clone()));
            }
        }
    }
    // 如果 models.json 没有模型列表，用默认模型 ID
    if models.is_empty() {
        for prov_name in auth.available_providers() {
            models.push((prov_name.clone(), "default".to_string(), prov_name.clone()));
        }
    }
    models
}

async fn run_list_sessions() -> Result<()> {
    let cwd = env::current_dir()?;
    let cfg = config::load_config(Some(&cwd));
    let session_dir = config::session_dir_for_cwd(&cfg, &cwd);

    let mgr = SessionManager::new(&session_dir);
    let sessions = mgr.list().await?;

    if sessions.is_empty() {
        println!("No sessions found.");
        return Ok(());
    }

    println!("Sessions ({} total):", sessions.len());
    println!("{:<25} {:<6} CWD", "ID", "Msgs");
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

/// 生成模板配置文件到 ~/.piso/。
async fn run_init() -> Result<()> {
    use std::io::Write;

    let config_dir =
        config::config_dir().ok_or_else(|| anyhow!("Cannot determine home directory"))?;

    // 创建目录
    std::fs::create_dir_all(&config_dir)?;
    std::fs::create_dir_all(config_dir.join("sessions"))?;
    std::fs::create_dir_all(config_dir.join("skills"))?;

    let models_path = config_dir.join("models.json");
    let settings_path = config_dir.join("settings.json");
    let keybindings_path = config_dir.join("keybindings.json");

    // models.json — 仅当不存在时创建
    if !models_path.exists() {
        let template = r#"{
  "providers": {
    "openai": {
      "name": "OpenAI",
      "baseUrl": "https://api.openai.com/v1",
      "apiKey": "sk-...",
      "api": "openai-completions",
      "models": [
        { "id": "gpt-4o", "name": "GPT-4o" },
        { "id": "gpt-4o-mini", "name": "GPT-4o Mini" }
      ]
    }
  }
}"#;
        let mut f = std::fs::File::create(&models_path)?;
        f.write_all(template.as_bytes())?;
        println!("Created {}", models_path.display());
        println!("  -> Edit this file to add your API key and providers");
    } else {
        println!("Exists: {}", models_path.display());
    }

    // settings.json
    if !settings_path.exists() {
        let template = r#"{
  "model": "gpt-4o",
  "provider": "openai",
  "max_tokens": 16384
}"#;
        let mut f = std::fs::File::create(&settings_path)?;
        f.write_all(template.as_bytes())?;
        println!("Created {}", settings_path.display());
    } else {
        println!("Exists: {}", settings_path.display());
    }

    // keybindings.json
    if !keybindings_path.exists() {
        let template = r#"{
  "submit": { "modifiers": "ctrl", "key": "o" },
  "quit": { "modifiers": "ctrl", "key": "c" },
  "cancel": { "modifiers": "none", "key": "escape" },
  "open_session_picker": { "modifiers": "ctrl", "key": "s" },
  "open_model_picker": { "modifiers": "ctrl", "key": "p" },
  "tab_complete": { "modifiers": "none", "key": "tab" }
}"#;
        let mut f = std::fs::File::create(&keybindings_path)?;
        f.write_all(template.as_bytes())?;
        println!("Created {}", keybindings_path.display());
    } else {
        println!("Exists: {}", keybindings_path.display());
    }

    println!("\npiso config directory: {}", config_dir.display());
    println!("Next steps:");
    println!("  1. Edit ~/.piso/models.json and add your API key");
    println!("  2. Run: piso --provider openai --model gpt-4o");

    Ok(())
}

/// 导出会话为 HTML。
async fn run_export(cli: &Cli, output_path: &str) -> Result<()> {
    let cwd = env::current_dir().context("Failed to get current directory")?;
    let cfg = config::load_config(Some(&cwd));
    let session_dir = config::session_dir_for_cwd(&cfg, &cwd);

    // 查找最新会话或指定会话
    let mgr = SessionManager::new(&session_dir);
    let session_id = if let Some(session_id) = &cli.session {
        session_id.clone()
    } else if cli.r#continue {
        let sessions = mgr.list().await?;
        sessions
            .first()
            .ok_or_else(|| anyhow!("No sessions found"))?
            .id
            .clone()
    } else {
        let sessions = mgr.list().await?;
        sessions
            .first()
            .ok_or_else(|| anyhow!("No sessions found. Use --session <id> or --continue"))?
            .id
            .clone()
    };
    let session = mgr.open(&session_id).await?;

    let html = render_session_html(&session);
    std::fs::write(output_path, &html)
        .context(format!("Failed to write HTML to {}", output_path))?;
    eprintln!("Exported session to {}", output_path);
    Ok(())
}

/// 将 session 渲染为 HTML。
fn render_session_html(session: &JsonlSession) -> String {
    use pi_types::session::SessionEntry;

    let mut body = String::new();

    for entry in session.entries() {
        if let SessionEntry::Message(msg) = entry {
            let text = extract_text_from_content(&msg.content);
            if text.is_empty() {
                continue;
            }
            let escaped = html_escape(&text);
            match msg.role.as_str() {
                "user" => {
                    body.push_str(&format!(
                        "<div class=\"msg user\"><div class=\"role\">You</div><div class=\"content\"><pre>{}</pre></div></div>\n",
                        escaped
                    ));
                }
                "assistant" => {
                    // Markdown → 简单 HTML
                    let html_content = markdown_to_simple_html(&text);
                    body.push_str(&format!(
                        "<div class=\"msg assistant\"><div class=\"role\">Assistant</div><div class=\"content\">{}</div></div>\n",
                        html_content
                    ));
                }
                _ => {}
            }
        }
    }

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>piso session {}</title>
<style>
body {{ font-family: -apple-system, sans-serif; max-width: 800px; margin: 0 auto; padding: 20px; background: #1a1a2e; color: #e0e0e0; }}
.msg {{ margin: 16px 0; padding: 12px 16px; border-radius: 8px; }}
.user {{ background: #16213e; border-left: 3px solid #0f3460; }}
.assistant {{ background: #1a1a2e; border-left: 3px solid #e94560; }}
.role {{ font-weight: bold; font-size: 0.85em; color: #888; margin-bottom: 4px; }}
.content {{ line-height: 1.6; white-space: pre-wrap; }}
.content pre {{ background: #0d1117; padding: 12px; border-radius: 4px; overflow-x: auto; }}
.content code {{ background: #0d1117; padding: 2px 6px; border-radius: 3px; font-size: 0.9em; }}
.content strong {{ color: #fff; }}
.content h1, .content h2, .content h3 {{ color: #e94560; margin-top: 12px; }}
.content ul, .content ol {{ padding-left: 20px; }}
.content li {{ margin: 4px 0; }}
</style>
</head>
<body>
<h1>piso session</h1>
<p style="color:#666">ID: {}</p>
{}
</body>
</html>"#,
        session.id(),
        session.id(),
        body
    )
}

/// 从 JSON content 数组提取纯文本（dispatch 内部辅助）。
fn extract_text_from_content(content: &serde_json::Value) -> String {
    content
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|block| {
                    if block.get("type").and_then(|v| v.as_str()) == Some("text") {
                        block
                            .get("text")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default()
}

/// 简单 Markdown → HTML 转换。
fn markdown_to_simple_html(md: &str) -> String {
    let mut html = String::new();
    let mut in_code_block = false;

    for line in md.lines() {
        if line.starts_with("```") {
            if in_code_block {
                html.push_str("</code></pre>\n");
            } else {
                html.push_str("<pre><code>");
            }
            in_code_block = !in_code_block;
            continue;
        }
        if in_code_block {
            html.push_str(&html_escape(line));
            html.push('\n');
            continue;
        }
        // 标题
        if let Some(rest) = line.strip_prefix("# ") {
            html.push_str(&format!("<h1>{}</h1>\n", html_escape(rest)));
            continue;
        }
        if let Some(rest) = line.strip_prefix("## ") {
            html.push_str(&format!("<h2>{}</h2>\n", html_escape(rest)));
            continue;
        }
        if let Some(rest) = line.strip_prefix("### ") {
            html.push_str(&format!("<h3>{}</h3>\n", html_escape(rest)));
            continue;
        }
        // 列表
        let trimmed = line.trim_start();
        if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
            let content = &trimmed[2..];
            html.push_str(&format!("<li>{}</li>\n", inline_html(content)));
            continue;
        }
        // 段落
        html.push_str(&format!("<p>{}</p>\n", inline_html(line)));
    }

    html
}

/// 行内 Markdown → HTML。
fn inline_html(text: &str) -> String {
    let text = html_escape(text);
    // **bold**
    let text = regex_replace(&text, r"\*\*([^*]+)\*\*", "<strong>$1</strong>");
    // `code`

    regex_replace(&text, r"`([^`]+)`", "<code>$1</code>")
}

/// 简单正则替换（避免引入 regex 依赖）。
fn regex_replace(text: &str, _pattern: &str, replacement: &str) -> String {
    // 手动解析 **bold** 和 `code`
    let mut result = String::new();
    let mut chars = text.chars().peekable();
    let mut current = String::new();

    while let Some(ch) = chars.next() {
        if ch == '`' {
            result.push_str(&current);
            current.clear();
            let mut code = String::new();
            while let Some(&c) = chars.peek() {
                if c == '`' {
                    chars.next();
                    break;
                }
                code.push(chars.next().unwrap());
            }
            result.push_str(&replacement.replace("$1", &code));
        } else if ch == '*' && chars.peek() == Some(&'*') {
            chars.next();
            result.push_str(&current);
            current.clear();
            let mut bold = String::new();
            while let Some(&c) = chars.peek() {
                if c == '*' {
                    chars.next();
                    if chars.peek() == Some(&'*') {
                        chars.next();
                        break;
                    } else {
                        bold.push('*');
                        continue;
                    }
                }
                bold.push(chars.next().unwrap());
            }
            result.push_str(&format!("<strong>{}</strong>", bold));
        } else {
            current.push(ch);
        }
    }
    result.push_str(&current);
    result
}

/// HTML 转义。
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_display_known() {
        assert_eq!(provider_display_name("glm"), "Zhipu AI");
        assert_eq!(provider_display_name("zhipu"), "Zhipu AI");
        assert_eq!(provider_display_name("openai"), "OpenAI");
        assert_eq!(provider_display_name("anthropic"), "Anthropic");
        assert_eq!(provider_display_name("deepseek"), "DeepSeek");
        assert_eq!(provider_display_name("gemini"), "Google Gemini");
        assert_eq!(provider_display_name("google"), "Google Gemini");
        assert_eq!(provider_display_name("google-vertex"), "Google Vertex AI");
    }

    #[test]
    fn parse_model_pattern_provider_and_thinking() {
        let (model, provider, thinking) = parse_model_pattern("openai/gpt-4o:high");
        assert_eq!(model, Some("gpt-4o"));
        assert_eq!(provider, Some("openai"));
        assert_eq!(thinking, Some("high"));
    }

    #[test]
    fn parse_model_pattern_thinking_only() {
        let (model, provider, thinking) = parse_model_pattern("sonnet:medium");
        assert_eq!(model, Some("sonnet"));
        assert_eq!(provider, None);
        assert_eq!(thinking, Some("medium"));
    }

    #[test]
    fn parse_model_pattern_provider_only() {
        let (model, provider, thinking) = parse_model_pattern("anthropic/claude-sonnet-4");
        assert_eq!(model, Some("claude-sonnet-4"));
        assert_eq!(provider, Some("anthropic"));
        assert_eq!(thinking, None);
    }

    #[test]
    fn parse_model_pattern_plain() {
        let (model, provider, thinking) = parse_model_pattern("gpt-4o");
        assert_eq!(model, Some("gpt-4o"));
        assert_eq!(provider, None);
        assert_eq!(thinking, None);
    }
}
