//! 配置加载。

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

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

/// 加载配置：全局 ~/.pi/settings.json + 项目 .pi/settings.json。
pub fn load_config(project_dir: Option<&Path>) -> Config {
    let mut config = Config::default();

    // 全局配置
    if let Some(home) = dirs::home_dir() {
        let global_path = home.join(".pi").join("settings.json");
        merge_from_file(&mut config, &global_path);
    }

    // 项目级配置（覆盖全局）
    if let Some(dir) = project_dir {
        let local_path = dir.join(".pi").join("settings.json");
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

/// 获取配置目录路径。
pub fn config_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".pi"))
}

/// 获取会话目录路径。
pub fn session_dir(config: &Config) -> Option<PathBuf> {
    if let Some(dir) = &config.session_dir {
        Some(PathBuf::from(dir))
    } else if let Some(home) = dirs::home_dir() {
        Some(home.join(".pi").join("agent").join("sessions"))
    } else {
        None
    }
}
