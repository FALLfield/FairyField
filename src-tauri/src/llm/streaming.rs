//! SSE 流式响应处理模块
//!
//! 解析 Server-Sent Events 格式的流式 LLM 响应。
//! 提供通用 SSE 工具函数供 provider 使用。

/// SSE 事件类型
#[derive(Debug, Clone, PartialEq)]
pub enum SseEvent {
    /// 数据事件
    Data(String),
    /// 错误事件
    Error(String),
    /// 流结束
    Done,
}

/// 解析单行 SSE 数据
pub fn parse_sse_line(line: &str) -> Option<SseEvent> {
    let line = line.trim();
    if line.is_empty() || line.starts_with(':') {
        return None;
    }

    if let Some(data) = line.strip_prefix("data: ") {
        if data.trim() == "[DONE]" {
            return Some(SseEvent::Done);
        }
        return Some(SseEvent::Data(data.to_string()));
    }

    if let Some(err) = line.strip_prefix("event: error") {
        return Some(SseEvent::Error(err.trim().to_string()));
    }

    None
}

/// 解析完整的 SSE 响应体
pub fn parse_sse_response(body: &str) -> Vec<SseEvent> {
    body.lines().filter_map(parse_sse_line).collect()
}

/// 流式处理器
pub struct StreamHandler {
    receiver: tokio::sync::mpsc::Receiver<SseEvent>,
}

impl StreamHandler {
    /// 创建新的流处理器，返回 (handler, sender)
    pub fn new() -> (Self, tokio::sync::mpsc::Sender<SseEvent>) {
        let (tx, rx) = tokio::sync::mpsc::channel(64);
        (Self { receiver: rx }, tx)
    }

    /// 获取下一个事件
    pub async fn next_event(&mut self) -> Option<SseEvent> {
        self.receiver.recv().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_data_event() {
        let event = parse_sse_line("data: {\"content\":\"hello\"}");
        assert_eq!(
            event,
            Some(SseEvent::Data("{\"content\":\"hello\"}".to_string()))
        );
    }

    #[test]
    fn test_parse_done_event() {
        let event = parse_sse_line("data: [DONE]");
        assert_eq!(event, Some(SseEvent::Done));
    }

    #[test]
    fn test_parse_empty_line() {
        assert_eq!(parse_sse_line(""), None);
    }

    #[test]
    fn test_parse_comment() {
        assert_eq!(parse_sse_line(": comment"), None);
    }

    #[test]
    fn test_parse_sse_response() {
        let body = "data: {\"content\":\"你\"}\n\ndata: {\"content\":\"好\"}\ndata: [DONE]\n";
        let events = parse_sse_response(body);
        assert_eq!(events.len(), 3);
        assert_eq!(events[2], SseEvent::Done);
    }

    #[tokio::test]
    async fn test_stream_handler() {
        let (mut handler, tx) = StreamHandler::new();
        tx.send(SseEvent::Data("hello".to_string())).await.unwrap();
        tx.send(SseEvent::Done).await.unwrap();

        assert_eq!(
            handler.next_event().await,
            Some(SseEvent::Data("hello".to_string()))
        );
        assert_eq!(handler.next_event().await, Some(SseEvent::Done));
        // Drop sender so receiver returns None
        drop(tx);
        assert_eq!(handler.next_event().await, None);
    }
}
