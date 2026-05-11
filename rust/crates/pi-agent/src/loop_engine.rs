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

use anyhow::{Context, Result, anyhow};
use futures::StreamExt;
use pi_llm::driver::{CompletionRequest, LlmDriver, StreamEvent};
use pi_session::JsonlSession;
use pi_tools::registry::ToolRegistry;
use pi_types::message::{AssistantMessage, ContentBlock, Message, StopReason};
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
}

/// Agent 循环输出 — 收集的最终响应。
#[derive(Debug, Clone)]
pub struct AgentOutput {
    pub text: String,
    pub thinking: String,
    pub tool_calls: usize,
    pub stop_reason: Option<StopReason>,
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

        let mut iteration = 0;
        let mut total_text = String::new();
        let mut total_thinking = String::new();
        let mut total_tool_calls = 0;

        loop {
            iteration += 1;
            if iteration > self.max_iterations {
                return Ok(AgentOutput {
                    text: "[agent loop exceeded max iterations]".to_string(),
                    thinking: total_thinking,
                    tool_calls: total_tool_calls,
                    stop_reason: Some(StopReason::Length),
                });
            }

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
                api_key: String::new(), // 从 ProviderRegistry 获取
                base_url: None,
            };

            // 调用 LLM
            let mut stream = self.driver.stream(request)?;
            let mut assistant_content: Vec<ContentBlock> = Vec::new();
            let mut stop_reason = None;
            let mut current_text = String::new();
            let mut current_thinking = String::new();

            while let Some(event) = stream.next().await {
                match event {
                    Ok(StreamEvent::TextDelta { text }) => {
                        current_text.push_str(&text);
                    }
                    Ok(StreamEvent::ThinkingDelta { thinking }) => {
                        current_thinking.push_str(&thinking);
                    }
                    Ok(StreamEvent::ToolCallStart { id, name, index }) => {
                        // 占位，等 ToolCallEnd 时添加
                    }
                    Ok(StreamEvent::ToolCallDelta { index, input }) => {}
                    Ok(StreamEvent::ToolCallEnd { index }) => {}
                    Ok(StreamEvent::Stop { reason }) => {
                        stop_reason = reason;
                    }
                    Ok(StreamEvent::Usage(_)) => {}
                    Ok(StreamEvent::Error { message }) => {
                        return Err(anyhow!("LLM error: {message}"));
                    }
                    Ok(StreamEvent::Start) => {}
                    Err(e) => {
                        return Err(anyhow!("Stream error: {e}"));
                    }
                }
            }

            // 添加文本内容
            if !current_text.is_empty() {
                assistant_content.push(ContentBlock::text(&current_text));
                total_text.push_str(&current_text);
            }
            if !current_thinking.is_empty() {
                total_thinking.push_str(&current_thinking);
            }

            // 检查是否有工具调用需要执行
            // 注意：当前简化版不解析流式 tool_call，直接返回文本
            // 完整的 tool_call 解析在 Phase 2 实现

            // 追加助手消息到会话
            let assistant_entry = MessageEntry {
                entry_type: "message".to_string(),
                id: generate_id(),
                parent_id: self.session.leaf_id().map(|s| s.to_string()),
                timestamp: chrono::Utc::now().to_rfc3339(),
                role: "assistant".to_string(),
                content: serde_json::json!(
                    assistant_content.iter().map(|c| serde_json::json!({"type": "text", "text": c.as_text().unwrap_or("")})).collect::<Vec<_>>()
                ),
                model: Some(self.model.clone()),
                stop_reason: stop_reason.map(|r| format!("{r:?}").to_lowercase()),
                usage: None,
            };
            self.session
                .append(pi_types::session::SessionEntry::Message(assistant_entry))
                .await
                .context("Failed to append assistant message")?;

            return Ok(AgentOutput {
                text: total_text,
                thinking: total_thinking,
                tool_calls: total_tool_calls,
                stop_reason,
            });
        }
    }

    /// 从会话条目构建消息历史。
    fn build_messages(&self) -> Result<Vec<Message>> {
        let mut messages = Vec::new();

        for entry in self.session.entries() {
            if let pi_types::session::SessionEntry::Message(me) = entry {
                if me.role == "user" {
                    // 从 content 提取文本
                    let text = extract_text_from_content(&me.content);
                    if !text.is_empty() {
                        messages.push(Message::user_text(&text));
                    }
                } else if me.role == "assistant" {
                    let text = extract_text_from_content(&me.content);
                    if !text.is_empty() {
                        messages.push(Message::assistant_text(&text));
                    }
                }
            }
        }

        Ok(messages)
    }
}

/// 从 JSON content 数组中提取文本。
fn extract_text_from_content(content: &serde_json::Value) -> String {
    content
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|block| {
                    if block.get("type").and_then(|v| v.as_str()) == Some("text") {
                        block.get("text").and_then(|v| v.as_str()).map(|s| s.to_string())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default()
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
     Be concise and direct."
        .to_string()
}
