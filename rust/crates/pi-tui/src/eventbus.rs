//! EventBus — 异步事件总线。
//!
//! Agent、TUI、Extension 之间的解耦通信机制。
//! 发布者通过 emit() 发送事件，订阅者通过 subscribe() 接收。
//!
//! 使用 tokio::broadcast channel 实现，支持多订阅者。

use std::fmt;
use std::sync::Arc;

use tokio::sync::broadcast;

/// 事件类型。
#[derive(Debug, Clone)]
pub enum Event {
    /// Agent 开始处理消息。
    AgentStart { prompt: String },
    /// Agent 完成处理。
    AgentDone { output: String },
    /// Agent 出错。
    AgentError { error: String },
    /// LLM 文本 delta。
    TextDelta { text: String },
    /// 思考 delta。
    ThinkingDelta { thinking: String },
    /// 工具调用开始。
    ToolCallStart { name: String, call_id: String },
    /// 工具调用完成。
    ToolCallEnd {
        name: String,
        output: String,
        is_error: bool,
    },
    /// Token 用量更新。
    Usage { input: u32, output: u32 },
    /// 会话切换。
    SessionSwitched { session_id: String },
    /// 模型切换。
    ModelSwitched { provider: String, model: String },
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Event::AgentStart { prompt } => write!(f, "AgentStart({} chars)", prompt.len()),
            Event::AgentDone { .. } => write!(f, "AgentDone"),
            Event::AgentError { error } => write!(f, "AgentError({})", error),
            Event::TextDelta { text } => write!(f, "TextDelta({} chars)", text.len()),
            Event::ThinkingDelta { .. } => write!(f, "ThinkingDelta"),
            Event::ToolCallStart { name, .. } => write!(f, "ToolCallStart({})", name),
            Event::ToolCallEnd { name, is_error, .. } => {
                write!(f, "ToolCallEnd({}, err={})", name, is_error)
            }
            Event::Usage { input, output } => write!(f, "Usage({}/{})", input, output),
            Event::SessionSwitched { session_id } => write!(f, "SessionSwitched({})", session_id),
            Event::ModelSwitched { provider, model } => {
                write!(f, "ModelSwitched({}/{})", provider, model)
            }
        }
    }
}

/// 事件总线。
#[derive(Clone)]
pub struct EventBus {
    tx: Arc<broadcast::Sender<Event>>,
}

impl EventBus {
    /// 创建新的事件总线。
    ///
    /// `capacity` — broadcast channel 缓冲区大小。
    pub fn new(capacity: usize) -> Self {
        let (tx, _rx) = broadcast::channel(capacity);
        Self { tx: Arc::new(tx) }
    }

    /// 发送事件。
    pub fn emit(&self, event: Event) {
        // 如果没有订阅者，丢弃事件（broadcast::send 返回 Err 但不是致命错误）
        let _ = self.tx.send(event);
    }

    /// 订阅事件流。
    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.tx.subscribe()
    }

    /// 订阅者数量。
    pub fn subscriber_count(&self) -> usize {
        self.tx.receiver_count()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new(256)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emit_and_receive() {
        let bus = EventBus::new(16);
        let mut rx = bus.subscribe();

        bus.emit(Event::AgentStart {
            prompt: "hello".into(),
        });

        let event = rx.try_recv().unwrap();
        match event {
            Event::AgentStart { prompt } => assert_eq!(prompt, "hello"),
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn multiple_subscribers() {
        let bus = EventBus::new(16);
        let mut rx1 = bus.subscribe();
        let mut rx2 = bus.subscribe();

        bus.emit(Event::Usage {
            input: 10,
            output: 20,
        });

        let e1 = rx1.try_recv().unwrap();
        let e2 = rx2.try_recv().unwrap();

        assert!(matches!(
            e1,
            Event::Usage {
                input: 10,
                output: 20
            }
        ));
        assert!(matches!(
            e2,
            Event::Usage {
                input: 10,
                output: 20
            }
        ));
        assert_eq!(bus.subscriber_count(), 2);
    }

    #[test]
    fn no_subscribers_ok() {
        let bus = EventBus::new(16);
        // 不应该 panic
        bus.emit(Event::AgentDone {
            output: "ok".into(),
        });
    }

    #[test]
    fn event_display() {
        let e = Event::ToolCallStart {
            name: "bash".into(),
            call_id: "123".into(),
        };
        assert_eq!(format!("{}", e), "ToolCallStart(bash)");
    }

    #[test]
    fn default_capacity() {
        let bus = EventBus::default();
        let mut rx = bus.subscribe();
        bus.emit(Event::AgentError {
            error: "test".into(),
        });
        assert!(rx.try_recv().is_ok());
    }
}
