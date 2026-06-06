//! 配置管理模块
//!
//! 负责应用配置的加载、保存和默认值管理。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// 配置目录名称
const CONFIG_DIR_NAME: &str = ".fairyfield";
/// 配置文件名称
const CONFIG_FILE_NAME: &str = "config.json";
/// 本地密钥文件名称。
///
/// TODO: V1.1 将此存储替换为 macOS Keychain / Windows Credential Manager /
/// Secret Service。当前实现刻意与 config.json 分离，避免公开配置和前端 IPC
/// 泄漏密钥，同时修复 Onboarding 后重启丢失 API Key 的问题。
const SECRETS_FILE_NAME: &str = "secrets.json";
const CURRENT_CONFIG_VERSION: u32 = 1;

fn default_config_version() -> u32 {
    CURRENT_CONFIG_VERSION
}

/// 应用全局配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// 配置格式版本（用于未来迁移）
    #[serde(default = "default_config_version")]
    pub version: u32,
    pub llm: LlmConfig,
    pub voice: VoiceConfig,
    pub character: CharacterConfig,
    pub window: WindowConfig,
    pub memory: MemoryConfig,
    pub gateway: GatewayConfig,
    pub ui: UiConfig,
}

/// LLM 提供商预设
///
/// 定义一个 LLM 提供商的完整配置，包含名称、类型、端点、模型和密钥。
/// 多个预设可同时存在于配置中，通过 `active_provider` 指定当前使用的提供商。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderPreset {
    /// 预设名称（用户可读，如 "OpenAI GPT-4o"、"智谱 GLM-4"）
    pub name: String,
    /// 提供商类型: openai / claude / glm
    /// glm 使用 OpenAI 兼容接口，但作为独立提供商类型方便识别
    pub provider_type: String,
    /// API 端点 URL
    pub api_endpoint: String,
    /// 模型名称
    pub model: String,
    /// API 密钥：序列化时跳过（不写入文件）
    #[serde(skip_serializing, default)]
    pub api_key: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct SecretStore {
    #[serde(default)]
    llm_provider_keys: HashMap<String, String>,
}

/// LLM 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// 当前活跃的提供商名称（对应 ProviderPreset.name）
    /// 为空或不存在时回退到 provider 字段（向后兼容）
    #[serde(default)]
    pub active_provider: String,
    /// 多提供商预设列表
    #[serde(default = "default_providers")]
    pub providers: Vec<ProviderPreset>,
    // ---- 以下为向后兼容字段（旧配置文件仍可使用） ----
    #[serde(default)]
    pub provider: String,
    #[serde(default)]
    pub model: String,
    /// API 密钥：序列化时跳过（不写入文件），反序列化时用 default 填充
    #[serde(skip_serializing, default)]
    pub api_key: String,
    #[serde(default)]
    pub api_endpoint: String,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
    #[serde(default = "default_temperature")]
    pub temperature: f32,
}

/// 默认提供商预设列表
fn default_providers() -> Vec<ProviderPreset> {
    vec![
        ProviderPreset {
            name: "DeepSeek V4 Flash".to_string(),
            provider_type: "openai".to_string(),
            api_endpoint: "https://api.deepseek.com/v1".to_string(),
            model: "deepseek-chat".to_string(),
            api_key: String::new(),
        },
        ProviderPreset {
            name: "OpenAI GPT-4o".to_string(),
            provider_type: "openai".to_string(),
            api_endpoint: "https://api.openai.com/v1".to_string(),
            model: "gpt-4o".to_string(),
            api_key: String::new(),
        },
        ProviderPreset {
            name: "Claude Sonnet".to_string(),
            provider_type: "claude".to_string(),
            api_endpoint: "https://api.anthropic.com".to_string(),
            model: "claude-sonnet-4-20250514".to_string(),
            api_key: String::new(),
        },
        ProviderPreset {
            name: "智谱 GLM-4".to_string(),
            provider_type: "glm".to_string(),
            api_endpoint: "https://open.bigmodel.cn/api/paas/v4".to_string(),
            model: "glm-4".to_string(),
            api_key: String::new(),
        },
    ]
}

impl LlmConfig {
    /// 获取当前活跃的提供商预设
    ///
    /// 优先按 `active_provider` 名称查找；若未找到则回退到旧字段组合。
    pub fn active_preset(&self) -> ProviderPreset {
        if !self.active_provider.is_empty() {
            if let Some(preset) = self
                .providers
                .iter()
                .find(|p| p.name == self.active_provider)
            {
                // 环境变量覆盖 api_key
                let mut preset = preset.clone();
                self.apply_env_key(&mut preset);
                return preset;
            }
        }
        // 回退到旧字段（向后兼容）
        let mut preset = ProviderPreset {
            name: format!("{}-{}", self.provider, self.model),
            provider_type: self.provider.clone(),
            api_endpoint: self.api_endpoint.clone(),
            model: self.model.clone(),
            api_key: self.api_key.clone(),
        };
        self.apply_env_key(&mut preset);
        preset
    }

    /// 通过名称切换活跃提供商
    pub fn switch_provider(&mut self, name: &str) -> Result<(), String> {
        if self.providers.iter().any(|p| p.name == name) {
            self.active_provider = name.to_string();
            Ok(())
        } else {
            Err(format!("提供商预设 '{}' 不存在", name))
        }
    }

    /// 返回所有提供商预设名称
    pub fn provider_names(&self) -> Vec<String> {
        self.providers.iter().map(|p| p.name.clone()).collect()
    }

    /// 环境变量覆盖 API key
    fn apply_env_key(&self, preset: &mut ProviderPreset) {
        if preset.api_key.is_empty() {
            if let Ok(key) = std::env::var("FAIRYFIELD_API_KEY") {
                preset.api_key = key;
                return;
            }
            let var = match preset.provider_type.as_str() {
                "claude" => "ANTHROPIC_API_KEY",
                "glm" => "GLM_API_KEY",
                "deepseek" => "DEEPSEEK_API_KEY",
                _ => "OPENAI_API_KEY",
            };
            if let Ok(key) = std::env::var(var) {
                preset.api_key = key;
                return;
            }
            // 最后回退：也检查通用的 OPENAI_API_KEY（DeepSeek 等 OpenAI 兼容提供商）
            if var != "OPENAI_API_KEY" {
                if let Ok(key) = std::env::var("OPENAI_API_KEY") {
                    preset.api_key = key;
                }
            }
        }
    }
}

/// 语音管道配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceConfig {
    #[serde(default)]
    pub asr_model: String,
    #[serde(default)]
    pub tts_model: String,
    #[serde(default)]
    pub vad_model: String,
    #[serde(default = "default_sample_rate")]
    pub sample_rate: u32,
    #[serde(default)]
    pub language: String,
    #[serde(default = "default_tts_speaker_id")]
    pub tts_speaker_id: i32,
    #[serde(default = "default_tts_speed")]
    pub tts_speed: f32,
}

fn default_sample_rate() -> u32 {
    16000
}

fn default_tts_speaker_id() -> i32 {
    0
}
fn default_tts_speed() -> f32 {
    1.0
}
fn default_max_tokens() -> u32 {
    2048
}
fn default_temperature() -> f32 {
    0.7
}

/// 角色配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterConfig {
    #[serde(default)]
    pub model_path: String,
    #[serde(default)]
    pub default_expression: String,
}

/// 窗口配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    #[serde(default = "default_window_width")]
    pub width: f64,
    #[serde(default = "default_window_height")]
    pub height: f64,
    #[serde(default = "default_true")]
    pub transparent: bool,
    #[serde(default = "default_true")]
    pub always_on_top: bool,
    #[serde(default)]
    pub decorations: bool,
    #[serde(default = "default_true")]
    pub click_through: bool,
}

fn default_window_width() -> f64 {
    400.0
}
fn default_window_height() -> f64 {
    700.0
}
fn default_true() -> bool {
    true
}

/// 记忆系统配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    #[serde(default)]
    pub db_path: String,
    #[serde(default = "default_embedding_dim")]
    pub embedding_dim: usize,
    #[serde(default = "default_wakeup_tokens")]
    pub wakeup_max_tokens: usize,
    #[serde(default = "default_dedup_threshold")]
    pub dedup_threshold: f32,
}

fn default_embedding_dim() -> usize {
    384
}
fn default_wakeup_tokens() -> usize {
    600
}
fn default_dedup_threshold() -> f32 {
    0.9
}

/// 通信网关配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayConfig {
    #[serde(default)]
    pub telegram_enabled: bool,
    /// Telegram Bot Token：序列化时跳过（不写入文件）
    #[serde(skip_serializing, default)]
    pub telegram_token: String,
    #[serde(default)]
    pub cron_enabled: bool,
}

/// UI 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    #[serde(default = "default_chat_bubble_width")]
    pub chat_bubble_max_width: u32,
    #[serde(default = "default_font_size")]
    pub message_font_size: u32,
    #[serde(default)]
    pub show_debug_panel: bool,
    #[serde(default)]
    pub show_fps: bool,
    #[serde(default = "default_true")]
    pub show_chat: bool,
    #[serde(default = "default_true")]
    pub window_always_on_top: bool,
}

fn default_chat_bubble_width() -> u32 {
    320
}
fn default_font_size() -> u32 {
    14
}

/// 返回配置目录路径（`~/.fairyfield/`）
pub fn config_dir() -> PathBuf {
    dirs_home_dir().join(CONFIG_DIR_NAME)
}

/// 返回配置文件路径（`~/.fairyfield/config.json`）
pub fn config_file_path() -> PathBuf {
    config_dir().join(CONFIG_FILE_NAME)
}

fn secrets_file_path() -> PathBuf {
    config_dir().join(SECRETS_FILE_NAME)
}

fn load_secret_store_from_path(path: &PathBuf) -> Result<SecretStore, String> {
    if !path.exists() {
        return Ok(SecretStore::default());
    }

    let content = fs::read_to_string(path).map_err(|e| format!("读取密钥文件失败: {e}"))?;
    serde_json::from_str(&content).map_err(|e| format!("解析密钥文件失败: {e}"))
}

fn save_secret_store_to_path(path: &PathBuf, store: &SecretStore) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建密钥目录失败: {e}"))?;
    }

    let json = serde_json::to_string_pretty(store).map_err(|e| format!("序列化密钥失败: {e}"))?;
    fs::write(path, json).map_err(|e| format!("写入密钥文件失败: {e}"))
}

fn load_secret_store() -> Result<SecretStore, String> {
    load_secret_store_from_path(&secrets_file_path())
}

fn save_secret_store(store: &SecretStore) -> Result<(), String> {
    save_secret_store_to_path(&secrets_file_path(), store)
}

fn hydrate_llm_secrets(config: &mut AppConfig) {
    if let Ok(store) = load_secret_store() {
        for preset in &mut config.llm.providers {
            if preset.api_key.is_empty() {
                if let Some(key) = store.llm_provider_keys.get(&preset.name) {
                    preset.api_key = key.clone();
                }
            }
        }

        if config.llm.api_key.is_empty() {
            if let Some(key) = store.llm_provider_keys.get(&config.llm.active_provider) {
                config.llm.api_key = key.clone();
            }
        }
    }
}

/// 保存指定 LLM 提供商的 API Key 到本地密钥文件。
///
/// 注意：不会写入 config.json，因此 `platform_load_config` 返回给前端时仍会
/// 通过命令层清空密钥字段。
pub fn save_llm_provider_api_key(provider_name: &str, api_key: &str) -> Result<(), String> {
    if provider_name.trim().is_empty() {
        return Err("provider_name 不能为空".into());
    }
    if api_key.trim().is_empty() {
        return Err("api_key 不能为空".into());
    }

    let mut store = load_secret_store()?;
    store
        .llm_provider_keys
        .insert(provider_name.to_string(), api_key.to_string());
    save_secret_store(&store)
}

/// 从文件加载配置
///
/// 读取 `~/.fairyfield/config.json`，如果文件不存在则创建默认配置并保存。
/// API 密钥支持通过 `FAIRYFIELD_API_KEY` 环境变量覆盖文件中的值。
pub fn load_from_file() -> Result<AppConfig, String> {
    let path = config_file_path();

    let config = if path.exists() {
        let content = fs::read_to_string(&path).map_err(|e| format!("读取配置文件失败: {e}"))?;
        let mut cfg: AppConfig =
            serde_json::from_str(&content).map_err(|e| format!("解析配置文件失败: {e}"))?;

        hydrate_llm_secrets(&mut cfg);

        // 环境变量覆盖 api_key
        if let Ok(key) = std::env::var("FAIRYFIELD_API_KEY") {
            cfg.llm.api_key = key;
        }

        // 版本兼容性检查
        if cfg.version < CURRENT_CONFIG_VERSION {
            tracing::info!(
                "配置文件版本过旧 (v{} → v{})，将自动升级",
                cfg.version,
                CURRENT_CONFIG_VERSION
            );
            cfg.version = CURRENT_CONFIG_VERSION;
        }
        cfg
    } else {
        let mut cfg = default_config();
        // 尝试保存默认配置；目录不存在则创建
        if let Some(parent) = path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                tracing::error!("创建配置目录失败: {e}");
            }
        }
        if let Ok(json) = serde_json::to_string_pretty(&cfg) {
            if let Err(e) = fs::write(&path, json) {
                tracing::error!("写入默认配置文件失败: {e}");
            }
        }

        hydrate_llm_secrets(&mut cfg);

        // 环境变量覆盖 api_key
        if let Ok(key) = std::env::var("FAIRYFIELD_API_KEY") {
            cfg.llm.api_key = key;
            return Ok(cfg);
        }

        cfg
    };

    Ok(config)
}

/// 保存配置到文件
///
/// 将配置写入 `~/.fairyfield/config.json`。
pub fn save_to_file(config: &AppConfig) -> Result<(), String> {
    let dir = config_dir();
    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(|e| format!("创建配置目录失败: {e}"))?;
    }

    let json = serde_json::to_string_pretty(config).map_err(|e| format!("序列化配置失败: {e}"))?;

    let path = config_file_path();
    fs::write(&path, json).map_err(|e| format!("写入配置文件失败: {e}"))?;

    Ok(())
}

/// 返回默认应用配置
pub fn default_config() -> AppConfig {
    AppConfig {
        version: CURRENT_CONFIG_VERSION,
        llm: LlmConfig {
            active_provider: "DeepSeek V4 Flash".to_string(),
            providers: default_providers(),
            provider: "openai".to_string(),
            model: "gpt-4o".to_string(),
            api_key: String::new(),
            api_endpoint: "https://api.openai.com/v1".to_string(),
            max_tokens: 2048,
            temperature: 0.7,
        },
        voice: VoiceConfig {
            asr_model: "paraformer".to_string(),
            tts_model: "matcha".to_string(),
            vad_model: "silero-vad".to_string(),
            sample_rate: 16000,
            language: "zh".to_string(),
            tts_speaker_id: 0,
            tts_speed: 1.0,
        },
        character: CharacterConfig {
            model_path: "models/default/2031903848872972007.glb".to_string(),
            default_expression: "neutral".to_string(),
        },
        window: WindowConfig {
            width: 400.0,
            height: 700.0,
            transparent: true,
            always_on_top: true,
            decorations: false,
            click_through: true,
        },
        memory: MemoryConfig {
            db_path: "fairyfield.db".to_string(),
            embedding_dim: 384,
            wakeup_max_tokens: 600,
            dedup_threshold: 0.9,
        },
        gateway: GatewayConfig {
            telegram_enabled: false,
            telegram_token: String::new(),
            cron_enabled: false,
        },
        ui: UiConfig {
            chat_bubble_max_width: 320,
            message_font_size: 14,
            show_debug_panel: false,
            show_fps: false,
            show_chat: true,
            window_always_on_top: true,
        },
    }
}

/// 模型文件目录（`~/.fairyfield/models/`）
pub fn models_dir() -> PathBuf {
    dirs_home_dir().join(CONFIG_DIR_NAME).join("models")
}

/// 跨平台获取 HOME 目录
fn dirs_home_dir() -> PathBuf {
    // 优先使用 HOME 环境变量（Unix/macOS）
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home);
    }
    // 回退到 USERPROFILE（Windows）
    if let Ok(profile) = std::env::var("USERPROFILE") {
        return PathBuf::from(profile);
    }
    // 最终回退
    PathBuf::from(".")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_valid() {
        let config = default_config();
        assert_eq!(config.llm.provider, "openai");
        assert_eq!(config.llm.model, "gpt-4o");
        assert!(config.llm.api_key.is_empty());
        assert_eq!(config.llm.active_provider, "DeepSeek V4 Flash");
        assert_eq!(config.llm.providers.len(), 4);
        assert_eq!(config.llm.providers[0].provider_type, "openai");
        assert_eq!(config.llm.providers[0].model, "deepseek-chat");
        assert_eq!(config.llm.providers[1].provider_type, "openai");
        assert_eq!(config.llm.providers[2].provider_type, "claude");
        assert_eq!(config.llm.providers[3].provider_type, "glm");
        assert_eq!(config.voice.sample_rate, 16000);
        assert_eq!(config.character.default_expression, "neutral");
        assert!(config.window.transparent);
        assert!(config.window.always_on_top);
        assert!(!config.window.decorations);
        assert!(config.window.click_through);
        assert_eq!(config.memory.embedding_dim, 384);
        assert_eq!(config.memory.wakeup_max_tokens, 600);
        assert!((config.memory.dedup_threshold - 0.9).abs() < f32::EPSILON);
        assert!(!config.gateway.telegram_enabled);
        assert!(config.gateway.telegram_token.is_empty());
        assert!(!config.gateway.cron_enabled);
        assert_eq!(config.ui.chat_bubble_max_width, 320);
        assert!(config.ui.show_chat);
        assert!(!config.ui.show_debug_panel);
    }

    #[test]
    fn config_serializes_to_json() {
        let config = default_config();
        let json = serde_json::to_string(&config).expect("序列化失败");
        let parsed: AppConfig = serde_json::from_str(&json).expect("反序列化失败");
        assert_eq!(parsed.llm.provider, config.llm.provider);
        assert_eq!(parsed.voice.sample_rate, config.voice.sample_rate);
        assert_eq!(parsed.memory.embedding_dim, config.memory.embedding_dim);
        assert!(!config.gateway.telegram_enabled);
    }

    #[test]
    fn api_key_not_serialized() {
        let mut config = default_config();
        config.llm.api_key = "sk-secret-key-12345".to_string();
        let json = serde_json::to_string(&config).expect("序列化失败");
        assert!(
            !json.contains("sk-secret-key-12345"),
            "api_key 不应出现在序列化 JSON 中"
        );
        assert!(
            !json.contains("api_key"),
            "api_key 字段不应出现在序列化 JSON 中"
        );
    }

    #[test]
    fn telegram_token_not_serialized() {
        let mut config = default_config();
        config.gateway.telegram_token = "123456:ABC-DEF".to_string();
        let json = serde_json::to_string(&config).expect("序列化失败");
        assert!(
            !json.contains("123456:ABC-DEF"),
            "telegram_token 不应出现在序列化 JSON 中"
        );
        assert!(
            !json.contains("telegram_token"),
            "telegram_token 字段不应出现在序列化 JSON 中"
        );
    }

    #[test]
    fn provider_preset_api_key_not_serialized() {
        let mut config = default_config();
        config.llm.providers[0].api_key = "sk-secret-key-99999".to_string();
        let json = serde_json::to_string(&config).expect("序列化失败");
        assert!(
            !json.contains("sk-secret-key-99999"),
            "provider preset api_key 不应出现在序列化 JSON 中"
        );
    }

    #[test]
    fn secret_store_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("secrets.json");
        let mut store = SecretStore::default();
        store
            .llm_provider_keys
            .insert("DeepSeek V4 Flash".into(), "sk-secret".into());

        save_secret_store_to_path(&path, &store).unwrap();
        let loaded = load_secret_store_from_path(&path).unwrap();

        assert_eq!(
            loaded.llm_provider_keys.get("DeepSeek V4 Flash"),
            Some(&"sk-secret".to_string())
        );
    }

    #[test]
    fn active_preset_returns_named_provider() {
        let config = default_config();
        let preset = config.llm.active_preset();
        assert_eq!(preset.provider_type, "openai");
        assert_eq!(preset.model, "deepseek-chat");
    }

    #[test]
    fn active_preset_fallback_to_legacy_fields() {
        let mut config = default_config();
        config.llm.active_provider = String::new(); // 清空 active_provider
        config.llm.provider = "glm".to_string();
        config.llm.model = "glm-4-flash".to_string();
        config.llm.api_endpoint = "https://open.bigmodel.cn/api/paas/v4".to_string();
        let preset = config.llm.active_preset();
        assert_eq!(preset.provider_type, "glm");
        assert_eq!(preset.model, "glm-4-flash");
    }

    #[test]
    fn switch_provider_succeeds() {
        let mut config = default_config();
        assert!(config.llm.switch_provider("Claude Sonnet").is_ok());
        assert_eq!(config.llm.active_provider, "Claude Sonnet");
    }

    #[test]
    fn switch_provider_unknown_fails() {
        let mut config = default_config();
        assert!(config.llm.switch_provider("Nonexistent").is_err());
    }

    #[test]
    fn provider_names_returns_all() {
        let config = default_config();
        let names = config.llm.provider_names();
        assert_eq!(names.len(), 4);
        assert!(names.contains(&"DeepSeek V4 Flash".to_string()));
        assert!(names.contains(&"OpenAI GPT-4o".to_string()));
        assert!(names.contains(&"Claude Sonnet".to_string()));
        assert!(names.contains(&"智谱 GLM-4".to_string()));
    }

    #[test]
    fn backward_compat_deserialize_old_config() {
        // 旧格式配置（无 active_provider / providers 字段）
        let old_json = r#"{
            "llm": {
                "provider": "glm",
                "model": "glm-4",
                "api_endpoint": "https://open.bigmodel.cn/api/paas/v4",
                "max_tokens": 4096,
                "temperature": 0.5
            },
            "voice": { "asr_model": "paraformer", "tts_model": "kokoro", "vad_model": "silero-vad", "sample_rate": 16000, "language": "zh" },
            "character": { "model_path": "models/default/2031903848872972007.glb", "default_expression": "neutral" },
            "window": { "width": 400, "height": 700, "transparent": true, "always_on_top": true, "decorations": false, "click_through": true },
            "memory": { "db_path": "fairyfield.db", "embedding_dim": 384, "wakeup_max_tokens": 600, "dedup_threshold": 0.9 },
            "gateway": { "telegram_enabled": false, "telegram_token": "", "cron_enabled": false },
            "ui": { "chat_bubble_max_width": 320, "message_font_size": 14, "show_debug_panel": false, "show_fps": false, "show_chat": true, "window_always_on_top": true }
        }"#;
        let parsed: AppConfig = serde_json::from_str(old_json).expect("旧配置反序列化失败");
        assert_eq!(parsed.llm.provider, "glm");
        assert_eq!(parsed.llm.model, "glm-4");
        assert!(parsed.llm.active_provider.is_empty());
        // active_preset 应回退到旧字段
        let preset = parsed.llm.active_preset();
        assert_eq!(preset.provider_type, "glm");
    }

    #[test]
    fn minimal_new_format_config() {
        // 用户最简配置（仅 providers + active_provider，无旧字段）
        let json = r#"{
            "llm": {
                "active_provider": "DeepSeek V4 Flash",
                "providers": [
                    {
                        "name": "DeepSeek V4 Flash",
                        "provider_type": "openai",
                        "api_endpoint": "https://api.deepseek.com/v1",
                        "model": "deepseek-chat",
                        "api_key": "sk-test"
                    }
                ],
                "max_tokens": 2048,
                "temperature": 0.7
            },
            "voice": {},
            "character": {},
            "window": {},
            "memory": {},
            "gateway": {},
            "ui": {}
        }"#;
        let parsed: AppConfig =
            serde_json::from_str(json).expect("最简配置反序列化应成功（所有字段有 default）");
        assert_eq!(parsed.llm.active_provider, "DeepSeek V4 Flash");
        assert_eq!(parsed.llm.providers[0].api_key, "sk-test");
        assert_eq!(parsed.window.width, 400.0);
        assert_eq!(parsed.window.height, 700.0);
        assert!(parsed.window.transparent);
        assert!(parsed.ui.show_chat);
        assert!(!parsed.ui.show_debug_panel);
    }

    #[test]
    fn new_format_without_legacy_fields() {
        // 新格式配置（只有 providers 数组，无 provider/model/api_endpoint 旧字段）
        let new_json = r#"{
            "llm": {
                "active_provider": "DeepSeek V4 Flash",
                "providers": [
                    {
                        "name": "DeepSeek V4 Flash",
                        "provider_type": "openai",
                        "api_endpoint": "https://api.deepseek.com/v1",
                        "model": "deepseek-chat",
                        "api_key": "sk-test"
                    }
                ],
                "max_tokens": 2048,
                "temperature": 0.7
            },
            "voice": { "asr_model": "paraformer", "tts_model": "kokoro", "vad_model": "silero-vad", "sample_rate": 16000, "language": "zh" },
            "character": { "model_path": "models/default/2031903848872972007.glb", "default_expression": "neutral" },
            "window": { "width": 400, "height": 700, "transparent": true, "always_on_top": true, "decorations": false, "click_through": true },
            "memory": { "db_path": "fairyfield.db", "embedding_dim": 384, "wakeup_max_tokens": 600, "dedup_threshold": 0.9 },
            "gateway": { "telegram_enabled": false, "telegram_token": "", "cron_enabled": false },
            "ui": { "chat_bubble_max_width": 320, "message_font_size": 14, "show_debug_panel": false, "show_fps": false, "show_chat": true, "window_always_on_top": true }
        }"#;
        let parsed: AppConfig = serde_json::from_str(new_json)
            .expect("新格式配置反序列化失败（缺少 provider 等旧字段应被 default 填充）");
        assert_eq!(parsed.llm.active_provider, "DeepSeek V4 Flash");
        assert_eq!(parsed.llm.providers.len(), 1);
        assert_eq!(parsed.llm.providers[0].name, "DeepSeek V4 Flash");
        assert_eq!(parsed.llm.providers[0].api_key, "sk-test");
        assert_eq!(parsed.llm.max_tokens, 2048);
        assert_eq!(parsed.llm.temperature, 0.7);
        // 旧字段应被 default 填充为空
        assert!(parsed.llm.provider.is_empty());
        assert!(parsed.llm.model.is_empty());
        // active_preset 应从 providers 查找
        let preset = parsed.llm.active_preset();
        assert_eq!(preset.provider_type, "openai");
        assert_eq!(preset.model, "deepseek-chat");
    }
}
