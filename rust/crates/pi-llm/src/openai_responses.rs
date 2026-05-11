//! OpenAI Responses API driver — 支持新的 responses 格式。
//!
//! OpenAI Responses API 使用 `input` 替代 `messages`，
//! 响应包含 `output` items。SSE 格式与 chat completions 相同。

use async_trait::async_trait;
use futures::StreamExt;
use reqwest::Client;
use serde_json::json;

use crate::driver::{CompletionRequest, LlmDriver, StreamEvent, StreamResult};
use crate::openai::parse_openai_events;

/// OpenAI Responses API driver。
pub struct OpenAiResponsesDriver {
    client: Client,
}

impl OpenAiResponsesDriver {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

impl Default for OpenAiResponsesDriver {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LlmDriver for OpenAiResponsesDriver {
    fn name(&self) -> &str {
        "openai-responses"
    }

    fn stream(&self, request: CompletionRequest) -> Result<StreamResult, anyhow::Error> {
        let url = request
            .base_url
            .as_ref()
            .map(|b| format!("{}/responses", b.trim_end_matches('/')))
            .unwrap_or_else(|| "https://api.openai.com/v1/responses".to_string());

        // 转换 messages 为 responses API 的 input 格式
        let mut input = Vec::new();
        for msg in &request.messages {
            match msg {
                pi_types::message::Message::User(u) => {
                    let texts: Vec<_> = u
                        .content
                        .iter()
                        .filter_map(|b| match b {
                            pi_types::message::ContentBlock::Text(t) => Some(t.text.as_str()),
                            _ => None,
                        })
                        .collect::<Vec<_>>();
                    input.push(json!({ "role": "user", "content": texts.join("\n") }));
                }
                pi_types::message::Message::Assistant(a) => {
                    let texts: Vec<_> = a
                        .content
                        .iter()
                        .filter_map(|b| match b {
                            pi_types::message::ContentBlock::Text(t) => Some(t.text.as_str()),
                            _ => None,
                        })
                        .collect::<Vec<_>>();
                    input.push(json!({ "role": "assistant", "content": texts.join("\n") }));
                }
                pi_types::message::Message::ToolResult(t) => {
                    input.push(json!({ "role": "user", "content": t.content }));
                }
            }
        }

        let mut body = json!({
            "model": request.model,
            "input": input,
            "stream": true,
        });

        if let Some(sp) = &request.system_prompt {
            body["instructions"] = json!(sp);
        }

        if !request.tools.is_empty() {
            let tools: Vec<_> = request
                .tools
                .iter()
                .map(|t| {
                    json!({
                        "type": "function",
                        "function": {
                            "name": t.name,
                            "description": t.description,
                            "parameters": t.parameters,
                        }
                    })
                })
                .collect();
            body["tools"] = json!(tools);
        }

        let api_key = request.api_key.clone();

        let response_future = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {api_key}"))
            .header("content-type", "application/json")
            .json(&body)
            .send();

        let stream = async_stream::stream! {
            let response = match response_future.await {
                Ok(r) => r,
                Err(e) => {
                    yield Err(anyhow::anyhow!("OpenAI Responses request failed: {e}"));
                    return;
                }
            };

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                yield Err(anyhow::anyhow!("OpenAI Responses error {status}: {body}"));
                return;
            }

            // SSE 格式与 chat completions 相同，复用解析器
            let mut stream = response.bytes_stream();
            let mut buffer = String::new();

            while let Some(chunk_result) = stream.next().await {
                let chunk = match chunk_result {
                    Ok(c) => c,
                    Err(e) => {
                        yield Err(anyhow::anyhow!("Responses stream error: {e}"));
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
        let driver = OpenAiResponsesDriver::new();
        assert_eq!(driver.name(), "openai-responses");
    }

    #[test]
    fn default_impl() {
        let driver = OpenAiResponsesDriver::default();
        assert_eq!(driver.name(), "openai-responses");
    }
}
