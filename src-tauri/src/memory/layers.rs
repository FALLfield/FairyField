//! 4-Layer Memory Stack
//!
//! L0: Identity (~100 tokens) — Fairy 的身份描述
//! L1: Working Memory (~500 tokens) — 近期对话摘要
//! L2: Palace Index (~300 tokens) — 记忆宫殿 wing/room 索引
//! L3: Full Recall — 完整 drawer 内容（按需检索）

use super::store::{Drawer, MemoryStore};
use serde::{Deserialize, Serialize};

/// Wake-up 上下文（L0 + L1 + L2 索引）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WakeUpContext {
    /// L0: 身份描述
    pub identity: String,
    /// L1: 近期摘要
    pub working_summary: String,
    /// L2: 宫殿索引
    pub palace_index: String,
    /// 估计 token 数
    pub estimated_tokens: usize,
}

/// 4-Layer 记忆栈
pub struct MemoryLayers {
    store: MemoryStore,
}

impl MemoryLayers {
    pub fn new(store: MemoryStore) -> Self {
        Self { store }
    }

    /// Wake-up: 加载 L0+L1+L2（~600 tokens 目标）
    pub fn wake_up(&self) -> Result<WakeUpContext, String> {
        let identity = self
            .store
            .get_meta("identity")
            .unwrap_or_else(|| "我是 Fairy，你的桌面 AI 伴侣。".to_string());

        let working_summary = self
            .store
            .get_meta("working_summary")
            .unwrap_or_else(|| "（暂无近期对话摘要）".to_string());

        let palace_index = self
            .store
            .get_meta("palace_index")
            .unwrap_or_else(|| "（暂无记忆索引）".to_string());

        let estimated_tokens = (identity.chars().count()
            + working_summary.chars().count()
            + palace_index.chars().count())
            * 2
            / 3;

        Ok(WakeUpContext {
            identity,
            working_summary,
            palace_index,
            estimated_tokens,
        })
    }

    /// L3: 全文搜索
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<Drawer>, String> {
        self.store.search_fts(query, limit)
    }

    /// 存入新记忆
    pub fn add_drawer(
        &self,
        content: &str,
        wing: &str,
        room: &str,
        hall: &str,
    ) -> Result<i64, String> {
        self.store
            .save_drawer(content, wing, room, hall, "general", "", 0.5)
    }

    /// 按 wing 查询
    pub fn recall(&self, wing: &str, limit: usize) -> Result<Vec<Drawer>, String> {
        self.store.get_drawers_by_wing(wing, limit)
    }

    /// 设置元数据（L0-L2 层）
    pub fn set_meta(&self, key: &str, value: &str, layer: i32) -> Result<(), String> {
        self.store.set_meta(key, value, layer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_layers() -> MemoryLayers {
        let store = MemoryStore::new(":memory:").unwrap();
        MemoryLayers::new(store)
    }

    #[test]
    fn wake_up_returns_defaults() {
        let layers = make_layers();
        let ctx = layers.wake_up().unwrap();
        assert!(ctx.identity.contains("Fairy"));
        assert!(ctx.estimated_tokens < 1000);
    }

    #[test]
    fn add_and_search() {
        let layers = make_layers();
        layers
            .add_drawer("I love programming in Rust", "daily", "hobbies", "tech")
            .unwrap();
        let results = layers.search("programming", 10).unwrap();
        assert!(!results.is_empty());
        assert!(results[0].content.contains("programming"));
    }
}
