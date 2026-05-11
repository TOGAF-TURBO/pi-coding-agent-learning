//! 认证管理。

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde_json::Value;

/// 环境变量 → provider 映射。
const ENV_KEY_MAP: &[(&str, &str)] = &[
    ("ANTHROPIC_API_KEY", "anthropic"),
    ("OPENAI_API_KEY", "openai"),
    ("GOOGLE_API_KEY", "google"),
    ("GEMINI_API_KEY", "google"),
    ("DEEPSEEK_API_KEY", "deepseek"),
    ("GROQ_API_KEY", "groq"),
    ("OPENROUTER_API_KEY", "openrouter"),
    ("TOGETHER_API_KEY", "together"),
    ("FIREWORKS_API_KEY", "fireworks"),
    ("MISTRAL_API_KEY", "mistral"),
    ("XAI_API_KEY", "xai"),
];

/// 认证存储 — 从多个来源解析 API Key。
pub struct AuthStorage {
    keys: HashMap<String, String>,
}

impl AuthStorage {
    /// 从环境变量和配置文件加载所有可用的 API Key。
    pub fn load(config_dir: Option<&Path>) -> Result<Self> {
        let mut keys = HashMap::new();

        // 1. 环境变量
        for (var, provider) in ENV_KEY_MAP {
            if let Ok(val) = std::env::var(var) {
                if !val.is_empty() {
                    keys.insert(provider.to_string(), val);
                }
            }
        }

        // 2. auth.json
        if let Some(dir) = config_dir {
            let auth_path = dir.join("auth.json");
            if auth_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&auth_path) {
                    if let Ok(map) = serde_json::from_str::<HashMap<String, String>>(&content) {
                        for (k, v) in map {
                            keys.insert(k, v);
                        }
                    }
                }
            }

            // 3. models.json 中的 apiKey
            let models_path = dir.join("agent").join("models.json");
            if models_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&models_path) {
                    if let Ok(doc) = serde_json::from_str::<Value>(&content) {
                        if let Some(providers) = doc.get("providers").and_then(|p| p.as_object()) {
                            for (name, config) in providers {
                                if let Some(key) = config.get("apiKey").and_then(|v| v.as_str()) {
                                    if !key.is_empty() {
                                        keys.insert(name.clone(), key.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(Self { keys })
    }

    /// 获取指定 provider 的 API Key。
    pub fn get_key(&self, provider: &str) -> Option<&str> {
        self.keys.get(provider).map(|s| s.as_str())
    }

    /// 列出有 API Key 的 provider。
    pub fn available_providers(&self) -> Vec<String> {
        self.keys.keys().cloned().collect()
    }
}
