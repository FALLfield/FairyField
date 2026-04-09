//! 主要 Agent 模块
//!
//! 负责对话管理、历史记录和情绪状态。

use crate::llm::{ChatConfig, LlmError, Message, MessageRole};
use std::sync::Arc;

/// Agent 配置
#[derive(Debug, Clone)]
pub struct AgentConfig {
    /// 系统提示词
    pub system_prompt: String,
    /// 温度
    pub temperature: f32,
    /// 最大 token 数
    pub max_tokens: u32,
    /// 最大历史消息数
    pub max_history: usize,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            system_prompt: "你是 Fairy，一个温柔、活泼、有点调皮的 AI 伴侣。".to_string(),
            temperature: 0.7,
            max_tokens: 2048,
            max_history: 50,
        }
    }
}

/// 情绪状态
#[derive(Debug, Clone, PartialEq)]
pub enum EmotionState {
    Happy,
    Sad,
    Angry,
    Surprised,
    Neutral,
    Thinking,
}

/// 主要对话 Agent
pub struct PrimaryAgent {
    provider: Arc<dyn crate::llm::provider::LlmProvider>,
    config: AgentConfig,
    history: std::sync::Mutex<Vec<Message>>,
    emotion: std::sync::Mutex<EmotionState>,
}

impl PrimaryAgent {
    pub fn new(
        provider: Arc<dyn crate::llm::provider::LlmProvider>,
        config: AgentConfig,
    ) -> Self {
        Self {
            provider,
            config,
            history: std::sync::Mutex::new(Vec::new()),
            emotion: std::sync::Mutex::new(EmotionState::Neutral),
        }
    }

    /// 处理用户消息，返回 AI 回复
    pub async fn chat(&self, message: &str) -> Result<String, LlmError> {
        let now = chrono_free_timestamp();

        // 添加用户消息
        {
            let mut history = self.history.lock().unwrap();
            history.push(Message {
                role: MessageRole::User,
                content: message.to_string(),
                timestamp: now,
            });
        }

        // 构建完整的消息列表
        let messages = {
            let history = self.history.lock().unwrap();
            let mut msgs = vec![Message {
                role: MessageRole::System,
                content: self.config.system_prompt.clone(),
                timestamp: now,
            }];
            msgs.extend(history.clone());
            msgs
        };

        let chat_config = ChatConfig {
            temperature: self.config.temperature,
            max_tokens: self.config.max_tokens,
            top_p: 0.9,
        };

        // 设置情绪为 Thinking
        self.set_emotion(EmotionState::Thinking);

        // 调用 LLM
        let reply = self.provider.chat(messages, chat_config).await?;

        // 添加助手回复
        {
            let mut history = self.history.lock().unwrap();
            history.push(Message {
                role: MessageRole::Assistant,
                content: reply.clone(),
                timestamp: chrono_free_timestamp(),
            });
            // 裁剪超长历史
            self.trim_history(&mut history);
        }

        // 恢复情绪为 Neutral
        self.set_emotion(EmotionState::Neutral);

        Ok(reply)
    }

    pub fn set_emotion(&self, emotion: EmotionState) {
        *self.emotion.lock().unwrap() = emotion;
    }

    pub fn get_emotion(&self) -> EmotionState {
        self.emotion.lock().unwrap().clone()
    }

    pub fn clear_history(&self) {
        self.history.lock().unwrap().clear();
    }

    pub fn get_history(&self) -> Vec<Message> {
        self.history.lock().unwrap().clone()
    }

    fn trim_history(&self, history: &mut Vec<Message>) {
        if history.len() > self.config.max_history {
            let remove_count = history.len() - self.config.max_history;
            history.drain(0..remove_count);
        }
    }
}

/// 获取当前时间戳（不依赖 chrono crate）
fn chrono_free_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::provider::MockProvider;

    fn now() -> i64 { 1700000000000 }

    fn make_agent() -> PrimaryAgent {
        let provider = Arc::new(MockProvider::new("收到！"));
        PrimaryAgent::new(provider, AgentConfig::default())
    }

    #[tokio::test]
    async fn test_chat_basic() {
        let agent = make_agent();
        let reply = agent.chat("你好").await.unwrap();
        assert_eq!(reply, "收到！");
    }

    #[tokio::test]
    async fn test_chat_history() {
        let agent = make_agent();
        agent.chat("第一条").await.unwrap();
        agent.chat("第二条").await.unwrap();
        let history = agent.get_history();
        assert_eq!(history.len(), 4); // 2 user + 2 assistant
    }

    #[tokio::test]
    async fn test_emotion_state() {
        let agent = make_agent();
        assert_eq!(agent.get_emotion(), EmotionState::Neutral);
        agent.set_emotion(EmotionState::Happy);
        assert_eq!(agent.get_emotion(), EmotionState::Happy);
    }

    #[test]
    fn test_clear_history() {
        let agent = make_agent();
        agent.clear_history();
        assert!(agent.get_history().is_empty());
    }

    #[test]
    fn test_default_config() {
        let config = AgentConfig::default();
        assert!(!config.system_prompt.is_empty());
        assert_eq!(config.temperature, 0.7);
    }
}
