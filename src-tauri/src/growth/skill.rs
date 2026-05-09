//! 技能管理
//!
//! TOML frontmatter + Markdown 格式的技能系统。
//! 支持渐进式披露：list → view → load。

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

/// 技能元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMeta {
    pub name: String,
    pub description: String,
    pub version: String,
    pub category: String,
    pub reference: Option<String>,
}

/// 技能完整内容
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub meta: SkillMeta,
    pub content: String,
}

/// 技能管理器
pub struct SkillManager {
    conn: Connection,
}

impl SkillManager {
    pub fn new(conn: Connection) -> Result<Self, String> {
        let mgr = Self { conn };
        mgr.init_tables()?;
        Ok(mgr)
    }

    pub fn new_in_memory() -> Result<Self, String> {
        Self::new(Connection::open_in_memory().map_err(|e| e.to_string())?)
    }

    fn init_tables(&self) -> Result<(), String> {
        self.conn
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS skills (
                    name TEXT PRIMARY KEY,
                    description TEXT NOT NULL,
                    version TEXT NOT NULL DEFAULT '1.0',
                    category TEXT NOT NULL DEFAULT 'general',
                    reference TEXT,
                    content TEXT NOT NULL,
                    created_at INTEGER NOT NULL,
                    updated_at INTEGER NOT NULL
                );",
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// 列出所有技能元数据
    pub fn list_skills(&self) -> Result<Vec<SkillMeta>, String> {
        let mut stmt = self
            .conn
            .prepare("SELECT name, description, version, category, reference FROM skills")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| {
                Ok(SkillMeta {
                    name: row.get(0)?,
                    description: row.get(1)?,
                    version: row.get(2)?,
                    category: row.get(3)?,
                    reference: row.get(4)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
    }

    /// 查看技能完整内容
    pub fn view_skill(&self, name: &str) -> Result<Option<Skill>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT name, description, version, category, reference, content FROM skills WHERE name = ?1",
            )
            .map_err(|e| e.to_string())?;
        let mut rows = stmt.query_map(params![name], |row| {
            Ok(Skill {
                meta: SkillMeta {
                    name: row.get(0)?,
                    description: row.get(1)?,
                    version: row.get(2)?,
                    category: row.get(3)?,
                    reference: row.get(4)?,
                },
                content: row.get(5)?,
            })
        }).map_err(|e| e.to_string())?;
        match rows.next() {
            Some(row) => Ok(Some(row.map_err(|e| e.to_string())?)),
            None => Ok(None),
        }
    }

    /// 创建新技能
    pub fn create_skill(
        &self,
        name: &str,
        description: &str,
        category: &str,
        content: &str,
    ) -> Result<(), String> {
        let ts = now_millis();
        self.conn.execute(
            "INSERT INTO skills (name, description, version, category, content, created_at, updated_at) VALUES (?1, ?2, '1.0', ?3, ?4, ?5, ?5)",
            params![name, description, category, content, ts],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    /// 更新技能内容
    pub fn update_skill(&self, name: &str, content: &str) -> Result<(), String> {
        let ts = now_millis();
        let rows = self.conn.execute(
            "UPDATE skills SET content = ?1, updated_at = ?2 WHERE name = ?3",
            params![content, ts, name],
        ).map_err(|e| e.to_string())?;
        if rows == 0 {
            return Err(format!("技能 '{}' 不存在", name));
        }
        Ok(())
    }

    /// 删除技能
    pub fn delete_skill(&self, name: &str) -> Result<(), String> {
        let rows = self
            .conn
            .execute("DELETE FROM skills WHERE name = ?1", params![name])
            .map_err(|e| e.to_string())?;
        if rows == 0 {
            return Err(format!("技能 '{}' 不存在", name));
        }
        Ok(())
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

    fn make_mgr() -> SkillManager {
        SkillManager::new_in_memory().unwrap()
    }

    #[test]
    fn create_and_list_skill() {
        let mgr = make_mgr();
        mgr.create_skill("greeting", "打招呼", "social", "用温暖的方式问好")
            .unwrap();
        let skills = mgr.list_skills().unwrap();
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "greeting");
    }

    #[test]
    fn view_skill() {
        let mgr = make_mgr();
        mgr.create_skill("greeting", "打招呼", "social", "你好呀~")
            .unwrap();
        let skill = mgr.view_skill("greeting").unwrap().unwrap();
        assert_eq!(skill.content, "你好呀~");
    }

    #[test]
    fn update_skill() {
        let mgr = make_mgr();
        mgr.create_skill("test", "测试", "general", "v1").unwrap();
        mgr.update_skill("test", "v2").unwrap();
        let skill = mgr.view_skill("test").unwrap().unwrap();
        assert_eq!(skill.content, "v2");
    }

    #[test]
    fn delete_skill() {
        let mgr = make_mgr();
        mgr.create_skill("temp", "临时", "general", "内容").unwrap();
        mgr.delete_skill("temp").unwrap();
        assert!(mgr.view_skill("temp").unwrap().is_none());
    }

    #[test]
    fn view_nonexistent() {
        let mgr = make_mgr();
        assert!(mgr.view_skill("不存在").unwrap().is_none());
    }
}
