//! Image generation/analysis tool (configuration-gated).

use super::simple::{config_required, env_present};
use crate::tools::executor::{Tool, ToolError};
use crate::tools::manifest::{ToolCategory, ToolManifest, ToolPermission};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ImageInput {
    action: String,
    prompt: String,
}

pub struct ImageTool;

impl ImageTool {
    pub fn new() -> Self {
        Self
    }
    pub fn manifest() -> ToolManifest {
        ToolManifest {
            id: "image".into(),
            name: "Image".into(),
            description:
                "Prepare image generation or analysis requests when a provider is configured."
                    .into(),
            category: ToolCategory::Media,
            parameters: Self.parameters_schema(),
            permissions: vec![ToolPermission::NetworkAccess],
            rate_limit: Some(5),
            timeout_ms: 30000,
            enabled_by_default: false,
            oauth: None,
            version: Some("1.0.0".into()),
            author: Some("FairyField".into()),
            long_description: None,
        }
    }
}

impl Tool for ImageTool {
    fn name(&self) -> &str {
        "image"
    }
    fn description(&self) -> &str {
        "Prepare image generation or analysis requests"
    }
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type":"object",
            "properties":{
                "action":{"type":"string","enum":["generate","analyze"],"description":"Image operation"},
                "prompt":{"type":"string","description":"Generation prompt or analysis instruction"}
            },
            "required":["action","prompt"]
        })
    }
    fn validate_input(&self, input: &str) -> bool {
        serde_json::from_str::<ImageInput>(input)
            .map(|p| {
                matches!(p.action.as_str(), "generate" | "analyze") && !p.prompt.trim().is_empty()
            })
            .unwrap_or(false)
    }
    fn execute(
        &self,
        input: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, ToolError>> + Send + '_>>
    {
        let parsed = serde_json::from_str::<ImageInput>(input);
        Box::pin(async move {
            let params = parsed.map_err(|e| ToolError::ValidationFailed(e.to_string()))?;
            if !env_present("IMAGE_API_KEY") && !env_present("OPENAI_API_KEY") {
                return config_required("image", &params.action, "IMAGE_API_KEY or OPENAI_API_KEY");
            }
            Ok(serde_json::json!({"status":"ready","service":"image","action":params.action,"network_call":"skipped"}).to_string())
        })
    }
}

impl Default for ImageTool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_generate() {
        assert!(ImageTool.validate_input(r#"{"action":"generate","prompt":"a tiny desk fairy"}"#));
    }
    #[test]
    fn rejects_empty_prompt() {
        assert!(!ImageTool.validate_input(r#"{"action":"generate","prompt":""}"#));
    }
}
