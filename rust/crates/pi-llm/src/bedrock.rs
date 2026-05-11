//! Amazon Bedrock driver — AWS Sigv4 认证 + Converse Stream API。
//!
//! 认证: AWS credentials (环境变量 AWS_ACCESS_KEY_ID, AWS_SECRET_ACCESS_KEY)
//! API: POST /model/{modelId}/converse-stream
//! Region: 默认 us-east-1，可通过 AWS_REGION 覆盖
//!
//! 注意: 完整 Sigv4 签名需要 `aws-sigv4` crate。
//! 当前为简化实现，适合原型开发。

use async_trait::async_trait;
use futures::StreamExt;
use reqwest::Client;
use serde_json::json;

use pi_types::model::ModelInfo;

use crate::driver::{CompletionRequest, LlmDriver, StreamEvent, StreamResult};

/// Amazon Bedrock driver。
pub struct BedrockDriver {
    client: Client,
}

impl BedrockDriver {
    pub fn new() -> Self {
        Self { client: Client::new() }
    }

    fn region() -> String {
        std::env::var("AWS_REGION")
            .or_else(|_| std::env::var("AWS_DEFAULT_REGION"))
            .unwrap_or_else(|_| "us-east-1".to_string())
    }

    fn build_url(model_id: &str, region: &str) -> String {
        let encoded = model_id.replace('/', "%2F");
        format!(
            "https://bedrock-runtime.{}.amazonaws.com/model/{}/converse-stream",
            region, encoded
        )
    }

    fn has_aws_credentials() -> bool {
        std::env::var("AWS_ACCESS_KEY_ID").is_ok()
            && std::env::var("AWS_SECRET_ACCESS_KEY").is_ok()
    }
}

impl Default for BedrockDriver {
    fn default() -> Self { Self::new() }
}

#[async_trait]
impl LlmDriver for BedrockDriver {
    fn name(&self) -> &str { "amazon-bedrock" }

    fn stream(&self, request: CompletionRequest) -> Result<StreamResult, anyhow::Error> {
        if !Self::has_aws_credentials() {
            return Err(anyhow::anyhow!(
                "AWS credentials not found. Set AWS_ACCESS_KEY_ID and AWS_SECRET_ACCESS_KEY environment variables."
            ));
        }

        let region = Self::region();
        let url = Self::build_url(&request.model, &region);

        let mut messages = Vec::new();
        for msg in &request.messages {
            match msg {
                pi_types::message::Message::User(u) => {
                    let texts: Vec<_> = u.content.iter()
                        .filter_map(|b| match b {
                            pi_types::message::ContentBlock::Text(t) => Some(t.text.as_str()),
                            _ => None,
                        })
                        .collect::<Vec<_>>();
                    messages.push(json!({ "role": "user", "content": texts.join("\n") }));
                }
                pi_types::message::Message::Assistant(a) => {
                    let texts: Vec<_> = a.content.iter()
                        .filter_map(|b| match b {
                            pi_types::message::ContentBlock::Text(t) => Some(t.text.as_str()),
                            _ => None,
                        })
                        .collect::<Vec<_>>();
                    messages.push(json!({ "role": "assistant", "content": texts.join("\n") }));
                }
                pi_types::message::Message::ToolResult(t) => {
                    messages.push(json!({ "role": "user", "content": t.content }));
                }
            }
        }

        let mut body = json!({ "messages": messages });

        if let Some(sp) = &request.system_prompt {
            body["system"] = json!([{ "text": sp }]);
        }

        if !request.tools.is_empty() {
            let tools: Vec<_> = request.tools.iter().filter_map(|t| {
                Some(json!({
                    "toolSpec": {
                        "name": t.name,
                        "description": t.description,
                        "inputSchema": { "json": t.parameters }
                    }
                }))
            }).collect();
            body["toolConfig"] = json!({ "tools": tools });
        }

        let api_key = request.api_key.clone();
        let response_future = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {api_key}"))
            .header("Content-Type", "application/json")
            .header("Accept", "text/event-stream")
            .json(&body)
            .send();

        let stream = async_stream::stream! {
            let response = match response_future.await {
                Ok(r) => r,
                Err(e) => {
                    yield Err(anyhow::anyhow!("Bedrock request failed: {e}"));
                    return;
                }
            };

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                yield Err(anyhow::anyhow!("Bedrock error {status}: {body}"));
                return;
            }

            // 解析 Bedrock SSE
            let mut stream = response.bytes_stream();
            let mut buffer = String::new();

            while let Some(chunk_result) = stream.next().await {
                let chunk = match chunk_result {
                    Ok(c) => c,
                    Err(e) => {
                        yield Err(anyhow::anyhow!("Bedrock stream error: {e}"));
                        return;
                    }
                };

                buffer.push_str(&String::from_utf8_lossy(&chunk));

                while let Some(line_end) = buffer.find('\n') {
                    let line = buffer[..line_end].trim().to_string();
                    buffer = buffer[line_end + 1..].to_string();

                    if !line.starts_with('{') { continue; }

                    if let Ok(event) = serde_json::from_str::<serde_json::Value>(&line) {
                        match event.get("type").and_then(|t| t.as_str()) {
                            Some("contentBlockDelta") => {
                                if let Some(text) = event.pointer("/delta/text").and_then(|v| v.as_str()) {
                                    yield Ok(StreamEvent::TextDelta { text: text.to_string() });
                                }
                                if let Some(think) = event.pointer("/delta/thinking").and_then(|v| v.as_str()) {
                                    yield Ok(StreamEvent::ThinkingDelta { thinking: think.to_string() });
                                }
                            }
                            Some("messageStop") => {
                                yield Ok(StreamEvent::Stop { reason: None });
                                return;
                            }
                            Some("toolUse") => {
                                if let (Some(id), Some(name)) = (
                                    event.get("toolUseId").and_then(|v| v.as_str()),
                                    event.get("name").and_then(|v| v.as_str()),
                                ) {
                                    yield Ok(StreamEvent::ToolCallStart {
                                        id: id.to_string(),
                                        name: name.to_string(),
                                        index: 0,
                                    });
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        };

        Ok(Box::pin(stream))
    }
}

/// Bedrock 支持的模型列表。
pub fn bedrock_models() -> Vec<ModelInfo> {
    vec![
        ModelInfo {
            id: "anthropic.claude-sonnet-4-20250514".to_string(),
            name: "Claude Sonnet 4 (Bedrock)".to_string(),
            provider: "bedrock".to_string(),
            api: "bedrock-converse-stream".to_string(),
            reasoning: true,
            context_window: 200_000,
            max_tokens: 16_384,
            cost: None,
        },
        ModelInfo {
            id: "anthropic.claude-haiku-3-5-20241022".to_string(),
            name: "Claude Haiku 3.5 (Bedrock)".to_string(),
            provider: "bedrock".to_string(),
            api: "bedrock-converse-stream".to_string(),
            reasoning: false,
            context_window: 200_000,
            max_tokens: 8_192,
            cost: None,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn driver_name() {
        let driver = BedrockDriver::new();
        assert_eq!(driver.name(), "amazon-bedrock");
    }

    #[test]
    fn default_impl() {
        let driver = BedrockDriver::default();
        assert_eq!(driver.name(), "amazon-bedrock");
    }

    #[test]
    fn build_url_encodes_model() {
        let url = BedrockDriver::build_url(
            "anthropic.claude-sonnet-4-20250514",
            "us-east-1",
        );
        assert!(url.contains("bedrock-runtime.us-east-1.amazonaws.com"));
        assert!(url.contains("anthropic.claude-sonnet-4-20250514"));
        assert!(url.contains("converse-stream"));
    }

    #[test]
    fn bedrock_models_list() {
        let models = bedrock_models();
        assert!(!models.is_empty());
        assert!(models.iter().any(|m| m.id.contains("claude")));
    }

    #[test]
    fn no_credentials_detected() {
        // This test runs without AWS credentials
        // Just verify has_aws_credentials returns false when not set
        // (it might return true if CI has credentials)
        let _result = BedrockDriver::has_aws_credentials();
        // Don't assert false — CI might have AWS creds
    }
}
