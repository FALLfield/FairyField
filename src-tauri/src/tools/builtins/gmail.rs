//! Gmail integration tool (safe configuration-gated surface).

use super::simple::{
    action_schema, config_required, env_present, service_manifest, validate_action_input,
};
use crate::tools::executor::{Tool, ToolError};
use crate::tools::manifest::{OAuthConfig, ToolCategory, ToolManifest, ToolPermission};

const ACTIONS: &[&str] = &[
    "search_messages",
    "get_message",
    "send_draft",
    "list_labels",
];

pub struct GmailTool;

impl GmailTool {
    pub fn new() -> Self {
        Self
    }
    pub fn manifest() -> ToolManifest {
        service_manifest(
            "gmail",
            "Gmail",
            "Search, read, and prepare Gmail messages.",
            ToolCategory::Productivity,
            vec![ToolPermission::NetworkAccess, ToolPermission::UserData],
            ACTIONS,
            Some(OAuthConfig {
                provider: "google".into(),
                auth_url: "https://accounts.google.com/o/oauth2/v2/auth".into(),
                token_url: "https://oauth2.googleapis.com/token".into(),
                scopes: vec![
                    "https://www.googleapis.com/auth/gmail.readonly".into(),
                    "https://www.googleapis.com/auth/gmail.compose".into(),
                ],
                pkce: true,
            }),
        )
    }
}

impl Tool for GmailTool {
    fn name(&self) -> &str {
        "gmail"
    }
    fn description(&self) -> &str {
        "Search, read, and prepare Gmail messages"
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
            if !env_present("GOOGLE_ACCESS_TOKEN") {
                return config_required("gmail", &action, "GOOGLE_ACCESS_TOKEN");
            }
            Ok(serde_json::json!({"status":"ready","service":"gmail","action":action,"network_call":"skipped"}).to_string())
        })
    }
}

impl Default for GmailTool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_message_search() {
        assert!(GmailTool.validate_input(r#"{"action":"search_messages"}"#));
    }
    #[tokio::test]
    async fn no_token_is_config_required() {
        std::env::remove_var("GOOGLE_ACCESS_TOKEN");
        assert!(GmailTool
            .execute(r#"{"action":"list_labels"}"#)
            .await
            .unwrap()
            .contains("configuration_required"));
    }
}
