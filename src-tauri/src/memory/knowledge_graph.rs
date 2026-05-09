//! 时序知识图谱
//!
//! 存储三元组 (subject, predicate, object) 并支持时间点查询。
//! 事实不删除，只通过 valid_to 标记失效。

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

/// 知识三元组
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Triple {
    pub id: i64,
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub valid_from: i64,       // Unix millis
    pub valid_to: Option<i64>, // None = still valid
    pub source: String,        // "conversation", "manual", "system"
    pub confidence: f32,
    pub created_at: i64,
}

/// 知识图谱
pub struct KnowledgeGraph {
    conn: Connection,
}

impl KnowledgeGraph {
    pub fn new(conn: Connection) -> Result<Self, String> {
        let kg = Self { conn };
        kg.init_tables()?;
        Ok(kg)
    }

    pub fn new_in_memory() -> Result<Self, String> {
        Self::new(Connection::open_in_memory().map_err(|e| e.to_string())?)
    }

    fn init_tables(&self) -> Result<(), String> {
        self.conn
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS kg_triples (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    subject TEXT NOT NULL,
                    predicate TEXT NOT NULL,
                    object TEXT NOT NULL,
                    valid_from INTEGER NOT NULL,
                    valid_to INTEGER,
                    source TEXT NOT NULL DEFAULT 'manual',
                    confidence REAL NOT NULL DEFAULT 0.8,
                    created_at INTEGER NOT NULL
                );
                CREATE INDEX IF NOT EXISTS idx_kg_subject ON kg_triples(subject);
                CREATE INDEX IF NOT EXISTS idx_kg_predicate ON kg_triples(predicate);
                CREATE INDEX IF NOT EXISTS idx_kg_valid ON kg_triples(valid_from, valid_to);",
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// 添加事实
    pub fn add_fact(
        &self,
        subject: &str,
        predicate: &str,
        object: &str,
        source: &str,
        confidence: f32,
    ) -> Result<i64, String> {
        let now = now_millis();
        self.conn
            .execute(
                "INSERT INTO kg_triples (subject, predicate, object, valid_from, valid_to, source, confidence, created_at)
                 VALUES (?1, ?2, ?3, ?4, NULL, ?5, ?6, ?4)",
                params![subject, predicate, object, now, source, confidence],
            )
            .map_err(|e| e.to_string())?;
        Ok(self.conn.last_insert_rowid())
    }

    /// 失效一个事实（设置 valid_to）
    pub fn invalidate(&self, id: i64) -> Result<(), String> {
        let now = now_millis();
        self.conn
            .execute(
                "UPDATE kg_triples SET valid_to = ?1 WHERE id = ?2 AND valid_to IS NULL",
                params![now, id],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// 查询实体的所有有效事实
    pub fn query_entity(&self, subject: &str, as_of: Option<i64>) -> Result<Vec<Triple>, String> {
        let as_of = as_of.unwrap_or_else(now_millis);
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, subject, predicate, object, valid_from, valid_to, source, confidence, created_at
                 FROM kg_triples
                 WHERE subject = ?1 AND valid_from <= ?2 AND (valid_to IS NULL OR valid_to > ?2)
                 ORDER BY confidence DESC, created_at DESC",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map(params![subject, as_of], |row| {
                Ok(Triple {
                    id: row.get(0)?,
                    subject: row.get(1)?,
                    predicate: row.get(2)?,
                    object: row.get(3)?,
                    valid_from: row.get(4)?,
                    valid_to: row.get(5)?,
                    source: row.get(6)?,
                    confidence: row.get(7)?,
                    created_at: row.get(8)?,
                })
            })
            .map_err(|e| e.to_string())?;

        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    /// 查询关系（subject + predicate）
    pub fn query_relation(&self, subject: &str, predicate: &str) -> Result<Vec<Triple>, String> {
        let now = now_millis();
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, subject, predicate, object, valid_from, valid_to, source, confidence, created_at
                 FROM kg_triples
                 WHERE subject = ?1 AND predicate = ?2 AND valid_from <= ?3 AND (valid_to IS NULL OR valid_to > ?3)
                 ORDER BY confidence DESC",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map(params![subject, predicate, now], |row| {
                Ok(Triple {
                    id: row.get(0)?,
                    subject: row.get(1)?,
                    predicate: row.get(2)?,
                    object: row.get(3)?,
                    valid_from: row.get(4)?,
                    valid_to: row.get(5)?,
                    source: row.get(6)?,
                    confidence: row.get(7)?,
                    created_at: row.get(8)?,
                })
            })
            .map_err(|e| e.to_string())?;

        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    /// 搜索（模糊匹配 subject 或 object）
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<Triple>, String> {
        let pattern = format!("%{}%", query);
        let now = now_millis();
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, subject, predicate, object, valid_from, valid_to, source, confidence, created_at
                 FROM kg_triples
                 WHERE (subject LIKE ?1 OR object LIKE ?1) AND valid_from <= ?2 AND (valid_to IS NULL OR valid_to > ?2)
                 ORDER BY confidence DESC, created_at DESC
                 LIMIT ?3",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map(params![pattern, now, limit as i64], |row| {
                Ok(Triple {
                    id: row.get(0)?,
                    subject: row.get(1)?,
                    predicate: row.get(2)?,
                    object: row.get(3)?,
                    valid_from: row.get(4)?,
                    valid_to: row.get(5)?,
                    source: row.get(6)?,
                    confidence: row.get(7)?,
                    created_at: row.get(8)?,
                })
            })
            .map_err(|e| e.to_string())?;

        Ok(rows.filter_map(|r| r.ok()).collect())
    }
}

fn now_millis() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_kg() -> KnowledgeGraph {
        KnowledgeGraph::new_in_memory().unwrap()
    }

    #[test]
    fn add_and_query() {
        let kg = make_kg();
        kg.add_fact("用户", "喜欢", "编程", "conversation", 0.9)
            .unwrap();
        let facts = kg.query_entity("用户", None).unwrap();
        assert_eq!(facts.len(), 1);
        assert_eq!(facts[0].object, "编程");
    }

    #[test]
    fn invalidate_fact() {
        let kg = make_kg();
        let id = kg
            .add_fact("用户", "居住", "北京", "conversation", 0.8)
            .unwrap();
        kg.invalidate(id).unwrap();
        let facts = kg.query_entity("用户", None).unwrap();
        assert!(facts.is_empty());
    }

    #[test]
    fn query_relation() {
        let kg = make_kg();
        kg.add_fact("用户", "喜欢", "Rust", "conversation", 0.9)
            .unwrap();
        kg.add_fact("用户", "喜欢", "TypeScript", "conversation", 0.7)
            .unwrap();
        kg.add_fact("用户", "工作", "编程", "conversation", 0.8)
            .unwrap();
        let likes = kg.query_relation("用户", "喜欢").unwrap();
        assert_eq!(likes.len(), 2);
    }

    #[test]
    fn search_fuzzy() {
        let kg = make_kg();
        kg.add_fact("猫", "是", "动物", "conversation", 0.9)
            .unwrap();
        kg.add_fact("狗", "是", "动物", "conversation", 0.9)
            .unwrap();
        let results = kg.search("动物", 10).unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn temporal_query() {
        let kg = make_kg();
        let id = kg
            .add_fact("用户", "状态", "开心", "conversation", 0.8)
            .unwrap();
        kg.invalidate(id).unwrap();
        kg.add_fact("用户", "状态", "平静", "conversation", 0.8)
            .unwrap();
        let current = kg.query_entity("用户", None).unwrap();
        assert_eq!(current.len(), 1);
        assert_eq!(current[0].object, "平静");
    }
}
