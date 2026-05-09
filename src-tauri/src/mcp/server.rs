//! MCP Server — Claude Code 集成
//!
//! 将 Fairy 的记忆系统暴露为 Claude Code 可用的工具。
//! 通过 Tauri IPC 命令提供 fairy_memory_search 和 fairy_wake_up。

use crate::memory::layers::MemoryLayers;
use std::sync::Arc;
use std::sync::Mutex;

/// MCP Server 管理状态
pub struct McpState {
    pub layers: Arc<Mutex<MemoryLayers>>,
}

/// MCP 工具定义
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// 返回所有可用的 MCP 工具定义
pub fn list_mcp_tools() -> Vec<McpTool> {
    vec![
        McpTool {
            name: "fairy_memory_search".into(),
            description: "搜索 Fairy 的长期记忆。返回与查询相关的记忆条目。".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "搜索关键词" },
                    "limit": { "type": "number", "description": "结果数量限制（默认5）" }
                },
                "required": ["query"]
            }),
        },
        McpTool {
            name: "fairy_wake_up".into(),
            description: "唤醒 Fairy，获取她的身份、核心记忆和近期重要信息（L0+L1+L2 索引，约600 tokens）。".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        McpTool {
            name: "fairy_recall_wing".into(),
            description: "按记忆区域（wing）召回记忆。wing 例如 '关系', '偏好', '事件'。".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "wing": { "type": "string", "description": "记忆区域名称" },
                    "limit": { "type": "number", "description": "结果数量限制（默认10）" }
                },
                "required": ["wing"]
            }),
        },
    ]
}

#[tauri::command]
pub fn mcp_list_tools() -> Vec<serde_json::Value> {
    list_mcp_tools()
        .into_iter()
        .map(|t| serde_json::to_value(t).unwrap_or_default())
        .collect()
}

#[tauri::command]
pub fn mcp_fairy_memory_search(
    query: String,
    limit: Option<usize>,
    state: tauri::State<'_, McpState>,
) -> Result<serde_json::Value, String> {
    let limit = limit.unwrap_or(5);
    let layers = state.layers.lock().map_err(|e| e.to_string())?;
    let results = layers.search(&query, limit)?;

    Ok(serde_json::json!({
        "results": results.iter().map(|d| serde_json::json!({
            "id": d.id,
            "content": d.content,
            "wing": d.wing,
            "room": d.room,
            "hall": d.hall,
            "created_at": d.created_at,
        })).collect::<Vec<_>>(),
        "count": results.len(),
        "query": query,
    }))
}

#[tauri::command]
pub fn mcp_fairy_wake_up(state: tauri::State<'_, McpState>) -> Result<serde_json::Value, String> {
    let layers = state.layers.lock().map_err(|e| e.to_string())?;
    let context = layers.wake_up()?;

    Ok(serde_json::json!({
        "identity": context.identity,
        "working_summary": context.working_summary,
        "palace_index": context.palace_index,
        "estimated_tokens": context.estimated_tokens,
    }))
}

#[tauri::command]
pub fn mcp_fairy_recall_wing(
    wing: String,
    limit: Option<usize>,
    state: tauri::State<'_, McpState>,
) -> Result<serde_json::Value, String> {
    let limit = limit.unwrap_or(10);
    let layers = state.layers.lock().map_err(|e| e.to_string())?;
    let results = layers.recall(&wing, limit)?;

    Ok(serde_json::json!({
        "wing": wing,
        "results": results.iter().map(|d| serde_json::json!({
            "id": d.id,
            "content": d.content,
            "room": d.room,
            "hall": d.hall,
            "created_at": d.created_at,
        })).collect::<Vec<_>>(),
        "count": results.len(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_mcp_tools_has_core_tools() {
        let tools = list_mcp_tools();
        assert!(tools.len() >= 3);
        assert!(tools.iter().any(|t| t.name == "fairy_memory_search"));
        assert!(tools.iter().any(|t| t.name == "fairy_wake_up"));
        assert!(tools.iter().any(|t| t.name == "fairy_recall_wing"));
    }

    #[test]
    fn mcp_tool_schemas_valid() {
        for tool in list_mcp_tools() {
            assert!(!tool.name.is_empty());
            assert!(!tool.description.is_empty());
            assert!(tool.parameters.is_object());
        }
    }
}
