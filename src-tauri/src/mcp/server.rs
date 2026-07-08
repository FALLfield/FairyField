//! MCP Server — Claude Code 集成
//!
//! 将 Fairy 的记忆系统暴露为 Claude Code 可用的工具。
//! 通过 Tauri IPC 命令提供 fairy_memory_search 和 fairy_wake_up。

use crate::agent::{Toolset, ToolsetError};
use crate::memory::layers::MemoryLayers;
use std::sync::Arc;
use std::sync::Mutex;

/// MCP Server 管理状态
pub struct McpState {
    pub layers: Arc<Mutex<MemoryLayers>>,
    pub toolset: Arc<Toolset>,
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
            description:
                "唤醒 Fairy，获取她的身份、核心记忆和近期重要信息（L0+L1+L2 索引，约600 tokens）。"
                    .into(),
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
        McpTool {
            name: "fairy_execute_tool".into(),
            description:
                "通过 Fairy 的工具系统执行操作。例如：搜索 Obsidian 笔记、管理 GitHub Issues、搜索网络。"
                    .into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "tool_id": {
                        "type": "string",
                        "description": "工具 ID，如 'web.search', 'obsidian.search_notes', 'github.list_issues'"
                    },
                    "params": {
                        "type": "object",
                        "description": "工具参数（JSON object）"
                    }
                },
                "required": ["tool_id", "params"]
            }),
        },
        McpTool {
            name: "fairy_get_context".into(),
            description: "获取 Fairy 的当前上下文：当前情绪、最近的对话摘要、相关记忆。".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "include_memory": {
                        "type": "boolean",
                        "description": "是否包含记忆搜索结果（默认true）"
                    }
                }
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

#[tauri::command]
pub async fn mcp_fairy_execute_tool(
    tool_id: String,
    params: serde_json::Value,
    state: tauri::State<'_, McpState>,
) -> Result<serde_json::Value, String> {
    if tool_id.trim().is_empty() {
        return Err("tool_id 不能为空".into());
    }
    if !params.is_object() {
        return Err("params 必须是 JSON object".into());
    }

    let tool_name = normalize_tool_id(&tool_id);
    let input = serde_json::to_string(&params).map_err(|e| e.to_string())?;
    let output = match state.toolset.execute(&tool_name, &input).await {
        Ok(output) => output,
        Err(first_err) if should_retry_raw_tool_id(&tool_id, &tool_name, &first_err) => state
            .toolset
            .execute(&tool_id, &input)
            .await
            .map_err(|fallback_err| {
                format!(
                    "工具执行失败（{}: {}; fallback {}: {}）",
                    tool_name, first_err, tool_id, fallback_err
                )
            })?,
        Err(err) => return Err(err.to_string()),
    };

    Ok(serde_json::json!({
        "status": "ok",
        "tool_id": tool_id,
        "tool_name": tool_name,
        "result": parse_tool_output(&output),
    }))
}

fn should_retry_raw_tool_id(tool_id: &str, normalized: &str, error: &ToolsetError) -> bool {
    tool_id != normalized
        && !matches!(
            error,
            ToolsetError::SecurityBlocked { .. } | ToolsetError::CommandNotApproved { .. }
        )
}

fn normalize_tool_id(tool_id: &str) -> String {
    let trimmed = tool_id.trim();
    match trimmed {
        "web.search" => "web_search".into(),
        "web.fetch" => "web_fetch".into(),
        "file.read" => "file_read".into(),
        "file.write" => "file_write".into(),
        "memory.search" => "memory_search".into(),
        "memory.save" => "memory_save".into(),
        "git.run" | "git.status" | "git.log" | "git.diff" => "git".into(),
        "terminal.run" | "shell.run" => "terminal".into(),
        "obsidian.search_notes" | "obsidian.search" | "obsidian" => "obsidian_search".into(),
        other
            if other.starts_with("github.")
                || other.starts_with("notion.")
                || other.starts_with("linear.")
                || other.starts_with("gmail.")
                || other.starts_with("calendar.")
                || other.starts_with("slack.")
                || other.starts_with("discord.")
                || other.starts_with("browser.")
                || other.starts_with("cron.")
                || other.starts_with("weather.")
                || other.starts_with("translate.")
                || other.starts_with("image.") =>
        {
            other.split('.').next().unwrap_or(other).into()
        }
        other => other.replace('.', "_"),
    }
}

fn parse_tool_output(output: &str) -> serde_json::Value {
    serde_json::from_str(output).unwrap_or_else(|_| serde_json::json!(output))
}

#[tauri::command]
pub fn mcp_fairy_get_context(
    include_memory: Option<bool>,
    state: tauri::State<'_, McpState>,
) -> Result<serde_json::Value, String> {
    let include_memory = include_memory.unwrap_or(true);
    let layers = state.layers.lock().map_err(|e| e.to_string())?;
    let wake_up = layers.wake_up()?;

    let mut context = serde_json::json!({
        "identity": wake_up.identity,
        "working_summary": wake_up.working_summary,
        "estimated_tokens": wake_up.estimated_tokens,
        "emotion": "neutral",
        "recent_topics": [],
    });

    if include_memory {
        // Add a memory snapshot from the palace index
        context["palace_index"] = serde_json::json!(wake_up.palace_index);
    }

    Ok(context)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_mcp_tools_has_core_tools() {
        let tools = list_mcp_tools();
        assert!(tools.len() >= 5);
        assert!(tools.iter().any(|t| t.name == "fairy_memory_search"));
        assert!(tools.iter().any(|t| t.name == "fairy_wake_up"));
        assert!(tools.iter().any(|t| t.name == "fairy_recall_wing"));
        assert!(tools.iter().any(|t| t.name == "fairy_execute_tool"));
        assert!(tools.iter().any(|t| t.name == "fairy_get_context"));
    }

    #[test]
    fn mcp_tool_schemas_valid() {
        for tool in list_mcp_tools() {
            assert!(!tool.name.is_empty());
            assert!(!tool.description.is_empty());
            assert!(tool.parameters.is_object());
        }
    }

    #[test]
    fn normalize_tool_ids() {
        assert_eq!(normalize_tool_id("web.search"), "web_search");
        assert_eq!(normalize_tool_id("github"), "github");
        assert_eq!(normalize_tool_id("github.list_issues"), "github");
        assert_eq!(normalize_tool_id("notion.search"), "notion");
        assert_eq!(
            normalize_tool_id("obsidian.search_notes"),
            "obsidian_search"
        );
        assert_eq!(normalize_tool_id("terminal.run"), "terminal");
    }

    #[test]
    fn parse_tool_output_json_or_text() {
        assert_eq!(parse_tool_output(r#"{"ok":true}"#)["ok"], true);
        assert_eq!(parse_tool_output("plain"), serde_json::json!("plain"));
    }

    #[test]
    fn mcp_does_not_retry_raw_id_after_security_blocks() {
        let security = ToolsetError::SecurityBlocked {
            patterns: vec!["ignore".into()],
            score: 0.9,
        };
        let command = ToolsetError::CommandNotApproved {
            command: "rm -rf /".into(),
            level: "Dangerous".into(),
            reason: "blocked".into(),
        };
        let not_found = ToolsetError::ExecutionFailed("工具未找到".into());

        assert!(!should_retry_raw_tool_id(
            "terminal.run",
            "terminal",
            &security
        ));
        assert!(!should_retry_raw_tool_id(
            "terminal.run",
            "terminal",
            &command
        ));
        assert!(should_retry_raw_tool_id(
            "custom.tool",
            "custom_tool",
            &not_found
        ));
    }
}
