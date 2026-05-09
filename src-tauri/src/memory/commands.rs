//! Tauri IPC commands for memory subsystem

use super::embedding::EmbeddingSearcher;
use super::knowledge_graph::KnowledgeGraph;
use super::layers::MemoryLayers;
use super::miner::ConversationMiner;
use super::palace::Palace;
use super::store::{ChatMessage, Drawer};
use super::WakeUpContext;
use std::sync::{Arc, Mutex};
use tauri::State;

/// Memory subsystem managed state
pub struct MemoryState {
    pub layers: Arc<Mutex<MemoryLayers>>,
    pub miner: Arc<Mutex<ConversationMiner>>,
    pub palace: Mutex<Palace>,
    pub knowledge_graph: Mutex<KnowledgeGraph>,
    pub searcher: Mutex<EmbeddingSearcher>,
}

#[tauri::command]
pub fn memory_wake_up(state: State<'_, MemoryState>) -> Result<WakeUpContext, String> {
    state.layers.lock().map_err(|e| e.to_string())?.wake_up()
}

#[tauri::command]
pub fn memory_search(
    query: String,
    limit: Option<usize>,
    state: State<'_, MemoryState>,
) -> Result<Vec<Drawer>, String> {
    state
        .layers
        .lock()
        .map_err(|e| e.to_string())?
        .search(&query, limit.unwrap_or(10))
}

#[tauri::command]
pub fn memory_recall(
    wing: String,
    limit: Option<usize>,
    state: State<'_, MemoryState>,
) -> Result<Vec<Drawer>, String> {
    state
        .layers
        .lock()
        .map_err(|e| e.to_string())?
        .recall(&wing, limit.unwrap_or(10))
}

#[tauri::command]
pub fn memory_add_drawer(
    content: String,
    wing: String,
    room: String,
    hall: String,
    state: State<'_, MemoryState>,
) -> Result<i64, String> {
    state
        .layers
        .lock()
        .map_err(|e| e.to_string())?
        .add_drawer(&content, &wing, &room, &hall)
}

#[tauri::command]
pub fn memory_mine(messages: String, state: State<'_, MemoryState>) -> Result<usize, String> {
    // Parse messages as JSON array of {role, content} objects
    let parsed: Vec<serde_json::Value> =
        serde_json::from_str(&messages).map_err(|e| format!("无效的消息格式: {}", e))?;

    let chat_messages: Vec<ChatMessage> = parsed
        .iter()
        .filter_map(|m| {
            let role = m.get("role").and_then(|r| r.as_str())?;
            let content = m.get("content").and_then(|c| c.as_str())?;
            Some(ChatMessage {
                id: String::new(),
                session_id: String::new(),
                role: role.to_string(),
                content: content.to_string(),
                emotion_tag: None,
                created_at: 0,
            })
        })
        .collect();

    let mined = state
        .miner
        .lock()
        .map_err(|e| e.to_string())?
        .mine(&chat_messages);
    let count = mined.len();
    if count > 0 {
        state
            .miner
            .lock()
            .map_err(|e| e.to_string())?
            .save_mined(&mined)?;
    }
    Ok(count)
}

#[tauri::command]
pub fn memory_set_meta(
    key: String,
    value: String,
    layer: Option<i32>,
    state: State<'_, MemoryState>,
) -> Result<(), String> {
    state
        .layers
        .lock()
        .map_err(|e| e.to_string())?
        .set_meta(&key, &value, layer.unwrap_or(0))
}

#[tauri::command]
pub fn memory_list_wings(state: State<'_, MemoryState>) -> Result<Vec<serde_json::Value>, String> {
    let wings = state.palace.lock().map_err(|e| e.to_string())?.list_wings();
    Ok(wings
        .into_iter()
        .map(|w| serde_json::to_value(w).unwrap_or_default())
        .collect())
}

// ========== 知识图谱 IPC 命令 ==========

#[tauri::command]
pub fn kg_add_fact(
    subject: String,
    predicate: String,
    object: String,
    source: Option<String>,
    confidence: Option<f32>,
    state: State<'_, MemoryState>,
) -> Result<i64, String> {
    state
        .knowledge_graph
        .lock()
        .map_err(|e| e.to_string())?
        .add_fact(
            &subject,
            &predicate,
            &object,
            &source.unwrap_or_default(),
            confidence.unwrap_or(1.0),
        )
}

#[tauri::command]
pub fn kg_invalidate(id: i64, state: State<'_, MemoryState>) -> Result<(), String> {
    state
        .knowledge_graph
        .lock()
        .map_err(|e| e.to_string())?
        .invalidate(id)
}

#[tauri::command]
pub fn kg_query_entity(
    subject: String,
    as_of: Option<i64>,
    state: State<'_, MemoryState>,
) -> Result<Vec<super::knowledge_graph::Triple>, String> {
    state
        .knowledge_graph
        .lock()
        .map_err(|e| e.to_string())?
        .query_entity(&subject, as_of)
}

#[tauri::command]
pub fn kg_query_relation(
    subject: String,
    predicate: String,
    state: State<'_, MemoryState>,
) -> Result<Vec<super::knowledge_graph::Triple>, String> {
    state
        .knowledge_graph
        .lock()
        .map_err(|e| e.to_string())?
        .query_relation(&subject, &predicate)
}

#[tauri::command]
pub fn kg_search(
    query: String,
    limit: Option<usize>,
    state: State<'_, MemoryState>,
) -> Result<Vec<super::knowledge_graph::Triple>, String> {
    state
        .knowledge_graph
        .lock()
        .map_err(|e| e.to_string())?
        .search(&query, limit.unwrap_or(10))
}

// ========== 向量搜索 IPC 命令 ==========

#[tauri::command]
pub fn memory_vector_search(
    query: String,
    limit: Option<usize>,
    state: State<'_, MemoryState>,
) -> Result<Vec<serde_json::Value>, String> {
    let results = state
        .searcher
        .lock()
        .map_err(|e| e.to_string())?
        .search(&query, limit.unwrap_or(5));
    Ok(results
        .into_iter()
        .map(|(id, score)| {
            serde_json::json!({ "id": id, "score": score })
        })
        .collect())
}
