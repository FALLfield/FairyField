//! 记忆工具模块
//!
//! memory_search 和 memory_save。
//! 支持 mock 模式（new()）和真实 MemoryLayers 访问（with_layers()）。

use super::executor::{Tool, ToolError};
use crate::memory::MemoryLayers;
use serde::Deserialize;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

// ---------------------------------------------------------------------------
// SearchParams
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct SearchParams {
    query: String,
    #[serde(default = "default_limit")]
    limit: u32,
}

fn default_limit() -> u32 {
    5
}

// ---------------------------------------------------------------------------
// MemorySearchTool
// ---------------------------------------------------------------------------

/// 搜索长期记忆（FTS5 全文搜索）
pub struct MemorySearchTool {
    layers: Option<Arc<Mutex<MemoryLayers>>>,
}

impl MemorySearchTool {
    /// Mock 模式——用于测试和未初始化记忆系统的场景
    pub fn new() -> Self {
        Self { layers: None }
    }

    /// 真实模式——接入 MemoryLayers
    pub fn with_layers(layers: Arc<Mutex<MemoryLayers>>) -> Self {
        Self {
            layers: Some(layers),
        }
    }
}

impl Tool for MemorySearchTool {
    fn name(&self) -> &str {
        "memory_search"
    }

    fn description(&self) -> &str {
        "搜索长期记忆"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "query": { "type": "string", "description": "搜索关键词" },
                "limit": { "type": "number", "description": "结果数量限制" }
            },
            "required": ["query"]
        })
    }

    fn validate_input(&self, input: &str) -> bool {
        serde_json::from_str::<SearchParams>(input).is_ok()
    }

    fn execute(
        &self,
        input: &str,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<String, ToolError>> + Send + '_>> {
        let params = match serde_json::from_str::<SearchParams>(input) {
            Ok(p) => p,
            Err(e) => {
                return Box::pin(async move { Err(ToolError::ValidationFailed(e.to_string())) })
            }
        };

        match &self.layers {
            Some(layers) => {
                let layers = Arc::clone(layers);
                Box::pin(async move {
                    let layers = layers
                        .lock()
                        .map_err(|e| ToolError::ExecutionFailed(format!("锁获取失败: {e}")))?;
                    let results = layers
                        .search(&params.query, params.limit as usize)
                        .map_err(|e| ToolError::ExecutionFailed(format!("记忆搜索失败: {e}")))?;
                    // 序列化搜索结果（只保留关键字段）
                    let summary: Vec<serde_json::Value> = results
                        .iter()
                        .map(|d| {
                            serde_json::json!({
                                "id": d.id,
                                "content": d.content,
                                "wing": d.wing,
                                "room": d.room,
                                "hall": d.hall,
                            })
                        })
                        .collect();
                    Ok(serde_json::to_string(&summary).unwrap_or_else(|_| "[]".into()))
                })
            }
            None => Box::pin(async move {
                Ok(format!(
                    "[memory_search mock] 搜索 '{}'，最多 {} 条",
                    params.query, params.limit
                ))
            }),
        }
    }
}

// ---------------------------------------------------------------------------
// SaveParams
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct SaveParams {
    content: String,
    #[serde(default = "default_category")]
    category: String,
}

fn default_category() -> String {
    "general".into()
}

/// 根据 content 关键词自动分类到 wing/room
/// 镜像 Palace::auto_classify 逻辑，无需 Palace 实例
fn auto_classify(content: &str) -> (&'static str, &'static str) {
    let lower = content.to_lowercase();

    for kw in &[
        "开心", "难过", "生气", "焦虑", "高兴", "sad", "happy", "angry", "喜欢", "讨厌", "害怕",
    ] {
        if lower.contains(kw) {
            return ("emotion", "feelings");
        }
    }

    for kw in &[
        "工作", "项目", "会议", "任务", "代码", "bug", "部署", "work", "project", "meeting",
    ] {
        if lower.contains(kw) {
            return ("work", "general");
        }
    }

    for kw in &[
        "游戏", "电影", "音乐", "运动", "旅游", "美食", "game", "movie", "music",
    ] {
        if lower.contains(kw) {
            return ("interest", "hobbies");
        }
    }

    for kw in &[
        "学习", "教程", "原理", "算法", "定义", "概念", "learn", "tutorial",
    ] {
        if lower.contains(kw) {
            return ("knowledge", "general");
        }
    }

    ("daily", "general")
}

// ---------------------------------------------------------------------------
// MemorySaveTool
// ---------------------------------------------------------------------------

/// 保存信息到长期记忆
pub struct MemorySaveTool {
    layers: Option<Arc<Mutex<MemoryLayers>>>,
}

impl MemorySaveTool {
    /// Mock 模式——用于测试和未初始化记忆系统的场景
    pub fn new() -> Self {
        Self { layers: None }
    }

    /// 真实模式——接入 MemoryLayers
    pub fn with_layers(layers: Arc<Mutex<MemoryLayers>>) -> Self {
        Self {
            layers: Some(layers),
        }
    }
}

impl Tool for MemorySaveTool {
    fn name(&self) -> &str {
        "memory_save"
    }

    fn description(&self) -> &str {
        "保存信息到长期记忆"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "content": { "type": "string", "description": "要保存的内容" },
                "category": { "type": "string", "description": "分类" }
            },
            "required": ["content"]
        })
    }

    fn validate_input(&self, input: &str) -> bool {
        serde_json::from_str::<SaveParams>(input).is_ok()
    }

    fn execute(
        &self,
        input: &str,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<String, ToolError>> + Send + '_>> {
        let params = match serde_json::from_str::<SaveParams>(input) {
            Ok(p) => p,
            Err(e) => {
                return Box::pin(async move { Err(ToolError::ValidationFailed(e.to_string())) })
            }
        };

        match &self.layers {
            Some(layers) => {
                let layers = Arc::clone(layers);
                Box::pin(async move {
                    let layers = layers
                        .lock()
                        .map_err(|e| ToolError::ExecutionFailed(format!("锁获取失败: {e}")))?;
                    // 根据 content 关键词自动分类 wing/room
                    let (wing, room) = auto_classify(&params.content);
                    let hall = &params.category;

                    let id = layers
                        .add_drawer(&params.content, wing, room, hall)
                        .map_err(|e| ToolError::ExecutionFailed(format!("记忆保存失败: {e}")))?;
                    Ok(serde_json::json!({
                        "status": "ok",
                        "id": id,
                        "wing": wing,
                        "room": room,
                    })
                    .to_string())
                })
            }
            None => Box::pin(async move {
                Ok(format!(
                    "[memory_save mock] 已保存 '{}' 分类为 '{}'",
                    &params.content[..params.content.len().min(50)],
                    params.category
                ))
            }),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- Mock path tests (backward compat) ---

    #[test]
    fn validate_search() {
        let tool = MemorySearchTool::new();
        assert!(tool.validate_input(r#"{"query": "test"}"#));
    }

    #[tokio::test]
    async fn execute_search_mock() {
        let tool = MemorySearchTool::new();
        let result = tool.execute(r#"{"query": "test"}"#).await;
        assert!(result.unwrap().contains("mock"));
    }

    #[test]
    fn validate_save() {
        let tool = MemorySaveTool::new();
        assert!(tool.validate_input(r#"{"content": "hello", "category": "test"}"#));
    }

    #[tokio::test]
    async fn execute_save_mock() {
        let tool = MemorySaveTool::new();
        let result = tool
            .execute(r#"{"content": "我喜欢编程", "category": "preference"}"#)
            .await;
        assert!(result.unwrap().contains("mock"));
    }

    // --- Real MemoryLayers tests ---

    fn make_layers() -> Arc<Mutex<MemoryLayers>> {
        let store = crate::memory::MemoryStore::new(":memory:").unwrap();
        Arc::new(Mutex::new(MemoryLayers::new(store)))
    }

    #[tokio::test]
    async fn search_with_layers_empty() {
        let layers = make_layers();
        let tool = MemorySearchTool::with_layers(layers);
        let result = tool.execute(r#"{"query": "test"}"#).await;
        let json = result.unwrap();
        // 空结果应返回 []
        assert_eq!(json, "[]");
    }

    #[tokio::test]
    async fn save_and_search_roundtrip() {
        let layers = make_layers();
        let save_tool = MemorySaveTool::with_layers(Arc::clone(&layers));

        // NOTE: FTS5 默认分词器不支持中文，使用 ASCII 内容测试 round-trip
        let save_result = save_tool
            .execute(r#"{"content": "user likes rust programming", "category": "preference"}"#)
            .await
            .unwrap();
        let save_json: serde_json::Value = serde_json::from_str(&save_result).unwrap();
        assert_eq!(save_json["status"], "ok");

        // 搜索应能找到
        let search_tool = MemorySearchTool::with_layers(layers);
        let search_result = search_tool
            .execute(r#"{"query": "rust"}"#)
            .await
            .unwrap();
        let found: Vec<serde_json::Value> = serde_json::from_str(&search_result).unwrap();
        assert_eq!(found.len(), 1);
        assert!(found[0]["content"].as_str().unwrap().contains("rust"));
    }

    #[tokio::test]
    async fn search_with_layers_returns_json() {
        let layers = make_layers();

        // 存入几条记忆
        {
            let l = layers.lock().unwrap();
            l.add_drawer("Rust 是安全的", "knowledge", "general", "tech")
                .unwrap();
            l.add_drawer("今天开会讨论了项目进度", "work", "general", "meeting")
                .unwrap();
        }

        let tool = MemorySearchTool::with_layers(layers);
        let result = tool
            .execute(r#"{"query": "Rust", "limit": 5}"#)
            .await;
        let json: Vec<serde_json::Value> = serde_json::from_str(&result.unwrap()).unwrap();
        assert!(!json.is_empty());
        assert!(json[0]["content"].as_str().unwrap().contains("Rust"));
        assert!(json[0]["wing"].as_str().unwrap().eq("knowledge"));
    }
}
