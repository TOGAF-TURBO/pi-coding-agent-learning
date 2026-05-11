//! 技能和提示模板类型。
//!
//! 对应 `packages/agent/src/harness/skills.ts` 和 `packages/coding-agent/src/core/prompt-templates.ts`。

use serde::{Deserialize, Serialize};

/// 技能定义 — 可注入到系统提示的领域知识包。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub description: String,
    /// 技能内容的文件路径或内联文本。
    pub content: String,
}

/// 提示模板 — 预定义的系统提示结构。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    pub name: String,
    pub description: String,
    pub template: String,
}
