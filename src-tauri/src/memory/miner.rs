//! 对话记忆挖掘
//!
//! 从对话中提取值得记住的信息。

use super::store::{ChatMessage, MemoryStore};
use serde::{Deserialize, Serialize};

/// 提取的记忆类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MinedMemory {
    Preference { content: String },
    Fact { content: String },
    Event { content: String },
    Emotion { content: String },
    Knowledge { content: String },
}

/// 对话记忆挖掘器
pub struct ConversationMiner {
    store: MemoryStore,
}

impl ConversationMiner {
    pub fn new(store: MemoryStore) -> Self {
        Self { store }
    }

    /// 从对话消息中挖掘记忆（简单关键词匹配）
    pub fn mine(&self, messages: &[ChatMessage]) -> Vec<MinedMemory> {
        let mut results = Vec::new();

        for msg in messages {
            if msg.role != "user" {
                continue;
            }
            let content = &msg.content;
            let lower = content.to_lowercase();

            // 偏好
            for pat in &[
                "我喜欢",
                "我不喜欢",
                "我偏好",
                "我爱",
                "我讨厌",
                "i like",
                "i hate",
                "i prefer",
            ] {
                if lower.contains(pat) {
                    results.push(MinedMemory::Preference {
                        content: content.clone(),
                    });
                    break;
                }
            }

            // 事实
            for pat in &[
                "我叫",
                "我的名字",
                "我是",
                "我在",
                "我住",
                "我的工作是",
                "my name is",
                "i live in",
                "i work at",
            ] {
                if lower.contains(pat) {
                    results.push(MinedMemory::Fact {
                        content: content.clone(),
                    });
                    break;
                }
            }

            // 事件
            for pat in &[
                "今天我",
                "昨天我",
                "明天我要",
                "我去",
                "i went to",
                "i did",
                "today i",
            ] {
                if lower.contains(pat) {
                    results.push(MinedMemory::Event {
                        content: content.clone(),
                    });
                    break;
                }
            }
        }

        // 去重：同一条消息只提取一次
        results.dedup_by(|a, b| match (a, b) {
            (MinedMemory::Preference { content: ca }, MinedMemory::Preference { content: cb }) => {
                ca == cb
            }
            (MinedMemory::Fact { content: ca }, MinedMemory::Fact { content: cb }) => ca == cb,
            (MinedMemory::Event { content: ca }, MinedMemory::Event { content: cb }) => ca == cb,
            _ => false,
        });

        results
    }

    /// 将挖掘结果存入记忆
    // TODO: 单线程下安全，但大量记忆入库时应用事务包裹以保证原子性
    pub fn save_mined(&self, memories: &[MinedMemory]) -> Result<Vec<i64>, String> {
        let mut ids = Vec::new();
        for memory in memories {
            let (content, category) = match memory {
                MinedMemory::Preference { content } => (content.as_str(), "preference"),
                MinedMemory::Fact { content } => (content.as_str(), "fact"),
                MinedMemory::Event { content } => (content.as_str(), "event"),
                MinedMemory::Emotion { content } => (content.as_str(), "emotion"),
                MinedMemory::Knowledge { content } => (content.as_str(), "knowledge"),
            };
            let id = self
                .store
                .save_drawer(content, "daily", category, "mined", "mined", "", 0.5)?;
            ids.push(id);
        }
        Ok(ids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_miner() -> ConversationMiner {
        ConversationMiner::new(MemoryStore::new(":memory:").unwrap())
    }

    fn msg(role: &str, content: &str) -> ChatMessage {
        ChatMessage {
            id: "test".into(),
            session_id: "test".into(),
            role: role.into(),
            content: content.into(),
            emotion_tag: None,
            created_at: 1000,
        }
    }

    #[test]
    fn mine_preference() {
        let miner = make_miner();
        let results = miner.mine(&[msg("user", "我喜欢编程")]);
        assert!(matches!(results[0], MinedMemory::Preference { .. }));
    }

    #[test]
    fn mine_fact() {
        let miner = make_miner();
        let results = miner.mine(&[msg("user", "我叫小明")]);
        assert!(matches!(results[0], MinedMemory::Fact { .. }));
    }

    #[test]
    fn mine_event() {
        let miner = make_miner();
        let results = miner.mine(&[msg("user", "今天我去爬山了")]);
        assert!(matches!(results[0], MinedMemory::Event { .. }));
    }

    #[test]
    fn mine_ignores_assistant() {
        let miner = make_miner();
        let results = miner.mine(&[msg("assistant", "我喜欢编程")]);
        assert!(results.is_empty());
    }

    #[test]
    fn save_mined() {
        let miner = make_miner();
        let memories = vec![MinedMemory::Preference {
            content: "我喜欢Rust".into(),
        }];
        let ids = miner.save_mined(&memories).unwrap();
        assert_eq!(ids.len(), 1);
        assert!(ids[0] > 0);
    }
}
