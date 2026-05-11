//! 技能加载 — 从 ~/.piso/skills/ 和 <project>/.piso/skills/ 加载 SKILL.md。
//!
//! 对应 `packages/coding-agent/src/core/skills.ts`。
//!
//! 技能目录结构：
//! ```text
//! ~/.piso/skills/
//! └── my-skill/
//!     └── SKILL.md          # name + description 在 frontmatter 或目录名
//!
//! <project>/.piso/skills/
//! └── project-skill/
//!     └── SKILL.md
//! ```
//!
//! 技能注入到系统提示，指导 LLM 在匹配任务时加载技能文件。

use std::path::{Path, PathBuf};

use anyhow::Result;

/// 加载的技能。
#[derive(Debug, Clone)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub file_path: PathBuf,
    /// 技能文件内容。
    pub content: String,
    /// 来源（"user" = 全局，"project" = 项目级）。
    pub source: String,
}

/// 从多个来源加载所有技能。
///
/// 加载顺序（后者覆盖同名）：
/// 1. 全局 `~/.piso/skills/`
/// 2. 项目 `<project>/.piso/skills/`
pub fn load_skills(cwd: &Path, global_skills_dir: Option<&Path>) -> Vec<Skill> {
    let mut skills = Vec::new();
    let mut seen_names = std::collections::HashSet::new();

    // 1. 全局技能
    if let Some(dir) = global_skills_dir {
        if dir.exists() {
            load_skills_from_dir(dir, "user", &mut skills, &mut seen_names);
        }
    }

    // 2. 项目技能
    let project_skills = cwd.join(".piso").join("skills");
    if project_skills.exists() {
        load_skills_from_dir(&project_skills, "project", &mut skills, &mut seen_names);
    }

    skills
}

/// 从单个目录加载所有技能。
fn load_skills_from_dir(
    dir: &Path,
    source: &str,
    skills: &mut Vec<Skill>,
    seen_names: &mut std::collections::HashSet<String>,
) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let skill_md = path.join("SKILL.md");
        if !skill_md.is_file() {
            continue;
        }

        let dir_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();

        match load_skill(&skill_md, &dir_name, source) {
            Ok(skill) => {
                if !seen_names.contains(&skill.name) {
                    seen_names.insert(skill.name.clone());
                    skills.push(skill);
                }
            }
            Err(e) => {
                eprintln!("[skill] Failed to load {}: {e}", skill_md.display());
            }
        }
    }
}

/// 加载单个 SKILL.md。
fn load_skill(path: &Path, fallback_name: &str, source: &str) -> Result<Skill> {
    let content = std::fs::read_to_string(path)?;

    // 解析 frontmatter（--- 包围的 YAML）
    let (frontmatter, body) = parse_frontmatter(&content);

    let name = frontmatter.get("name")
        .and_then(|v| v.as_str())
        .unwrap_or(fallback_name)
        .to_string();

    let description = frontmatter.get("description")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| {
            // 从 body 第一行提取描述
            body.lines()
                .find(|l| !l.trim().is_empty() && !l.starts_with('#'))
                .unwrap_or(fallback_name)
        })
        .to_string();

    Ok(Skill {
        name,
        description,
        file_path: path.to_path_buf(),
        content: body.to_string(),
        source: source.to_string(),
    })
}

/// 简单 frontmatter 解析（仅支持扁平 key: value）。
fn parse_frontmatter(content: &str) -> (serde_json::Value, &str) {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return (serde_json::Value::Object(Default::default()), content);
    }

    // 找到结束的 ---
    let after_first = &trimmed[3..];
    if let Some(end) = after_first.find("\n---") {
        let yaml_str = &after_first[..end];
        let body = &after_first[end + 4..];

        // 简单解析 key: value
        let mut map = serde_json::Map::new();
        for line in yaml_str.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim().to_string();
                let value = value.trim().trim_matches('"').trim_matches('\'').to_string();
                map.insert(key, serde_json::Value::String(value));
            }
        }

        return (serde_json::Value::Object(map), body);
    }

    (serde_json::Value::Object(Default::default()), content)
}

/// 将技能列表格式化为系统提示文本。
///
/// 格式与 TS 版本的 `formatSkillsForSystemPrompt` 兼容。
pub fn format_skills_for_prompt(skills: &[Skill]) -> String {
    if skills.is_empty() {
        return String::new();
    }

    let mut lines = vec![
        "The following skills provide specialized instructions for specific tasks.".to_string(),
        "Use the read tool to load a skill's file when the task matches its description.".to_string(),
        "When a skill file references a relative path, resolve it against the skill directory.".to_string(),
        String::new(),
        "<available_skills>".to_string(),
    ];

    for skill in skills {
        lines.push("  <skill>".to_string());
        lines.push(format!("    <name>{}</name>", escape_xml(&skill.name)));
        lines.push(format!("    <description>{}</description>", escape_xml(&skill.description)));
        let path_str = skill.file_path.display().to_string();
        lines.push(format!("    <location>{}</location>", escape_xml(&path_str)));
        lines.push("  </skill>".to_string());
    }

    lines.push("</available_skills>".to_string());
    lines.join("\n")
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn loads_skill_from_directory() {
        let dir = TempDir::new().unwrap();
        let skill_dir = dir.path().join("my-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: my-skill\ndescription: A test skill\n---\n# My Skill\nDo stuff.",
        )
        .unwrap();

        let mut skills = Vec::new();
        let mut seen = std::collections::HashSet::new();
        load_skills_from_dir(dir.path(), "test", &mut skills, &mut seen);

        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "my-skill");
        assert_eq!(skills[0].description, "A test skill");
        assert!(skills[0].content.contains("Do stuff."));
    }

    #[test]
    fn skill_without_frontmatter_uses_dir_name() {
        let dir = TempDir::new().unwrap();
        let skill_dir = dir.path().join("auto-name");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(skill_dir.join("SKILL.md"), "Just some content.").unwrap();

        let mut skills = Vec::new();
        let mut seen = std::collections::HashSet::new();
        load_skills_from_dir(dir.path(), "test", &mut skills, &mut seen);

        assert_eq!(skills[0].name, "auto-name");
    }

    #[test]
    fn format_skills_produces_xml() {
        let skills = vec![Skill {
            name: "test".to_string(),
            description: "A test".to_string(),
            file_path: PathBuf::from("/tmp/test/SKILL.md"),
            content: String::new(),
            source: "test".to_string(),
        }];
        let formatted = format_skills_for_prompt(&skills);
        assert!(formatted.contains("<available_skills>"));
        assert!(formatted.contains("<name>test</name>"));
        assert!(formatted.contains("</available_skills>"));
    }

    #[test]
    fn deduplicates_by_name() {
        let dir = TempDir::new().unwrap();

        // 两个目录各有一个同名技能
        let s1 = dir.path().join("global").join("my-skill");
        fs::create_dir_all(&s1).unwrap();
        fs::write(s1.join("SKILL.md"), "---\nname: my-skill\n---\nGlobal.").unwrap();

        let s2 = dir.path().join("project").join("my-skill");
        fs::create_dir_all(&s2).unwrap();
        fs::write(s2.join("SKILL.md"), "---\nname: my-skill\n---\nProject.").unwrap();

        let mut skills = Vec::new();
        let mut seen = std::collections::HashSet::new();
        load_skills_from_dir(&dir.path().join("global"), "global", &mut skills, &mut seen);
        load_skills_from_dir(&dir.path().join("project"), "project", &mut skills, &mut seen);

        // 同名只保留第一个
        assert_eq!(skills.len(), 1);
    }
}
