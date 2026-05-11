//! ExtensionApi — 扩展系统的核心 trait。
//!
//! 对应 TS 版的 `ExtensionAPI` interface，但精简为核心功能：
//! - 事件订阅（on_start, on_tool_call, on_message 等）
//! - 工具注册
//! - 命令注册
//!
//! 扩展通过 `ExtensionFactory` 工厂函数注册，类似于 TS 版的
//! `export default function(pi: ExtensionAPI) { ... }`。


/// 扩展工厂函数类型。
/// 接收 ExtensionApi 引用，在函数内注册钩子/工具/命令。
pub type ExtensionFactory = fn(&mut dyn ExtensionApi);

/// 扩展 API trait — 暴露给扩展的核心接口。
pub trait ExtensionApi {
    /// 注册扩展名称（用于日志和调试）。
    fn name(&self) -> &str;

    /// 订阅 agent 启动事件。
    fn on_agent_start(&mut self, handler: Box<dyn Fn(&str) + Send + Sync>);

    /// 订阅 agent 完成事件。
    fn on_agent_done(&mut self, handler: Box<dyn Fn(&str) + Send + Sync>);

    /// 订阅工具调用开始事件。
    fn on_tool_call_start(&mut self, handler: Box<dyn Fn(&str, &str) + Send + Sync>);

    /// 订阅工具调用完成事件。
    fn on_tool_call_end(&mut self, handler: Box<dyn Fn(&str, &str, bool) + Send + Sync>);

    /// 订阅模型切换事件。
    fn on_model_switched(&mut self, handler: Box<dyn Fn(&str, &str) + Send + Sync>);

    /// 订阅 turn 开始事件。
    fn on_turn_start(&mut self, handler: Box<dyn Fn(&str) + Send + Sync>);

    /// 订阅 turn 结束事件。
    fn on_turn_end(&mut self, handler: Box<dyn Fn(&str) + Send + Sync>);

    /// 订阅消息开始事件。
    fn on_message_start(&mut self, handler: Box<dyn Fn(&str) + Send + Sync>);

    /// 订阅消息结束事件。
    fn on_message_end(&mut self, handler: Box<dyn Fn(&str) + Send + Sync>);

    /// 订阅工具执行开始事件（含参数）。
    fn on_tool_execution_start(&mut self, handler: Box<dyn Fn(&str, &str) + Send + Sync>);

    /// 订阅工具执行结束事件。
    fn on_tool_execution_end(&mut self, handler: Box<dyn Fn(&str, &str, bool) + Send + Sync>);

    /// 订阅会话开始事件。
    fn on_session_start(&mut self, handler: Box<dyn Fn(&str) + Send + Sync>);

    /// 订阅会话压缩事件。
    fn on_session_compact(&mut self, handler: Box<dyn Fn(usize) + Send + Sync>);

    /// 订阅会话关闭事件。
    fn on_session_shutdown(&mut self, handler: Box<dyn Fn() + Send + Sync>);

    /// 订阅错误事件。
    fn on_error(&mut self, handler: Box<dyn Fn(&str) + Send + Sync>);

    /// 订阅 provider 请求前事件。
    fn on_before_provider_request(&mut self, handler: Box<dyn Fn(&str, &str) + Send + Sync>);

    /// 订阅 provider 响应后事件。
    fn on_after_provider_response(&mut self, handler: Box<dyn Fn(&str, u64, u64) + Send + Sync>);

    /// 注册自定义工具。
    /// 参数：(name, description, handler)
    /// handler 接收 JSON 输入，返回 JSON 输出。
    fn register_tool(
        &mut self,
        name: &str,
        description: &str,
        handler: Box<dyn Fn(&str) -> Result<String, String> + Send + Sync>,
    );

    /// 注册 slash 命令。
    /// 参数：(name, description, handler)
    /// handler 接收命令参数字符串。
    fn register_command(
        &mut self,
        name: &str,
        description: &str,
        handler: Box<dyn Fn(&str) -> Result<(), String> + Send + Sync>,
    );

    /// 注册自定义消息渲染器。
    /// handler 接收 (role, content)，返回格式化后的 String。
    /// 返回 None 表示使用默认渲染。
    fn register_message_renderer(
        &mut self,
        handler: Box<dyn Fn(&str, &str) -> Option<String> + Send + Sync>,
    );

    /// 注册编辑器组件 — 注入自定义提示文本到编辑区。
    /// handler 返回要显示的提示字符串（如状态信息）。
    fn register_editor_hint(
        &mut self,
        handler: Box<dyn Fn() -> String + Send + Sync>,
    );

    /// 获取扩展存储（持久化键值对）。
    fn store(&self) -> &ExtensionStore;

    // ========================================================================
    // 高级 API (#21)
    // ========================================================================

    /// 注册自定义 LLM provider。
    /// provider_id: 唯一标识符 (e.g. "my-proxy")
    /// base_url: API endpoint
    /// api_key: 认证密钥
    /// api_type: 协议类型 (e.g. "openai-completions", "anthropic-messages")
    fn register_provider(
        &mut self,
        provider_id: &str,
        base_url: &str,
        api_key: &str,
        api_type: &str,
        models: Vec<ProviderModelInfo>,
    );

    /// 注册快捷键。
    /// key: 快捷键描述 (e.g. "ctrl+alt+p")
    /// description: 功能描述
    /// handler: 回调
    fn register_shortcut(
        &mut self,
        key: &str,
        description: &str,
        handler: Box<dyn Fn() + Send + Sync>,
    );

    /// 注入消息到会话。
    fn send_message(&mut self, content: &str);

    /// 注入用户消息到会话。
    fn send_user_message(&mut self, content: &str);

    /// 设置会话显示名。
    fn set_session_name(&mut self, name: &str);

    /// 设置标签（用于分类）。
    fn set_label(&mut self, key: &str, value: &str);
}

/// 扩展持久化存储。
#[derive(Debug, Clone, Default)]
pub struct ExtensionStore {
    /// 扩展名 → 键值对。
    data: std::collections::HashMap<String, String>,
}

impl ExtensionStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.data.get(key).map(|s| s.as_str())
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.data.insert(key.to_string(), value.to_string());
    }

    pub fn remove(&mut self, key: &str) {
        self.data.remove(key);
    }
}

/// 钩子处理器集合 — 存储所有注册的回调。
#[derive(Default)]
pub struct HookRegistry {
    pub on_agent_start: Vec<Box<dyn Fn(&str) + Send + Sync>>,
    pub on_agent_done: Vec<Box<dyn Fn(&str) + Send + Sync>>,
    pub on_tool_call_start: Vec<Box<dyn Fn(&str, &str) + Send + Sync>>,
    pub on_tool_call_end: Vec<Box<dyn Fn(&str, &str, bool) + Send + Sync>>,
    pub on_model_switched: Vec<Box<dyn Fn(&str, &str) + Send + Sync>>,
    pub on_turn_start: Vec<Box<dyn Fn(&str) + Send + Sync>>,
    pub on_turn_end: Vec<Box<dyn Fn(&str) + Send + Sync>>,
    pub on_message_start: Vec<Box<dyn Fn(&str) + Send + Sync>>,
    pub on_message_end: Vec<Box<dyn Fn(&str) + Send + Sync>>,
    pub on_tool_execution_start: Vec<Box<dyn Fn(&str, &str) + Send + Sync>>,
    pub on_tool_execution_end: Vec<Box<dyn Fn(&str, &str, bool) + Send + Sync>>,
    pub on_session_start: Vec<Box<dyn Fn(&str) + Send + Sync>>,
    pub on_session_compact: Vec<Box<dyn Fn(usize) + Send + Sync>>,
    pub on_session_shutdown: Vec<Box<dyn Fn() + Send + Sync>>,
    pub on_error: Vec<Box<dyn Fn(&str) + Send + Sync>>,
    pub on_before_provider_request: Vec<Box<dyn Fn(&str, &str) + Send + Sync>>,
    pub on_after_provider_response: Vec<Box<dyn Fn(&str, u64, u64) + Send + Sync>>,
    pub tools: Vec<ToolEntry>,
    pub commands: Vec<CommandEntry>,
    pub message_renderers: Vec<Box<dyn Fn(&str, &str) -> Option<String> + Send + Sync>>,
    pub editor_hints: Vec<Box<dyn Fn() -> String + Send + Sync>>,
    pub providers: Vec<ProviderEntry>,
    pub shortcuts: Vec<ShortcutEntry>,
    pub pending_messages: Vec<(String, String)>, // (role, content)
    pub session_name: Option<String>,
    pub labels: Vec<(String, String)>,
}

/// 注册的工具条目。
pub struct ToolEntry {
    pub name: String,
    pub description: String,
    pub handler: Box<dyn Fn(&str) -> Result<String, String> + Send + Sync>,
}

/// 注册的命令条目。
pub struct CommandEntry {
    pub name: String,
    pub description: String,
    pub handler: Box<dyn Fn(&str) -> Result<(), String> + Send + Sync>,
}

/// Provider 模型信息。
#[derive(Debug, Clone)]
pub struct ProviderModelInfo {
    pub id: String,
    pub name: String,
    pub context_window: Option<usize>,
}

/// 注册的 provider 条目。
pub struct ProviderEntry {
    pub provider_id: String,
    pub base_url: String,
    pub api_key: String,
    pub api_type: String,
    pub models: Vec<ProviderModelInfo>,
}

/// 注册的快捷键条目。
pub struct ShortcutEntry {
    pub key: String,
    pub description: String,
    pub handler: Box<dyn Fn() + Send + Sync>,
}

/// 基础 ExtensionApi 实现。
pub struct BasicExtensionApi {
    name: String,
    hooks: HookRegistry,
    store: ExtensionStore,
}

impl BasicExtensionApi {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            hooks: HookRegistry::default(),
            store: ExtensionStore::new(),
        }
    }

    /// 获取所有注册的钩子。
    pub fn into_hooks(self) -> HookRegistry {
        self.hooks
    }
}

impl ExtensionApi for BasicExtensionApi {
    fn name(&self) -> &str {
        &self.name
    }

    fn on_agent_start(&mut self, handler: Box<dyn Fn(&str) + Send + Sync>) {
        self.hooks.on_agent_start.push(handler);
    }

    fn on_agent_done(&mut self, handler: Box<dyn Fn(&str) + Send + Sync>) {
        self.hooks.on_agent_done.push(handler);
    }

    fn on_tool_call_start(&mut self, handler: Box<dyn Fn(&str, &str) + Send + Sync>) {
        self.hooks.on_tool_call_start.push(handler);
    }

    fn on_tool_call_end(&mut self, handler: Box<dyn Fn(&str, &str, bool) + Send + Sync>) {
        self.hooks.on_tool_call_end.push(handler);
    }

    fn on_model_switched(&mut self, handler: Box<dyn Fn(&str, &str) + Send + Sync>) {
        self.hooks.on_model_switched.push(handler);
    }

    fn on_turn_start(&mut self, handler: Box<dyn Fn(&str) + Send + Sync>) {
        self.hooks.on_turn_start.push(handler);
    }

    fn on_turn_end(&mut self, handler: Box<dyn Fn(&str) + Send + Sync>) {
        self.hooks.on_turn_end.push(handler);
    }

    fn on_message_start(&mut self, handler: Box<dyn Fn(&str) + Send + Sync>) {
        self.hooks.on_message_start.push(handler);
    }

    fn on_message_end(&mut self, handler: Box<dyn Fn(&str) + Send + Sync>) {
        self.hooks.on_message_end.push(handler);
    }

    fn on_tool_execution_start(&mut self, handler: Box<dyn Fn(&str, &str) + Send + Sync>) {
        self.hooks.on_tool_execution_start.push(handler);
    }

    fn on_tool_execution_end(&mut self, handler: Box<dyn Fn(&str, &str, bool) + Send + Sync>) {
        self.hooks.on_tool_execution_end.push(handler);
    }

    fn on_session_start(&mut self, handler: Box<dyn Fn(&str) + Send + Sync>) {
        self.hooks.on_session_start.push(handler);
    }

    fn on_session_compact(&mut self, handler: Box<dyn Fn(usize) + Send + Sync>) {
        self.hooks.on_session_compact.push(handler);
    }

    fn on_session_shutdown(&mut self, handler: Box<dyn Fn() + Send + Sync>) {
        self.hooks.on_session_shutdown.push(handler);
    }

    fn on_error(&mut self, handler: Box<dyn Fn(&str) + Send + Sync>) {
        self.hooks.on_error.push(handler);
    }

    fn on_before_provider_request(&mut self, handler: Box<dyn Fn(&str, &str) + Send + Sync>) {
        self.hooks.on_before_provider_request.push(handler);
    }

    fn on_after_provider_response(&mut self, handler: Box<dyn Fn(&str, u64, u64) + Send + Sync>) {
        self.hooks.on_after_provider_response.push(handler);
    }

    fn register_tool(
        &mut self,
        name: &str,
        description: &str,
        handler: Box<dyn Fn(&str) -> Result<String, String> + Send + Sync>,
    ) {
        self.hooks.tools.push(ToolEntry {
            name: name.to_string(),
            description: description.to_string(),
            handler,
        });
    }

    fn register_command(
        &mut self,
        name: &str,
        description: &str,
        handler: Box<dyn Fn(&str) -> Result<(), String> + Send + Sync>,
    ) {
        self.hooks.commands.push(CommandEntry {
            name: name.to_string(),
            description: description.to_string(),
            handler,
        });
    }

    fn register_message_renderer(
        &mut self,
        handler: Box<dyn Fn(&str, &str) -> Option<String> + Send + Sync>,
    ) {
        self.hooks.message_renderers.push(handler);
    }

    fn register_editor_hint(
        &mut self,
        handler: Box<dyn Fn() -> String + Send + Sync>,
    ) {
        self.hooks.editor_hints.push(handler);
    }

    fn store(&self) -> &ExtensionStore {
        &self.store
    }

    fn register_provider(
        &mut self,
        provider_id: &str,
        base_url: &str,
        api_key: &str,
        api_type: &str,
        models: Vec<ProviderModelInfo>,
    ) {
        self.hooks.providers.push(ProviderEntry {
            provider_id: provider_id.to_string(),
            base_url: base_url.to_string(),
            api_key: api_key.to_string(),
            api_type: api_type.to_string(),
            models,
        });
    }

    fn register_shortcut(
        &mut self,
        key: &str,
        description: &str,
        handler: Box<dyn Fn() + Send + Sync>,
    ) {
        self.hooks.shortcuts.push(ShortcutEntry {
            key: key.to_string(),
            description: description.to_string(),
            handler,
        });
    }

    fn send_message(&mut self, content: &str) {
        self.hooks.pending_messages.push(("assistant".to_string(), content.to_string()));
    }

    fn send_user_message(&mut self, content: &str) {
        self.hooks.pending_messages.push(("user".to_string(), content.to_string()));
    }

    fn set_session_name(&mut self, name: &str) {
        self.hooks.session_name = Some(name.to_string());
    }

    fn set_label(&mut self, key: &str, value: &str) {
        self.hooks.labels.push((key.to_string(), value.to_string()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_and_invoke_hooks() {
        let mut api = BasicExtensionApi::new("test");
        api.on_agent_start(Box::new(|prompt| {
            assert_eq!(prompt, "hello");
        }));
        api.on_tool_call_end(Box::new(|name, _output, is_error| {
            assert_eq!(name, "bash");
            assert!(!is_error);
        }));

        let hooks = api.into_hooks();
        assert_eq!(hooks.on_agent_start.len(), 1);
        assert_eq!(hooks.on_tool_call_end.len(), 1);

        // 调用钩子
        (hooks.on_agent_start[0])("hello");
        (hooks.on_tool_call_end[0])("bash", "ok", false);
    }

    #[test]
    fn register_tool_and_command() {
        let mut api = BasicExtensionApi::new("test-ext");

        api.register_tool(
            "my_tool",
            "A test tool",
            Box::new(|input| Ok(format!("processed: {}", input))),
        );

        api.register_command(
            "hello",
            "Say hello",
            Box::new(|args| {
                if args.is_empty() {
                    Err("missing args".into())
                } else {
                    Ok(())
                }
            }),
        );

        let hooks = api.into_hooks();
        assert_eq!(hooks.tools.len(), 1);
        assert_eq!(hooks.tools[0].name, "my_tool");
        let result = (hooks.tools[0].handler)("{\"a\":1}");
        assert_eq!(result.unwrap(), "processed: {\"a\":1}");

        assert_eq!(hooks.commands.len(), 1);
        assert!((hooks.commands[0].handler)("world").is_ok());
        assert!((hooks.commands[0].handler)("").is_err());
    }

    #[test]
    fn extension_store() {
        let mut store = ExtensionStore::new();
        assert!(store.get("key").is_none());
        store.set("key", "value");
        assert_eq!(store.get("key"), Some("value"));
        store.remove("key");
        assert!(store.get("key").is_none());
    }
}
