//! Slack integration tool (safe configuration-gated surface).

use super::simple::{
    action_schema, config_required, env_present, service_manifest, validate_action_input,
};
use crate::tools::executor::{Tool, ToolError};
use crate::tools::manifest::{OAuthConfig, ToolCategory, ToolManifest, ToolPermission};

const ACTIONS: &[&str] = &[
    "list_channels",
    "search_messages",
    "send_message",
    "add_reaction",
];

pub struct SlackTool;

impl SlackTool {
    pub fn new() -> Self {
        Self
    }
    pub fn manifest() -> ToolManifest {
        service_manifest(
            "slack",
            "Slack",
            "Search channels and prepare Slack messages.",
            ToolCategory::Communication,
            vec![ToolPermission::NetworkAccess, ToolPermission::UserData],
            ACTIONS,
            Some(OAuthConfig {
                provider: "slack".into(),
                auth_url: "https://slack.com/oauth/v2/authorize".into(),
                token_url: "https://slack.com/api/oauth.v2.access".into(),
                scopes: vec![
                    "channels:read".into(),
                    "chat:write".into(),
                    "search:read".into(),
                ],
                pkce: false,
            }),
        )
    }
}

impl Tool for SlackTool {
    fn name(&self) -> &str {
        "slack"
    }
    fn description(&self) -> &str {
        "Search channels and prepare Slack messages"
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
            if !env_present("SLACK_BOT_TOKEN") {
                return config_required("slack", &action, "SLACK_BOT_TOKEN");
            }
            Ok(serde_json::json!({"status":"ready","service":"slack","action":action,"network_call":"skipped"}).to_string())
        })
    }
}

impl Default for SlackTool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_channels() {
        assert!(SlackTool.validate_input(r#"{"action":"list_channels"}"#));
    }
    #[tokio::test]
    async fn no_token_is_config_required() {
        std::env::remove_var("SLACK_BOT_TOKEN");
        assert!(SlackTool
            .execute(r#"{"action":"send_message"}"#)
            .await
            .unwrap()
            .contains("SLACK_BOT_TOKEN"));
    }
}
