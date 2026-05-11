//! OpenAI Chat Completions API 驱动。
//!
//! 覆盖 30+ 兼容 provider：OpenAI、GLM、DeepSeek、Groq、OpenRouter 等。
//! 所有使用 `/chat/completions` 端点的 provider 共享此驱动。
//!
//! 与 Anthropic driver 的关键差异：
//! - SSE `data:` 行是完整 JSON（不是 event + data 分离）
//! - 工具调用在 `delta.tool_calls` 数组中增量到达
//! - 思考内容在 `delta.reasoning_content` 或 `delta.reasoning` 中
//! - 停止信号是 `data: [DONE]`

use anyhow::{Result, anyhow};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::{json, Value};

use crate::driver::{CompletionRequest, LlmDriver, StreamEvent, StreamResult};

const OPENAI_API_URL: &str = "https://api.openai.com/v1/chat/completions";

/// 累积的 tool call 状态。
#[derive(Default)]
struct OpenAiToolCall {
    id: String,
    name: String,
    arguments_json: String,
}

/// 从 delta 块中累积 tool call 数据。
fn accumulate_tool_calls(tool_calls: &mut Vec<OpenAiToolCall>, delta: &Value) {
    let Some(tcs) = delta.get("tool_calls").and_then(|v| v.as_array()) else {
        return;
    };
    for tc in tcs {
        let index = tc.get("index").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
        while tool_calls.len() <= index {
            tool_calls.push(OpenAiToolCall::default());
        }
        if let Some(id) = tc.get("id").and_then(|v| v.as_str()) {
            tool_calls[index].id = id.to_string();
        }
        if let Some(func) = tc.get("function") {
            if let Some(name) = func.get("name").and_then(|v| v.as_str()) {
                tool_calls[index].name = name.to_string();
            }
            if let Some(args) = func.get("arguments").and_then(|v| v.as_str()) {
                tool_calls[index].arguments_json.push_str(args);
            }
        }
    }
}

/// 将累积的 tool calls 转为 (index, id, name, input) 列表。
fn take_tool_calls(tool_calls: &mut Vec<OpenAiToolCall>) -> Vec<(usize, String, String, Value)> {
    let calls = std::mem::take(tool_calls);
    calls.into_iter().enumerate()
        .filter(|(_, tc)| !tc.id.is_empty() || !tc.name.is_empty())
        .map(|(i, tc)| {
            let input: Value = serde_json::from_str(&tc.arguments_json).unwrap_or(Value::Null);
            (i, tc.id, tc.name, input)
        })
        .collect()
}

/// OpenAI Chat Completions API 驱动。
pub struct OpenAiDriver {
    client: Client,
}

impl OpenAiDriver {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

#[async_trait]
impl LlmDriver for OpenAiDriver {
    fn stream(&self, request: CompletionRequest) -> Result<StreamResult> {
        let base_url = request
            .base_url
            .clone()
            .unwrap_or_else(|| OPENAI_API_URL.to_string());

        let body = build_openai_request(&request);
        let api_key = request.api_key.clone();

        let response_future = self.client
            .post(&base_url)
            .header("authorization", format!("Bearer {api_key}"))
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
                yield Err(anyhow!("OpenAI API error {status}: {body}"));
                return;
            }

            yield Ok(StreamEvent::Start);

            let mut tool_calls: Vec<OpenAiToolCall> = Vec::new();
            let byte_stream = response.bytes_stream();
            let mut parser = OpenAiSseParser::new();

            for await chunk in byte_stream {
                let chunk = match chunk {
                    Ok(c) => c,
                    Err(e) => {
                        // 流中断：flush 未结束的 tool calls
                        for (i, id, name, input) in take_tool_calls(&mut tool_calls) {
                            yield Ok(StreamEvent::ToolCallEnd { index: i, id, name, input });
                        }
                        yield Err(anyhow!("Stream read error: {e}"));
                        return;
                    }
                };

                parser.feed(&chunk);
                while let Some(event) = parser.next_event() {
                    match event {
                        SseEvent::Data(data) => {
                            if data == "[DONE]" {
                                for (i, id, name, input) in take_tool_calls(&mut tool_calls) {
                                    yield Ok(StreamEvent::ToolCallEnd { index: i, id, name, input });
                                }
                                yield Ok(StreamEvent::Stop { reason: None });
                                continue;
                            }

                            match serde_json::from_str::<Value>(&data) {
                                Ok(chunk_val) => {
                                    let delta = chunk_val.get("choices")
                                        .and_then(|c| c.as_array())
                                        .and_then(|a| a.first())
                                        .and_then(|c| c.get("delta"));

                                    // 累积 tool call 数据（不 emit）
                                    if let Some(d) = &delta {
                                        accumulate_tool_calls(&mut tool_calls, d);
                                    }

                                    // 检查 finish_reason
                                    let finish = chunk_val.get("choices")
                                        .and_then(|c| c.as_array())
                                        .and_then(|a| a.first())
                                        .and_then(|c| c.get("finish_reason"))
                                        .and_then(|v| v.as_str());

                                    if finish == Some("tool_calls") {
                                        // flush 所有累积的 tool calls
                                        for (i, id, name, input) in take_tool_calls(&mut tool_calls) {
                                            yield Ok(StreamEvent::ToolCallEnd { index: i, id, name, input });
                                        }
                                    }

                                    // emit 其他事件（text、thinking、usage、stop）
                                    if let Some(events) = parse_openai_events(&chunk_val) {
                                        for ev in events {
                                            yield Ok(ev);
                                        }
                                    }
                                }
                                Err(e) => {
                                    tracing::debug!("Skipping unparseable SSE data: {e}");
                                }
                            }
                        }
                        SseEvent::Error(msg) => {
                            yield Ok(StreamEvent::Error { message: msg });
                        }
                    }
                }
            }

            // 安全 flush
            for (i, id, name, input) in take_tool_calls(&mut tool_calls) {
                yield Ok(StreamEvent::ToolCallEnd { index: i, id, name, input });
            }
        };

        Ok(Box::pin(stream))
    }

    fn name(&self) -> &str {
        "openai"
    }
}

/// 解析 SSE chunk，只返回 text/thinking/usage/stop 事件（不含 tool call）。
fn parse_openai_events(chunk: &Value) -> Option<Vec<StreamEvent>> {
    let mut events = Vec::new();

    let choices = chunk.get("choices")?.as_array()?;
    if choices.is_empty() {
        return None;
    }

    let choice = &choices[0];

    // 用量
    if let Some(usage) = chunk.get("usage") {
        events.push(StreamEvent::Usage(pi_types::message::Usage {
            input_tokens: usage.get("prompt_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
            output_tokens: usage.get("completion_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
            ..Default::default()
        }));
    }

    // 结束原因
    if let Some(finish) = choice.get("finish_reason").and_then(|v| v.as_str()) {
        let reason = match finish {
            "stop" => Some(pi_types::message::StopReason::Stop),
            "tool_calls" => Some(pi_types::message::StopReason::ToolUse),
            "length" => Some(pi_types::message::StopReason::Length),
            "content_filter" => Some(pi_types::message::StopReason::Stop),
            _ => None,
        };
        events.push(StreamEvent::Stop { reason });
    }

    if let Some(delta) = choice.get("delta") {
        // 文本内容
        if let Some(content) = delta.get("content").and_then(|v| v.as_str()) {
            if !content.is_empty() {
                events.push(StreamEvent::TextDelta { text: content.to_string() });
            }
        }

        // 思考内容（DeepSeek / GLM 风格）
        for field in &["reasoning_content", "reasoning"] {
            if let Some(thinking) = delta.get(field).and_then(|v| v.as_str()) {
                if !thinking.is_empty() {
                    events.push(StreamEvent::ThinkingDelta { thinking: thinking.to_string() });
                }
            }
        }
        // 注意：tool_calls 事件由 accumulate + flush 处理，不在此处 emit
    }

    if events.is_empty() {
        None
    } else {
        Some(events)
    }
}

/// 构建 OpenAI Chat Completions API 请求体。
fn build_openai_request(req: &CompletionRequest) -> Value {
    let mut messages = Vec::new();

    // 系统提示
    if let Some(system) = &req.system_prompt {
        messages.push(json!({
            "role": "system",
            "content": system,
        }));
    }

    for msg in &req.messages {
        match msg {
            pi_types::message::Message::User(u) => {
                // OpenAI 要求 tool_result 作为独立的 role: "tool" 消息
                // 而不是放在 user 消息的 content 中
                let mut texts: Vec<Value> = Vec::new();
                for block in &u.content {
                    match block {
                        pi_types::message::ContentBlock::Text(t) => {
                            texts.push(json!(t.text));
                        }
                        pi_types::message::ContentBlock::ToolResult(r) => {
                            // 先 flush 之前的文本
                            if !texts.is_empty() {
                                let text = texts.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join("");
                                messages.push(json!({"role": "user", "content": text}));
                                texts.clear();
                            }
                            // 作为独立的 tool 消息
                            messages.push(json!({
                                "role": "tool",
                                "tool_call_id": r.tool_use_id,
                                "content": r.content,
                            }));
                        }
                        pi_types::message::ContentBlock::Image(img) => {
                            texts.push(json!(format!("data:{};base64,{}", img.media_type, img.data)));
                        }
                        _ => {}
                    }
                }
                if !texts.is_empty() {
                    if texts.len() == 1 {
                        messages.push(json!({"role": "user", "content": texts[0]}));
                    } else {
                        messages.push(json!({"role": "user", "content": texts}));
                    }
                }
            }
            pi_types::message::Message::Assistant(a) => {
                let mut content_parts: Vec<Value> = Vec::new();
                let mut tool_calls: Vec<Value> = Vec::new();

                for block in &a.content {
                    match block {
                        pi_types::message::ContentBlock::Text(t) => {
                            content_parts.push(json!(t.text));
                        }
                        pi_types::message::ContentBlock::ToolUse(tc) => {
                            tool_calls.push(json!({
                                "id": tc.id,
                                "type": "function",
                                "function": {
                                    "name": tc.name,
                                    "arguments": serde_json::to_string(&tc.input).unwrap_or_else(|_| "{}".to_string()),
                                }
                            }));
                        }
                        _ => {}
                    }
                }

                let text = content_parts.into_iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect::<Vec<_>>()
                    .join("");

                let mut msg = json!({
                    "role": "assistant",
                    "content": if text.is_empty() { Value::Null } else { json!(text) },
                });

                if !tool_calls.is_empty() {
                    msg["tool_calls"] = json!(tool_calls);
                }

                messages.push(msg);
            }
            pi_types::message::Message::ToolResult(tr) => {
                for block in &tr.content {
                    if let pi_types::message::ContentBlock::ToolResult(r) = block {
                        messages.push(json!({
                            "role": "tool",
                            "tool_call_id": r.tool_use_id,
                            "content": r.content,
                        }));
                    }
                }
            }
        }
    }

    let mut body = json!({
        "model": req.model,
        "max_tokens": req.max_tokens,
        "messages": messages,
        "stream": true,
    });

    if !req.tools.is_empty() {
        let tools: Vec<Value> = req.tools.iter().map(|t| {
            json!({
                "type": "function",
                "function": {
                    "name": t.name,
                    "description": t.description,
                    "parameters": t.parameters,
                }
            })
        }).collect();
        body["tools"] = json!(tools);
    }

    body
}

// ─── SSE 解析器 ────────────────────────────────────────────────

/// OpenAI 风格的 SSE 事件。
enum SseEvent {
    /// `data: {...}` 行内容。
    Data(String),
    /// 解析错误。
    Error(String),
}

/// OpenAI SSE 解析器。
struct OpenAiSseParser {
    buffer: String,
}

impl OpenAiSseParser {
    fn new() -> Self {
        Self { buffer: String::new() }
    }

    fn feed(&mut self, bytes: &[u8]) {
        self.buffer.push_str(&String::from_utf8_lossy(bytes));
    }

    fn next_event(&mut self) -> Option<SseEvent> {
        loop {
            let line_end = self.buffer.find('\n')?;
            let line = self.buffer[..line_end].trim_end_matches('\r').to_string();
            self.buffer = self.buffer[line_end + 1..].to_string();

            if line.is_empty() {
                continue;
            }

            if let Some(data) = line.strip_prefix("data: ") {
                let content = data.trim().to_string();
                if !content.is_empty() {
                    return Some(SseEvent::Data(content));
                }
                continue;
            }

            if let Some(data) = line.strip_prefix("data:") {
                let content = data.trim().to_string();
                if !content.is_empty() {
                    return Some(SseEvent::Data(content));
                }
                continue;
            }

            if let Some(msg) = line.strip_prefix("error: ") {
                return Some(SseEvent::Error(msg.trim().to_string()));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn openai_sse_parser_basic() {
        let mut parser = OpenAiSseParser::new();
        parser.feed(b"data: {\"choices\":[{\"delta\":{\"content\":\"hello\"}}]}\n\ndata: {\"choices\":[{\"delta\":{\"content\":\" world\"}}]}\n\ndata: [DONE]\n\n");

        match parser.next_event() {
            Some(SseEvent::Data(d)) => assert!(d.contains("hello")),
            _ => panic!("expected Data"),
        }
        match parser.next_event() {
            Some(SseEvent::Data(d)) => assert!(d.contains("world")),
            _ => panic!("expected Data"),
        }
        match parser.next_event() {
            Some(SseEvent::Data(d)) => assert_eq!(d, "[DONE]"),
            _ => panic!("expected Data"),
        }
        assert!(parser.next_event().is_none());
    }

    #[test]
    fn openai_sse_parser_incremental() {
        let mut parser = OpenAiSseParser::new();
        parser.feed(b"data: {\"choices\":[{\"delta\":");
        assert!(parser.next_event().is_none());
        parser.feed(b"{\"content\":\"hi\"}}]}\n\n");
        match parser.next_event() {
            Some(SseEvent::Data(d)) => assert!(d.contains("hi")),
            _ => panic!("expected Data"),
        }
    }

    #[test]
    fn parse_events_text_delta() {
        let chunk = json!({
            "choices": [{
                "delta": {"content": "hello"},
                "index": 0,
            }]
        });
        let events = parse_openai_events(&chunk).unwrap();
        assert_eq!(events.len(), 1);
        match &events[0] {
            StreamEvent::TextDelta { text } => assert_eq!(text, "hello"),
            _ => panic!("expected TextDelta"),
        }
    }

    #[test]
    fn parse_events_finish_with_usage() {
        let chunk = json!({
            "choices": [{
                "delta": {},
                "finish_reason": "tool_calls",
                "index": 0,
            }],
            "usage": {
                "prompt_tokens": 100,
                "completion_tokens": 50,
            }
        });
        let events = parse_openai_events(&chunk).unwrap();
        assert!(events.iter().any(|e| matches!(e, StreamEvent::Stop { reason: Some(pi_types::message::StopReason::ToolUse) })));
        assert!(events.iter().any(|e| matches!(e, StreamEvent::Usage(u) if u.input_tokens == 100)));
    }

    #[test]
    fn accumulate_tool_calls_multi_chunk() {
        let mut tc = Vec::new();

        // chunk 1: tool call start with partial arguments
        let delta1: Value = serde_json::from_str(
            r#"{"tool_calls":[{"index":0,"id":"call_abc","function":{"name":"bash","arguments":"false"}}]}"#
        ).unwrap();
        accumulate_tool_calls(&mut tc, &delta1);
        assert_eq!(tc.len(), 1);
        assert_eq!(tc[0].id, "call_abc");
        assert_eq!(tc[0].name, "bash");
        assert_eq!(tc[0].arguments_json, "false");

        // chunk 2: arguments continuation
        let delta2: Value = serde_json::from_str(
            r#"{"tool_calls":[{"index":0,"function":{"arguments":"}"}}]}"#
        ).unwrap();
        accumulate_tool_calls(&mut tc, &delta2);
        assert_eq!(tc[0].arguments_json, "false}");

        // flush: "false}" is not valid JSON, so input should be Null
        let result = take_tool_calls(&mut tc);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, 0);
        assert_eq!(result[0].1, "call_abc");
        assert_eq!(result[0].2, "bash");
        assert!(result[0].3.is_null()); // invalid JSON -> Null
    }

    #[test]
    fn accumulate_tool_calls_complete_json() {
        let mut tc = Vec::new();

        // Single chunk with complete arguments
        let delta: Value = serde_json::from_str(
            r###"{"tool_calls":[{"index":0,"id":"call_123","function":{"name":"read","arguments":"{\"path\":\"/tmp/test.txt\"}"}}]}"###
        ).unwrap();
        accumulate_tool_calls(&mut tc, &delta);
        assert_eq!(tc[0].arguments_json, r#"{"path":"/tmp/test.txt"}"#);

        let result = take_tool_calls(&mut tc);
        assert_eq!(result[0].3["path"], "/tmp/test.txt");
    }

    #[test]
    fn build_request_with_tools() {
        let req = CompletionRequest {
            model: "glm-5.1".to_string(),
            system_prompt: Some("You are helpful".to_string()),
            messages: vec![pi_types::message::Message::user_text("run ls")],
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
            base_url: Some("https://open.bigmodel.cn/api/coding/paas/v4/chat/completions".to_string()),
        };
        let body = build_openai_request(&req);
        assert_eq!(body["model"], "glm-5.1");
        assert!(body["tools"].as_array().unwrap().len() == 1);
        assert_eq!(body["tools"][0]["type"], "function");
        assert_eq!(body["tools"][0]["function"]["name"], "bash");
        assert_eq!(body["messages"][0]["role"], "system");
    }
}
