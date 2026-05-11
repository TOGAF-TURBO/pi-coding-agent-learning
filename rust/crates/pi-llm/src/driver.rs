//! LLM Driver trait — 所有 provider 的统一异步接口。

use async_trait::async_trait;
use futures::Stream;
use serde::{Deserialize, Serialize};
use std::pin::Pin;

use pi_types::message::{StopReason, Usage};
use pi_types::tool::ToolDefinition;

/// 发送给 LLM 的完成请求。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    pub model: String,
    pub system_prompt: Option<String>,
    pub messages: Vec<pi_types::message::Message>,
    pub tools: Vec<ToolDefinition>,
    pub thinking_enabled: bool,
    pub thinking_budget: Option<u32>,
    pub max_tokens: u32,
    pub api_key: String,
    pub base_url: Option<String>,
}

/// LLM 流式响应事件。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StreamEvent {
    /// 流开始。
    #[serde(rename = "start")]
    Start,
    /// 文本增量。
    #[serde(rename = "text_delta")]
    TextDelta { text: String },
    /// 思考增量。
    #[serde(rename = "thinking_delta")]
    ThinkingDelta { thinking: String },
    /// 工具调用开始。
    #[serde(rename = "tool_call_start")]
    ToolCallStart {
        id: String,
        name: String,
        index: usize,
    },
    /// 工具调用输入增量（JSON 片段）。
    #[serde(rename = "tool_call_delta")]
    ToolCallDelta { index: usize, input: String },
    /// 工具调用结束（输入 JSON 已完整累积）。
    #[serde(rename = "tool_call_end")]
    ToolCallEnd {
        index: usize,
        id: String,
        name: String,
        input: serde_json::Value,
    },
    /// 工具执行结果（由 AgentLoop 在工具执行后发出）。
    #[serde(rename = "tool_result")]
    ToolResult {
        id: String,
        name: String,
        output: String,
        is_error: bool,
    },
    /// 用量统计。
    #[serde(rename = "usage")]
    Usage(Usage),
    /// 流结束。
    #[serde(rename = "stop")]
    Stop { reason: Option<StopReason> },
    /// 错误。
    #[serde(rename = "error")]
    Error { message: String },
}

/// 流类型别名。
pub type StreamResult = Pin<Box<dyn Stream<Item = Result<StreamEvent, anyhow::Error>> + Send>>;

/// LLM 驱动 trait — 所有 provider 实现此接口。
#[async_trait]
pub trait LlmDriver: Send + Sync {
    /// 流式完成请求。
    fn stream(&self, request: CompletionRequest) -> Result<StreamResult, anyhow::Error>;

    /// 返回 driver 名称（如 "anthropic", "openai"）。
    fn name(&self) -> &str;
}
