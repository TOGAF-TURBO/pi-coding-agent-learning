//! Azure OpenAI driver — 支持 Azure 部署映射和 API 密钥认证。
//!
//! 认证方式: `api-key: <key>` 头部
//! URL: `https://<resource>.openai.azure.com/openai/deployments/<deployment>/chat/completions?api-version=2024-02-15-preview`
//!
//! 请求/响应格式与 OpenAI chat completions 相同，仅认证和 URL 不同。

use async_trait::async_trait;
use futures::StreamExt;
use reqwest::Client;
use serde_json::json;

use crate::driver::{CompletionRequest, LlmDriver, StreamEvent, StreamResult};
use crate::openai::{build_openai_request, parse_openai_events};

/// Azure OpenAI driver。
pub struct AzureOpenAiDriver {
    client: Client,
}

impl AzureOpenAiDriver {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    fn build_url(base_url: &str, deployment: &str) -> String {
        let base = base_url.trim_end_matches('/');
        format!(
            "{}/openai/deployments/{}/chat/completions?api-version=2024-02-15-preview",
            base, deployment
        )
    }
}

impl Default for AzureOpenAiDriver {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LlmDriver for AzureOpenAiDriver {
    fn name(&self) -> &str {
        "azure-openai"
    }

    fn stream(&self, request: CompletionRequest) -> Result<StreamResult, anyhow::Error> {
        let base = request
            .base_url
            .clone()
            .unwrap_or_else(|| "https://api.openai.azure.com".to_string());
        let url = Self::build_url(&base, &request.model);
        let body = build_openai_request(&request);
        let api_key = request.api_key.clone();

        let response_future = self
            .client
            .post(&url)
            .header("api-key", &api_key)
            .header("content-type", "application/json")
            .json(&body)
            .send();

        let stream = async_stream::stream! {
            let response = match response_future.await {
                Ok(r) => r,
                Err(e) => {
                    yield Err(anyhow::anyhow!("Azure OpenAI request failed: {e}"));
                    return;
                }
            };

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                yield Err(anyhow::anyhow!("Azure error {status}: {body}"));
                return;
            }

            let mut stream = response.bytes_stream();
            let mut buffer = String::new();

            while let Some(chunk_result) = stream.next().await {
                let chunk = match chunk_result {
                    Ok(c) => c,
                    Err(e) => {
                        yield Err(anyhow::anyhow!("Azure stream error: {e}"));
                        return;
                    }
                };

                buffer.push_str(&String::from_utf8_lossy(&chunk));

                while let Some(line_end) = buffer.find('\n') {
                    let line = buffer[..line_end].trim().to_string();
                    buffer = buffer[line_end + 1..].to_string();

                    if !line.starts_with("data: ") { continue; }
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
        let driver = AzureOpenAiDriver::new();
        assert_eq!(driver.name(), "azure-openai");
    }

    #[test]
    fn build_url_format() {
        let url = AzureOpenAiDriver::build_url(
            "https://myresource.openai.azure.com",
            "gpt-4o-deployment",
        );
        assert!(url.contains("myresource.openai.azure.com"));
        assert!(url.contains("gpt-4o-deployment"));
        assert!(url.contains("api-version=2024-02-15-preview"));
    }

    #[test]
    fn default_impl() {
        let driver = AzureOpenAiDriver::default();
        assert_eq!(driver.name(), "azure-openai");
    }
}
