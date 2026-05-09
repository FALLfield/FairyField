//! 持久化存储模块
//!
//! 基于 SQLite 的对话记忆和 Palace 层级存储。
//! 包含 FTS5 全文搜索、对话消息、记忆抽屉和元数据表。

use jieba_rs::Jieba;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

// ===========================================================================
// 对话消息（向后兼容）
// ===========================================================================

/// 对话消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub session_id: String,
    pub role: String,
    pub content: String,
    pub emotion_tag: Option<String>,
    pub created_at: i64,
}

// ===========================================================================
// 记忆抽屉（Phase 3）
// ===========================================================================

/// 记忆抽屉
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Drawer {
    pub id: i64,
    pub content: String,
    pub category: String,
    pub wing: String,
    pub room: String,
    pub hall: String,
    pub tags: String,
    pub importance: f32,
    pub created_at: i64,
    pub accessed_at: i64,
    pub access_count: i64,
}

// ===========================================================================
// MemoryStore
// ===========================================================================

/// 对话存储
pub struct MemoryStore {
    conn: Connection,
    jieba: Jieba,
}

impl MemoryStore {
    /// 创建新的存储实例
    pub fn new(db_path: &str) -> Result<Self, String> {
        let conn = if db_path == ":memory:" {
            Connection::open_in_memory()
        } else {
            Connection::open(db_path)
        }
        .map_err(|e| e.to_string())?;

        let store = Self { conn, jieba: Jieba::new() };
        store.init_tables()?;
        Ok(store)
    }

    /// 初始化所有数据库表
    fn init_tables(&self) -> Result<(), String> {
        // 对话消息表（向后兼容）
        self.conn
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS chat_messages (
                    id TEXT PRIMARY KEY,
                    session_id TEXT NOT NULL,
                    role TEXT NOT NULL,
                    content TEXT NOT NULL,
                    emotion_tag TEXT,
                    created_at INTEGER NOT NULL
                );",
            )
            .map_err(|e| e.to_string())?;

        // 记忆抽屉表
        self.conn
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS memory_drawers (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    content TEXT NOT NULL,
                    category TEXT NOT NULL DEFAULT 'general',
                    wing TEXT NOT NULL DEFAULT 'daily',
                    room TEXT NOT NULL DEFAULT 'default',
                    hall TEXT NOT NULL DEFAULT 'default',
                    tags TEXT NOT NULL DEFAULT '',
                    importance REAL NOT NULL DEFAULT 0.5,
                    created_at INTEGER NOT NULL,
                    accessed_at INTEGER NOT NULL,
                    access_count INTEGER NOT NULL DEFAULT 0
                );",
            )
            .map_err(|e| e.to_string())?;

        // FTS5 全文搜索
        self.conn
            .execute_batch(
                "CREATE VIRTUAL TABLE IF NOT EXISTS memory_fts USING fts5(
                    content,
                    category,
                    wing,
                    room,
                    content='memory_drawers',
                    content_rowid='id'
                );",
            )
            .map_err(|e| e.to_string())?;

        // 元数据表（L0-L2 层）
        self.conn
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS memory_meta (
                    key TEXT PRIMARY KEY,
                    value TEXT NOT NULL,
                    layer INTEGER NOT NULL DEFAULT 0,
                    updated_at INTEGER NOT NULL
                );",
            )
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    // -----------------------------------------------------------------------
    // 对话消息（向后兼容）
    // -----------------------------------------------------------------------

    /// 保存消息
    pub fn save_message(&self, msg: &ChatMessage) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT OR REPLACE INTO chat_messages (id, session_id, role, content, emotion_tag, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![msg.id, msg.session_id, msg.role, msg.content, msg.emotion_tag, msg.created_at],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// 获取会话历史
    pub fn get_history(&self, session_id: &str, limit: u32) -> Result<Vec<ChatMessage>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, session_id, role, content, emotion_tag, created_at
                 FROM chat_messages WHERE session_id = ?1 ORDER BY created_at DESC LIMIT ?2",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map(params![session_id, limit], |row| {
                Ok(ChatMessage {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    role: row.get(2)?,
                    content: row.get(3)?,
                    emotion_tag: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })
            .map_err(|e| e.to_string())?;

        let mut messages: Vec<ChatMessage> = Vec::new();
        for r in rows {
            match r {
                Ok(msg) => messages.push(msg),
                Err(e) => {
                    eprintln!("Warning: row deserialization failed: {}", e);
                }
            }
        }
        messages.reverse();
        Ok(messages)
    }

    /// 清除会话历史
    pub fn clear_history(&self, session_id: &str) -> Result<(), String> {
        self.conn
            .execute(
                "DELETE FROM chat_messages WHERE session_id = ?1",
                params![session_id],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// 使用 jieba 对中文文本进行分词，返回空格分隔的词序列
    /// 英文/ASCII 文本保持不变（jieba 对 ASCII 不做处理）
    fn tokenize_chinese(&self, text: &str) -> String {
        let words: Vec<&str> = self.jieba.cut(text, false);
        words.join(" ")
    }

    // -----------------------------------------------------------------------
    // 记忆抽屉（Phase 3）
    // -----------------------------------------------------------------------

    /// 存入记忆抽屉（事务保护：drawer + FTS 同步）
    pub fn save_drawer(
        &self,
        content: &str,
        wing: &str,
        room: &str,
        hall: &str,
        category: &str,
        tags: &str,
        importance: f32,
    ) -> Result<i64, String> {
        let now = now();
        let tx = self
            .conn
            .unchecked_transaction()
            .map_err(|e| e.to_string())?;

        tx.execute(
            "INSERT INTO memory_drawers (content, category, wing, room, hall, tags, importance, created_at, accessed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8)",
            params![content, category, wing, room, hall, tags, importance, now],
        )
        .map_err(|e| e.to_string())?;

        let id = self.conn.last_insert_rowid();

        tx.execute(
            "INSERT INTO memory_fts (rowid, content, category, wing, room) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                id,
                self.tokenize_chinese(content),
                self.tokenize_chinese(category),
                self.tokenize_chinese(wing),
                self.tokenize_chinese(room),
            ],
        )
        .map_err(|e| format!("FTS 同步失败: {}", e))?;

        tx.commit().map_err(|e| e.to_string())?;
        Ok(id)
    }

    /// FTS5 全文搜索（自动对中文查询分词）
    pub fn search_fts(&self, query: &str, limit: usize) -> Result<Vec<Drawer>, String> {
        let tokenized = self.tokenize_chinese(query);
        let mut stmt = self
            .conn
            .prepare(
                "SELECT d.id, d.content, d.category, d.wing, d.room, d.hall, d.tags,
                        d.importance, d.created_at, d.accessed_at, d.access_count
                 FROM memory_drawers d
                 JOIN memory_fts f ON d.id = f.rowid
                 WHERE memory_fts MATCH ?1
                 ORDER BY rank
                 LIMIT ?2",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map(params![tokenized, limit as i64], |row| {
                Ok(Drawer {
                    id: row.get(0)?,
                    content: row.get(1)?,
                    category: row.get(2)?,
                    wing: row.get(3)?,
                    room: row.get(4)?,
                    hall: row.get(5)?,
                    tags: row.get(6)?,
                    importance: row.get(7)?,
                    created_at: row.get(8)?,
                    accessed_at: row.get(9)?,
                    access_count: row.get(10)?,
                })
            })
            .map_err(|e| e.to_string())?;

        let mut results: Vec<Drawer> = Vec::new();
        for r in rows {
            match r {
                Ok(drawer) => results.push(drawer),
                Err(e) => {
                    eprintln!("Warning: FTS row deserialization failed: {}", e);
                }
            }
        }
        Ok(results)
    }

    /// 按 wing 查询抽屉
    pub fn get_drawers_by_wing(&self, wing: &str, limit: usize) -> Result<Vec<Drawer>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, content, category, wing, room, hall, tags, importance, created_at, accessed_at, access_count
                 FROM memory_drawers WHERE wing = ?1 ORDER BY importance DESC, created_at DESC LIMIT ?2",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map(params![wing, limit as i64], |row| {
                Ok(Drawer {
                    id: row.get(0)?,
                    content: row.get(1)?,
                    category: row.get(2)?,
                    wing: row.get(3)?,
                    room: row.get(4)?,
                    hall: row.get(5)?,
                    tags: row.get(6)?,
                    importance: row.get(7)?,
                    created_at: row.get(8)?,
                    accessed_at: row.get(9)?,
                    access_count: row.get(10)?,
                })
            })
            .map_err(|e| e.to_string())?;

        let mut results: Vec<Drawer> = Vec::new();
        for r in rows {
            match r {
                Ok(drawer) => results.push(drawer),
                Err(e) => {
                    eprintln!("Warning: drawer row deserialization failed: {}", e);
                }
            }
        }
        Ok(results)
    }

    /// 更新访问记录
    pub fn update_access(&self, id: i64) -> Result<(), String> {
        let now = now();
        self.conn
            .execute(
                "UPDATE memory_drawers SET accessed_at = ?1, access_count = access_count + 1 WHERE id = ?2",
                params![now, id],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    // -----------------------------------------------------------------------
    // 元数据（L0-L2）
    // -----------------------------------------------------------------------

    /// 获取元数据
    pub fn get_meta(&self, key: &str) -> Option<String> {
        self.conn
            .query_row(
                "SELECT value FROM memory_meta WHERE key = ?1",
                params![key],
                |row| row.get(0),
            )
            .ok()
    }

    /// 设置元数据
    pub fn set_meta(&self, key: &str, value: &str, layer: i32) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT OR REPLACE INTO memory_meta (key, value, layer, updated_at) VALUES (?1, ?2, ?3, ?4)",
                params![key, value, layer, now()],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}

fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

// ===========================================================================
// 测试
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_store() -> MemoryStore {
        MemoryStore::new(":memory:").unwrap()
    }

    fn make_message(id: &str, role: &str, content: &str) -> ChatMessage {
        ChatMessage {
            id: id.into(),
            session_id: "test".into(),
            role: role.into(),
            content: content.into(),
            emotion_tag: None,
            created_at: 1000,
        }
    }

    #[test]
    fn test_new_memory_store() {
        make_store();
    }

    #[test]
    fn test_save_and_get_messages() {
        let store = make_store();
        store
            .save_message(&make_message("1", "user", "hello"))
            .unwrap();
        store
            .save_message(&make_message("2", "assistant", "hi"))
            .unwrap();
        let msgs = store.get_history("test", 10).unwrap();
        assert_eq!(msgs.len(), 2);
    }

    #[test]
    fn test_clear_history() {
        let store = make_store();
        store
            .save_message(&make_message("1", "user", "hello"))
            .unwrap();
        store.clear_history("test").unwrap();
        assert!(store.get_history("test", 10).unwrap().is_empty());
    }

    #[test]
    fn test_save_and_search_drawer() {
        // TODO: FTS5 default tokenizer doesn't handle CJK; switch to jieba tokenizer for Chinese
        let store = make_store();
        store
            .save_drawer(
                "I love programming in Rust",
                "daily",
                "hobbies",
                "tech",
                "preference",
                "",
                0.8,
            )
            .unwrap();
        store
            .save_drawer(
                "Went hiking today",
                "daily",
                "activities",
                "outdoor",
                "event",
                "",
                0.5,
            )
            .unwrap();

        let results = store.search_fts("programming", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].content.contains("programming"));
    }

    #[test]
    fn test_get_drawers_by_wing() {
        let store = make_store();
        store
            .save_drawer("工作笔记", "work", "notes", "general", "general", "", 0.5)
            .unwrap();
        store
            .save_drawer("生活记录", "daily", "log", "general", "general", "", 0.5)
            .unwrap();

        let daily = store.get_drawers_by_wing("daily", 10).unwrap();
        let work = store.get_drawers_by_wing("work", 10).unwrap();
        assert_eq!(daily.len(), 1);
        assert_eq!(work.len(), 1);
    }

    #[test]
    fn test_meta() {
        let store = make_store();
        assert!(store.get_meta("identity").is_none());
        store.set_meta("identity", "我是 Fairy", 0).unwrap();
        assert_eq!(store.get_meta("identity").unwrap(), "我是 Fairy");
    }

    #[test]
    fn test_update_access() {
        let store = make_store();
        let id = store
            .save_drawer("test", "daily", "default", "default", "general", "", 0.5)
            .unwrap();
        store.update_access(id).unwrap();
        let results = store.get_drawers_by_wing("daily", 10).unwrap();
        assert_eq!(results[0].access_count, 1);
    }
}
