//! 持久化记忆模块
//!
//! 基于 SQLite 的对话记忆、4-Layer Stack、Palace 层级存储。

pub mod agentmemory_backend;
pub mod commands;
pub mod embedding;
pub mod knowledge_graph;
pub mod layers;
pub mod miner;
pub mod palace;
pub mod store;

pub use agentmemory_backend::{
    AgentDiaryEntry, AgentMemoryBackend, DuplicateCheck, DuplicateMatch,
};
pub use knowledge_graph::KnowledgeGraph;
pub use layers::{MemoryLayers, WakeUpContext};
pub use miner::ConversationMiner;
pub use palace::Palace;
pub use store::Drawer;
pub use store::{ChatMessage, MemoryStore};
