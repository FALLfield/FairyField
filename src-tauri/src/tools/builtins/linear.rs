//! Linear integration tool (safe configuration-gated surface).

use super::simple::{
    action_schema, config_required, env_present, service_manifest, validate_action_input,
};
use crate::tools::executor::{Tool, ToolError};
use crate::tools::manifest::{ToolCategory, ToolManifest, ToolPermission};

const ACTIONS: &[&str] = &[
    "search_issues",
    "get_issue",
    "list_projects",
    "create_issue",
];

pub struct LinearTool;

impl LinearTool {
    pub fn new() -> Self {
        Self
    }
    pub fn manifest() -> ToolManifest {
        service_manifest(
            "linear",
            "Linear",
            "Search and manage Linear issues and projects.",
            ToolCategory::Developer,
            vec![ToolPermission::NetworkAccess, ToolPermission::UserData],
            ACTIONS,
            None,
        )
    }
}

impl Tool for LinearTool {
    fn name(&self) -> &str {
        "linear"
    }
    fn description(&self) -> &str {
        "Search and manage Linear issues and projects"
    }
    fn parameters_schema(&self) -> serde_json::Value {
        action_schema(ACTIONS)
    }
    fn validate_input(&self, input: &str) -> bool {
        validate_action_input(input, ACTIONS)
    }
    fn execute(
        &self,
        input: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, ToolError>> + Send + '_>>
    {
        let action = serde_json::from_str::<serde_json::Value>(input)
            .ok()
            .and_then(|v| v.get("action").and_then(|a| a.as_str()).map(str::to_string))
            .unwrap_or_default();
        Box::pin(async move {
            if !env_present("LINEAR_API_KEY") {
                return config_required("linear", &action, "LINEAR_API_KEY");
            }
            Ok(serde_json::json!({"status":"ready","service":"linear","action":action,"network_call":"skipped"}).to_string())
        })
    }
}

impl Default for LinearTool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_issue_search() {
        assert!(LinearTool.validate_input(r#"{"action":"search_issues"}"#));
    }
    #[tokio::test]
    async fn no_env_is_clear() {
        std::env::remove_var("LINEAR_API_KEY");
        assert!(LinearTool
            .execute(r#"{"action":"get_issue"}"#)
            .await
            .unwrap()
            .contains("LINEAR_API_KEY"));
    }
}
