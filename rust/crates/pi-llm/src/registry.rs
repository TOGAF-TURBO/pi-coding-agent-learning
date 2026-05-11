//! Provider 注册表 — 按名称注册和查找 LLM 驱动。

use std::collections::HashMap;
use std::sync::Arc;

use parking_lot::RwLock;

use crate::driver::LlmDriver;
use crate::gemini::GeminiDriver;
use crate::openai::OpenAiDriver;
use crate::providers::AnthropicDriver;

/// Provider 注册表。
pub struct ProviderRegistry {
    drivers: RwLock<HashMap<String, Arc<dyn LlmDriver>>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        let mut drivers = HashMap::new();
        // 注册内置 provider
        let anthropic: Arc<dyn LlmDriver> = Arc::new(AnthropicDriver::new());
        drivers.insert("anthropic".to_string(), anthropic);

        let openai: Arc<dyn LlmDriver> = Arc::new(OpenAiDriver::new());
        drivers.insert("openai".to_string(), openai);

        // OpenAI-compatible providers（共享 OpenAI driver）
        for name in &["glm", "deepseek", "groq", "openrouter", "together", "fireworks", "mistral", "xai"] {
            drivers.insert((*name).to_string(), Arc::new(OpenAiDriver::new()));
        }

        // Google Gemini
        let google: Arc<dyn LlmDriver> = Arc::new(GeminiDriver::new());
        drivers.insert("google".to_string(), google);
        drivers.insert("gemini".to_string(), Arc::new(GeminiDriver::new()));

        Self {
            drivers: RwLock::new(drivers),
        }
    }

    /// 注册一个 provider。
    pub fn register(&self, name: impl Into<String>, driver: Arc<dyn LlmDriver>) {
        self.drivers.write().insert(name.into(), driver);
    }

    /// 按 provider 名称查找驱动。
    pub fn get(&self, name: &str) -> Option<Arc<dyn LlmDriver>> {
        self.drivers.read().get(name).cloned()
    }

    /// 列出所有已注册的 provider 名称。
    pub fn names(&self) -> Vec<String> {
        self.drivers.read().keys().cloned().collect()
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_has_builtin_providers() {
        let reg = ProviderRegistry::new();
        let names = reg.names();
        assert!(names.contains(&"anthropic".to_string()));
        assert!(names.contains(&"openai".to_string()));
        assert!(names.contains(&"google".to_string()));
        assert!(names.contains(&"glm".to_string()));
        assert!(names.contains(&"deepseek".to_string()));
    }

    #[test]
    fn get_existing_provider() {
        let reg = ProviderRegistry::new();
        let driver = reg.get("anthropic");
        assert!(driver.is_some());
        assert_eq!(driver.unwrap().name(), "anthropic");
    }

    #[test]
    fn get_nonexistent_provider() {
        let reg = ProviderRegistry::new();
        assert!(reg.get("nonexistent").is_none());
    }

    #[test]
    fn register_custom_provider() {
        let reg = ProviderRegistry::new();
        reg.register("custom", Arc::new(OpenAiDriver::new()));
        assert!(reg.get("custom").is_some());
        assert_eq!(reg.get("custom").unwrap().name(), "openai");
    }

    #[test]
    fn openai_compatible_providers() {
        let reg = ProviderRegistry::new();
        for name in &["glm", "deepseek", "groq", "openrouter", "mistral"] {
            let driver = reg.get(name);
            assert!(driver.is_some(), "missing provider: {}", name);
            assert_eq!(driver.unwrap().name(), "openai");
        }
    }
}
