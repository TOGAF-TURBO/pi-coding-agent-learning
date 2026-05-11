//! 事件类型 — Agent 运行时的生命周期事件。
//!
//! 对应 `packages/coding-agent/src/core/extensions/types.ts` 的事件体系。
//!
//! 设计原则（继承自 pi）：
//! - 事件通过 EventBus 广播，扩展通过 `on(event, handler)` 订阅
//! - Before 事件允许扩展拦截和修改行为（返回 Result）
//! - After 事件仅通知，不允许修改

use serde::{Deserialize, Serialize};

/// Agent 生命周期事件。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AgentEvent {
    /// Agent 启动前 — 可阻止启动或修改系统提示。
    #[serde(rename = "before_agent_start")]
    BeforeStart(BeforeAgentStartEvent),
    /// Agent 已启动。
    #[serde(rename = "agent_start")]
    Start(AgentStartEvent),
    /// Agent 已结束。
    #[serde(rename = "agent_end")]
    End(AgentEndEvent),

    /// Turn 开始。
    #[serde(rename = "turn_start")]
    TurnStart(TurnEvent),
    /// Turn 结束。
    #[serde(rename = "turn_end")]
    TurnEnd(TurnEvent),

    /// LLM 消息流开始。
    #[serde(rename = "message_start")]
    MessageStart,
    /// LLM 消息流更新（增量内容）。
    #[serde(rename = "message_update")]
    MessageUpdate(MessageUpdateEvent),
    /// LLM 消息流结束。
    #[serde(rename = "message_end")]
    MessageEnd(MessageEndEvent),

    /// 工具执行开始。
    #[serde(rename = "tool_execution_start")]
    ToolStart(ToolExecutionEvent),
    /// 工具执行更新（进度输出）。
    #[serde(rename = "tool_execution_update")]
    ToolUpdate(ToolExecutionEvent),
    /// 工具执行结束。
    #[serde(rename = "tool_execution_end")]
    ToolEnd(ToolExecutionEvent),

    /// 工具调用拦截 — 可阻止或修改工具调用。
    #[serde(rename = "tool_call")]
    ToolCall(ToolCallEvent),
    /// 工具结果拦截 — 可修改工具输出。
    #[serde(rename = "tool_result")]
    ToolResult(ToolResultEvent),

    /// 会话开始。
    #[serde(rename = "session_start")]
    SessionStart,
    /// 会话关闭。
    #[serde(rename = "session_shutdown")]
    SessionShutdown,

    /// 用户输入事件 — 可拦截原始按键。
    #[serde(rename = "input")]
    Input(InputEvent),

    /// 用户 Bash 命令拦截 — 可阻止危险命令。
    #[serde(rename = "user_bash")]
    UserBash(UserBashEvent),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeforeAgentStartEvent {
    pub session_id: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStartEvent {
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentEndEvent {
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnEvent {
    pub turn_number: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageUpdateEvent {
    pub content_delta: Option<String>,
    pub thinking_delta: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEndEvent {
    pub stop_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionEvent {
    pub tool_name: String,
    pub tool_use_id: String,
    pub output: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallEvent {
    pub tool_name: String,
    pub tool_use_id: String,
    pub input: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResultEvent {
    pub tool_name: String,
    pub tool_use_id: String,
    pub output: String,
    pub is_error: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputEvent {
    pub data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserBashEvent {
    pub command: String,
}
