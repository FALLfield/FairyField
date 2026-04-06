//! 配置管理模块
//!
//! 提供默认配置和应用配置结构的定义。

use serde::{Deserialize, Serialize};

/// 应用全局配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// LLM 相关配置
    pub llm: LlmConfig,
    /// 语音管道配置
    pub voice: VoiceConfig,
    /// 角色相关配置
    pub character: CharacterConfig,
    /// 窗口相关配置
    pub window: WindowConfig,
}

/// LLM 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// 提供商：openai / claude
    pub provider: String,
    /// API 端点 URL
    pub api_endpoint: String,
    /// 模型名称
    pub model: String,
    /// 系统提示词
    pub system_prompt: String,
    /// 最大上下文长度
    pub max_context_tokens: u32,
    /// 温度参数
    pub temperature: f32,
}

/// 语音管道配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceConfig {
    /// 是否启用语音识别
    pub asr_enabled: bool,
    /// 是否启用语音合成
    pub tts_enabled: bool,
    /// 是否启用语音活动检测
    pub vad_enabled: bool,
    /// TTS 说话人 ID
    pub tts_speaker: String,
    /// ASR 模型路径
    pub asr_model_path: String,
    /// TTS 模型路径
    pub tts_model_path: String,
    /// VAD 模型路径
    pub vad_model_path: String,
}

/// 角色配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterConfig {
    /// VRM 模型路径
    pub model_path: String,
    /// 默认表情
    pub default_expression: String,
    /// 角色名称
    pub name: String,
}

/// 窗口配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    /// 窗口宽度
    pub width: u32,
    /// 窗口高度
    pub height: u32,
    /// 是否透明
    pub transparent: bool,
    /// 是否置顶
    pub always_on_top: bool,
    /// 是否允许点击穿透
    pub click_through: bool,
}

/// 返回默认应用配置
pub fn default_config() -> AppConfig {
    AppConfig {
        llm: LlmConfig {
            provider: "openai".to_string(),
            api_endpoint: "https://api.openai.com/v1".to_string(),
            model: "gpt-4o".to_string(),
            system_prompt: "你是 FairyField 的 AI 伴侣角色，性格活泼可爱。".to_string(),
            max_context_tokens: 4096,
            temperature: 0.8,
        },
        voice: VoiceConfig {
            asr_enabled: false,
            tts_enabled: false,
            vad_enabled: false,
            tts_speaker: "default".to_string(),
            asr_model_path: String::new(),
            tts_model_path: String::new(),
            vad_model_path: String::new(),
        },
        character: CharacterConfig {
            model_path: "models/default/2031903848872972972007.glb".to_string(),
            default_expression: "neutral".to_string(),
            name: "Fairy".to_string(),
        },
        window: WindowConfig {
            width: 512,
            height: 768,
            transparent: true,
            always_on_top: true,
            click_through: true,
        },
    }
}
