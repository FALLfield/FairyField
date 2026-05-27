//! 成长引擎
//!
//! 从日常交互中沉淀经验，构建用户画像。

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

/// 经验条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Experience {
    pub id: i64,
    pub category: String,
    pub content: String,
    pub emotional_valence: Option<f32>,
    pub timestamp: i64,
}

/// 用户画像
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub preferences: Vec<String>,
    pub interests: Vec<String>,
    pub communication_style: String,
    pub experience_count: usize,
}

/// 成长引擎
pub struct GrowthEngine {
    conn: Connection,
}

impl GrowthEngine {
    pub fn new(conn: Connection) -> Result<Self, String> {
        let engine = Self { conn };
        engine.init_tables()?;
        Ok(engine)
    }

    pub fn new_in_memory() -> Result<Self, String> {
        Self::new(Connection::open_in_memory().map_err(|e| e.to_string())?)
    }

    fn init_tables(&self) -> Result<(), String> {
        self.conn
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS experiences (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    category TEXT NOT NULL,
                    content TEXT NOT NULL,
                    emotional_valence REAL,
                    timestamp INTEGER NOT NULL
                );
                CREATE INDEX IF NOT EXISTS idx_experiences_category ON experiences(category);
                CREATE INDEX IF NOT EXISTS idx_experiences_ts ON experiences(timestamp);",
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// 保存经验
    pub fn save_experience(
        &self,
        category: &str,
        content: &str,
        emotional_valence: Option<f32>,
    ) -> Result<i64, String> {
        let ts = now_millis();
        self.conn
            .execute(
                "INSERT INTO experiences (category, content, emotional_valence, timestamp) VALUES (?1, ?2, ?3, ?4)",
                params![category, content, emotional_valence, ts],
            )
            .map_err(|e| e.to_string())?;
        Ok(self.conn.last_insert_rowid())
    }

    /// 获取最近经验
    pub fn get_recent_experiences(&self, limit: usize) -> Result<Vec<Experience>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, category, content, emotional_valence, timestamp FROM experiences ORDER BY timestamp DESC LIMIT ?1",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![limit as i64], |row| {
                Ok(Experience {
                    id: row.get(0)?,
                    category: row.get(1)?,
                    content: row.get(2)?,
                    emotional_valence: row.get(3)?,
                    timestamp: row.get(4)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    /// 获取用户画像（从经验中提取）
    pub fn get_user_profile(&self) -> Result<UserProfile, String> {
        let mut pref_stmt = self
            .conn
            .prepare(
                "SELECT DISTINCT content FROM experiences WHERE category = 'preference' LIMIT 20",
            )
            .map_err(|e| e.to_string())?;
        let preferences: Vec<String> = pref_stmt
            .query_map([], |row| row.get(0))
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();

        let mut int_stmt = self
            .conn
            .prepare(
                "SELECT DISTINCT content FROM experiences WHERE category = 'interest' LIMIT 20",
            )
            .map_err(|e| e.to_string())?;
        let interests: Vec<String> = int_stmt
            .query_map([], |row| row.get(0))
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();

        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM experiences", [], |row| row.get(0))
            .map_err(|e| e.to_string())?;

        Ok(UserProfile {
            preferences,
            interests,
            communication_style: String::new(),
            experience_count: count as usize,
        })
    }

    /// 获取各分类经验数量
    pub fn get_category_stats(&self) -> Result<Vec<(String, i64)>, String> {
        let mut stmt = self
            .conn
            .prepare("SELECT category, COUNT(*) FROM experiences GROUP BY category")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
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

    fn make_engine() -> GrowthEngine {
        GrowthEngine::new_in_memory().unwrap()
    }

    #[test]
    fn save_and_retrieve_experience() {
        let engine = make_engine();
        let id = engine
            .save_experience("preference", "喜欢猫", None)
            .unwrap();
        assert!(id > 0);
        let recent = engine.get_recent_experiences(10).unwrap();
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].content, "喜欢猫");
    }

    #[test]
    fn user_profile_empty() {
        let engine = make_engine();
        let profile = engine.get_user_profile().unwrap();
        assert_eq!(profile.experience_count, 0);
    }

    #[test]
    fn category_stats() {
        let engine = make_engine();
        engine.save_experience("preference", "A", None).unwrap();
        engine.save_experience("interest", "B", None).unwrap();
        engine.save_experience("preference", "C", None).unwrap();
        let stats = engine.get_category_stats().unwrap();
        assert_eq!(stats.len(), 2);
    }
}
