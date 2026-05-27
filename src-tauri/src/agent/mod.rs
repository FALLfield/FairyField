//! AI 大脑模块
//!
//! 基于 LLM API 的 AI Agent 系统，包含：
//! - 主要对话 Agent（primary）
//! - 上下文压缩器（compressor）
//! - 委托管理器（delegation）
//! - 工具执行 Agent（tool_agent）
//! - 可组合工具集（toolset）

pub mod coding_agent;
pub mod compressor;
pub mod delegation;
pub mod primary;
pub mod tool_agent;
pub mod toolset;

// Re-export 主要类型和 Tauri commands 方便 lib.rs 使用
pub use primary::{
    agent_chat, agent_chat_stream, agent_clear_history, agent_get_emotion, AgentConfig, AgentState,
    ChatResponse, EmotionKind, EmotionState, PrimaryAgent,
};
pub use toolset::{Toolset, ToolsetError};
