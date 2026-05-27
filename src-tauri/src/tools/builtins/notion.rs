//! Notion integration tool (safe configuration-gated surface).

use super::simple::{
    action_schema, config_required, env_present, service_manifest, validate_action_input,
};
use crate::tools::executor::{Tool, ToolError};
use crate::tools::manifest::{OAuthConfig, ToolCategory, ToolManifest, ToolPermission};

const ACTIONS: &[&str] = &["search", "get_page", "list_databases", "create_page"];

pub struct NotionTool;

impl NotionTool {
    pub fn new() -> Self {
        Self
    }

    pub fn manifest() -> ToolManifest {
        service_manifest(
            "notion",
            "Notion",
            "Search and manage Notion pages and databases.",
            ToolCategory::Productivity,
            vec![ToolPermission::NetworkAccess, ToolPermission::UserData],
            ACTIONS,
            Some(OAuthConfig {
                provider: "notion".into(),
                auth_url: "https://api.notion.com/v1/oauth/authorize".into(),
                token_url: "https://api.notion.com/v1/oauth/token".into(),
                scopes: vec![
                    "read_content".into(),
                    "insert_content".into(),
                    "update_content".into(),
                ],
                pkce: false,
            }),
        )
    }
}

impl Tool for NotionTool {
    fn name(&self) -> &str {
        "notion"
    }
    fn description(&self) -> &str {
        "Search and manage Notion pages and databases"
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
            if !env_present("NOTION_API_KEY") {
                return config_required("notion", &action, "NOTION_API_KEY");
            }
            Ok(serde_json::json!({"status":"ready","service":"notion","action":action,"network_call":"skipped"}).to_string())
        })
    }
}

impl Default for NotionTool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_known_action() {
        assert!(NotionTool.validate_input(r#"{"action":"search"}"#));
    }
    #[test]
    fn rejects_unknown_action() {
        assert!(!NotionTool.validate_input(r#"{"action":"drop"}"#));
    }
    #[tokio::test]
    async fn returns_config_required_without_env() {
        std::env::remove_var("NOTION_API_KEY");
        let out = NotionTool.execute(r#"{"action":"search"}"#).await.unwrap();
        assert!(out.contains("configuration_required"));
    }
}
