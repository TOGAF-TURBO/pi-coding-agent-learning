//! Agent 循环 — 编排 LLM 调用、工具执行、会话状态。
//!
//! 核心循环：
//! ```text
//! loop {
//!     1. 构建请求（系统提示 + 会话历史 + 工具定义）
//!     2. 调用 LLM（流式接收响应）
//!     3. 如果响应包含工具调用 → 执行工具 → 追加结果 → 回到 1
//!     4. 如果响应是最终文本 → 返回
//! }
//! ```

use std::io::Write;

use anyhow::{Context, Result, anyhow};
use futures::StreamExt;
use pi_llm::driver::{CompletionRequest, LlmDriver, StreamEvent};
use pi_session::JsonlSession;
use pi_tools::registry::ToolRegistry;
use pi_types::message::{ContentBlock, Message, StopReason};
use pi_types::session::MessageEntry;
use pi_types::tool::ToolResult;

/// Agent 循环运行时。
pub struct AgentLoop {
    session: JsonlSession,
    driver: Box<dyn LlmDriver>,
    tools: ToolRegistry,
    system_prompt: String,
    model: String,
    max_tokens: u32,
    max_iterations: usize,
    api_key: String,
    base_url: Option<String>,
}

/// Agent 循环输出 — 收集的最终响应。
#[derive(Debug, Clone)]
pub struct AgentOutput {
    pub text: String,
    pub thinking: String,
    pub tool_calls: usize,
    pub stop_reason: Option<StopReason>,
}

/// 流式收集的一次 LLM 响应。
#[derive(Debug, Clone, Default)]
struct LlmResponse {
    text: String,
    thinking: String,
    tool_calls: Vec<ToolCallInfo>,
    stop_reason: Option<StopReason>,
}

/// 完整的工具调用信息。
#[derive(Debug, Clone)]
struct ToolCallInfo {
    id: String,
    name: String,
    input: serde_json::Value,
}

impl AgentLoop {
    pub fn new(
        session: JsonlSession,
        driver: Box<dyn LlmDriver>,
        tools: ToolRegistry,
        model: impl Into<String>,
    ) -> Self {
        let model_name = model.into();
        Self {
            session,
            driver,
            tools,
            system_prompt: build_default_system_prompt(),
            model: model_name,
            max_tokens: 16384,
            max_iterations: 50,
            api_key: String::new(),
            base_url: None,
        }
    }

    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = prompt.into();
        self
    }

    pub fn with_max_tokens(mut self, tokens: u32) -> Self {
        self.max_tokens = tokens;
        self
    }

    pub fn with_api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = key.into();
        self
    }

    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    /// 发送用户消息并运行 agent 循环直到完成。
    pub async fn run(&mut self, user_message: &str) -> Result<AgentOutput> {
        // 追加用户消息到会话
        let user_entry = MessageEntry {
            entry_type: "message".to_string(),
            id: generate_id(),
            parent_id: self.session.leaf_id().map(|s| s.to_string()),
            timestamp: chrono::Utc::now().to_rfc3339(),
            role: "user".to_string(),
            content: serde_json::json!([{"type": "text", "text": user_message}]),
            model: None,
            stop_reason: None,
            usage: None,
        };
        self.session
            .append(pi_types::session::SessionEntry::Message(user_entry))
            .await
            .context("Failed to append user message")?;

        let mut total_text = String::new();
        let mut total_thinking = String::new();
        let mut total_tool_calls = 0;

        for _iteration in 1..=self.max_iterations {
            // 构建消息历史
            let messages = self.build_messages()?;

            // 构建请求
            let request = CompletionRequest {
                model: self.model.clone(),
                system_prompt: Some(self.system_prompt.clone()),
                messages,
                tools: self.tools.definitions(),
                thinking_enabled: false,
                thinking_budget: None,
                max_tokens: self.max_tokens,
                api_key: self.api_key.clone(),
                base_url: self.base_url.clone(),
            };

            // 调用 LLM，收集流式响应
            let response = self.call_llm(request).await?;

            // 收集内容块
            let mut assistant_content: Vec<ContentBlock> = Vec::new();

            if !response.text.is_empty() {
                assistant_content.push(ContentBlock::text(&response.text));
                total_text.push_str(&response.text);
            }
            if !response.thinking.is_empty() {
                total_thinking.push_str(&response.thinking);
            }
            for tc in &response.tool_calls {
                assistant_content.push(ContentBlock::tool_call(&tc.id, &tc.name, tc.input.clone()));
            }

            // 追加助手消息到会话
            let assistant_entry = MessageEntry {
                entry_type: "message".to_string(),
                id: generate_id(),
                parent_id: self.session.leaf_id().map(|s| s.to_string()),
                timestamp: chrono::Utc::now().to_rfc3339(),
                role: "assistant".to_string(),
                content: serde_json::json!(assistant_content.iter().map(|c| content_block_to_json(c)).collect::<Vec<_>>()),
                model: Some(self.model.clone()),
                stop_reason: response.stop_reason.map(|r| format!("{r:?}").to_lowercase()),
                usage: None,
            };
            self.session
                .append(pi_types::session::SessionEntry::Message(assistant_entry))
                .await
                .context("Failed to append assistant message")?;

            // 如果没有工具调用，循环结束
            if response.tool_calls.is_empty() {
                return Ok(AgentOutput {
                    text: total_text,
                    thinking: total_thinking,
                    tool_calls: total_tool_calls,
                    stop_reason: response.stop_reason,
                });
            }

            // 执行工具调用
            total_tool_calls += response.tool_calls.len();
            let mut tool_results: Vec<ContentBlock> = Vec::new();

            for tc in &response.tool_calls {
                eprintln!("[tool] {}({})", tc.name, tc.input);

                let result = self.execute_tool(&tc.name, tc.input.clone()).await;

                let (output, is_error) = match result {
                    Ok(r) => (r.output, r.is_error),
                    Err(e) => (format!("Tool execution error: {e}"), true),
                };

                eprintln!("[tool] → {}{}", 
                    if is_error { "ERROR: " } else { "" },
                    if output.len() > 200 { &output[..200] } else { &output }
                );

                tool_results.push(ContentBlock::tool_result(&tc.id, &output, is_error));
            }

            // 追加工具结果到会话
            let tool_result_entry = MessageEntry {
                entry_type: "message".to_string(),
                id: generate_id(),
                parent_id: self.session.leaf_id().map(|s| s.to_string()),
                timestamp: chrono::Utc::now().to_rfc3339(),
                role: "user".to_string(),
                content: serde_json::json!(tool_results.iter().map(|c| content_block_to_json(c)).collect::<Vec<_>>()),
                model: None,
                stop_reason: None,
                usage: None,
            };
            self.session
                .append(pi_types::session::SessionEntry::Message(tool_result_entry))
                .await
                .context("Failed to append tool results")?;

            // 继续循环 — LLM 将看到工具结果并决定下一步
        }

        Ok(AgentOutput {
            text: if total_text.is_empty() {
                "[agent loop exceeded max iterations]".to_string()
            } else {
                total_text
            },
            thinking: total_thinking,
            tool_calls: total_tool_calls,
            stop_reason: Some(StopReason::Length),
        })
    }

    /// 调用 LLM 并收集完整流式响应。
    async fn call_llm(&self, request: CompletionRequest) -> Result<LlmResponse> {
        let mut stream = self.driver.stream(request)?;
        let mut resp = LlmResponse::default();

        while let Some(event) = stream.next().await {
            match event {
                Ok(StreamEvent::TextDelta { text }) => {
                    resp.text.push_str(&text);
                    // 实时输出文本到 stderr（不干扰 stdout 的最终输出）
                    eprint!("{}", text);
                    let _ = std::io::stderr().flush();
                }
                Ok(StreamEvent::ThinkingDelta { thinking }) => {
                    resp.thinking.push_str(&thinking);
                }
                Ok(StreamEvent::ToolCallStart { id, name, index }) => {
                    // 确保有足够空间
                    while resp.tool_calls.len() <= index {
                        resp.tool_calls.push(ToolCallInfo {
                            id: String::new(),
                            name: String::new(),
                            input: serde_json::Value::Null,
                        });
                    }
                    resp.tool_calls[index].id = id;
                    resp.tool_calls[index].name = name;
                }
                Ok(StreamEvent::ToolCallEnd { index, id, name, input }) => {
                    while resp.tool_calls.len() <= index {
                        resp.tool_calls.push(ToolCallInfo {
                            id: String::new(),
                            name: String::new(),
                            input: serde_json::Value::Null,
                        });
                    }
                    resp.tool_calls[index] = ToolCallInfo { id, name, input };
                }
                Ok(StreamEvent::Stop { reason }) => {
                    resp.stop_reason = reason;
                }
                Ok(StreamEvent::Usage(_)) => {}
                Ok(StreamEvent::Error { message }) => {
                    return Err(anyhow!("LLM error: {message}"));
                }
                Ok(StreamEvent::Start) => {}
                Ok(StreamEvent::ToolCallDelta { .. }) => {}
                Err(e) => {
                    return Err(anyhow!("Stream error: {e}"));
                }
            }
        }

        eprintln!(); // 流结束后换行
        Ok(resp)
    }

    /// 执行单个工具调用。
    async fn execute_tool(&self, name: &str, input: serde_json::Value) -> Result<ToolResult> {
        match self.tools.get(name) {
            Some(executor) => {
                executor.execute(input)
                    .await
                    .map_err(|e| anyhow!("Tool '{}' execution failed: {}", name, e))
            }
            None => Err(anyhow!("Unknown tool: {name}")),
        }
    }

    /// 从会话条目构建消息历史。
    fn build_messages(&self) -> Result<Vec<Message>> {
        let mut messages = Vec::new();

        for entry in self.session.entries() {
            if let pi_types::session::SessionEntry::Message(me) = entry {
                match me.role.as_str() {
                    "user" => {
                        // content 可能包含 text 和 tool_result 块
                        let blocks = parse_content_blocks(&me.content);
                        if !blocks.is_empty() {
                            messages.push(Message::User(pi_types::message::UserMessage {
                                role: "user".to_string(),
                                content: blocks,
                            }));
                        }
                    }
                    "assistant" => {
                        let blocks = parse_content_blocks(&me.content);
                        if !blocks.is_empty() {
                            messages.push(Message::Assistant(pi_types::message::AssistantMessage {
                                role: "assistant".to_string(),
                                content: blocks,
                                model: me.model.clone(),
                                stop_reason: me.stop_reason.as_deref().and_then(parse_stop_reason),
                                usage: None,
                                timestamp: Some(me.timestamp.clone()),
                            }));
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(messages)
    }
}

/// 从 JSON content 数组解析 ContentBlock 列表。
fn parse_content_blocks(content: &serde_json::Value) -> Vec<ContentBlock> {
    content.as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|block| {
                    let btype = block.get("type").and_then(|v| v.as_str())?;
                    match btype {
                        "text" => {
                            let text = block.get("text").and_then(|v| v.as_str()).unwrap_or("");
                            Some(ContentBlock::text(text))
                        }
                        "tool_use" => {
                            let id = block.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                            let name = block.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                            let input = block.get("input").cloned().unwrap_or(serde_json::Value::Null);
                            Some(ContentBlock::tool_call(id, name, input))
                        }
                        "tool_result" => {
                            let tool_use_id = block.get("tool_use_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                            let result_content = block.get("content").and_then(|v| v.as_str()).unwrap_or("").to_string();
                            let is_error = block.get("is_error").and_then(|v| v.as_bool()).unwrap_or(false);
                            Some(ContentBlock::tool_result(tool_use_id, result_content, is_error))
                        }
                        _ => None,
                    }
                })
                .collect()
        })
        .unwrap_or_default()
}

/// 将 ContentBlock 转为 JSON（用于会话存储）。
fn content_block_to_json(block: &ContentBlock) -> serde_json::Value {
    match block {
        ContentBlock::Text(t) => serde_json::json!({"type": "text", "text": t.text}),
        ContentBlock::ToolUse(tc) => serde_json::json!({
            "type": "tool_use",
            "id": tc.id,
            "name": tc.name,
            "input": tc.input,
        }),
        ContentBlock::ToolResult(r) => serde_json::json!({
            "type": "tool_result",
            "tool_use_id": r.tool_use_id,
            "content": r.content,
            "is_error": r.is_error,
        }),
        _ => serde_json::json!({"type": "text", "text": "[unsupported]"}),
    }
}

/// 解析 stop reason 字符串。
fn parse_stop_reason(s: &str) -> Option<StopReason> {
    match s {
        "stop" | "end_turn" => Some(StopReason::Stop),
        "tool_use" => Some(StopReason::ToolUse),
        "length" | "max_tokens" => Some(StopReason::Length),
        "error" => Some(StopReason::Error),
        "aborted" => Some(StopReason::Aborted),
        _ => None,
    }
}

fn generate_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{:08x}", (t as u32) ^ ((t >> 32) as u32))
}

fn build_default_system_prompt() -> String {
    "You are a helpful coding assistant running in the user's terminal. \
     You can execute commands and edit files to help the user with their tasks. \
     Be concise and direct.".to_string()
}
