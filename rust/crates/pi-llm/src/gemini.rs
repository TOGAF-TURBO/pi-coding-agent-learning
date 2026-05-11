//! Google Gemini API 驱动。
//!
//! Google Gemini 使用独立的 REST API，与 OpenAI/Anthropic 都不同。
//! 通过 HTTP 直接调用 Gemini `generateContent` 端点。
//!
//! 关键差异：
//! - 端点：`/v1beta/models/{model}:streamGenerateContent`
//! - 认证：`x-goog-api-key` header（不是 Bearer token）
//! - 工具调用在 `functionCall` 中
//! - SSE 不是标准格式，而是 JSON 数组分块传输

use anyhow::{Result, anyhow};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::{json, Value};

use crate::driver::{CompletionRequest, LlmDriver, StreamEvent, StreamResult};

const GEMINI_API_URL: &str = "https://generativelanguage.googleapis.com";

/// Google Gemini API 驱动。
pub struct GeminiDriver {
    client: Client,
}

impl GeminiDriver {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

#[async_trait]
impl LlmDriver for GeminiDriver {
    fn stream(&self, request: CompletionRequest) -> Result<StreamResult> {
        let base_url = request
            .base_url
            .clone()
            .unwrap_or_else(|| GEMINI_API_URL.to_string());

        let body = build_gemini_request(&request);
        let api_key = request.api_key.clone();
        let model = request.model.clone();

        let url = format!(
            "{}/v1beta/models/{}:streamGenerateContent?alt=sse",
            base_url.trim_end_matches('/'),
            model
        );

        let response_future = self.client
            .post(&url)
            .header("x-goog-api-key", &api_key)
            .header("content-type", "application/json")
            .json(&body)
            .send();

        let stream = async_stream::stream! {
            let response = match response_future.await {
                Ok(r) => r,
                Err(e) => {
                    yield Err(anyhow!("HTTP request failed: {e}"));
                    return;
                }
            };

            let status = response.status();
            if !status.is_success() {
                let body = response.text().await.unwrap_or_default();
                yield Err(anyhow!("Gemini API error {status}: {body}"));
                return;
            }

            yield Ok(StreamEvent::Start);

            let mut tool_calls: Vec<GeminiToolCall> = Vec::new();
            let byte_stream = response.bytes_stream();
            let mut parser = super::openai::OpenAiSseParser::new();

            for await chunk in byte_stream {
                let chunk = match chunk {
                    Ok(c) => c,
                    Err(e) => {
                        yield Err(anyhow!("Stream read error: {e}"));
                        return;
                    }
                };

                parser.feed(&chunk);
                while let Some(event) = parser.next_event() {
                    match event {
                        super::openai::SseEvent::Data(data) => {
                            if data == "[DONE]" {
                                // flush tool calls
                                for (i, tc) in std::mem::take(&mut tool_calls).into_iter().enumerate() {
                                    let input: Value = serde_json::from_str(&tc.args_json).unwrap_or(Value::Null);
                                    yield Ok(StreamEvent::ToolCallEnd {
                                        index: i,
                                        id: tc.id,
                                        name: tc.name,
                                        input,
                                    });
                                }
                                yield Ok(StreamEvent::Stop { reason: None });
                                continue;
                            }

                            match serde_json::from_str::<Value>(&data) {
                                Ok(chunk_val) => {
                                    if let Some(events) = parse_gemini_chunk(&chunk_val, &mut tool_calls) {
                                        for ev in events {
                                            yield Ok(ev);
                                        }
                                    }
                                }
                                Err(_) => continue,
                            }
                        }
                        _ => {}
                    }
                }
            }
        };

        Ok(Box::pin(stream))
    }

    fn name(&self) -> &str {
        "google"
    }
}

/// Gemini tool call accumulator.
struct GeminiToolCall {
    id: String,
    name: String,
    args_json: String,
}

/// Parse a Gemini SSE chunk.
fn parse_gemini_chunk(
    chunk: &Value,
    tool_calls: &mut Vec<GeminiToolCall>,
) -> Option<Vec<StreamEvent>> {
    let mut events = Vec::new();

    let candidates = chunk.get("candidates")?.as_array()?;
    if candidates.is_empty() {
        return None;
    }

    let candidate = &candidates[0];

    // Finish reason
    if let Some(reason) = candidate.get("finishReason").and_then(|v| v.as_str()) {
        let stop = match reason {
            "STOP" => Some(pi_types::message::StopReason::Stop),
            "MAX_TOKENS" => Some(pi_types::message::StopReason::Length),
            "SAFETY" => Some(pi_types::message::StopReason::Stop),
            _ => None,
        };
        if let Some(s) = stop {
            events.push(StreamEvent::Stop { reason: Some(s) });
        }
    }

    // Usage metadata
    if let Some(usage) = chunk.get("usageMetadata") {
        events.push(StreamEvent::Usage(pi_types::message::Usage {
            input_tokens: usage.get("promptTokenCount").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
            output_tokens: usage.get("candidatesTokenCount").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
            cache_read_input_tokens: usage.get("cachedContentTokenCount").and_then(|v| v.as_u64()).map(|v| v as u32),
            ..Default::default()
        }));
    }

    // Content parts
    if let Some(content) = candidate.get("content") {
        if let Some(parts) = content.get("parts").and_then(|p| p.as_array()) {
            for part in parts {
                // Text
                if let Some(text) = part.get("text").and_then(|v| v.as_str()) {
                    events.push(StreamEvent::TextDelta { text: text.to_string() });
                }

                // Thinking
                if let Some(thought) = part.get("thought").and_then(|v| v.as_bool()) {
                    if thought {
                        if let Some(text) = part.get("text").and_then(|v| v.as_str()) {
                            events.push(StreamEvent::ThinkingDelta { thinking: text.to_string() });
                        }
                    }
                }

                // Function call
                if let Some(fc) = part.get("functionCall") {
                    let name = fc.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let args = fc.get("args").cloned().unwrap_or(Value::Null);
                    let id = format!("gc_{}", tool_calls.len());
                    let args_json = serde_json::to_string(&args).unwrap_or_default();

                    tool_calls.push(GeminiToolCall {
                        id: id.clone(),
                        name: name.clone(),
                        args_json: args_json.clone(),
                    });

                    events.push(StreamEvent::ToolCallEnd {
                        index: tool_calls.len() - 1,
                        id,
                        name,
                        input: args,
                    });
                }
            }
        }
    }

    if events.is_empty() {
        None
    } else {
        Some(events)
    }
}

/// Build Gemini API request body.
fn build_gemini_request(req: &CompletionRequest) -> Value {
    let mut contents = Vec::new();

    for msg in &req.messages {
        match msg {
            pi_types::message::Message::User(u) => {
                let parts: Vec<Value> = u.content.iter().map(|block| {
                    match block {
                        pi_types::message::ContentBlock::Text(t) => json!({"text": t.text}),
                        pi_types::message::ContentBlock::ToolResult(r) => json!({
                            "functionResponse": {
                                "name": r.tool_use_id,
                                "response": {
                                    "content": r.content,
                                }
                            }
                        }),
                        _ => json!({"text": "[unsupported]"}),
                    }
                }).collect();
                contents.push(json!({"role": "user", "parts": parts}));
            }
            pi_types::message::Message::Assistant(a) => {
                let parts: Vec<Value> = a.content.iter().map(|block| {
                    match block {
                        pi_types::message::ContentBlock::Text(t) => json!({"text": t.text}),
                        pi_types::message::ContentBlock::ToolUse(tc) => json!({
                            "functionCall": {
                                "name": tc.name,
                                "args": tc.input,
                            }
                        }),
                        pi_types::message::ContentBlock::Thinking(t) => {
                            json!({"thought": true, "text": t.thinking})
                        }
                        _ => json!({"text": ""}),
                    }
                }).collect();
                contents.push(json!({"role": "model", "parts": parts}));
            }
            pi_types::message::Message::ToolResult(tr) => {
                let parts: Vec<Value> = tr.content.iter().map(|block| {
                    if let pi_types::message::ContentBlock::ToolResult(r) = block {
                        json!({
                            "functionResponse": {
                                "name": r.tool_use_id,
                                "response": {"content": r.content}
                            }
                        })
                    } else {
                        json!({"text": "[unsupported]"})
                    }
                }).collect();
                contents.push(json!({"role": "user", "parts": parts}));
            }
        }
    }

    let mut body = json!({
        "contents": contents,
        "generationConfig": {
            "maxOutputTokens": req.max_tokens,
        }
    });

    // System instruction
    if let Some(system) = &req.system_prompt {
        body["systemInstruction"] = json!({"parts": [{"text": system}]});
    }

    // Tools
    if !req.tools.is_empty() {
        let declarations: Vec<Value> = req.tools.iter().map(|t| {
            json!({
                "name": t.name,
                "description": t.description,
                "parameters": t.parameters,
            })
        }).collect();
        body["tools"] = json!([{"functionDeclarations": declarations}]);
    }

    body
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_gemini_request_basic() {
        let req = CompletionRequest {
            model: "gemini-2.5-pro".to_string(),
            system_prompt: Some("You are helpful".to_string()),
            messages: vec![pi_types::message::Message::user_text("hello")],
            tools: vec![],
            thinking_enabled: false,
            thinking_budget: None,
            max_tokens: 4096,
            api_key: String::new(),
            base_url: None,
        };
        let body = build_gemini_request(&req);
        assert!(body.get("contents").unwrap().as_array().unwrap().len() == 1);
        assert_eq!(body["contents"][0]["role"], "user");
        assert!(body.get("systemInstruction").is_some());
    }

    #[test]
    fn build_gemini_request_with_tools() {
        let req = CompletionRequest {
            model: "gemini-2.5-pro".to_string(),
            system_prompt: None,
            messages: vec![],
            tools: vec![pi_types::tool::ToolDefinition {
                name: "bash".to_string(),
                description: "Execute a shell command".to_string(),
                parameters: serde_json::json!({"type": "object", "properties": {"command": {"type": "string"}}, "required": ["command"]}),
                requires_approval: false,
            }],
            thinking_enabled: false,
            thinking_budget: None,
            max_tokens: 4096,
            api_key: String::new(),
            base_url: None,
        };
        let body = build_gemini_request(&req);
        let tools = body["tools"][0]["functionDeclarations"].as_array().unwrap();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0]["name"], "bash");
    }

    #[test]
    fn parse_gemini_text_chunk() {
        let chunk = json!({
            "candidates": [{
                "content": {
                    "parts": [{"text": "hello"}],
                    "role": "model"
                }
            }]
        });
        let mut tc = Vec::new();
        let events = parse_gemini_chunk(&chunk, &mut tc).unwrap();
        assert!(events.iter().any(|e| matches!(e, StreamEvent::TextDelta { text } if text == "hello")));
    }
}
