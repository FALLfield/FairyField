//! Obsidian vault integration tool
//!
//! Provides read/write access to an Obsidian vault for note-taking
//! and knowledge management.

use crate::tools::executor::{Tool, ToolError};
use crate::tools::manifest::{ToolCategory, ToolManifest, ToolPermission};
use serde::Deserialize;
use std::path::PathBuf;

pub struct ObsidianTool {
    vault_path: Option<PathBuf>,
}

impl ObsidianTool {
    pub fn new() -> Self {
        Self { vault_path: None }
    }

    pub fn with_vault(vault_path: PathBuf) -> Self {
        Self {
            vault_path: Some(vault_path),
        }
    }

    /// Generate the ToolManifest for this tool
    pub fn manifest() -> ToolManifest {
        ToolManifest {
            id: "obsidian.search_notes".into(),
            name: "Obsidian Search".into(),
            description: "Search notes in the Obsidian vault by keyword. Returns matching note titles and previews.".into(),
            long_description: Some("Searches all Markdown files in the configured Obsidian vault directory. Returns file paths, titles, and content previews (first 200 chars).".into()),
            category: ToolCategory::Productivity,
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search keyword or phrase"
                    },
                    "max_results": {
                        "type": "integer",
                        "description": "Maximum number of results (default: 10)",
                        "default": 10
                    }
                },
                "required": ["query"]
            }),
            permissions: vec![ToolPermission::ReadFiles],
            rate_limit: Some(20),
            timeout_ms: 15000,
            enabled_by_default: false, // requires vault path config
            oauth: None,
            version: Some("1.0.0".into()),
            author: Some("FairyField".into()),
        }
    }

    /// Read a specific note from the vault
    pub fn read_note(&self, title: &str) -> Result<String, std::io::Error> {
        let vault_path = self.vault_path.as_ref().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Obsidian vault path not configured",
            )
        })?;
        let file_path = vault_path.join(format!("{}.md", title));
        std::fs::read_to_string(&file_path)
    }

    /// Write or update a note in the vault
    pub fn write_note(&self, title: &str, content: &str) -> Result<(), std::io::Error> {
        let vault_path = self.vault_path.as_ref().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Obsidian vault path not configured",
            )
        })?;
        // Sanitize filename
        let safe_title = title.replace(['/', '\\', ':'], "_");
        let file_path = vault_path.join(format!("{}.md", safe_title));
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&file_path, content)
    }

    /// Search notes containing the query string
    pub fn search_notes(
        &self,
        query: &str,
        max_results: usize,
    ) -> Result<Vec<SearchResult>, std::io::Error> {
        let vault_path = self.vault_path.as_ref().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Obsidian vault path not configured",
            )
        })?;

        let mut results = Vec::new();
        let query_lower = query.to_lowercase();

        // Walk the vault directory for .md files
        if vault_path.exists() {
            for entry in walkdir::WalkDir::new(vault_path)
                .max_depth(5)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if !entry.file_type().is_file() {
                    continue;
                }
                if entry.path().extension().is_none_or(|ext| ext != "md") {
                    continue;
                }

                let content = match std::fs::read_to_string(entry.path()) {
                    Ok(c) => c,
                    Err(_) => continue,
                };

                if content.to_lowercase().contains(&query_lower) {
                    let relative_path = entry
                        .path()
                        .strip_prefix(vault_path)
                        .unwrap_or(entry.path());
                    let title = relative_path
                        .with_extension("")
                        .to_string_lossy()
                        .to_string();
                    let preview = content.chars().take(200).collect::<String>();

                    results.push(SearchResult {
                        title,
                        path: relative_path.to_string_lossy().to_string(),
                        preview,
                    });

                    if results.len() >= max_results {
                        break;
                    }
                }
            }
        }

        Ok(results)
    }
}

#[derive(Debug, serde::Serialize)]
pub struct SearchResult {
    pub title: String,
    pub path: String,
    pub preview: String,
}

impl Default for ObsidianTool {
    fn default() -> Self {
        Self::new()
    }
}

impl Tool for ObsidianTool {
    fn name(&self) -> &str {
        "obsidian_search"
    }

    fn description(&self) -> &str {
        "Search notes in the Obsidian vault by keyword"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "query": {"type": "string", "description": "Search keyword"},
                "max_results": {"type": "integer", "description": "Max results", "default": 10}
            },
            "required": ["query"]
        })
    }

    fn validate_input(&self, input: &str) -> bool {
        serde_json::from_str::<ObsidianInput>(input).is_ok()
    }

    fn execute(
        &self,
        input: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, ToolError>> + Send + '_>>
    {
        let params = match serde_json::from_str::<ObsidianInput>(input) {
            Ok(p) => p,
            Err(e) => {
                return Box::pin(async move { Err(ToolError::ValidationFailed(e.to_string())) })
            }
        };

        let vault_path = match &self.vault_path {
            Some(p) => p.clone(),
            None => {
                return Box::pin(async move {
                    Err(ToolError::ExecutionFailed(
                        "Obsidian vault path not configured".to_string(),
                    ))
                })
            }
        };

        Box::pin(async move {
            let tool = ObsidianTool::with_vault(vault_path);
            let results = tool
                .search_notes(&params.query, params.max_results)
                .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

            Ok(serde_json::to_string(&results).unwrap_or_else(|_| "[]".into()))
        })
    }
}

#[derive(Deserialize)]
struct ObsidianInput {
    query: String,
    #[serde(default = "default_max_results")]
    max_results: usize,
}

fn default_max_results() -> usize {
    10
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_test_vault() -> (TempDir, ObsidianTool) {
        let dir = TempDir::new().unwrap();
        let vault_path = dir.path().to_path_buf();
        std::fs::write(
            vault_path.join("test note.md"),
            "# Test Note\n\nThis is a test note about Rust programming.",
        )
        .unwrap();
        std::fs::create_dir_all(vault_path.join("journal")).unwrap();
        std::fs::write(
            vault_path.join("journal/daily.md"),
            "# Daily Journal\n\nToday I learned about async Rust.",
        )
        .unwrap();
        (dir, ObsidianTool::with_vault(vault_path))
    }

    #[test]
    fn searches_notes_by_keyword() {
        let (_dir, tool) = setup_test_vault();
        let results = tool.search_notes("Rust", 10).unwrap();
        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.title.contains("test note")));
        assert!(results.iter().any(|r| r.title.contains("journal/daily")));
    }

    #[test]
    fn search_no_match_returns_empty() {
        let (_dir, tool) = setup_test_vault();
        let results = tool.search_notes("nonexistent_keyword_xyz", 10).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn search_respects_max_results() {
        let (_dir, tool) = setup_test_vault();
        let results = tool.search_notes("Rust", 1).unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn read_note_returns_content() {
        let (_dir, tool) = setup_test_vault();
        let content = tool.read_note("test note").unwrap();
        assert!(content.contains("Rust programming"));
    }

    #[test]
    fn write_and_read_note() {
        let dir = TempDir::new().unwrap();
        let tool = ObsidianTool::with_vault(dir.path().to_path_buf());
        tool.write_note("new idea", "# New Idea\n\nThis is a new note.")
            .unwrap();
        let content = tool.read_note("new idea").unwrap();
        assert!(content.contains("New Idea"));
    }

    #[test]
    fn manifest_has_correct_fields() {
        let manifest = ObsidianTool::manifest();
        assert_eq!(manifest.id, "obsidian.search_notes");
        assert_eq!(manifest.category, ToolCategory::Productivity);
        assert!(!manifest.enabled_by_default);
    }

    #[tokio::test]
    async fn tool_trait_execute_searches() {
        let (_dir, tool) = setup_test_vault();
        let result = tool
            .execute(r#"{"query": "async", "max_results": 5}"#)
            .await
            .unwrap();
        assert!(result.contains("journal/daily"));
    }

    #[tokio::test]
    async fn tool_trait_execute_no_vault_errors() {
        let tool = ObsidianTool::new();
        let result = tool
            .execute(r#"{"query": "anything", "max_results": 5}"#)
            .await;
        assert!(result.is_err());
    }
}
