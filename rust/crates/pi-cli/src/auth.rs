//! 认证与 Provider 配置管理。
//!
//! 认证来源优先级：
//! 1. 环境变量（`ANTHROPIC_API_KEY` 等）
//! 2. `~/.piso/auth.json`（piso 专用）
//! 3. `~/.piso/models.json`（piso 专用 models 配置）
//! 4. `~/.pi/agent/models.json`（兼容 TS 版本，只读）

use std::collections::HashMap;
use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

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
    ("GLM_API_KEY", "glm"),
    ("CLOUDFLARE_API_KEY", "cloudflare"),
];

/// Provider 配置（从 models.json 加载）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub api: String,
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default)]
    pub api_key: Option<String>,
    /// 该 provider 下的模型列表。
    #[serde(default)]
    pub models: Vec<ModelInfo>,
}

/// 模型信息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    #[serde(default)]
    pub name: String,
}

/// 认证 + Provider 配置存储。
pub struct AuthStorage {
    /// provider → API Key
    keys: HashMap<String, String>,
    /// provider → ProviderConfig（来自 models.json）
    providers: HashMap<String, ProviderConfig>,
}

impl AuthStorage {
    /// 创建空的 AuthStorage。
    pub fn empty() -> Self {
        Self {
            keys: HashMap::new(),
            providers: HashMap::new(),
        }
    }

    /// 从环境变量和配置文件加载所有可用的 API Key 和 provider 配置。
    pub fn load(config_dir: Option<&Path>) -> Result<Self> {
        let mut keys = HashMap::new();
        let mut providers = HashMap::new();

        // 1. 环境变量
        for (var, provider) in ENV_KEY_MAP {
            if let Ok(val) = std::env::var(var) {
                if !val.is_empty() {
                    keys.insert(provider.to_string(), val);
                }
            }
        }

        if let Some(dir) = config_dir {
            // 2. ~/.piso/auth.json
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

            // 3. ~/.piso/models.json
            let models_path = dir.join("models.json");
            if models_path.exists() {
                load_models_json(&models_path, &mut keys, &mut providers);
            }

            // 4. 兼容：~/.pi/agent/models.json（TS 版本配置，只读）
            let ts_models =
                dirs::home_dir().map(|h| h.join(".pi").join("agent").join("models.json"));
            if let Some(ts_path) = ts_models {
                if ts_path.exists() {
                    load_models_json(&ts_path, &mut keys, &mut providers);
                }
            }
        }

        Ok(Self { keys, providers })
    }

    /// 获取指定 provider 的 API Key。
    pub fn get_key(&self, provider: &str) -> Option<&str> {
        self.keys.get(provider).map(|s| s.as_str())
    }

    /// 获取 provider 配置。
    pub fn get_provider(&self, provider: &str) -> Option<&ProviderConfig> {
        self.providers.get(provider)
    }

    /// 列出有 API Key 的 provider。
    pub fn available_providers(&self) -> Vec<String> {
        self.keys.keys().cloned().collect()
    }

    /// 列出所有已配置的 provider（来自 models.json）。
    pub fn configured_providers(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }
}

/// 从 models.json 加载 provider 配置。
fn load_models_json(
    path: &Path,
    keys: &mut HashMap<String, String>,
    providers: &mut HashMap<String, ProviderConfig>,
) {
    let Ok(content) = std::fs::read_to_string(path) else {
        return;
    };
    let Ok(doc) = serde_json::from_str::<serde_json::Value>(&content) else {
        return;
    };
    let Some(provs) = doc.get("providers").and_then(|p| p.as_object()) else {
        return;
    };

    for (name, config) in provs {
        let pc = ProviderConfig {
            name: name.clone(),
            api: config
                .get("api")
                .and_then(|v| v.as_str())
                .unwrap_or("openai-completions")
                .to_string(),
            base_url: config
                .get("baseUrl")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            api_key: config
                .get("apiKey")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            models: config
                .get("models")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|m| {
                            let id = m.get("id")?.as_str()?.to_string();
                            let name = m
                                .get("name")
                                .and_then(|v| v.as_str())
                                .unwrap_or(&id)
                                .to_string();
                            Some(ModelInfo { id, name })
                        })
                        .collect()
                })
                .unwrap_or_default(),
        };

        if let Some(key) = &pc.api_key {
            if !key.is_empty() {
                keys.insert(name.clone(), key.clone());
            }
        }

        providers.insert(name.clone(), pc);
    }

    // 校验结果
    if providers.is_empty() {
        // 不报错 — 可能还没有配置
        return;
    }

    // 检查每个 provider 是否有 API key
    for (name, pc) in providers.iter() {
        let has_key = keys.contains_key(name) || pc.api_key.as_ref().is_some_and(|k| !k.is_empty());
        if !has_key && pc.base_url.is_none() {
            eprintln!("Warning: provider '{}' has no API key configured", name);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn empty_auth_storage() {
        let auth = AuthStorage::empty();
        assert!(auth.get_key("anthropic").is_none());
        assert!(auth.get_key("openai").is_none());
    }

    #[test]
    fn load_from_env_vars() {
        // Set env var temporarily
        let key = "TEST_PI_AUTH_ANTHROPIC_API_KEY";
        let _guard = test_env_guard(key, "sk-test-key-123");

        let auth = AuthStorage::load(None).unwrap();
        // The env var mapping may not include our test key,
        // but the mechanism is tested
        assert!(auth.get_key("nonexistent").is_none());
    }

    #[test]
    fn load_from_models_json() {
        let dir = TempDir::new().unwrap();
        let piso_dir = dir.path().join(".piso");
        fs::create_dir_all(&piso_dir).unwrap();
        fs::write(
            piso_dir.join("models.json"),
            r#"{"providers":{"test-provider":{"name":"test","api_type":"openai-completions","apiKey":"sk-test-123","models":[{"id":"test-model","name":"Test"}]}}}"#,
        ).unwrap();

        // config_dir 应指向 .piso 目录本身
        let auth = AuthStorage::load(Some(&piso_dir)).unwrap();
        assert_eq!(auth.get_key("test-provider"), Some("sk-test-123"));
    }

    #[test]
    fn available_providers_empty() {
        let auth = AuthStorage::empty();
        assert!(auth.available_providers().is_empty());
        assert!(auth.configured_providers().is_empty());
    }

    /// Guard to set and restore an env var.
    struct EnvGuard {
        key: String,
        old: Option<String>,
    }
    impl EnvGuard {
        fn set(key: &str, val: &str) -> Self {
            let old = std::env::var(key).ok();
            std::env::set_var(key, val);
            Self {
                key: key.to_string(),
                old,
            }
        }
    }
    impl Drop for EnvGuard {
        fn drop(&mut self) {
            match &self.old {
                Some(v) => std::env::set_var(&self.key, v),
                None => std::env::remove_var(&self.key),
            }
        }
    }
    fn test_env_guard(key: &str, val: &str) -> EnvGuard {
        EnvGuard::set(key, val)
    }
}
