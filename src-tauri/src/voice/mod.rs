//! 语音管道模块
//!
//! 基于 sherpa-onnx 的语音处理管道，包含：
//! - 语音识别 ASR（asr）
//! - 语音合成 TTS（tts）
//! - 语音活动检测 VAD（vad）
//!
//! Phase 2 实现目标。

pub mod asr;
pub mod tts;
pub mod vad;
