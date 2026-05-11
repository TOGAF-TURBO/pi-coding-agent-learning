//! 扩展加载器 — 发现和加载扩展。
//!
//! 当前支持 Rust 原生扩展（通过 ExtensionFactory 函数指针）。
//! 未来可扩展为 WASM/JS。

use std::path::Path;

use crate::api::{BasicExtensionApi, ExtensionFactory, HookRegistry};

/// 扩展注册表 — 管理所有已加载的扩展。
pub struct ExtensionLoader {
    factories: Vec<(String, ExtensionFactory)>,
}

impl ExtensionLoader {
    pub fn new() -> Self {
        Self {
            factories: Vec::new(),
        }
    }

    /// 注册一个内置扩展工厂。
    pub fn register(&mut self, name: &str, factory: ExtensionFactory) {
        self.factories.push((name.to_string(), factory));
    }

    /// 从目录扫描扩展（当前为 NOOP，未来扫描 .wasm/.js 文件）。
    pub fn scan_dir(&mut self, _dir: &Path) -> anyhow::Result<()> {
        // TODO: 未来扫描动态库/WASM/JS 文件
        Ok(())
    }

    /// 初始化所有扩展，返回合并的 HookRegistry。
    pub fn load_all(self) -> HookRegistry {
        let mut merged = HookRegistry::default();

        for (name, factory) in self.factories {
            let mut api = BasicExtensionApi::new(&name);
            factory(&mut api);
            let hooks = api.into_hooks();

            // 合并钩子
            merged.on_agent_start.extend(hooks.on_agent_start);
            merged.on_agent_done.extend(hooks.on_agent_done);
            merged.on_tool_call_start.extend(hooks.on_tool_call_start);
            merged.on_tool_call_end.extend(hooks.on_tool_call_end);
            merged.on_model_switched.extend(hooks.on_model_switched);
            merged.on_turn_start.extend(hooks.on_turn_start);
            merged.on_turn_end.extend(hooks.on_turn_end);
            merged.on_message_start.extend(hooks.on_message_start);
            merged.on_message_end.extend(hooks.on_message_end);
            merged
                .on_tool_execution_start
                .extend(hooks.on_tool_execution_start);
            merged
                .on_tool_execution_end
                .extend(hooks.on_tool_execution_end);
            merged.on_session_start.extend(hooks.on_session_start);
            merged.on_session_compact.extend(hooks.on_session_compact);
            merged.on_session_shutdown.extend(hooks.on_session_shutdown);
            merged.on_error.extend(hooks.on_error);
            merged
                .on_before_provider_request
                .extend(hooks.on_before_provider_request);
            merged
                .on_after_provider_response
                .extend(hooks.on_after_provider_response);
            merged.providers.extend(hooks.providers);
            merged.shortcuts.extend(hooks.shortcuts);
            merged.pending_messages.extend(hooks.pending_messages);
            if hooks.session_name.is_some() {
                merged.session_name = hooks.session_name;
            }
            merged.labels.extend(hooks.labels);
            merged.flags.extend(hooks.flags);
            merged.tools.extend(hooks.tools);
            merged.commands.extend(hooks.commands);
            merged.message_renderers.extend(hooks.message_renderers);
            merged.editor_hints.extend(hooks.editor_hints);
        }

        merged
    }
}

impl Default for ExtensionLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_multiple_extensions() {
        let mut loader = ExtensionLoader::new();

        // 扩展 1：注册工具
        loader.register("ext1", |api| {
            api.register_tool(
                "tool1",
                "First tool",
                Box::new(|input| Ok(format!("ext1: {}", input))),
            );
        });

        // 扩展 2：注册钩子 + 工具
        loader.register("ext2", |api| {
            api.on_agent_start(Box::new(|_prompt| {}));
            api.register_tool(
                "tool2",
                "Second tool",
                Box::new(|input| Ok(format!("ext2: {}", input))),
            );
        });

        let hooks = loader.load_all();
        assert_eq!(hooks.tools.len(), 2);
        assert_eq!(hooks.on_agent_start.len(), 1);

        // 验证工具可调用
        let result = (hooks.tools[0].handler)("test");
        assert_eq!(result.unwrap(), "ext1: test");
        let result = (hooks.tools[1].handler)("test");
        assert_eq!(result.unwrap(), "ext2: test");
    }

    #[test]
    fn empty_loader_ok() {
        let loader = ExtensionLoader::new();
        let hooks = loader.load_all();
        assert!(hooks.tools.is_empty());
        assert!(hooks.commands.is_empty());
    }
}
