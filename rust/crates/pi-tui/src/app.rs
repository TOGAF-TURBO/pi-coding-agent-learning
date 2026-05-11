//! App 状态 — TUI 和 Agent 循环共享的运行时状态。
//!
//! 通过 `tokio::watch` 在 agent task 和 TUI task 之间同步。


use parking_lot::RwLock;

/// Agent 运行状态。
#[derive(Debug, Clone)]
pub enum AgentState {
    /// 空闲，等待用户输入。
    Idle,
    /// 正在等待 LLM 响应。
    Thinking,
    /// 正在流式接收文本。
    Streaming,
    /// 正在执行工具。
    ToolRunning { name: String },
    /// 出错。
    Error(String),
}

/// 消息角色。
#[derive(Debug, Clone)]
pub enum ChatRole {
    User,
    Assistant,
    System,
    Tool { name: String, is_error: bool },
}

/// 聊天消息条目。
#[derive(Debug, Clone)]
pub struct ChatEntry {
    pub role: ChatRole,
    pub content: String,
    /// 是否正在流式接收（最后一行可能未完成）。
    pub streaming: bool,
}

/// 页脚状态数据。
#[derive(Debug, Clone)]
pub struct FooterData {
    pub model: String,
    pub provider: String,
    pub state: AgentState,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub duration_secs: u64,
}

/// 共享的 App 状态。
pub struct AppState {
    /// 聊天记录。
    pub entries: RwLock<Vec<ChatEntry>>,
    /// 页脚数据。
    pub footer: RwLock<FooterData>,
}

impl AppState {
    pub fn new(model: &str, provider: &str) -> Self {
        Self {
            entries: RwLock::new(Vec::new()),
            footer: RwLock::new(FooterData {
                model: model.to_string(),
                provider: provider.to_string(),
                state: AgentState::Idle,
                input_tokens: 0,
                output_tokens: 0,
                duration_secs: 0,
            }),
        }
    }

    /// 追加用户消息。
    pub fn push_user(&self, text: &str) {
        self.entries.write().push(ChatEntry {
            role: ChatRole::User,
            content: text.to_string(),
            streaming: false,
        });
    }

    /// 追加或更新助手流式文本。
    pub fn push_assistant_delta(&self, text: &str) {
        let mut entries = self.entries.write();
        // 如果最后一条是 streaming 的 assistant，追加
        if let Some(last) = entries.last_mut() {
            if matches!(last.role, ChatRole::Assistant) && last.streaming {
                last.content.push_str(text);
                return;
            }
        }
        entries.push(ChatEntry {
            role: ChatRole::Assistant,
            content: text.to_string(),
            streaming: true,
        });
    }

    /// 完成当前 streaming 助手消息。
    pub fn finish_assistant(&self) {
        if let Some(last) = self.entries.write().last_mut() {
            last.streaming = false;
        }
    }

    /// 追加工具结果。
    pub fn push_tool_result(&self, name: &str, output: &str, is_error: bool) {
        self.entries.write().push(ChatEntry {
            role: ChatRole::Tool {
                name: name.to_string(),
                is_error,
            },
            content: output.to_string(),
            streaming: false,
        });
    }

    /// 更新 agent 状态。
    pub fn set_state(&self, state: AgentState) {
        self.footer.write().state = state;
    }

    /// 获取当前 agent 状态。
    pub fn agent_state(&self) -> AgentState {
        self.footer.read().state.clone()
    }

    /// 更新 token 用量。
    pub fn set_usage(&self, input: u32, output: u32) {
        let mut f = self.footer.write();
        f.input_tokens = input;
        f.output_tokens = output;
    }
}
