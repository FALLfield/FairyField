//! ToolManifest — declaration-based tool description
//!
//! Each tool has a manifest that describes its identity, parameters,
//! permissions, rate limits, and integration requirements (e.g. OAuth).
//! Manifests are used for LLM function-calling schema generation and
//! for the tool registry UI.

use serde::{Deserialize, Serialize};

/// Tool category classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ToolCategory {
    Builtin,
    FileSystem,
    Network,
    Browser,
    Cron,
    Voice,
    Memory,
    Productivity,
    Developer,
    Communication,
    Media,
    Agent,
    Community,
}

/// Permission required by a tool
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ToolPermission {
    ReadFiles,
    WriteFiles,
    ExecuteCommand,
    NetworkAccess,
    FileSystem,
    UserData,
    CodeExecution,
    BrowserControl,
    Notifications,
}

/// OAuth 2.0 configuration for third-party integrations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthConfig {
    pub provider: String,
    pub auth_url: String,
    pub token_url: String,
    pub scopes: Vec<String>,
    pub pkce: bool,
}

/// ToolManifest — declarative description of a tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolManifest {
    /// Unique identifier (e.g. "obsidian.search_notes")
    pub id: String,
    /// Display name (human-readable)
    pub name: String,
    /// Short description for LLM function calling (max ~200 chars)
    pub description: String,
    /// Detailed usage instructions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub long_description: Option<String>,
    /// Category
    pub category: ToolCategory,
    /// JSON Schema parameter definition for LLM tool_use
    pub parameters: serde_json::Value,
    /// Required permissions
    pub permissions: Vec<ToolPermission>,
    /// Rate limit (max calls per second)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_limit: Option<u32>,
    /// Timeout in milliseconds
    pub timeout_ms: u64,
    /// Whether enabled by default
    pub enabled_by_default: bool,
    /// OAuth config for third-party integrations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oauth: Option<OAuthConfig>,
    /// Version (community plugins)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Author (community plugins)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
}

impl Default for ToolManifest {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            description: String::new(),
            long_description: None,
            category: ToolCategory::Builtin,
            parameters: serde_json::json!({}),
            permissions: Vec::new(),
            rate_limit: None,
            timeout_ms: 30000,
            enabled_by_default: true,
            oauth: None,
            version: None,
            author: None,
        }
    }
}

impl ToolManifest {
    /// Create a new builtin tool manifest
    pub fn builtin(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        parameters: serde_json::Value,
        permissions: Vec<ToolPermission>,
        timeout_ms: u64,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            long_description: None,
            category: ToolCategory::Builtin,
            parameters,
            permissions,
            rate_limit: None,
            timeout_ms,
            enabled_by_default: true,
            oauth: None,
            version: None,
            author: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_serializes_correctly() {
        let manifest = ToolManifest::builtin(
            "web.search",
            "Web Search",
            "Search the web",
            serde_json::json!({"query": {"type": "string"}}),
            vec![ToolPermission::NetworkAccess],
            30000,
        );
        let json = serde_json::to_string(&manifest).unwrap();
        assert!(json.contains("web.search"));
        assert!(json.contains("Web Search"));
        let deserialized: ToolManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "web.search");
    }

    #[test]
    fn manifest_with_oauth_serializes() {
        let manifest = ToolManifest {
            id: "github.issues".into(),
            name: "GitHub Issues".into(),
            description: "Manage GitHub issues".into(),
            category: ToolCategory::Developer,
            parameters: serde_json::json!({}),
            permissions: vec![ToolPermission::NetworkAccess, ToolPermission::UserData],
            timeout_ms: 30000,
            enabled_by_default: false,
            oauth: Some(OAuthConfig {
                provider: "github".into(),
                auth_url: "https://github.com/login/oauth/authorize".into(),
                token_url: "https://github.com/login/oauth/access_token".into(),
                scopes: vec!["repo".into(), "read:user".into()],
                pkce: false,
            }),
            ..Default::default()
        };
        let json = serde_json::to_string(&manifest).unwrap();
        assert!(json.contains("github"));
    }

    #[test]
    fn default_manifest_has_sensible_values() {
        let manifest = ToolManifest::default();
        assert_eq!(manifest.id, "");
        assert_eq!(manifest.category, ToolCategory::Builtin);
        assert_eq!(manifest.timeout_ms, 30000);
        assert!(manifest.enabled_by_default);
    }

    #[test]
    fn deserialize_minimal_manifest() {
        let json = r#"{
            "id": "test.tool",
            "name": "Test Tool",
            "description": "A test",
            "category": "builtin",
            "parameters": {},
            "permissions": [],
            "timeout_ms": 10000,
            "enabled_by_default": true
        }"#;
        let manifest: ToolManifest = serde_json::from_str(json).unwrap();
        assert_eq!(manifest.id, "test.tool");
        assert!(manifest.oauth.is_none());
        assert!(manifest.version.is_none());
    }
}
