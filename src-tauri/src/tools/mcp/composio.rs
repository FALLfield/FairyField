//! Lightweight Composio catalog discovery.
//!
//! This module maps JSON-declared Composio tool manifests into MCP tool
//! definitions. It intentionally does not execute Composio calls yet.

use crate::tools::manifest::ToolManifest;
use crate::tools::mcp::transport::McpTool;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComposioCatalogFile {
    pub tools: Vec<ComposioCatalogEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComposioCatalogEntry {
    pub manifest: ToolManifest,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app: Option<String>,
}

fn default_enabled() -> bool {
    true
}

#[derive(Debug, Clone, Default)]
pub struct ComposioToolCatalog {
    tools: Vec<ComposioCatalogEntry>,
}

impl ComposioToolCatalog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_json_str(input: &str) -> Result<Self, serde_json::Error> {
        let file: ComposioCatalogFile = serde_json::from_str(input)?;
        Ok(Self { tools: file.tools })
    }

    pub fn load_file(path: impl AsRef<Path>) -> Result<Self, ComposioCatalogError> {
        let text = std::fs::read_to_string(path)?;
        Ok(Self::from_json_str(&text)?)
    }

    pub fn discover_dir(path: impl AsRef<Path>) -> Result<Self, ComposioCatalogError> {
        let mut catalog = Self::new();
        if !path.as_ref().exists() {
            return Ok(catalog);
        }
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            if entry.path().extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let mut file_catalog = Self::load_file(entry.path())?;
            catalog.tools.append(&mut file_catalog.tools);
        }
        Ok(catalog)
    }

    pub fn enabled_manifests(&self) -> Vec<&ToolManifest> {
        self.tools
            .iter()
            .filter(|entry| entry.enabled && entry.manifest.enabled_by_default)
            .map(|entry| &entry.manifest)
            .collect()
    }

    pub fn all_manifests(&self) -> Vec<&ToolManifest> {
        self.tools.iter().map(|entry| &entry.manifest).collect()
    }

    pub fn to_mcp_tools(&self) -> Vec<McpTool> {
        self.enabled_manifests()
            .into_iter()
            .map(|manifest| McpTool {
                name: manifest.id.clone(),
                description: manifest.description.clone(),
                input_schema: manifest.parameters.clone(),
            })
            .collect()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ComposioCatalogError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_catalog() -> String {
        r#"{
            "tools": [
                {
                    "enabled": true,
                    "app": "notion",
                    "manifest": {
                        "id": "composio.notion.search",
                        "name": "Notion Search",
                        "description": "Search Notion via Composio",
                        "category": "productivity",
                        "parameters": {"type":"object","properties":{"query":{"type":"string"}},"required":["query"]},
                        "permissions": ["network_access", "user_data"],
                        "timeout_ms": 30000,
                        "enabled_by_default": true
                    }
                },
                {
                    "enabled": false,
                    "manifest": {
                        "id": "composio.gmail.send",
                        "name": "Gmail Send",
                        "description": "Send Gmail via Composio",
                        "category": "productivity",
                        "parameters": {"type":"object"},
                        "permissions": ["network_access", "user_data"],
                        "timeout_ms": 30000,
                        "enabled_by_default": true
                    }
                }
            ]
        }"#.into()
    }

    #[test]
    fn loads_and_filters_enabled_tools() {
        let catalog = ComposioToolCatalog::from_json_str(&sample_catalog()).unwrap();
        assert_eq!(catalog.all_manifests().len(), 2);
        let enabled = catalog.enabled_manifests();
        assert_eq!(enabled.len(), 1);
        assert_eq!(enabled[0].id, "composio.notion.search");
    }

    #[test]
    fn maps_to_mcp_tools() {
        let catalog = ComposioToolCatalog::from_json_str(&sample_catalog()).unwrap();
        let tools = catalog.to_mcp_tools();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "composio.notion.search");
        assert_eq!(tools[0].input_schema["required"][0], "query");
    }
}
