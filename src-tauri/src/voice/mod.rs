//! 语音管道模块
//!
//! 基于 sherpa-onnx 的语音处理管道，包含：
//! - 语音识别 ASR（asr）
//! - 语音合成 TTS（tts）
//! - 语音活动检测 VAD（vad）
//!
//! Phase 2 实现目标。当前提供 trait 抽象和 mock 实现。

pub mod asr;
pub mod tts;
pub mod vad;

use std::fmt;

/// 语音管道错误类型
#[derive(Debug)]
pub enum VoiceError {
    /// 引擎未初始化
    NotInitialized,
    /// 模型加载失败
    ModelLoadFailed(String),
    /// 音频处理错误
    AudioError(String),
    /// 功能尚未实现
    NotSupported(String),
}

impl fmt::Display for VoiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VoiceError::NotInitialized => write!(f, "语音引擎未初始化"),
            VoiceError::ModelLoadFailed(msg) => write!(f, "模型加载失败: {msg}"),
            VoiceError::AudioError(msg) => write!(f, "音频处理错误: {msg}"),
            VoiceError::NotSupported(msg) => write!(f, "不支持的功能: {msg}"),
        }
    }
}

impl std::error::Error for VoiceError {}
