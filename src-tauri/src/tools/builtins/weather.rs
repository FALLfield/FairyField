//! Weather tool (configuration-gated, no test network calls).

use super::simple::{config_required, env_present};
use crate::tools::executor::{Tool, ToolError};
use crate::tools::manifest::{ToolCategory, ToolManifest, ToolPermission};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct WeatherInput {
    city: String,
    #[serde(default = "default_days")]
    days: u8,
}

fn default_days() -> u8 {
    1
}

pub struct WeatherTool;

impl WeatherTool {
    pub fn new() -> Self {
        Self
    }
    pub fn manifest() -> ToolManifest {
        ToolManifest {
            id: "weather".into(),
            name: "Weather".into(),
            description: "Fetch weather forecasts when a weather API key is configured.".into(),
            category: ToolCategory::Network,
            parameters: Self.parameters_schema(),
            permissions: vec![ToolPermission::NetworkAccess],
            rate_limit: Some(10),
            timeout_ms: 10000,
            enabled_by_default: true,
            oauth: None,
            version: Some("1.0.0".into()),
            author: Some("FairyField".into()),
            long_description: None,
        }
    }
}

impl Tool for WeatherTool {
    fn name(&self) -> &str {
        "weather"
    }
    fn description(&self) -> &str {
        "Fetch weather forecasts when a weather API key is configured"
    }
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type":"object",
            "properties":{
                "city":{"type":"string","description":"City name"},
                "days":{"type":"integer","description":"Forecast days, 1-7","minimum":1,"maximum":7}
            },
            "required":["city"]
        })
    }
    fn validate_input(&self, input: &str) -> bool {
        serde_json::from_str::<WeatherInput>(input)
            .map(|p| !p.city.trim().is_empty() && (1..=7).contains(&p.days))
            .unwrap_or(false)
    }
    fn execute(
        &self,
        input: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, ToolError>> + Send + '_>>
    {
        let parsed = serde_json::from_str::<WeatherInput>(input);
        Box::pin(async move {
            let params = parsed.map_err(|e| ToolError::ValidationFailed(e.to_string()))?;
            if !env_present("OPENWEATHER_API_KEY") {
                return config_required("weather", "forecast", "OPENWEATHER_API_KEY");
            }
            Ok(serde_json::json!({"status":"ready","service":"weather","city":params.city,"days":params.days,"network_call":"skipped"}).to_string())
        })
    }
}

impl Default for WeatherTool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_city() {
        assert!(WeatherTool.validate_input(r#"{"city":"Tokyo","days":3}"#));
    }
    #[test]
    fn rejects_too_many_days() {
        assert!(!WeatherTool.validate_input(r#"{"city":"Tokyo","days":8}"#));
    }
    #[tokio::test]
    async fn no_key_is_config_required() {
        std::env::remove_var("OPENWEATHER_API_KEY");
        assert!(WeatherTool
            .execute(r#"{"city":"Tokyo"}"#)
            .await
            .unwrap()
            .contains("configuration_required"));
    }
}
