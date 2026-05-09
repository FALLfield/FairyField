//! LLM 客户端模块
//!
//! 提供统一的 LLM 调用接口，支持多提供商和流式响应。

pub mod caching;
pub mod provider;
pub mod routing;
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
    /// 工具调用 ID（Tool 角色消息对应）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    /// LLM 请求的工具调用列表（Assistant 消息携带）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCallRequest>>,
}

/// 消息角色
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
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

/// 流式响应的 token 片段
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChunk {
    /// token 文本
    pub text: String,
    /// 是否为最后一个片段
    pub done: bool,
}

/// LLM 提供商配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmProviderConfig {
    /// 提供商类型: openai / claude
    pub provider: String,
    /// API key
    pub api_key: String,
    /// API 端点 URL（OpenAI 兼容接口可自定义）
    pub api_endpoint: String,
    /// 模型名称
    pub model: String,
}

impl LlmProviderConfig {
    /// 从 AppConfig 的 LlmConfig 构建
    ///
    /// 优先使用多提供商预设（active_preset），回退到旧字段（向后兼容）。
    pub fn from_llm_config(llm: &crate::config::LlmConfig) -> Self {
        let preset = llm.active_preset();
        Self {
            provider: preset.provider_type,
            api_key: preset.api_key,
            api_endpoint: preset.api_endpoint,
            model: preset.model,
        }
    }

    /// 从环境变量填充 API key
    pub fn with_env_api_key(mut self) -> Self {
        let key = match self.provider.as_str() {
            "claude" => std::env::var("ANTHROPIC_API_KEY")
                .or_else(|_| std::env::var("CLAUDE_API_KEY"))
                .unwrap_or_default(),
            _ => std::env::var("FAIRYFIELD_API_KEY")
                .or_else(|_| std::env::var("OPENAI_API_KEY"))
                .unwrap_or_default(),
        };
        self.api_key = key;
        self
    }

    /// 设置 API key（用于测试或显式配置）
    pub fn with_api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = key.into();
        self
    }
}

/// LLM 工具调用请求
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolCallRequest {
    /// 工具调用唯一 ID（由 LLM 返回）
    pub id: String,
    /// 工具名称
    pub name: String,
    /// 工具参数（JSON 字符串）
    pub arguments: String,
}

/// 工具定义（Provider 无关格式）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolDefinition {
    /// 工具名称
    pub name: String,
    /// 工具描述
    pub description: String,
    /// 参数 JSON Schema
    pub parameters: serde_json::Value,
}

/// LLM 响应 — 纯文本或工具调用
#[derive(Debug, Clone, PartialEq)]
pub enum LlmResponse {
    /// 纯文本回复
    Text(String),
    /// 工具调用（可能附带文本）
    ToolCalls {
        calls: Vec<ToolCallRequest>,
        text: Option<String>,
    },
}
