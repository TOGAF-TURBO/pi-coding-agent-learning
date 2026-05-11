//! Google Vertex AI driver — 使用 Gemini API 格式 + Service Account 认证。
//!
//! Vertex AI 使用 Google Cloud service account 而非 API key。
//! 环境变量：GOOGLE_APPLICATION_CREDENTIALS, GOOGLE_CLOUD_PROJECT, GOOGLE_CLOUD_LOCATION

use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;
use std::path::PathBuf;

use crate::driver::{CompletionRequest, LlmDriver, StreamResult};
use crate::gemini::GeminiDriver;

/// Google Vertex AI driver。
///
/// 复用 Gemini driver 的 SSE 解析，但使用 Vertex AI 的 URL 格式和认证。
/// URL: `https://{location}-aiplatform.googleapis.com/v1/projects/{project}/locations/{location}/publishers/google/models/{model}:streamGenerateContent`
pub struct VertexDriver {
    inner: GeminiDriver,
    client: Client,
}

impl VertexDriver {
    pub fn new() -> Self {
        Self {
            inner: GeminiDriver::new(),
            client: Client::new(),
        }
    }

    fn project() -> Option<String> {
        std::env::var("GOOGLE_CLOUD_PROJECT")
            .or_else(|_| std::env::var("GCLOUD_PROJECT"))
            .ok()
    }

    fn location() -> String {
        std::env::var("GOOGLE_CLOUD_LOCATION")
            .or_else(|_| std::env::var("GCLOUD_LOCATION"))
            .unwrap_or_else(|_| "us-central1".to_string())
    }

    fn credentials_path() -> Option<PathBuf> {
        std::env::var("GOOGLE_APPLICATION_CREDENTIALS")
            .ok()
            .map(PathBuf::from)
    }

    fn build_url(project: &str, location: &str, model: &str) -> String {
        format!(
            "https://{}-aiplatform.googleapis.com/v1/projects/{}/locations/{}/publishers/google/models/{}:streamGenerateContent?alt=sse",
            location, project, location, model
        )
    }

    /// 读取 service account token（简化版 — 生产环境应使用 gcloud auth）。
    async fn get_access_token(&self) -> Option<String> {
        // 尝试从 gcloud CLI 获取 token
        let output = tokio::process::Command::new("gcloud")
            .args(["auth", "print-access-token"])
            .output()
            .await
            .ok()?;

        if output.status.success() {
            let token = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !token.is_empty() {
                return Some(token);
            }
        }
        None
    }
}

impl Default for VertexDriver {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LlmDriver for VertexDriver {
    fn name(&self) -> &str {
        "google-vertex"
    }

    fn stream(&self, request: CompletionRequest) -> Result<StreamResult, anyhow::Error> {
        let project =
            Self::project().ok_or_else(|| anyhow::anyhow!("GOOGLE_CLOUD_PROJECT not set"))?;
        let location = Self::location();
        let url = Self::build_url(&project, &location, &request.model);

        // 构建 Gemini 格式的请求体
        let mut contents = Vec::new();
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
                    if !texts.is_empty() {
                        contents.push(json!({
                            "role": "user",
                            "parts": texts.iter().map(|t| json!({"text": t})).collect::<Vec<_>>()
                        }));
                    }
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
                    if !texts.is_empty() {
                        contents.push(json!({
                            "role": "model",
                            "parts": texts.iter().map(|t| json!({"text": t})).collect::<Vec<_>>()
                        }));
                    }
                }
                pi_types::message::Message::ToolResult(_) => {}
            }
        }

        let mut body = json!({
            "contents": contents,
        });

        if let Some(sp) = &request.system_prompt {
            body["systemInstruction"] = json!({
                "parts": [{"text": sp}]
            });
        }

        if !request.tools.is_empty() {
            let decls: Vec<_> = request
                .tools
                .iter()
                .map(|t| {
                    json!({
                        "name": t.name,
                        "description": t.description,
                        "parameters": t.parameters,
                    })
                })
                .collect();
            body["tools"] = json!([{"function_declarations": decls}]);
        }

        let api_key = request.api_key.clone();
        let response_future = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {api_key}"))
            .header("Content-Type", "application/json")
            .json(&body)
            .send();

        // 复用 Gemini SSE 解析
        let stream = async_stream::stream! {
            let response = match response_future.await {
                Ok(r) => r,
                Err(e) => {
                    yield Err(anyhow::anyhow!("Vertex AI request failed: {e}"));
                    return;
                }
            };

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                yield Err(anyhow::anyhow!("Vertex AI error {status}: {body}"));
                return;
            }

            // Gemini SSE 格式解析
            use futures::StreamExt;
            let mut stream = response.bytes_stream();
            let mut buffer = String::new();

            while let Some(chunk_result) = stream.next().await {
                let chunk = match chunk_result {
                    Ok(c) => c,
                    Err(e) => {
                        yield Err(anyhow::anyhow!("Vertex stream error: {e}"));
                        return;
                    }
                };

                buffer.push_str(&String::from_utf8_lossy(&chunk));

                while let Some(line_end) = buffer.find('\n') {
                    let line = buffer[..line_end].trim().to_string();
                    buffer = buffer[line_end + 1..].to_string();

                    if !line.starts_with("data: ") { continue; }
                    let data = &line[6..];
                    if data == "[DONE]" { return; }

                    if let Ok(chunk_val) = serde_json::from_str::<serde_json::Value>(data) {
                        // Gemini 格式：candidates[0].content.parts[0].text
                        if let Some(candidates) = chunk_val.get("candidates").and_then(|c| c.as_array()) {
                            for candidate in candidates {
                                if let Some(parts) = candidate.pointer("/content/parts").and_then(|p| p.as_array()) {
                                    for part in parts {
                                        if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                                            yield Ok(crate::driver::StreamEvent::TextDelta { text: text.to_string() });
                                        }
                                        if let Some(thinking) = part.get("thought").and_then(|t| t.as_str()) {
                                            yield Ok(crate::driver::StreamEvent::ThinkingDelta { thinking: thinking.to_string() });
                                        }
                                    }
                                }
                                if let Some(reason) = candidate.pointer("/finishReason").and_then(|r| r.as_str()) {
                                    if reason == "STOP" {
                                        yield Ok(crate::driver::StreamEvent::Stop { reason: None });
                                        return;
                                    }
                                }
                            }
                        }
                        // Usage metadata
                        if let Some(usage) = chunk_val.get("usageMetadata") {
                            let input = usage.get("promptTokenCount").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                            let output = usage.get("candidatesTokenCount").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                            yield Ok(crate::driver::StreamEvent::Usage(pi_types::message::Usage {
                                input_tokens: input,
                                output_tokens: output,
                                cache_creation_input_tokens: None,
                                cache_read_input_tokens: None,
                            }));
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
        let driver = VertexDriver::new();
        assert_eq!(driver.name(), "google-vertex");
    }

    #[test]
    fn default_impl() {
        let driver = VertexDriver::default();
        assert_eq!(driver.name(), "google-vertex");
    }

    #[test]
    fn build_url_format() {
        let url = VertexDriver::build_url("my-project", "us-central1", "gemini-2.0-flash");
        assert!(url.contains("us-central1-aiplatform.googleapis.com"));
        assert!(url.contains("my-project"));
        assert!(url.contains("gemini-2.0-flash"));
        assert!(url.contains("streamGenerateContent"));
    }

    #[test]
    fn location_default() {
        // Without env var, should default to us-central1
        let loc = std::env::var("GOOGLE_CLOUD_LOCATION")
            .or_else(|_| std::env::var("GCLOUD_LOCATION"))
            .unwrap_or_else(|_| "us-central1".to_string());
        assert_eq!(loc, "us-central1");
    }
}
