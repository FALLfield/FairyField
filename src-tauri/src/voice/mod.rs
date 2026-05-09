//! 语音管道模块
//!
//! 整合 VAD（语音活动检测）、ASR（语音识别）、TTS（语音合成）。
//! 默认使用 Mock 实现，启用 `sherpa-onnx` feature 后使用真实引擎。

pub mod asr;
pub mod audio;
pub mod audio_input;
pub mod tts;
pub mod vad;

use asr::{AsrEngine, AsrResult};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tts::{TtsEngine, TtsManager, TtsResult};
use vad::{VadEngine, VadState};

/// 语音管道事件（通过 Tauri event 发送到前端）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum VoiceEvent {
    /// VAD 状态变化
    VadStateChanged { is_speech: bool },
    /// ASR 识别结果
    AsrResult { text: String, confidence: f32 },
    /// TTS 开始播放
    TtsStarted { text: String },
    /// TTS 播放完成
    TtsFinished,
    /// PCM 音频数据（发送到前端做口型同步）
    PcmData { samples: Vec<f32>, sample_rate: u32 },
    /// 错误
    Error { message: String },
}

/// 语音管道错误
#[derive(Debug, thiserror::Error)]
pub enum PipelineError {
    #[error("ASR 错误: {0}")]
    Asr(#[from] asr::AsrError),
    #[error("TTS 错误: {0}")]
    Tts(#[from] tts::TtsError),
    #[error("VAD 错误: {0}")]
    Vad(#[from] vad::VadError),
    #[error("管道未初始化")]
    NotInitialized,
    #[error("管道正在运行")]
    AlreadyRunning,
}

/// 语音管道 — 协调 VAD → ASR → TTS 的完整流程
pub struct VoicePipeline {
    /// ASR 引擎
    asr: Box<dyn AsrEngine>,
    /// VAD 引擎
    vad: Box<dyn VadEngine>,
    /// TTS 管理器（需要 &mut，用 Mutex 包装以便共享）
    tts: Arc<Mutex<TtsManager>>,
    /// 当前 VAD 状态
    vad_state: VadState,
}

impl VoicePipeline {
    /// 从配置创建语音管道
    pub fn from_config(
        config: &crate::config::settings::VoiceConfig,
    ) -> Result<Self, PipelineError> {
        let asr = asr::create_asr_engine(config);
        let vad = vad::create_vad_engine(config);
        let tts = TtsManager::from_config(config)?;

        Ok(Self {
            asr,
            vad,
            tts: Arc::new(Mutex::new(tts)),
            vad_state: VadState::Silence,
        })
    }

    /// 使用默认 Mock 引擎创建管道
    ///
    /// macOS 上 TTS 使用 `say` 命令（真实语音），其他平台使用 MockTts。
    pub fn new_mock() -> Result<Self, PipelineError> {
        let asr = Box::new(asr::MockAsr::new());
        let vad = Box::new(vad::MockVad::new());

        #[cfg(target_os = "macos")]
        let tts_engine: Box<dyn TtsEngine> = Box::new(tts::MacSayTts::new());
        #[cfg(not(target_os = "macos"))]
        let tts_engine: Box<dyn TtsEngine> = Box::new(tts::MockTts::new(24000));

        let tts = TtsManager::new(tts_engine)?;

        Ok(Self {
            asr,
            vad,
            tts: Arc::new(Mutex::new(tts)),
            vad_state: VadState::Silence,
        })
    }

    /// 检测音频中的语音活动
    pub fn detect_vad(
        &mut self,
        samples: &[f32],
        sample_rate: u32,
    ) -> Result<VadState, PipelineError> {
        let state = self.vad.detect(samples, sample_rate)?;
        self.vad_state = state;
        Ok(state)
    }

    /// 识别音频中的文本
    pub fn recognize(&self, samples: &[f32], sample_rate: u32) -> Result<AsrResult, PipelineError> {
        let result = self.asr.recognize(samples, sample_rate)?;
        Ok(result)
    }

    /// 合成文本为音频（不播放，仅获取 PCM 数据）
    pub fn synthesize(&self, text: &str) -> Result<TtsResult, PipelineError> {
        let tts = self.tts.blocking_lock();
        let result = tts.synthesize_only(text)?;
        Ok(result)
    }

    /// 合成文本并播放到扬声器
    pub async fn speak(&self, text: &str) -> Result<TtsResult, PipelineError> {
        let mut tts = self.tts.lock().await;
        let result = tts.speak(text)?;
        Ok(result)
    }

    /// 停止 TTS 播放
    pub async fn stop_tts(&self) {
        let mut tts = self.tts.lock().await;
        tts.stop();
    }

    /// 获取当前 VAD 状态
    pub fn vad_state(&self) -> bool {
        self.vad_state == VadState::Speech
    }

    /// 获取 TTS 管理器的 Arc 引用（用于在 Tauri command 中共享）
    pub fn tts_manager(&self) -> Arc<Mutex<TtsManager>> {
        self.tts.clone()
    }
}
