//! 持久化存储模块
//!
//! 使用 rusqlite (SQLite) 存储对话历史和任务记录。

use rusqlite::{params, Connection};
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
pub struct MemoryStore {
    conn: std::sync::Mutex<Connection>,
}

impl MemoryStore {
    /// 创建新的存储实例
    /// 使用 ":memory:" 创建内存数据库（测试用）
    /// 使用文件路径创建持久化数据库
    pub fn new(db_path: &str) -> Result<Self, String> {
        let conn = Connection::open(db_path)
            .map_err(|e| format!("数据库连接失败: {e}"))?;
        let store = Self {
            conn: std::sync::Mutex::new(conn),
        };
        store.init_tables()?;
        Ok(store)
    }

    /// 初始化数据库表
    fn init_tables(&self) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                emotion TEXT,
                FOREIGN KEY (session_id) REFERENCES sessions(id)
            );
            CREATE INDEX IF NOT EXISTS idx_messages_session ON messages(session_id, timestamp);"
        ).map_err(|e| format!("建表失败: {e}"))?;
        Ok(())
    }

    /// 保存消息
    pub fn save_message(&self, msg: &ChatMessage) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        // 确保会话存在
        conn.execute(
            "INSERT OR IGNORE INTO sessions (id, created_at, updated_at) VALUES (?1, ?2, ?2)",
            params![msg.session_id, msg.timestamp],
        )
        .map_err(|e| format!("保存会话失败: {e}"))?;

        conn.execute(
            "INSERT INTO messages (id, session_id, role, content, timestamp, emotion) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![msg.id, msg.session_id, msg.role, msg.content, msg.timestamp, msg.emotion],
        )
        .map_err(|e| format!("保存消息失败: {e}"))?;

        // 更新会话时间
        conn.execute(
            "UPDATE sessions SET updated_at = ?1 WHERE id = ?2",
            params![msg.timestamp, msg.session_id],
        )
        .map_err(|e| format!("更新会话失败: {e}"))?;

        Ok(())
    }

    /// 获取会话历史
    pub fn get_history(&self, session_id: &str, limit: u32) -> Result<Vec<ChatMessage>, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT id, session_id, role, content, timestamp, emotion FROM messages WHERE session_id = ?1 ORDER BY timestamp ASC LIMIT ?2",
            )
            .map_err(|e| format!("查询失败: {e}"))?;

        let rows = stmt
            .query_map(params![session_id, limit], |row| {
                Ok(ChatMessage {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    role: row.get(2)?,
                    content: row.get(3)?,
                    timestamp: row.get(4)?,
                    emotion: row.get(5)?,
                })
            })
            .map_err(|e| format!("查询失败: {e}"))?;

        let mut messages = Vec::new();
        for row in rows {
            messages.push(row.map_err(|e| format!("读取消息失败: {e}"))?);
        }
        Ok(messages)
    }

    /// 清除会话历史
    pub fn clear_history(&self, session_id: &str) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM messages WHERE session_id = ?1", params![session_id])
            .map_err(|e| format!("清除消息失败: {e}"))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn now() -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64
    }

    fn make_message(id: &str, role: &str, content: &str) -> ChatMessage {
        ChatMessage {
            id: id.to_string(),
            session_id: "test-session".to_string(),
            role: role.to_string(),
            content: content.to_string(),
            timestamp: now(),
            emotion: None,
        }
    }

    #[test]
    fn test_new_memory_store() {
        let store = MemoryStore::new(":memory:");
        assert!(store.is_ok());
    }

    #[test]
    fn test_save_and_get_messages() {
        let store = MemoryStore::new(":memory:").unwrap();
        store.save_message(&make_message("1", "user", "你好")).unwrap();
        store.save_message(&make_message("2", "assistant", "你好！有什么可以帮你的？")).unwrap();

        let history = store.get_history("test-session", 10).unwrap();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].role, "user");
        assert_eq!(history[1].content, "你好！有什么可以帮你的？");
    }

    #[test]
    fn test_get_history_with_limit() {
        let store = MemoryStore::new(":memory:").unwrap();
        for i in 0..5 {
            store.save_message(&make_message(&i.to_string(), "user", &format!("消息 {i}"))).unwrap();
        }
        let history = store.get_history("test-session", 3).unwrap();
        assert_eq!(history.len(), 3);
    }

    #[test]
    fn test_clear_history() {
        let store = MemoryStore::new(":memory:").unwrap();
        store.save_message(&make_message("1", "user", "你好")).unwrap();
        store.save_message(&make_message("2", "assistant", "你好！")).unwrap();

        store.clear_history("test-session").unwrap();
        let history = store.get_history("test-session", 10).unwrap();
        assert!(history.is_empty());
    }

    #[test]
    fn test_different_sessions() {
        let store = MemoryStore::new(":memory:").unwrap();
        store.save_message(&ChatMessage {
            id: "1".to_string(),
            session_id: "session-a".to_string(),
            role: "user".to_string(),
            content: "A 的消息".to_string(),
            timestamp: now(),
            emotion: None,
        }).unwrap();
        store.save_message(&ChatMessage {
            id: "2".to_string(),
            session_id: "session-b".to_string(),
            role: "user".to_string(),
            content: "B 的消息".to_string(),
            timestamp: now(),
            emotion: None,
        }).unwrap();

        let history_a = store.get_history("session-a", 10).unwrap();
        let history_b = store.get_history("session-b", 10).unwrap();
        assert_eq!(history_a.len(), 1);
        assert_eq!(history_b.len(), 1);
        assert_eq!(history_a[0].content, "A 的消息");
    }
}
