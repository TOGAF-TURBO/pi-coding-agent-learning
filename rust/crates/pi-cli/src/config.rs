//! 配置加载。
//!
//! piso 使用独立的 `~/.piso/` 配置目录，不与 TS 版本的 `~/.pi/` 冲突。
//!
//! 目录结构：
//! ```text
//! ~/.piso/
//! ├── settings.json       # 全局配置
//! ├── auth.json           # API key 存储
//! ├── models.json         # Provider + model 定义
//! ├── sessions/           # 会话存储（集中，不污染项目）
//! │   └── --home-user-project--/   # CWD 编码为安全目录名
//! │       └── <session-id>.jsonl
//! └── skills/             # 全局技能
//!     └── <skill-name>/
//!         └── SKILL.md
//!
//! <project>/.piso/
//! ├── settings.json       # 项目级配置（覆盖全局）
//! └── skills/             # 项目级技能
//! ```

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// piso 配置目录名。
pub const CONFIG_DIR_NAME: &str = ".piso";

/// 应用配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub model: Option<String>,
    pub provider: Option<String>,
    pub thinking: Option<String>,
    pub max_tokens: u32,
    pub session_dir: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            model: None,
            provider: None,
            thinking: None,
            max_tokens: 16384,
            session_dir: None,
        }
    }
}

/// 加载配置：全局 ~/.piso/settings.json + 项目 .piso/settings.json。
pub fn load_config(project_dir: Option<&Path>) -> Config {
    let mut config = Config::default();

    // 全局配置
    if let Some(home) = dirs::home_dir() {
        let global_path = home.join(CONFIG_DIR_NAME).join("settings.json");
        merge_from_file(&mut config, &global_path);
    }

    // 项目级配置（覆盖全局）
    if let Some(dir) = project_dir {
        let local_path = dir.join(CONFIG_DIR_NAME).join("settings.json");
        merge_from_file(&mut config, &local_path);
    }

    config
}

fn merge_from_file(config: &mut Config, path: &Path) {
    if let Ok(content) = std::fs::read_to_string(path) {
        if let Ok(overrides) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(v) = overrides.get("model").and_then(|v| v.as_str()) {
                config.model = Some(v.to_string());
            }
            if let Some(v) = overrides.get("provider").and_then(|v| v.as_str()) {
                config.provider = Some(v.to_string());
            }
            if let Some(v) = overrides.get("thinking").and_then(|v| v.as_str()) {
                config.thinking = Some(v.to_string());
            }
        }
    }
}

/// 获取 piso 全局配置目录：`~/.piso/`
pub fn config_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(CONFIG_DIR_NAME))
}

/// 获取会话目录路径：`~/.piso/sessions/--cwd-safe-path--/`
///
/// 与 TS 版 pi 兼容的编码方式：cwd 去前导 `/`，路径分隔符替换为 `-`。
pub fn session_dir_for_cwd(config: &Config, cwd: &Path) -> PathBuf {
    if let Some(dir) = &config.session_dir {
        PathBuf::from(dir)
    } else {
        let safe = cwd
            .to_string_lossy()
            .trim_start_matches('/')
            .replace(['/', '\\', ':'], "-");
        let safe_dir = format!("--{safe}--");
        config_dir()
            .map(|d| d.join("sessions").join(safe_dir))
            .unwrap_or_else(|| PathBuf::from(".piso/sessions"))
    }
}

/// 获取全局技能目录：`~/.piso/skills/`
pub fn skills_dir() -> Option<PathBuf> {
    config_dir().map(|d| d.join("skills"))
}

/// 获取项目级技能目录：`<project>/.piso/skills/`
pub fn project_skills_dir(project_dir: &Path) -> PathBuf {
    project_dir.join(CONFIG_DIR_NAME).join("skills")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn default_config() {
        let config = Config::default();
        assert!(config.model.is_none());
        assert!(config.provider.is_none());
        assert_eq!(config.max_tokens, 16384);
        assert!(config.session_dir.is_none());
    }

    #[test]
    fn load_config_merges_files() {
        let dir = TempDir::new().unwrap();
        let piso_dir = dir.path().join(".piso");
        fs::create_dir_all(&piso_dir).unwrap();
        fs::write(
            piso_dir.join("settings.json"),
            r#"{"model":"glm-5.1","provider":"glm","max_tokens":8192}"#,
        )
        .unwrap();

        let config = load_config(Some(dir.path()));
        assert_eq!(config.model.as_deref(), Some("glm-5.1"));
        assert_eq!(config.provider.as_deref(), Some("glm"));
    }

    #[test]
    fn session_dir_encoding() {
        let config = Config::default();
        let cwd = Path::new("/home/user/my-project");
        let dir = session_dir_for_cwd(&config, cwd);
        let dir_str = dir.to_string_lossy();
        assert!(dir_str.contains("--home-user-my-project--"));
    }

    #[test]
    fn session_dir_custom_override() {
        let config = Config {
            session_dir: Some("/custom/sessions".to_string()),
            ..Config::default()
        };
        let cwd = Path::new("/home/user/project");
        let dir = session_dir_for_cwd(&config, cwd);
        assert_eq!(dir, PathBuf::from("/custom/sessions"));
    }

    #[test]
    fn project_skills_dir_path() {
        let dir = project_skills_dir(Path::new("/project"));
        assert_eq!(dir, PathBuf::from("/project/.piso/skills"));
    }

    #[test]
    fn config_serialization_roundtrip() {
        let config = Config {
            model: Some("claude-sonnet-4".to_string()),
            provider: Some("anthropic".to_string()),
            thinking: Some("high".to_string()),
            max_tokens: 32768,
            session_dir: None,
        };
        let json = serde_json::to_string(&config).unwrap();
        let parsed: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.model, config.model);
        assert_eq!(parsed.provider, config.provider);
        assert_eq!(parsed.max_tokens, config.max_tokens);
    }
}
