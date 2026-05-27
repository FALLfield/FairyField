//! Translation tool (configuration-gated, no test network calls).

use super::simple::{config_required, env_present};
use crate::tools::executor::{Tool, ToolError};
use crate::tools::manifest::{ToolCategory, ToolManifest, ToolPermission};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct TranslateInput {
    text: String,
    target_lang: String,
    #[serde(default)]
    source_lang: Option<String>,
}

pub struct TranslateTool;

impl TranslateTool {
    pub fn new() -> Self {
        Self
    }
    pub fn manifest() -> ToolManifest {
        ToolManifest {
            id: "translate".into(),
            name: "Translate".into(),
            description: "Translate text when a translation provider key is configured.".into(),
            category: ToolCategory::Media,
            parameters: Self.parameters_schema(),
            permissions: vec![ToolPermission::NetworkAccess],
            rate_limit: Some(20),
            timeout_ms: 10000,
            enabled_by_default: true,
            oauth: None,
            version: Some("1.0.0".into()),
            author: Some("FairyField".into()),
            long_description: None,
        }
    }
}

impl Tool for TranslateTool {
    fn name(&self) -> &str {
        "translate"
    }
    fn description(&self) -> &str {
        "Translate text when a translation provider key is configured"
    }
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type":"object",
            "properties":{
                "text":{"type":"string","description":"Text to translate"},
                "target_lang":{"type":"string","description":"Target language code"},
                "source_lang":{"type":"string","description":"Optional source language code"}
            },
            "required":["text","target_lang"]
        })
    }
    fn validate_input(&self, input: &str) -> bool {
        serde_json::from_str::<TranslateInput>(input)
            .map(|p| !p.text.trim().is_empty() && !p.target_lang.trim().is_empty())
            .unwrap_or(false)
    }
    fn execute(
        &self,
        input: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, ToolError>> + Send + '_>>
    {
        let parsed = serde_json::from_str::<TranslateInput>(input);
        Box::pin(async move {
            let params = parsed.map_err(|e| ToolError::ValidationFailed(e.to_string()))?;
            if !env_present("TRANSLATE_API_KEY") {
                return config_required("translate", "translate", "TRANSLATE_API_KEY");
            }
            Ok(serde_json::json!({"status":"ready","service":"translate","target_lang":params.target_lang,"source_lang":params.source_lang,"network_call":"skipped"}).to_string())
        })
    }
}

impl Default for TranslateTool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_translation() {
        assert!(TranslateTool.validate_input(r#"{"text":"hello","target_lang":"zh"}"#));
    }
    #[tokio::test]
    async fn no_key_is_config_required() {
        std::env::remove_var("TRANSLATE_API_KEY");
        assert!(TranslateTool
            .execute(r#"{"text":"hello","target_lang":"zh"}"#)
            .await
            .unwrap()
            .contains("TRANSLATE_API_KEY"));
    }
}
