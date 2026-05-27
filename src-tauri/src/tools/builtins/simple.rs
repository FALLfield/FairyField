//! Shared helpers for safe Phase 6 integration tools.

use crate::tools::executor::ToolError;
use crate::tools::manifest::{OAuthConfig, ToolCategory, ToolManifest, ToolPermission};
use serde::Serialize;

pub fn service_manifest(
    id: &str,
    name: &str,
    description: &str,
    category: ToolCategory,
    permissions: Vec<ToolPermission>,
    actions: &[&str],
    oauth: Option<OAuthConfig>,
) -> ToolManifest {
    ToolManifest {
        id: id.into(),
        name: name.into(),
        description: description.into(),
        long_description: Some(format!(
            "{name} integration surface. Returns configuration guidance until credentials are connected."
        )),
        category,
        parameters: action_schema(actions),
        permissions,
        rate_limit: Some(10),
        timeout_ms: 30000,
        enabled_by_default: false,
        oauth,
        version: Some("1.0.0".into()),
        author: Some("FairyField".into()),
    }
}

pub fn action_schema(actions: &[&str]) -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "action": {
                "type": "string",
                "enum": actions,
                "description": "Operation to validate and prepare"
            },
            "query": {
                "type": "string",
                "description": "Search query, prompt, URL, or text depending on the action"
            },
            "target": {
                "type": "string",
                "description": "Optional target resource such as channel, page, issue, calendar, or recipient"
            },
            "payload": {
                "type": "object",
                "description": "Optional structured payload for the operation"
            }
        },
        "required": ["action"]
    })
}

pub fn validate_action_input(input: &str, actions: &[&str]) -> bool {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(input) else {
        return false;
    };
    let Some(action) = value.get("action").and_then(|v| v.as_str()) else {
        return false;
    };
    actions.contains(&action)
}

#[derive(Debug, Serialize)]
pub struct ConfigRequiredResponse<'a> {
    pub status: &'static str,
    pub service: &'a str,
    pub action: &'a str,
    pub required_env: &'a str,
    pub message: String,
}

pub fn config_required(
    service: &str,
    action: &str,
    required_env: &str,
) -> Result<String, ToolError> {
    serde_json::to_string(&ConfigRequiredResponse {
        status: "configuration_required",
        service,
        action,
        required_env,
        message: format!("{service} is not connected. Set {required_env} or complete OAuth before using this tool."),
    })
    .map_err(|e| ToolError::ExecutionFailed(e.to_string()))
}

pub fn env_present(name: &str) -> bool {
    std::env::var(name)
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false)
}
