//! Palace 层级管理
//!
//! Wing（翼）→ Room（房间）→ Drawer（抽屉）的三层记忆组织结构。

use super::store::MemoryStore;
use serde::{Deserialize, Serialize};

/// Wing（翼）— 顶层分类
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WingInfo {
    pub name: String,
    pub room_count: usize,
    pub drawer_count: usize,
}

/// Palace 层级管理
pub struct Palace {
    store: MemoryStore,
}

impl Palace {
    pub fn new(store: MemoryStore) -> Self {
        Self { store }
    }

    /// 列出所有 wing 及其统计
    // TODO: O(5N) 查询，大量数据时需优化为 GROUP BY 聚合查询
    pub fn list_wings(&self) -> Vec<WingInfo> {
        let wings = ["daily", "work", "interest", "emotion", "knowledge"];
        wings
            .iter()
            .map(|w| {
                let drawers = self.store.get_drawers_by_wing(w, 100).unwrap_or_default();
                let rooms: std::collections::HashSet<&str> =
                    drawers.iter().map(|d| d.room.as_str()).collect();
                WingInfo {
                    name: w.to_string(),
                    room_count: rooms.len(),
                    drawer_count: drawers.len(),
                }
            })
            .collect()
    }

    /// 列出 wing 下的 room
    pub fn list_rooms(&self, wing: &str) -> Vec<String> {
        let drawers = self
            .store
            .get_drawers_by_wing(wing, 1000)
            .unwrap_or_default();
        let mut rooms: Vec<String> = drawers.iter().map(|d| d.room.clone()).collect();
        rooms.sort();
        rooms.dedup();
        rooms
    }

    /// 简单关键词自动分类
    pub fn auto_classify(&self, content: &str) -> (&'static str, &'static str) {
        let lower = content.to_lowercase();

        // 情感类
        for kw in &[
            "开心", "难过", "生气", "焦虑", "高兴", "sad", "happy", "angry", "喜欢", "讨厌", "害怕",
        ] {
            if lower.contains(kw) {
                return ("emotion", "feelings");
            }
        }

        // 工作类
        for kw in &[
            "工作", "项目", "会议", "任务", "代码", "bug", "部署", "work", "project", "meeting",
        ] {
            if lower.contains(kw) {
                return ("work", "general");
            }
        }

        // 兴趣类
        for kw in &[
            "游戏", "电影", "音乐", "运动", "旅游", "美食", "game", "movie", "music",
        ] {
            if lower.contains(kw) {
                return ("interest", "hobbies");
            }
        }

        // 知识类
        for kw in &[
            "学习", "教程", "原理", "算法", "定义", "概念", "learn", "tutorial",
        ] {
            if lower.contains(kw) {
                return ("knowledge", "general");
            }
        }

        ("daily", "general")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_palace() -> Palace {
        Palace::new(MemoryStore::new(":memory:").unwrap())
    }

    #[test]
    fn list_wings() {
        let palace = make_palace();
        let wings = palace.list_wings();
        assert_eq!(wings.len(), 5);
        assert!(wings.iter().any(|w| w.name == "daily"));
    }

    #[test]
    fn auto_classify_emotion() {
        let palace = make_palace();
        let (wing, room) = palace.auto_classify("今天很开心");
        assert_eq!(wing, "emotion");
    }

    #[test]
    fn auto_classify_work() {
        let palace = make_palace();
        let (wing, _) = palace.auto_classify("我有一个项目要完成");
        assert_eq!(wing, "work");
    }

    #[test]
    fn auto_classify_default() {
        let palace = make_palace();
        let (wing, _) = palace.auto_classify("今天天气不错");
        assert_eq!(wing, "daily");
    }
}
