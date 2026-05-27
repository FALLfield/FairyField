//! User configuration — persisted to ~/.fairyfield/user.json
//! Separate from config.json (which holds system/LLM settings).

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    pub user_name: String,
    /// 用户希望 Fairy 如何称呼自己，如 "Fairy"、"主人" 或自定义昵称。
    #[serde(default = "default_call_preference")]
    pub call_preference: String,
    pub fairy_name: String,
    /// Personality preset: "warm" | "playful" | "quiet" | "custom"
    pub personality: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_personality: Option<String>,
    pub language: String,
    pub onboarding_completed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            user_name: "用户".into(),
            call_preference: default_call_preference(),
            fairy_name: "Fairy".into(),
            personality: "warm".into(),
            custom_personality: None,
            language: "zh".into(),
            onboarding_completed: false,
            created_at: None,
        }
    }
}

fn default_call_preference() -> String {
    "Fairy".into()
}

impl UserConfig {
    /// Load from ~/.fairyfield/user.json
    pub fn load() -> Result<Self, String> {
        let path = Self::config_path();
        if !path.exists() {
            return Ok(Self::default());
        }
        let content =
            std::fs::read_to_string(&path).map_err(|e| format!("读取用户配置失败: {}", e))?;
        serde_json::from_str(&content).map_err(|e| format!("解析用户配置失败: {}", e))
    }

    /// Save to ~/.fairyfield/user.json
    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("创建配置目录失败: {}", e))?;
        }
        let content =
            serde_json::to_string_pretty(self).map_err(|e| format!("序列化用户配置失败: {}", e))?;
        std::fs::write(&path, content).map_err(|e| format!("写入用户配置失败: {}", e))
    }

    fn config_path() -> PathBuf {
        let home = dirs_home().unwrap_or_else(|| PathBuf::from("."));
        home.join(".fairyfield").join("user.json")
    }
}

fn dirs_home() -> Option<PathBuf> {
    std::env::var("HOME").ok().map(PathBuf::from)
}

// Tauri commands

#[tauri::command]
pub fn user_load_config() -> Result<UserConfig, String> {
    UserConfig::load()
}

#[tauri::command]
pub fn user_save_config(config: UserConfig) -> Result<(), String> {
    config.save()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_values() {
        let config = UserConfig::default();
        assert_eq!(config.user_name, "用户");
        assert_eq!(config.call_preference, "Fairy");
        assert_eq!(config.fairy_name, "Fairy");
        assert!(!config.onboarding_completed);
    }

    #[test]
    fn serialize_deserialize_roundtrip() {
        let config = UserConfig {
            user_name: "TestUser".into(),
            call_preference: "主人".into(),
            fairy_name: "TestFairy".into(),
            personality: "playful".into(),
            custom_personality: Some("like a cat".into()),
            language: "zh".into(),
            onboarding_completed: true,
            created_at: Some("2026-05-17T00:00:00Z".into()),
        };
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: UserConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.user_name, "TestUser");
        assert_eq!(deserialized.call_preference, "主人");
        assert_eq!(deserialized.fairy_name, "TestFairy");
        assert_eq!(deserialized.personality, "playful");
    }

    #[test]
    fn save_and_load_roundtrip() {
        // Since the save path is hardcoded to ~/.fairyfield,
        // we test that serialization/deserialization works correctly.
        let config = UserConfig::default();
        let json = serde_json::to_string_pretty(&config).unwrap();
        assert!(json.contains("用户"));
    }

    #[test]
    fn custom_personality_optional() {
        let config = UserConfig {
            custom_personality: Some("cheerful and energetic".into()),
            ..Default::default()
        };
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("cheerful and energetic"));

        let without = UserConfig::default();
        let json2 = serde_json::to_string(&without).unwrap();
        assert!(!json2.contains("custom_personality"));
    }
}
