//! OAuth token metadata and local token storage.
//!
//! Tokens are stored under `~/.fairyfield/tokens/<provider>.json`.
//! TODO(security): replace plaintext files with the platform keychain boundary
//! (Keychain on macOS, Credential Manager on Windows, Secret Service on Linux)
//! or envelope encryption once FairyField adds a cross-platform secret store.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OAuthConfig {
    pub provider: String,
    pub client_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_secret_env: Option<String>,
    pub auth_url: String,
    pub token_url: String,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
    pub pkce: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OAuthToken {
    pub provider: String,
    pub access_token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_type: Option<String>,
    pub scopes: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at_unix: Option<u64>,
    pub created_at_unix: u64,
    pub updated_at_unix: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RedactedTokenInfo {
    pub provider: String,
    pub has_access_token: bool,
    pub has_refresh_token: bool,
    pub scopes: Vec<String>,
    pub expires_at_unix: Option<u64>,
    pub refresh_needed: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum OAuthStoreError {
    #[error("home directory is not available")]
    MissingHome,
    #[error("invalid provider id: {0}")]
    InvalidProvider(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

impl OAuthToken {
    pub fn new(provider: impl Into<String>, access_token: impl Into<String>) -> Self {
        let now = now_unix();
        Self {
            provider: provider.into(),
            access_token: access_token.into(),
            refresh_token: None,
            token_type: Some("Bearer".into()),
            scopes: Vec::new(),
            expires_at_unix: None,
            created_at_unix: now,
            updated_at_unix: now,
        }
    }

    pub fn refresh_needed(&self, skew_seconds: u64) -> bool {
        match self.expires_at_unix {
            Some(expiry) => now_unix().saturating_add(skew_seconds) >= expiry,
            None => false,
        }
    }

    pub fn redacted(&self) -> RedactedTokenInfo {
        RedactedTokenInfo {
            provider: self.provider.clone(),
            has_access_token: !self.access_token.is_empty(),
            has_refresh_token: self
                .refresh_token
                .as_ref()
                .map(|t| !t.is_empty())
                .unwrap_or(false),
            scopes: self.scopes.clone(),
            expires_at_unix: self.expires_at_unix,
            refresh_needed: self.refresh_needed(300),
        }
    }
}

pub struct OAuthTokenStore {
    root: PathBuf,
}

impl OAuthTokenStore {
    pub fn default_dir() -> Result<PathBuf, OAuthStoreError> {
        let home = std::env::var_os("HOME").ok_or(OAuthStoreError::MissingHome)?;
        Ok(PathBuf::from(home).join(".fairyfield").join("tokens"))
    }

    pub fn new_default() -> Result<Self, OAuthStoreError> {
        Ok(Self::new(Self::default_dir()?))
    }

    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn save(&self, token: &OAuthToken) -> Result<(), OAuthStoreError> {
        validate_provider(&token.provider)?;
        std::fs::create_dir_all(&self.root)?;
        let mut token = token.clone();
        token.updated_at_unix = now_unix();
        let bytes = serde_json::to_vec_pretty(&token)?;
        std::fs::write(self.token_path(&token.provider)?, bytes)?;
        Ok(())
    }

    pub fn load(&self, provider: &str) -> Result<OAuthToken, OAuthStoreError> {
        let path = self.token_path(provider)?;
        let text = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&text)?)
    }

    pub fn delete(&self, provider: &str) -> Result<bool, OAuthStoreError> {
        let path = self.token_path(provider)?;
        if path.exists() {
            std::fs::remove_file(path)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn list_redacted(&self) -> Result<Vec<RedactedTokenInfo>, OAuthStoreError> {
        if !self.root.exists() {
            return Ok(Vec::new());
        }
        let mut tokens = Vec::new();
        for entry in std::fs::read_dir(&self.root)? {
            let entry = entry?;
            if entry.path().extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let text = std::fs::read_to_string(entry.path())?;
            let token: OAuthToken = serde_json::from_str(&text)?;
            tokens.push(token.redacted());
        }
        tokens.sort_by(|a, b| a.provider.cmp(&b.provider));
        Ok(tokens)
    }

    fn token_path(&self, provider: &str) -> Result<PathBuf, OAuthStoreError> {
        validate_provider(provider)?;
        Ok(self.root.join(format!("{provider}.json")))
    }
}

fn validate_provider(provider: &str) -> Result<(), OAuthStoreError> {
    let valid = !provider.is_empty()
        && provider
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-'));
    if valid {
        Ok(())
    } else {
        Err(OAuthStoreError::InvalidProvider(provider.into()))
    }
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_roundtrip_and_redaction() {
        let dir = tempfile::tempdir().unwrap();
        let store = OAuthTokenStore::new(dir.path());
        let mut token = OAuthToken::new("github", "secret-access-token");
        token.refresh_token = Some("secret-refresh-token".into());
        token.scopes = vec!["repo".into()];

        store.save(&token).unwrap();
        let loaded = store.load("github").unwrap();
        assert_eq!(loaded.access_token, "secret-access-token");

        let listed = store.list_redacted().unwrap();
        assert_eq!(listed.len(), 1);
        assert!(listed[0].has_access_token);
        assert!(listed[0].has_refresh_token);
        let listed_json = serde_json::to_string(&listed).unwrap();
        assert!(!listed_json.contains("secret-access-token"));
    }

    #[test]
    fn refresh_needed_uses_expiry_skew() {
        let mut token = OAuthToken::new("gmail", "token");
        token.expires_at_unix = Some(now_unix() + 100);
        assert!(token.refresh_needed(300));
        token.expires_at_unix = Some(now_unix() + 1000);
        assert!(!token.refresh_needed(300));
    }

    #[test]
    fn rejects_path_like_provider() {
        let dir = tempfile::tempdir().unwrap();
        let store = OAuthTokenStore::new(dir.path());
        let err = store.load("../github").unwrap_err();
        assert!(matches!(err, OAuthStoreError::InvalidProvider(_)));
    }
}
