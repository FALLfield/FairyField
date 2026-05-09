//! 配置管理模块
//!
//! 负责应用配置的加载、保存和默认值管理。

pub mod settings;

pub use settings::{
    config_dir, config_file_path, default_config, load_from_file, save_to_file, AppConfig,
    CharacterConfig, GatewayConfig, LlmConfig, MemoryConfig, ProviderPreset, UiConfig, VoiceConfig,
    WindowConfig,
};
