//! 流式工具 — AssistantMessageEventStream 的异步辅助。

// TODO: 高级事件流封装

#[cfg(test)]
mod tests {
    use crate::driver::StreamEvent;

    #[test]
    fn stream_event_serialization() {
        let event = StreamEvent::TextDelta { text: "hello".to_string() };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("text_delta"));
        assert!(json.contains("hello"));
    }

    #[test]
    fn stream_event_thinking_delta() {
        let event = StreamEvent::ThinkingDelta { thinking: "deep thought".to_string() };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("thinking_delta"));
    }

    #[test]
    fn stream_event_tool_call() {
        let event = StreamEvent::ToolCallStart {
            id: "call_123".to_string(),
            name: "bash".to_string(),
            index: 0,
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("tool_call_start"));
        assert!(json.contains("bash"));
    }

    #[test]
    fn stream_event_usage() {
        let usage = pi_types::message::Usage {
            input_tokens: 100,
            output_tokens: 50,
            cache_creation_input_tokens: Some(0),
            cache_read_input_tokens: Some(0),
        };
        let event = StreamEvent::Usage(usage);
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("usage"));
    }

    #[test]
    fn stream_event_stop() {
        let event = StreamEvent::Stop { reason: Some(pi_types::message::StopReason::Stop) };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("stop"));
    }
}
