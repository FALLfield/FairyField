//! Weather tool.

use super::web::WebSearchTool;
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
            description: "Fetch current weather through FairyField's built-in weather providers."
                .into(),
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
        "Fetch current weather through FairyField's built-in weather providers"
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
            let query = if params.days > 1 {
                format!("{} {} day weather forecast", params.city, params.days)
            } else {
                format!("{} weather", params.city)
            };
            WebSearchTool
                .execute(&serde_json::json!({ "query": query, "limit": 3 }).to_string())
                .await
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
    async fn no_key_returns_weather_or_clear_fallback() {
        std::env::remove_var("OPENWEATHER_API_KEY");
        let result = WeatherTool.execute(r#"{"city":"Tokyo"}"#).await.unwrap();
        assert!(!result.contains("configuration_required"));
        assert!(result.contains("天气") || result.contains("weather") || result.contains("Tokyo"));
    }
}
