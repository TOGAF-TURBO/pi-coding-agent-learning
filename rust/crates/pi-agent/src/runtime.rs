//! 运行时 — 组装 Agent 运行时依赖。
//!
//! 将 model resolver、auth、session、tools、extensions 统一组装，
//! 为 agent loop 提供完整的运行时上下文。

use std::path::PathBuf;

/// Agent 运行时配置 — 所有一次 agent turn 需要的参数。
#[derive(Debug, Clone)]
pub struct AgentRuntime {
    /// 工作目录。
    pub cwd: PathBuf,
    /// 模型 ID。
    pub model: String,
    /// Provider 名称。
    pub provider: String,
    /// API key。
    pub api_key: String,
    /// API 类型（openai-completions / anthropic / gemini）。
    pub api_type: String,
    /// 自定义 base URL。
    pub base_url: Option<String>,
    /// 系统提示。
    pub system_prompt: String,
    /// 最大输出 token。
    pub max_tokens: u32,
}

impl AgentRuntime {
    /// 从 CLI 参数和环境构建运行时。
    pub fn new(
        cwd: PathBuf,
        model: String,
        provider: String,
        api_key: String,
        api_type: String,
    ) -> Self {
        Self {
            cwd,
            model,
            provider,
            api_key,
            api_type,
            base_url: None,
            system_prompt: String::new(),
            max_tokens: 16384,
        }
    }

    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = prompt.into();
        self
    }

    pub fn with_max_tokens(mut self, tokens: u32) -> Self {
        self.max_tokens = tokens;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_builder() {
        let rt = AgentRuntime::new(
            PathBuf::from("/project"),
            "gpt-4".into(),
            "openai".into(),
            "sk-xxx".into(),
            "openai-completions".into(),
        )
        .with_base_url("https://api.example.com")
        .with_max_tokens(8192);

        assert_eq!(rt.model, "gpt-4");
        assert_eq!(rt.base_url, Some("https://api.example.com".to_string()));
        assert_eq!(rt.max_tokens, 8192);
    }
}
