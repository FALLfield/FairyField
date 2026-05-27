//! Shared AgentMemory backend
//!
//! Inspired by MemPalace's MCP-facing memory tools: one small facade for
//! wake-up, search, verbatim writes, duplicate checks, and per-agent diaries.

use super::layers::{MemoryLayers, WakeUpContext};
use super::store::Drawer;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

/// Duplicate-check match returned by [`AgentMemoryBackend::duplicate_check`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateMatch {
    pub id: i64,
    pub wing: String,
    pub room: String,
    pub hall: String,
    pub similarity: f32,
    pub content: String,
}

/// Conservative duplicate-check result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateCheck {
    pub is_duplicate: bool,
    pub matches: Vec<DuplicateMatch>,
}

/// Agent diary entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDiaryEntry {
    pub agent: String,
    pub content: String,
    pub wing: String,
    pub room: String,
    pub hall: String,
    pub created_at: i64,
}

/// Shared backend exposed to coding agents and MCP adapters.
///
/// Content is stored verbatim. This layer does not summarize or rewrite user
/// text before writing it into the memory store.
#[derive(Clone)]
pub struct AgentMemoryBackend {
    layers: Arc<Mutex<MemoryLayers>>,
}

impl AgentMemoryBackend {
    pub fn new(layers: Arc<Mutex<MemoryLayers>>) -> Self {
        Self { layers }
    }

    /// Wake-up context: L0 identity + L1 working summary + L2 palace index.
    pub fn wake_up(&self) -> Result<WakeUpContext, String> {
        self.with_layers(|layers| layers.wake_up())
    }

    /// Search L3 memory using the existing FTS-backed layer API.
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<Drawer>, String> {
        self.with_layers(|layers| layers.search(query, limit))
    }

    /// Add verbatim content into the memory palace.
    pub fn add(&self, content: &str, wing: &str, room: &str, hall: &str) -> Result<i64, String> {
        self.with_layers(|layers| layers.add_drawer(content, wing, room, hall))
    }

    /// Check whether content already appears in nearby search results.
    ///
    /// FairyField's current layer API does not expose vector similarity, so this
    /// uses conservative normalized-text equality over candidates from FTS
    /// search. Exact normalized matches are reported as similarity 1.0.
    pub fn duplicate_check(&self, content: &str, threshold: f32) -> Result<DuplicateCheck, String> {
        let normalized = normalize_text(content);
        if normalized.is_empty() {
            return Ok(DuplicateCheck {
                is_duplicate: false,
                matches: Vec::new(),
            });
        }

        let min_similarity = threshold.clamp(0.0, 1.0);
        let mut seen_ids = HashSet::new();
        let mut matches = Vec::new();

        for query in duplicate_queries(content) {
            let candidates = self.search(&query, 25).unwrap_or_default();
            for drawer in candidates {
                if !seen_ids.insert(drawer.id) {
                    continue;
                }

                let similarity = normalized_similarity(&normalized, &drawer.content);
                if similarity >= min_similarity {
                    matches.push(DuplicateMatch {
                        id: drawer.id,
                        wing: drawer.wing,
                        room: drawer.room,
                        hall: drawer.hall,
                        similarity,
                        content: drawer.content,
                    });
                }
            }
        }

        Ok(DuplicateCheck {
            is_duplicate: !matches.is_empty(),
            matches,
        })
    }

    /// Write a verbatim diary entry for an agent.
    pub fn diary_write(&self, agent: &str, content: &str) -> Result<i64, String> {
        let wing = diary_wing(agent);
        self.add(content, &wing, "diary", "hall_diary")
    }

    /// Read an agent's recent diary entries, newest first.
    pub fn diary_read(&self, agent: &str, limit: usize) -> Result<Vec<AgentDiaryEntry>, String> {
        let wing = diary_wing(agent);
        self.with_layers(|layers| {
            let mut entries: Vec<AgentDiaryEntry> = layers
                .recall(&wing, limit.saturating_mul(4).max(limit))
                .unwrap_or_default()
                .into_iter()
                .filter(|drawer| drawer.room == "diary")
                .map(|drawer| AgentDiaryEntry {
                    agent: agent.to_string(),
                    content: drawer.content,
                    wing: drawer.wing,
                    room: drawer.room,
                    hall: drawer.hall,
                    created_at: drawer.created_at,
                })
                .collect();

            entries.sort_by(|a, b| b.created_at.cmp(&a.created_at));
            entries.truncate(limit);
            Ok(entries)
        })
    }

    fn with_layers<T>(
        &self,
        f: impl FnOnce(&MemoryLayers) -> Result<T, String>,
    ) -> Result<T, String> {
        let layers = self.layers.lock().map_err(|e| e.to_string())?;
        f(&layers)
    }
}

fn diary_wing(agent: &str) -> String {
    let slug = normalize_slug(agent);
    if slug.is_empty() {
        "wing_agent".to_string()
    } else {
        format!("wing_{}", slug)
    }
}

fn normalize_slug(text: &str) -> String {
    text.chars()
        .flat_map(char::to_lowercase)
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || is_cjk(ch) {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>()
        .split('_')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("_")
}

fn duplicate_queries(content: &str) -> Vec<String> {
    let normalized = normalize_text(content);
    let terms: Vec<&str> = normalized.split_whitespace().collect();
    let mut queries = Vec::new();

    if !content.trim().is_empty() {
        queries.push(content.trim().to_string());
    }

    if !terms.is_empty() {
        queries.push(terms.iter().take(8).copied().collect::<Vec<_>>().join(" "));
    }

    if let Some(longest) = terms.iter().max_by_key(|term| term.chars().count()) {
        queries.push((*longest).to_string());
    }

    queries.sort();
    queries.dedup();
    queries
}

fn normalized_similarity(normalized_left: &str, right: &str) -> f32 {
    let normalized_right = normalize_text(right);
    if normalized_left == normalized_right {
        1.0
    } else {
        0.0
    }
}

fn normalize_text(text: &str) -> String {
    text.chars()
        .flat_map(char::to_lowercase)
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || is_cjk(ch) {
                ch
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn is_cjk(ch: char) -> bool {
    ('\u{4e00}'..='\u{9fff}').contains(&ch)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::store::MemoryStore;

    fn make_backend() -> AgentMemoryBackend {
        let store = MemoryStore::new(":memory:").unwrap();
        let layers = MemoryLayers::new(store);
        AgentMemoryBackend::new(Arc::new(Mutex::new(layers)))
    }

    #[test]
    fn add_and_search_preserves_verbatim_content() {
        let backend = make_backend();
        let content = "SESSION:2026-05-26|memory backend stores verbatim text";
        backend
            .add(content, "wing_code", "memory", "hall_facts")
            .unwrap();

        let results = backend.search("verbatim", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].content, content);
    }

    #[test]
    fn duplicate_check_detects_normalized_exact_match() {
        let backend = make_backend();
        backend
            .add(
                "Fairy remembers the Rust memory backend.",
                "wing_code",
                "memory",
                "hall_facts",
            )
            .unwrap();

        let check = backend
            .duplicate_check("fairy remembers the rust memory backend", 0.9)
            .unwrap();
        assert!(check.is_duplicate);
        assert_eq!(check.matches[0].similarity, 1.0);
    }

    #[test]
    fn duplicate_check_ignores_non_exact_match() {
        let backend = make_backend();
        backend
            .add(
                "Fairy remembers the Rust memory backend.",
                "wing_code",
                "memory",
                "hall_facts",
            )
            .unwrap();

        let check = backend
            .duplicate_check("Fairy remembers a different backend", 0.9)
            .unwrap();
        assert!(!check.is_duplicate);
    }

    #[test]
    fn diary_write_and_read_are_agent_scoped() {
        let backend = make_backend();
        backend
            .diary_write("memory Agent", "SESSION: wrote shared backend")
            .unwrap();
        backend
            .diary_write("tools Agent", "SESSION: unrelated tool work")
            .unwrap();

        let entries = backend.diary_read("memory Agent", 10).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].agent, "memory Agent");
        assert_eq!(entries[0].wing, "wing_memory_agent");
        assert_eq!(entries[0].room, "diary");
        assert_eq!(entries[0].hall, "hall_diary");
        assert_eq!(entries[0].content, "SESSION: wrote shared backend");
    }

    #[test]
    fn wake_up_delegates_to_layers() {
        let backend = make_backend();
        let ctx = backend.wake_up().unwrap();
        assert!(ctx.identity.contains("Fairy"));
    }
}
