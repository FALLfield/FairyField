//! Tauri IPC commands for tools subsystem

use super::executor::Tool;
use super::registry::ToolRegistry;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::Emitter;
use tauri::State;

const TOOL_IPC_TIMEOUT_SECS: u64 = 30;

/// Tools subsystem managed state
///
/// Uses std::sync::Mutex because ToolRegistry internally uses std::sync::Mutex
/// for its tool map. Using tokio::sync::Mutex here would cause Send issues
/// when holding the guard across async tool execution.
pub struct ToolsState {
    pub registry: std::sync::Mutex<ToolRegistry>,
}

#[tauri::command]
pub fn tools_list(state: State<'_, ToolsState>) -> Result<Vec<serde_json::Value>, String> {
    let registry = state.registry.lock().map_err(|e| e.to_string())?;
    Ok(registry
        .tool_descriptions()
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
    // Access the ToolExecutor's internal tool map to clone the Arc<dyn Tool>
    // before releasing the lock, so the future is Send-safe.
    let tool: Arc<dyn Tool> = {
        let registry = state.registry.lock().map_err(|e| e.to_string())?;
        // Access the internal executor through registry to find the tool
        registry
            .find_tool(&tool_name)
            .ok_or_else(|| format!("工具未找到: {}", tool_name))?
    };

    if !tool.validate_input(&input) {
        return Err(format!("参数验证失败: {}", input));
    }

    let start = Instant::now();
    let result = tokio::time::timeout(
        Duration::from_secs(TOOL_IPC_TIMEOUT_SECS),
        tool.execute(&input),
    )
    .await
    .map_err(|_| format!("工具执行超时: {}", tool_name))
    .and_then(|result| result.map_err(|e| e.to_string()));
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
