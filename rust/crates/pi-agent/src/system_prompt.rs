//! 系统提示构建 — 组装发送给 LLM 的系统提示。
//!
//! 包含：
//! - 角色定义（终端编码助手）
//! - 工作目录信息
//! - 工具使用指南
//! - 上下文文件（AGENTS.md 等，Phase 2 后续）

/// 系统提示构建器。
pub struct SystemPromptBuilder {
    cwd: String,
    model_info: Option<String>,
    custom_prompt: Option<String>,
    append_prompts: Vec<String>,
    tool_guides: Vec<String>,
}

impl SystemPromptBuilder {
    pub fn new(cwd: impl Into<String>) -> Self {
        Self {
            cwd: cwd.into(),
            model_info: None,
            custom_prompt: None,
            append_prompts: Vec::new(),
            tool_guides: Vec::new(),
        }
    }

    /// 设置模型信息（名称 + provider），注入 system prompt。
    pub fn with_model_info(mut self, model_name: &str, provider: &str) -> Self {
        self.model_info = Some(format!("You are {} served by {}.", model_name, provider));
        self
    }

    /// 覆盖默认系统提示。
    pub fn with_custom_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.custom_prompt = Some(prompt.into());
        self
    }

    /// 追加额外提示（可多次调用）。
    pub fn append(mut self, prompt: impl Into<String>) -> Self {
        self.append_prompts.push(prompt.into());
        self
    }

    /// 启用内置工具指南。
    pub fn with_tool_guides(mut self) -> Self {
        self.tool_guides = vec![
            tool_guide_bash(),
            tool_guide_read(),
            tool_guide_write(),
            tool_guide_edit(),
            tool_guide_find(),
            tool_guide_grep(),
        ];
        self
    }

    /// 加载项目级自定义提示（从 .piso/system.md）。
    pub fn with_project_prompt(mut self, cwd: &str) -> Self {
        let system_md = std::path::Path::new(cwd).join(".piso").join("system.md");
        if let Ok(content) = std::fs::read_to_string(&system_md) {
            self.append_prompts.push(content);
        }
        self
    }

    /// 构建最终系统提示。
    pub fn build(&self) -> String {
        let mut parts = Vec::new();

        // 1. 核心角色提示
        if let Some(custom) = &self.custom_prompt {
            parts.push(custom.clone());
        } else {
            parts.push(default_role_prompt(&self.cwd));
        }

        // 1.5 模型身份（紧跟角色提示之后）
        if let Some(info) = &self.model_info {
            parts.push(String::new());
            parts.push(info.clone());
        }

        // 2. 工具使用指南
        if !self.tool_guides.is_empty() {
            parts.push(String::new());
            parts.push("# Tool Usage Guide".to_string());
            parts.push(String::new());
            for guide in &self.tool_guides {
                parts.push(guide.clone());
            }
        }

        // 3. 追加提示
        for extra in &self.append_prompts {
            parts.push(String::new());
            parts.push(extra.clone());
        }

        parts.join("\n")
    }
}

/// 默认角色提示。
fn default_role_prompt(cwd: &str) -> String {
    format!(
        r#"You are a helpful coding assistant running in the user's terminal.

## Environment

- Working directory: {cwd}
- You can execute commands and edit files to help the user with their tasks.
- Be concise and direct in your responses.

## Rules

- Current date: {date}
- Current working directory: {cwd}

- When making file edits, use the `edit` tool for precise changes. Only use `write` for new files or complete rewrites.
- Before editing files you have not already inspected, read them first.
- Keep edits minimal and focused on the user's request.
- After code changes, consider running relevant tests or checks.
- Use the `read` tool to inspect files before making wide-ranging changes.
- Never use `git add -A` or `git add .` — always use `git add <specific-file-paths>`.
- When you don't know something, say so. Don't make up information.
"#,
        cwd = cwd,
        date = chrono::Utc::now().format("%Y-%m-%d")
    )
}

fn tool_guide_bash() -> String {
    r#"### bash — Execute shell commands
- Use for running build commands, tests, git operations, and other shell tasks.
- The `command` parameter is required.
- Default timeout is 120 seconds. Use `timeout` parameter for longer/shorter.
- Output is truncated at 2000 lines / 1MB.
"#
    .to_string()
}

fn tool_guide_read() -> String {
    r#"### read — Read file contents
- Returns file content with line numbers.
- Use `offset` (1-indexed) and `limit` for pagination of large files.
- Always read files before editing them.
"#
    .to_string()
}

fn tool_guide_write() -> String {
    r#"### write — Create or overwrite files
- Creates parent directories automatically.
- Only use for new files or complete rewrites. Use `edit` for targeted changes.
"#
    .to_string()
}

fn tool_guide_edit() -> String {
    r#"### edit — Make precise file edits
- Uses exact text replacement: `oldText` must match exactly in the file.
- `oldText` must be unique in the file (no ambiguous matches).
- Include enough context in `oldText` to make it unique.
- When changing multiple locations, make separate `edit` calls for each.
"#
    .to_string()
}

fn tool_guide_find() -> String {
    r#"### find — Find files and directories
- Supports glob patterns (e.g. `**/*.rs`, `src/**/*.ts`).
- Automatically skips `.git`, `node_modules`, `target`, `dist`.
- Use `path` to set base directory (default: cwd).
"#
    .to_string()
}

fn tool_guide_grep() -> String {
    r#"### grep — Search file contents
- Supports regex patterns.
- Use `include` glob to filter files (e.g. `*.rs`, `*.ts`).
- Use `case_insensitive: true` for case-insensitive search.
- Results format: `file:line\tcontent`.
"#
    .to_string()
}

impl Default for SystemPromptBuilder {
    fn default() -> Self {
        Self::new(".")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn basic_prompt_contains_tool_guide() {
        let builder = SystemPromptBuilder::new("/tmp").with_tool_guides();
        let prompt = builder.build();
        assert!(prompt.contains("read"));
        assert!(prompt.contains("bash"));
        assert!(prompt.contains("edit"));
    }

    #[test]
    fn custom_prompt_override() {
        let builder =
            SystemPromptBuilder::new("/tmp").with_custom_prompt("You are a test assistant.");
        let prompt = builder.build();
        assert!(prompt.contains("test assistant"));
    }

    #[test]
    fn append_prompt() {
        let builder = SystemPromptBuilder::new("/tmp").append("Extra instructions here.");
        let prompt = builder.build();
        assert!(prompt.contains("Extra instructions"));
    }

    #[test]
    fn project_prompt_from_file() {
        let dir = TempDir::new().unwrap();
        let piso_dir = dir.path().join(".piso");
        fs::create_dir_all(&piso_dir).unwrap();
        fs::write(piso_dir.join("system.md"), "Project-specific instructions").unwrap();

        let builder = SystemPromptBuilder::new(dir.path().to_string_lossy())
            .with_project_prompt(&dir.path().to_string_lossy());
        let prompt = builder.build();
        assert!(prompt.contains("Project-specific instructions"));
    }

    #[test]
    fn model_info_injected() {
        let builder = SystemPromptBuilder::new("/tmp").with_model_info("glm-5.1", "zhipu");
        let prompt = builder.build();
        assert!(prompt.contains("You are glm-5.1 served by zhipu."));
    }

    #[test]
    fn no_project_prompt_when_missing() {
        let dir = TempDir::new().unwrap();
        let builder = SystemPromptBuilder::new(dir.path().to_string_lossy())
            .with_project_prompt(&dir.path().to_string_lossy());
        let prompt = builder.build();
        // Should not contain anything from .piso/system.md (doesn't exist)
        assert!(!prompt.contains("Project-specific"));
    }

    #[test]
    fn date_injected_in_prompt() {
        let builder = SystemPromptBuilder::new("/tmp");
        let prompt = builder.build();
        assert!(prompt.contains("Current date: 20"));
        assert!(prompt.contains("Current working directory: /tmp"));
    }
}
