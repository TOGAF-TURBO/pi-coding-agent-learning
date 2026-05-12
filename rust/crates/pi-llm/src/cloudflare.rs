//! Cloudflare Workers AI 提供商实现。
//!
//! 通过 OpenAI 兼容 API 适配 Cloudflare Workers AI：
//! - 直接端点：`api.cloudflare.com/client/v4/accounts/{account_id}/ai/v1`
//! - AI Gateway：`gateway.ai.cloudflare.com/v1/{account_id}/{gateway_id}/openai`
//! - 环境变量：`CLOUDFLARE_API_KEY`, `CLOUDFLARE_ACCOUNT_ID`, `CLOUDFLARE_GATEWAY_ID`

use async_trait::async_trait;
use futures::StreamExt;
use reqwest::Client;

use crate::driver::{CompletionRequest, LlmDriver, StreamEvent, StreamResult};
use crate::openai::{build_openai_request, parse_openai_events};

/// Cloudflare Workers AI / AI Gateway driver。
///
/// 使用 OpenAI 兼容格式，仅需自定义 URL 构建。
#[derive(Default)]
pub struct CloudflareDriver {
    client: Client,
}

impl CloudflareDriver {
    pub fn new() -> Self {
        Self::default()
    }

    /// 根据环境变量构建 base URL。
    fn build_base_url(request: &CompletionRequest) -> anyhow::Result<String> {
        // 优先使用 request.base_url（用户显式配置）
        if let Some(base) = &request.base_url {
            return Ok(base.clone());
        }

        let account_id = std::env::var("CLOUDFLARE_ACCOUNT_ID")
            .map_err(|_| anyhow::anyhow!("CLOUDFLARE_ACCOUNT_ID not set"))?;

        let base = match std::env::var("CLOUDFLARE_GATEWAY_ID") {
            Ok(gateway_id) => format!(
                "https://gateway.ai.cloudflare.com/v1/{account_id}/{gateway_id}/openai"
            ),
            Err(_) => format!(
                "https://api.cloudflare.com/client/v4/accounts/{account_id}/ai/v1"
            ),
        };

        Ok(base)
    }

    /// 构建 Cloudflare chat completions URL。
    fn build_url(request: &CompletionRequest) -> anyhow::Result<String> {
        let base = Self::build_base_url(request)?;
        Ok(format!("{}/chat/completions", base))
    }
}

#[async_trait]
impl LlmDriver for CloudflareDriver {
    fn name(&self) -> &str {
        "cloudflare"
    }

    fn stream(&self, request: CompletionRequest) -> Result<StreamResult, anyhow::Error> {
        let url = Self::build_url(&request)?;
        let api_key = if !request.api_key.is_empty() {
            request.api_key.clone()
        } else {
            std::env::var("CLOUDFLARE_API_KEY")
                .map_err(|_| anyhow::anyhow!("CLOUDFLARE_API_KEY not set"))?
        };

        let body = build_openai_request(&request);

        let response_future = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {api_key}"))
            .header("Content-Type", "application/json")
            .json(&body)
            .send();

        let stream = async_stream::stream! {
            let response = match response_future.await {
                Ok(r) => r,
                Err(e) => {
                    yield Err(anyhow::anyhow!("Cloudflare request failed: {e}"));
                    return;
                }
            };

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                yield Err(anyhow::anyhow!("Cloudflare error {status}: {body}"));
                return;
            }

            let mut stream = response.bytes_stream();
            let mut buffer = String::new();

            while let Some(chunk_result) = stream.next().await {
                let chunk = match chunk_result {
                    Ok(c) => c,
                    Err(e) => {
                        yield Err(anyhow::anyhow!("Cloudflare stream error: {e}"));
                        return;
                    }
                };

                buffer.push_str(&String::from_utf8_lossy(&chunk));

                while let Some(line_end) = buffer.find('\n') {
                    let line = buffer[..line_end].trim().to_string();
                    buffer = buffer[line_end + 1..].to_string();

                    if !line.starts_with("data: ") {
                        continue;
                    }
                    let data = &line[6..];
                    if data == "[DONE]" {
                        yield Ok(StreamEvent::Stop { reason: None });
                        return;
                    }

                    if let Ok(chunk_val) = serde_json::from_str::<serde_json::Value>(data) {
                        if let Some(events) = parse_openai_events(&chunk_val) {
                            for event in events {
                                yield Ok(event);
                            }
                        }
                    }
                }
            }
        };

        Ok(Box::pin(stream))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn driver_name() {
        let driver = CloudflareDriver::new();
        assert_eq!(driver.name(), "cloudflare");
    }

    #[test]
    fn build_url_no_account() {
        std::env::remove_var("CLOUDFLARE_ACCOUNT_ID");
        let req = CompletionRequest { model: "test".to_string(), system_prompt: None, messages: vec![], tools: vec![], thinking_enabled: false, thinking_budget: None, max_tokens: 4096, api_key: String::new(), base_url: None };
        let result = CloudflareDriver::build_url(&req);
        assert!(result.is_err());
    }

    #[test]
    fn build_url_direct() {
        std::env::set_var("CLOUDFLARE_ACCOUNT_ID", "test-acc-123");
        std::env::remove_var("CLOUDFLARE_GATEWAY_ID");
        let req = CompletionRequest { model: "test".to_string(), system_prompt: None, messages: vec![], tools: vec![], thinking_enabled: false, thinking_budget: None, max_tokens: 4096, api_key: String::new(), base_url: None };
        let url = CloudflareDriver::build_url(&req).unwrap();
        assert!(url.contains("api.cloudflare.com"));
        assert!(url.contains("test-acc-123"));
        assert!(url.contains("/ai/v1/chat/completions"));
        std::env::remove_var("CLOUDFLARE_ACCOUNT_ID");
    }

    #[test]
    fn build_url_gateway() {
        std::env::set_var("CLOUDFLARE_ACCOUNT_ID", "test-acc-123");
        std::env::set_var("CLOUDFLARE_GATEWAY_ID", "my-gw");
        let req = CompletionRequest { model: "test".to_string(), system_prompt: None, messages: vec![], tools: vec![], thinking_enabled: false, thinking_budget: None, max_tokens: 4096, api_key: String::new(), base_url: None };
        let url = CloudflareDriver::build_url(&req).unwrap();
        assert!(url.contains("gateway.ai.cloudflare.com"));
        assert!(url.contains("my-gw"));
        std::env::remove_var("CLOUDFLARE_ACCOUNT_ID");
        std::env::remove_var("CLOUDFLARE_GATEWAY_ID");
    }

    #[test]
    fn build_url_explicit_base() {
        let req = CompletionRequest {
            base_url: Some("https://custom.example.com/v1".to_string()),
            model: "test".to_string(),
            system_prompt: None,
            messages: vec![],
            tools: vec![],
            thinking_enabled: false,
            thinking_budget: None,
            max_tokens: 4096,
            api_key: String::new(),
        };
        let url = CloudflareDriver::build_url(&req).unwrap();
        assert_eq!(url, "https://custom.example.com/v1/chat/completions");
    }

    #[test]
    fn cloudflare_registered_in_provider_registry() {
        let reg = crate::registry::ProviderRegistry::new();
        let driver = reg.get("cloudflare");
        assert!(driver.is_some(), "cloudflare should be registered");
        assert_eq!(driver.unwrap().name(), "cloudflare");
    }
}
