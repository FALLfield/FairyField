//! 持久化存储模块
//!
//! 使用 rusqlite (SQLite) 存储对话历史和任务记录。

use serde::{Deserialize, Serialize};

/// 对话消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// 消息唯一 ID
    pub id: String,
    /// 会话 ID
    pub session_id: String,
    /// 角色：user / assistant / system
    pub role: String,
    /// 消息内容
    pub content: String,
    /// 时间戳（Unix 毫秒）
    pub timestamp: i64,
    /// 关联的情绪状态
    pub emotion: Option<String>,
}

/// 对话存储
///
/// Phase 2 实现：使用 rusqlite 管理对话历史的持久化存储，
/// 支持按会话查询、按时间范围筛选等操作。
#[allow(dead_code)]
pub struct MemoryStore {
    // TODO: SQLite 连接
}
