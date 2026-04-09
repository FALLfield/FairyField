//! LLM 客户端模块
//!
//! 提供统一的 LLM 调用接口，支持多提供商和流式响应。

pub mod provider;
pub mod streaming;

use serde::{Deserialize, Serialize};

/// 对话消息
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Message {
    /// 角色
    pub role: MessageRole,
    /// 内容
    pub content: String,
    /// 时间戳（Unix 毫秒）
    pub timestamp: i64,
}

/// 消息角色
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

/// 聊天配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatConfig {
    /// 温度（0-2）
    pub temperature: f32,
    /// 最大 token 数
    pub max_tokens: u32,
    /// top-p（0-1）
    pub top_p: f32,
}

impl Default for ChatConfig {
    fn default() -> Self {
        Self {
            temperature: 0.7,
            max_tokens: 2048,
            top_p: 0.9,
        }
    }
}

/// LLM 错误类型
#[derive(Debug, Clone)]
pub struct LlmError(pub String);

impl std::fmt::Display for LlmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LLM 错误: {}", self.0)
    }
}

impl std::error::Error for LlmError {}

impl From<String> for LlmError {
    fn from(s: String) -> Self {
        LlmError(s)
    }
}
