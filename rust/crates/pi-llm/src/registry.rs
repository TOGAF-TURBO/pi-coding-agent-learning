//! Provider 注册表 — 按名称注册和查找 LLM 驱动。

use std::collections::HashMap;
use std::sync::Arc;

use parking_lot::RwLock;

use crate::driver::LlmDriver;
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
