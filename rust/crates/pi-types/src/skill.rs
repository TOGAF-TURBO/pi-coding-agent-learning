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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skill_serialization() {
        let skill = Skill {
            name: "test-skill".to_string(),
            description: "A test skill".to_string(),
            content: "Do something useful".to_string(),
        };
        let json = serde_json::to_string(&skill).unwrap();
        let parsed: Skill = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "test-skill");
        assert_eq!(parsed.content, "Do something useful");
    }

    #[test]
    fn prompt_template() {
        let tmpl = PromptTemplate {
            name: "review".to_string(),
            description: "Code review template".to_string(),
            template: "Review this code: {{input}}".to_string(),
        };
        assert_eq!(tmpl.name, "review");
        assert!(tmpl.template.contains("{{input}}"));
    }
}
