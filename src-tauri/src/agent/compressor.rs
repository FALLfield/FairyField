//! 上下文压缩器
//!
//! 智能压缩对话历史，确保不超出 LLM 上下文窗口。

use crate::llm::{Message, MessageRole};

/// 压缩策略
#[derive(Debug, Clone)]
pub enum CompressionStrategy {
    SlidingWindow {
        keep_recent: usize,
    },
    Summarize {
        max_summary_tokens: usize,
    },
    Hybrid {
        keep_recent: usize,
        max_summary_tokens: usize,
    },
}

/// 压缩结果
#[derive(Debug)]
pub struct CompressedHistory {
    pub messages: Vec<Message>,
    pub original_count: usize,
    pub estimated_tokens: usize,
    pub has_summary: bool,
}

/// 上下文压缩器
pub struct ContextCompressor {
    strategy: CompressionStrategy,
    compression_threshold: usize,
}

impl ContextCompressor {
    pub fn new(strategy: CompressionStrategy) -> Self {
        Self {
            strategy,
            compression_threshold: 4000,
        }
    }

    pub fn with_threshold(mut self, tokens: usize) -> Self {
        self.compression_threshold = tokens;
        self
    }

    pub fn threshold(&self) -> usize {
        self.compression_threshold
    }

    /// 估算 token 数量（CJK 字符 ~2 tokens each，ASCII ~0.5 tokens each）
    #[must_use]
    pub fn estimate_tokens(text: &str) -> usize {
        if text.is_empty() {
            return 0;
        }
        let mut count = 0usize;
        for ch in text.chars() {
            if ('\u{4E00}'..='\u{9FFF}').contains(&ch) {
                count += 2; // CJK: ~2 tokens each
            } else {
                count += 1; // ASCII: ~4 chars per token, counting chars not bytes
            }
        }
        count / 2
    }

    #[must_use]
    pub fn needs_compression(&self, messages: &[Message]) -> bool {
        let total: usize = messages
            .iter()
            .map(|m| Self::estimate_tokens(&m.content))
            .sum();
        total > self.compression_threshold
    }

    #[must_use]
    pub fn compress(&self, messages: &[Message]) -> CompressedHistory {
        let original_count = messages.len();
        if original_count <= 2 {
            let est: usize = messages
                .iter()
                .map(|m| Self::estimate_tokens(&m.content))
                .sum();
            return CompressedHistory {
                messages: messages.to_vec(),
                original_count,
                estimated_tokens: est,
                has_summary: false,
            };
        }

        match &self.strategy {
            CompressionStrategy::SlidingWindow { keep_recent } => {
                self.compress_sliding_window(messages, *keep_recent)
            }
            CompressionStrategy::Summarize { max_summary_tokens } => {
                self.compress_summarize(messages, *max_summary_tokens)
            }
            CompressionStrategy::Hybrid {
                keep_recent,
                max_summary_tokens,
            } => self.compress_hybrid(messages, *keep_recent, *max_summary_tokens),
        }
    }

    fn compress_sliding_window(
        &self,
        messages: &[Message],
        keep_recent: usize,
    ) -> CompressedHistory {
        let mut result = Vec::new();
        let start = if !messages.is_empty() && messages[0].role == MessageRole::System {
            result.push(messages[0].clone());
            1
        } else {
            0
        };

        let remaining = &messages[start..];
        let skip = remaining.len().saturating_sub(keep_recent);
        for msg in remaining.iter().skip(skip) {
            result.push(msg.clone());
        }

        let est: usize = result
            .iter()
            .map(|m| Self::estimate_tokens(&m.content))
            .sum();
        CompressedHistory {
            messages: result,
            original_count: messages.len(),
            estimated_tokens: est,
            has_summary: false,
        }
    }

    fn compress_summarize(&self, messages: &[Message], max_tokens: usize) -> CompressedHistory {
        let mut result: Vec<Message> = messages
            .iter()
            .filter(|m| m.role == MessageRole::System)
            .cloned()
            .collect();

        let non_system: Vec<&Message> = messages
            .iter()
            .filter(|m| m.role != MessageRole::System)
            .collect();
        if !non_system.is_empty() {
            let summary = build_summary(&non_system, max_tokens);
            result.push(Message {
                role: MessageRole::System,
                content: format!("[对话历史摘要]\n{}", summary),
                timestamp: non_system.last().map(|m| m.timestamp).unwrap_or(0),
                tool_call_id: None,
                tool_calls: None,
            });
        }

        let est: usize = result
            .iter()
            .map(|m| Self::estimate_tokens(&m.content))
            .sum();
        CompressedHistory {
            messages: result,
            original_count: messages.len(),
            estimated_tokens: est,
            has_summary: true,
        }
    }

    fn compress_hybrid(
        &self,
        messages: &[Message],
        keep_recent: usize,
        max_tokens: usize,
    ) -> CompressedHistory {
        let mut result = Vec::new();
        let start = if !messages.is_empty() && messages[0].role == MessageRole::System {
            result.push(messages[0].clone());
            1
        } else {
            0
        };

        let remaining = &messages[start..];
        if remaining.len() > keep_recent {
            let split = remaining.len() - keep_recent;
            debug_assert!(
                split > 0,
                "split should always be positive when remaining.len() > keep_recent"
            );
            let old_refs: Vec<&Message> = remaining[..split].iter().collect();
            let summary = build_summary(&old_refs, max_tokens);
            result.push(Message {
                role: MessageRole::System,
                content: format!("[早期对话摘要]\n{}", summary),
                timestamp: remaining[split - 1].timestamp,
                tool_call_id: None,
                tool_calls: None,
            });
            for msg in remaining[split..].iter() {
                result.push(msg.clone());
            }
        } else {
            for msg in remaining {
                result.push(msg.clone());
            }
        }

        let est: usize = result
            .iter()
            .map(|m| Self::estimate_tokens(&m.content))
            .sum();
        CompressedHistory {
            messages: result,
            original_count: messages.len(),
            estimated_tokens: est,
            has_summary: remaining.len() > keep_recent,
        }
    }
}

fn build_summary(messages: &[&Message], max_tokens: usize) -> String {
    let lines: Vec<String> = messages
        .iter()
        .map(|m| {
            let role = match m.role {
                MessageRole::User => "用户",
                MessageRole::Assistant => "助手",
                MessageRole::System => "系统",
                MessageRole::Tool => "工具",
            };
            format!("- [{}] {}", role, m.content)
        })
        .collect();

    let full = lines.join("\n");
    let max_chars = max_tokens * 4;
    if full.len() > max_chars {
        // Find a safe char boundary <= max_chars to avoid slicing mid-byte on CJK chars
        let mut end = max_chars;
        while !full.is_char_boundary(end) && end > 0 {
            end -= 1;
        }
        let truncated = &full[..end];
        match truncated.rfind('\n') {
            Some(pos) => format!("{}...(已截断)", &full[..pos]),
            None => format!("{}...(已截断)", truncated),
        }
    } else {
        full
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn msg(role: MessageRole, content: &str) -> Message {
        Message {
            role,
            content: content.into(),
            timestamp: 1000,
            tool_call_id: None,
            tool_calls: None,
        }
    }

    #[test]
    fn estimate_tokens() {
        assert_eq!(ContextCompressor::estimate_tokens(""), 0);
        // 14 ASCII chars: 14 * 1 / 2 = 7
        assert_eq!(ContextCompressor::estimate_tokens("hello world!!!!"), 7);
        // 4 CJK chars: 4 * 2 / 2 = 4
        assert_eq!(ContextCompressor::estimate_tokens("你好世界"), 4);
        // mixed: 2 CJK (2*2=4) + 1 space (1) + 5 ASCII (5) = 10 / 2 = 5
        assert_eq!(ContextCompressor::estimate_tokens("你好 hello"), 5);
    }

    #[test]
    fn sliding_window_keeps_recent() {
        let c = ContextCompressor::new(CompressionStrategy::SlidingWindow { keep_recent: 2 });
        let msgs = vec![
            msg(MessageRole::System, "sys"),
            msg(MessageRole::User, "m1"),
            msg(MessageRole::Assistant, "m2"),
            msg(MessageRole::User, "m3"),
            msg(MessageRole::Assistant, "m4"),
        ];
        let result = c.compress(&msgs);
        assert_eq!(result.messages.len(), 3); // system + 2 recent
        assert_eq!(result.messages[0].content, "sys");
        assert_eq!(result.messages[2].content, "m4");
    }

    #[test]
    fn summarize_creates_summary() {
        let c = ContextCompressor::new(CompressionStrategy::Summarize {
            max_summary_tokens: 1000,
        });
        let msgs = vec![
            msg(MessageRole::System, "prompt"),
            msg(MessageRole::User, "hello"),
            msg(MessageRole::Assistant, "hi"),
        ];
        let result = c.compress(&msgs);
        assert!(result.has_summary);
        assert!(result.messages[1].content.contains("[对话历史摘要]"));
    }

    #[test]
    fn hybrid_combines() {
        let c = ContextCompressor::new(CompressionStrategy::Hybrid {
            keep_recent: 1,
            max_summary_tokens: 1000,
        });
        let msgs = vec![
            msg(MessageRole::System, "sys"),
            msg(MessageRole::User, "old1"),
            msg(MessageRole::Assistant, "old2"),
            msg(MessageRole::User, "recent"),
        ];
        let result = c.compress(&msgs);
        assert_eq!(result.messages.len(), 3); // system + summary + 1 recent
        assert!(result.messages[1].content.contains("[早期对话摘要]"));
        assert_eq!(result.messages[2].content, "recent");
    }

    #[test]
    fn needs_compression() {
        let c = ContextCompressor::new(CompressionStrategy::SlidingWindow { keep_recent: 5 })
            .with_threshold(10);
        let long = "abcdefghij".repeat(100);
        assert!(c.needs_compression(&[msg(MessageRole::User, &long)]));
        assert!(!c.needs_compression(&[msg(MessageRole::User, "short")]));
    }
}
