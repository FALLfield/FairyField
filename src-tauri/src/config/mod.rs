//! 配置管理模块
//!
//! 负责应用配置的加载、保存和默认值管理。

pub mod settings;

pub use settings::{
    default_config, AppConfig, CharacterConfig, LlmConfig, VoiceConfig, WindowConfig,
};
