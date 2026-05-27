//! Discord integration tool (safe configuration-gated surface).

use super::simple::{
    action_schema, config_required, env_present, service_manifest, validate_action_input,
};
use crate::tools::executor::{Tool, ToolError};
use crate::tools::manifest::{ToolCategory, ToolManifest, ToolPermission};

const ACTIONS: &[&str] = &[
    "list_guilds",
    "list_channels",
    "send_message",
    "get_messages",
];

pub struct DiscordTool;

impl DiscordTool {
    pub fn new() -> Self {
        Self
    }
    pub fn manifest() -> ToolManifest {
        service_manifest(
            "discord",
            "Discord",
            "Search Discord channels and prepare messages.",
            ToolCategory::Communication,
            vec![ToolPermission::NetworkAccess, ToolPermission::UserData],
            ACTIONS,
            None,
        )
    }
}

impl Tool for DiscordTool {
    fn name(&self) -> &str {
        "discord"
    }
    fn description(&self) -> &str {
        "Search Discord channels and prepare messages"
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
            if !env_present("DISCORD_BOT_TOKEN") {
                return config_required("discord", &action, "DISCORD_BOT_TOKEN");
            }
            Ok(serde_json::json!({"status":"ready","service":"discord","action":action,"network_call":"skipped"}).to_string())
        })
    }
}

impl Default for DiscordTool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_messages() {
        assert!(DiscordTool.validate_input(r#"{"action":"get_messages"}"#));
    }
    #[tokio::test]
    async fn no_token_is_config_required() {
        std::env::remove_var("DISCORD_BOT_TOKEN");
        assert!(DiscordTool
            .execute(r#"{"action":"list_guilds"}"#)
            .await
            .unwrap()
            .contains("DISCORD_BOT_TOKEN"));
    }
}
