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
//! ├── sessions/           # 会话存储
//! │   └── <session-id>/
//! │       └── session.jsonl
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

/// 获取会话目录路径：`~/.piso/sessions/`
pub fn session_dir(config: &Config) -> Option<PathBuf> {
    if let Some(dir) = &config.session_dir {
        Some(PathBuf::from(dir))
    } else {
        config_dir().map(|d| d.join("sessions"))
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
