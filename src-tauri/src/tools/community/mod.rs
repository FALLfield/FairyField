//! Community JSON plugin loader.
//!
//! Phase 6 only loads, validates, and lists plugin manifests. Provider
//! execution is intentionally out of scope for this step.

use crate::tools::manifest::ToolManifest;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityPlugin {
    pub manifest: ToolManifest,
    pub provider: PluginProvider,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PluginProvider {
    Http {
        endpoint: String,
        #[serde(default = "default_http_method")]
        method: String,
        #[serde(default)]
        headers: serde_json::Value,
        #[serde(default)]
        query_params: serde_json::Value,
        #[serde(default)]
        response_mapping: serde_json::Value,
    },
    LocalCommand {
        command: String,
        #[serde(default)]
        args: Vec<String>,
    },
    StaticResponse {
        response: serde_json::Value,
    },
}

fn default_http_method() -> String {
    "GET".into()
}

#[derive(Debug, Default)]
pub struct CommunityPluginLoader {
    plugins: Vec<CommunityPlugin>,
}

impl CommunityPluginLoader {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_json_str(input: &str) -> Result<CommunityPlugin, PluginLoadError> {
        let plugin: CommunityPlugin = serde_json::from_str(input)?;
        validate_plugin(&plugin)?;
        Ok(plugin)
    }

    pub fn load_file(path: impl AsRef<Path>) -> Result<CommunityPlugin, PluginLoadError> {
        let text = std::fs::read_to_string(path)?;
        Self::from_json_str(&text)
    }

    pub fn load_dir(path: impl AsRef<Path>) -> Result<Self, PluginLoadError> {
        let mut loader = Self::new();
        if !path.as_ref().exists() {
            return Ok(loader);
        }
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            if entry.path().extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            loader.plugins.push(Self::load_file(entry.path())?);
        }
        loader.ensure_unique_ids()?;
        loader
            .plugins
            .sort_by(|a, b| a.manifest.id.cmp(&b.manifest.id));
        Ok(loader)
    }

    pub fn plugins(&self) -> &[CommunityPlugin] {
        &self.plugins
    }

    pub fn manifests(&self) -> Vec<&ToolManifest> {
        self.plugins.iter().map(|p| &p.manifest).collect()
    }

    pub fn enabled_manifests(&self) -> Vec<&ToolManifest> {
        self.plugins
            .iter()
            .filter(|p| p.manifest.enabled_by_default)
            .map(|p| &p.manifest)
            .collect()
    }

    fn ensure_unique_ids(&self) -> Result<(), PluginLoadError> {
        let mut seen = HashSet::new();
        for plugin in &self.plugins {
            if !seen.insert(plugin.manifest.id.clone()) {
                return Err(PluginLoadError::DuplicateId(plugin.manifest.id.clone()));
            }
        }
        Ok(())
    }
}

fn validate_plugin(plugin: &CommunityPlugin) -> Result<(), PluginLoadError> {
    if plugin.manifest.id.trim().is_empty() {
        return Err(PluginLoadError::Invalid("manifest.id is required".into()));
    }
    if plugin.manifest.name.trim().is_empty() {
        return Err(PluginLoadError::Invalid("manifest.name is required".into()));
    }
    if plugin.manifest.description.trim().is_empty() {
        return Err(PluginLoadError::Invalid(
            "manifest.description is required".into(),
        ));
    }
    if !plugin.manifest.parameters.is_object() {
        return Err(PluginLoadError::Invalid(
            "manifest.parameters must be a JSON object".into(),
        ));
    }
    match &plugin.provider {
        PluginProvider::Http {
            endpoint, method, ..
        } => {
            if !endpoint.starts_with("https://") && !endpoint.starts_with("http://") {
                return Err(PluginLoadError::Invalid(
                    "http provider endpoint must be http/https".into(),
                ));
            }
            if !matches!(method.as_str(), "GET" | "POST" | "PUT" | "PATCH" | "DELETE") {
                return Err(PluginLoadError::Invalid("unsupported http method".into()));
            }
        }
        PluginProvider::LocalCommand { command, .. } => {
            if command.trim().is_empty() || command.contains('/') || command.contains('\\') {
                return Err(PluginLoadError::Invalid(
                    "local_command provider must name a command, not a path".into(),
                ));
            }
        }
        PluginProvider::StaticResponse { .. } => {}
    }
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum PluginLoadError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("invalid plugin: {0}")]
    Invalid(String),
    #[error("duplicate plugin id: {0}")]
    DuplicateId(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_static_response_plugin() {
        let plugin =
            CommunityPluginLoader::from_json_str(include_str!("examples/calculator.json")).unwrap();
        assert_eq!(plugin.manifest.id, "community.calculator");
        assert!(matches!(
            plugin.provider,
            PluginProvider::StaticResponse { .. }
        ));
    }

    #[test]
    fn loads_all_examples() {
        let loader = CommunityPluginLoader::load_dir("src/tools/community/examples")
            .or_else(|_| CommunityPluginLoader::load_dir("src-tauri/src/tools/community/examples"))
            .unwrap();
        assert_eq!(loader.plugins().len(), 3);
        assert_eq!(loader.enabled_manifests().len(), 1);
    }

    #[test]
    fn rejects_path_command_provider() {
        let bad = r#"{
            "manifest": {
                "id": "community.bad",
                "name": "Bad",
                "description": "Bad command",
                "category": "community",
                "parameters": {},
                "permissions": ["execute_command"],
                "timeout_ms": 1000,
                "enabled_by_default": false
            },
            "provider": {"type":"local_command","command":"./script.sh"}
        }"#;
        assert!(CommunityPluginLoader::from_json_str(bad).is_err());
    }
}
