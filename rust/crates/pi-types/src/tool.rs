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

impl ToolDefinition {
    /// 校验工具输入参数。
    ///
    /// 检查：
    /// 1. input 必须是 object
    /// 2. required 字段必须存在
    /// 3. properties 中声明的类型必须匹配
    ///
    /// 返回 Ok(input) 或 Err(错误描述)。
    pub fn validate_input(&self, input: &Value) -> Result<Value, String> {
        let obj = input
            .as_object()
            .ok_or_else(|| "tool arguments must be a JSON object".to_string())?;

        // 1. required 字段检查
        if let Some(required) = self.parameters.get("required").and_then(|r| r.as_array()) {
            for field in required {
                if let Some(name) = field.as_str() {
                    if !obj.contains_key(name) {
                        return Err(format!(
                            "missing required parameter '{}' for tool '{}'",
                            name, self.name
                        ));
                    }
                }
            }
        }

        // 2. 类型校验
        if let Some(properties) = self.parameters.get("properties").and_then(|p| p.as_object()) {
            for (key, schema) in properties {
                if let Some(value) = obj.get(key) {
                    if let Some(expected_type) = schema.get("type").and_then(|t| t.as_str()) {
                        let actual_type = json_type_of(value);
                        if actual_type != expected_type
                            && !(expected_type == "number" && actual_type == "integer")
                        {
                            return Err(format!(
                                "parameter '{}' expected type '{}', got '{}'",
                                key, expected_type, actual_type
                            ));
                        }
                    }
                }
            }
        }

        Ok(input.clone())
    }
}

/// JSON 值的类型名称。
fn json_type_of(v: &Value) -> &'static str {
    match v {
        Value::String(_) => "string",
        Value::Number(n) => {
            if n.is_f64() { "number" } else { "integer" }
        }
        Value::Bool(_) => "boolean",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
        Value::Null => "null",
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_def(name: &str, params: Value) -> ToolDefinition {
        ToolDefinition {
            name: name.to_string(),
            description: "test".to_string(),
            parameters: params,
            requires_approval: false,
        }
    }

    #[test]
    fn validate_missing_required() {
        let def = make_def("bash", json!({
            "type": "object",
            "properties": { "command": { "type": "string" } },
            "required": ["command"]
        }));
        let result = def.validate_input(&json!({}));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("missing required"));
        assert!(err.contains("command"));
    }

    #[test]
    fn validate_wrong_type() {
        let def = make_def("bash", json!({
            "type": "object",
            "properties": { "command": { "type": "string" } },
            "required": ["command"]
        }));
        let result = def.validate_input(&json!({ "command": 123 }));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("expected type 'string'"));
    }

    #[test]
    fn validate_ok() {
        let def = make_def("bash", json!({
            "type": "object",
            "properties": { "command": { "type": "string" } },
            "required": ["command"]
        }));
        let result = def.validate_input(&json!({ "command": "echo hi" }));
        assert!(result.is_ok());
    }

    #[test]
    fn validate_unknown_field_ok() {
        let def = make_def("bash", json!({
            "type": "object",
            "properties": { "command": { "type": "string" } },
            "required": ["command"]
        }));
        // extra fields should be tolerated
        let result = def.validate_input(&json!({ "command": "ls", "extra": 42 }));
        assert!(result.is_ok());
    }

    #[test]
    fn validate_number_accepts_integer() {
        let def = make_def("bash", json!({
            "type": "object",
            "properties": { "timeout": { "type": "number" } }
        }));
        // integer should be accepted as number
        let result = def.validate_input(&json!({ "timeout": 30 }));
        assert!(result.is_ok());
    }
}
