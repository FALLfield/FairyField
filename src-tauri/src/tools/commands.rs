//! Tauri IPC commands for tools subsystem

use crate::agent::Toolset;
use std::sync::Arc;
use std::time::Instant;
use tauri::Emitter;
use tauri::State;

/// Tools subsystem managed state
pub struct ToolsState {
    pub toolset: Arc<Toolset>,
}

#[tauri::command]
pub fn tools_list(state: State<'_, ToolsState>) -> Result<Vec<serde_json::Value>, String> {
    Ok(state
        .toolset
        .list_tools()
        .into_iter()
        .map(|info| {
            serde_json::json!({
                "name": info.name,
                "description": info.description,
                "parameters": info.parameters_schema,
            })
        })
        .collect())
}

#[tauri::command]
pub async fn tools_execute(
    tool_name: String,
    input: String,
    state: State<'_, ToolsState>,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    let start = Instant::now();
    let result = state
        .toolset
        .execute(&tool_name, &input)
        .await
        .map_err(|error| error.to_string());
    let duration_ms = start.elapsed().as_millis() as u64;

    let success = result.is_ok();
    let _ = app_handle.emit(
        "tool-executed",
        serde_json::json!({
            "tool": tool_name,
            "duration_ms": duration_ms,
            "success": success,
        }),
    );

    result
}
