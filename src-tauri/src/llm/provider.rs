//! LLM Provider 模块
//!
//! 定义 LLM 提供商 trait 和各提供商实现。

use crate::llm::{ChatConfig, LlmError, Message};

/// LLM 提供商 trait
pub trait LlmProvider: Send + Sync {
    /// 发送聊天请求
    fn chat(
        &self,
        messages: Vec<Message>,
        config: ChatConfig,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<String, LlmError>> + Send + '_>,
    >;
    /// 提供商名称
    fn name(&self) -> &str;
}

/// Mock LLM 提供商（用于测试）
pub struct MockProvider {
    reply: String,
    messages_received: std::sync::Mutex<Vec<Message>>,
}

impl MockProvider {
    pub fn new(reply: impl Into<String>) -> Self {
        Self {
            reply: reply.into(),
            messages_received: std::sync::Mutex::new(Vec::new()),
        }
    }

    pub fn messages_received(&self) -> Vec<Message> {
        self.messages_received.lock().unwrap().clone()
    }
}

impl LlmProvider for MockProvider {
    fn chat(
        &self,
        messages: Vec<Message>,
        _config: ChatConfig,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<String, LlmError>> + Send + '_>,
    > {
        self.messages_received
            .lock()
            .unwrap()
            .extend(messages.clone());
        let reply = self.reply.clone();
        Box::pin(async move { Ok(reply) })
    }

    fn name(&self) -> &str {
        "mock"
    }
}

/// OpenAI 提供商（类型定义完整，实现待 Phase 3）
pub struct OpenAiProvider {
    api_key: String,
    base_url: String,
    model: String,
}

impl OpenAiProvider {
    pub fn new(api_key: String, base_url: String, model: String) -> Result<Self, LlmError> {
        if api_key.is_empty() {
            return Err(LlmError("API Key 不能为空".to_string()));
        }
        Ok(Self {
            api_key,
            base_url,
            model,
        })
    }

    pub fn model(&self) -> &str {
        &self.model
    }
}

impl LlmProvider for OpenAiProvider {
    fn chat(
        &self,
        _messages: Vec<Message>,
        _config: ChatConfig,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<String, LlmError>> + Send + '_>,
    > {
        Box::pin(async move {
            Err(LlmError("OpenAI provider 尚未实现（Phase 3）".to_string()))
        })
    }

    fn name(&self) -> &str {
        "openai"
    }
}

/// Claude 提供商（类型定义完整，实现待 Phase 3）
pub struct ClaudeProvider {
    api_key: String,
    model: String,
}

impl ClaudeProvider {
    pub fn new(api_key: String, model: String) -> Result<Self, LlmError> {
        if api_key.is_empty() {
            return Err(LlmError("API Key 不能为空".to_string()));
        }
        Ok(Self { api_key, model })
    }
}

impl LlmProvider for ClaudeProvider {
    fn chat(
        &self,
        _messages: Vec<Message>,
        _config: ChatConfig,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<String, LlmError>> + Send + '_>,
    > {
        Box::pin(async move {
            Err(LlmError("Claude provider 尚未实现（Phase 3）".to_string()))
        })
    }

    fn name(&self) -> &str {
        "claude"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::{ChatConfig, MessageRole};

    fn now() -> i64 {
        1700000000000
    }

    #[tokio::test]
    async fn test_mock_provider_reply() {
        let provider = MockProvider::new("你好！");
        let messages = vec![Message {
            role: MessageRole::User,
            content: "嗨".to_string(),
            timestamp: now(),
        }];
        let result = provider.chat(messages, ChatConfig::default()).await;
        assert_eq!(result.unwrap(), "你好！");
    }

    #[tokio::test]
    async fn test_mock_provider_records_messages() {
        let provider = MockProvider::new("回复");
        let messages = vec![Message {
            role: MessageRole::User,
            content: "你好".to_string(),
            timestamp: now(),
        }];
        let _ = provider.chat(messages, ChatConfig::default()).await;
        let received = provider.messages_received();
        assert_eq!(received.len(), 1);
    }

    #[test]
    fn test_openai_provider_new() {
        let provider = OpenAiProvider::new(
            "sk-test".to_string(),
            "https://api.openai.com/v1".to_string(),
            "gpt-4".to_string(),
        );
        assert!(provider.is_ok());
    }

    #[test]
    fn test_openai_provider_empty_key() {
        let result = OpenAiProvider::new(String::new(), "https://api.openai.com/v1".to_string(), "gpt-4".to_string());
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_openai_not_implemented() {
        let provider = OpenAiProvider::new("sk-test".to_string(), "https://api.openai.com/v1".to_string(), "gpt-4".to_string()).unwrap();
        let result = provider.chat(vec![], ChatConfig::default()).await;
        assert!(result.is_err());
    }
}
