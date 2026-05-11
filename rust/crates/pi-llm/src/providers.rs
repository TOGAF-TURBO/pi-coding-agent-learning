//! Anthropic Messages API 驱动。
//!
//! 对应 `packages/ai/src/providers/anthropic.ts`。
//!
//! 通过 HTTP 直接调用 Anthropic Messages API，使用 SSE 解析流式响应。
//! 不依赖 Anthropic SDK，保持最小依赖。

use anyhow::{Context, Result, anyhow};
use async_trait::async_trait;
use futures::{Stream, StreamExt};
use reqwest::Client;
use serde_json::{json, Value};

use crate::driver::{CompletionRequest, LlmDriver, StreamEvent, StreamResult};

const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_VERSION: &str = "2023-06-01";

/// Anthropic Messages API 驱动。
pub struct AnthropicDriver {
    client: Client,
}

impl AnthropicDriver {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

#[async_trait]
impl LlmDriver for AnthropicDriver {
    fn stream(&self, request: CompletionRequest) -> Result<StreamResult, anyhow::Error> {
        let base_url = request
            .base_url
            .clone()
            .unwrap_or_else(|| ANTHROPIC_API_URL.to_string());

        let body = build_anthropic_request(&request);
        let api_key = request.api_key.clone();

        let response_future = self.client.post(&base_url)
            .header("x-api-key", &api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
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
                yield Err(anyhow!("Anthropic API error {status}: {body}"));
                return;
            }

            yield Ok(StreamEvent::Start);

            let mut tool_calls: Vec<ToolCallAccumulator> = Vec::new();
            let mut current_block_type: Option<String> = None;
            let mut current_block_index: usize = 0;
            let mut usage: Option<pi_types::message::Usage> = None;

            let byte_stream = response.bytes_stream();
            let mut lines = SseLineParser::new();

            for await chunk in byte_stream {
                let chunk = match chunk {
                    Ok(c) => c,
                    Err(e) => {
                        yield Err(anyhow!("Stream read error: {e}"));
                        return;
                    }
                };

                lines.feed(&chunk);
                while let Some((event_type, data)) = lines.next_event() {
                    match event_type.as_str() {
                        "message_start" => {
                            if let Ok(msg) = serde_json::from_str::<Value>(&data) {
                                if let Some(u) = msg.get("message").and_then(|m| m.get("usage")) {
                                    usage = Some(pi_types::message::Usage {
                                        input_tokens: u.get("input_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                                        output_tokens: u.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                                        cache_creation_input_tokens: u.get("cache_creation_input_tokens").and_then(|v| v.as_u64()).map(|v| v as u32),
                                        cache_read_input_tokens: u.get("cache_read_input_tokens").and_then(|v| v.as_u64()).map(|v| v as u32),
                                    });
                                }
                            }
                        }
                        "content_block_start" => {
                            if let Ok(block) = serde_json::from_str::<Value>(&data) {
                                current_block_index = block.get("index").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                                if let Some(cb) = block.get("content_block") {
                                    current_block_type = cb.get("type").and_then(|v| v.as_str()).map(|s| s.to_string());
                                    match current_block_type.as_deref() {
                                        Some("tool_use") => {
                                            let id = cb.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                            let name = cb.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                            while tool_calls.len() <= current_block_index {
                                                tool_calls.push(ToolCallAccumulator::default());
                                            }
                                            tool_calls[current_block_index] = ToolCallAccumulator {
                                                id,
                                                name,
                                                input_json: String::new(),
                                            };
                                            yield Ok(StreamEvent::ToolCallStart {
                                                id: tool_calls[current_block_index].id.clone(),
                                                name: tool_calls[current_block_index].name.clone(),
                                                index: current_block_index,
                                            });
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                        "content_block_delta" => {
                            if let Ok(delta) = serde_json::from_str::<Value>(&data) {
                                if let Some(d) = delta.get("delta") {
                                    let dtype = d.get("type").and_then(|v| v.as_str()).unwrap_or("");
                                    match dtype {
                                        "text_delta" => {
                                            let text = d.get("text").and_then(|v| v.as_str()).unwrap_or("");
                                            yield Ok(StreamEvent::TextDelta { text: text.to_string() });
                                        }
                                        "thinking_delta" => {
                                            let thinking = d.get("thinking").and_then(|v| v.as_str()).unwrap_or("");
                                            yield Ok(StreamEvent::ThinkingDelta { thinking: thinking.to_string() });
                                        }
                                        "input_json_delta" => {
                                            let input = d.get("partial_json").and_then(|v| v.as_str()).unwrap_or("");
                                            let idx = delta.get("index").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                                            if idx < tool_calls.len() {
                                                tool_calls[idx].input_json.push_str(input);
                                            }
                                            yield Ok(StreamEvent::ToolCallDelta {
                                                index: idx,
                                                input: input.to_string(),
                                            });
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                        "content_block_stop" => {
                            if current_block_type.as_deref() == Some("tool_use") {
                                yield Ok(StreamEvent::ToolCallEnd { index: current_block_index });
                            }
                        }
                        "message_delta" => {
                            if let Ok(delta) = serde_json::from_str::<Value>(&data) {
                                if let Some(u) = delta.get("usage") {
                                    let out = u.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                                    if let Some(ref mut us) = usage {
                                        us.output_tokens += out;
                                    }
                                }
                                let stop = delta.get("delta")
                                    .and_then(|d| d.get("stop_reason"))
                                    .and_then(|v| v.as_str());
                                let reason = match stop {
                                    Some("end_turn") | Some("stop") => Some(pi_types::message::StopReason::Stop),
                                    Some("tool_use") => Some(pi_types::message::StopReason::ToolUse),
                                    Some("max_tokens") => Some(pi_types::message::StopReason::Length),
                                    _ => None,
                                };
                                if let Some(u) = usage.take() {
                                    yield Ok(StreamEvent::Usage(u));
                                }
                                yield Ok(StreamEvent::Stop { reason });
                            }
                        }
                        "error" => {
                            let msg = serde_json::from_str::<Value>(&data)
                                .ok()
                                .and_then(|v| v.get("error").and_then(|e| e.get("message")).and_then(|m| m.as_str()).map(|s| s.to_string()))
                                .unwrap_or_else(|| data.clone());
                            yield Ok(StreamEvent::Error { message: msg });
                        }
                        _ => {}
                    }
                }
            }
        };

        Ok(Box::pin(stream))
    }

    fn name(&self) -> &str {
        "anthropic"
    }
}

#[derive(Default)]
struct ToolCallAccumulator {
    id: String,
    name: String,
    input_json: String,
}

/// 构建 Anthropic Messages API 请求体。
fn build_anthropic_request(req: &CompletionRequest) -> Value {
    let mut messages = Vec::new();

    for msg in &req.messages {
        match msg {
            pi_types::message::Message::User(u) => {
                let content: Vec<Value> = u.content.iter().map(|block| {
                    match block {
                        pi_types::message::ContentBlock::Text(t) => json!({"type": "text", "text": t.text}),
                        pi_types::message::ContentBlock::Image(img) => json!({
                            "type": "image",
                            "source": {
                                "type": "base64",
                                "media_type": img.media_type,
                                "data": img.data,
                            }
                        }),
                        _ => json!({"type": "text", "text": "[unsupported]"}),
                    }
                }).collect();
                messages.push(json!({"role": "user", "content": content}));
            }
            pi_types::message::Message::Assistant(a) => {
                let content: Vec<Value> = a.content.iter().map(|block| {
                    match block {
                        pi_types::message::ContentBlock::Text(t) => json!({"type": "text", "text": t.text}),
                        pi_types::message::ContentBlock::Thinking(t) => json!({"type": "thinking", "thinking": t.thinking}),
                        pi_types::message::ContentBlock::ToolUse(tc) => json!({
                            "type": "tool_use",
                            "id": tc.id,
                            "name": tc.name,
                            "input": tc.input,
                        }),
                        _ => json!({"type": "text", "text": "[unsupported]"}),
                    }
                }).collect();
                messages.push(json!({"role": "assistant", "content": content}));
            }
            pi_types::message::Message::ToolResult(tr) => {
                let content: Vec<Value> = tr.content.iter().map(|block| {
                    match block {
                        pi_types::message::ContentBlock::ToolResult(r) => json!({
                            "type": "tool_result",
                            "tool_use_id": r.tool_use_id,
                            "content": r.content,
                            "is_error": r.is_error,
                        }),
                        _ => json!({"type": "text", "text": "[unsupported]"}),
                    }
                }).collect();
                messages.push(json!({"role": "user", "content": content}));
            }
        }
    }

    let mut body = json!({
        "model": req.model,
        "max_tokens": req.max_tokens,
        "messages": messages,
        "stream": true,
    });

    if let Some(system) = &req.system_prompt {
        body["system"] = json!(system);
    }

    if !req.tools.is_empty() {
        let tools: Vec<Value> = req.tools.iter().map(|t| {
            json!({
                "name": t.name,
                "description": t.description,
                "input_schema": t.parameters,
            })
        }).collect();
        body["tools"] = json!(tools);
    }

    if req.thinking_enabled {
        let budget = req.thinking_budget.unwrap_or(10000);
        body["thinking"] = json!({
            "type": "enabled",
            "budget_tokens": budget,
        });
    }

    body
}

/// SSE 行解析器 — 从 HTTP byte stream 中提取 event/data 对。
struct SseLineParser {
    buffer: String,
}

impl SseLineParser {
    fn new() -> Self {
        Self {
            buffer: String::new(),
        }
    }

    fn feed(&mut self, bytes: &[u8]) {
        self.buffer
            .push_str(&String::from_utf8_lossy(bytes));
    }

    fn next_event(&mut self) -> Option<(String, String)> {
        loop {
            // Look for double newline (end of SSE event)
            let event_end = self.buffer.find("\n\n")?;
            let event_text = self.buffer[..event_end].to_string();
            self.buffer = self.buffer[event_end + 2..].to_string();

            let mut event_type = String::from("message");
            let mut data = String::new();

            for line in event_text.lines() {
                if let Some(t) = line.strip_prefix("event: ") {
                    event_type = t.trim().to_string();
                } else if let Some(d) = line.strip_prefix("data: ") {
                    data = d.trim().to_string();
                } else if let Some(d) = line.strip_prefix("data:") {
                    data = d.trim().to_string();
                }
            }

            if !data.is_empty() {
                return Some((event_type, data));
            }
        }
    }
}
