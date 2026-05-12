//! 工具类型 — LLM 可调用工具的定义和执行接口。
//!
//! 对应 `packages/ai/src/types.ts` 的 `Tool<T>` 和
//! `packages/coding-agent/src/core/extensions/types.ts` 的 `ToolDefinition`。

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// 工具参数的 JSON Schema 定义。
pub type ParameterSchema = Value;

/// 工具定义 — 描述一个 LLM 可调用的工具。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: ParameterSchema,
    /// 是否需要用户批准才能执行。
    #[serde(default)]
    pub requires_approval: bool,
}

/// 工具执行结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_use_id: String,
    pub output: String,
    pub is_error: bool,
    /// 执行耗时（毫秒）。
    pub duration_ms: Option<u64>,
    /// 工具请求终止 Agent 循环。
    /// 当所有工具结果都设置 terminate=true 时，ReAct 循环退出。
    #[serde(default)]
    pub terminate: bool,
}

/// 工具执行器的 trait。
///
/// 每个内置工具实现此 trait。扩展注册的工具通过 ExtensionBridge 适配。
#[async_trait::async_trait]
pub trait ToolExecutor: Send + Sync {
    /// 返回工具的定义（name、description、parameters schema）。
    fn definition(&self) -> ToolDefinition;

    /// 执行工具调用。
    async fn execute(&self, input: Value) -> Result<ToolResult, crate::error::PiError>;
}
