//! 语音活动检测模块 (VAD)
//!
//! 支持 mock 和 sherpa-onnx silero-vad（离线 VAD）。

/// VAD 错误
#[derive(Debug, thiserror::Error)]
pub enum VadError {
    #[error("VAD 检测失败: {0}")]
    Detection(String),
    #[error("模型未加载")]
    ModelNotLoaded,
}

/// VAD 检测结果
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VadState {
    /// 检测到语音
    Speech,
    /// 静音
    Silence,
}

/// VAD 引擎 trait
pub trait VadEngine: Send + Sync {
    /// 检测一段音频是否有语音活动
    fn detect(&self, samples: &[f32], sample_rate: u32) -> Result<VadState, VadError>;

    /// 引擎名称
    fn name(&self) -> &str;
}

// ========== Mock VAD ==========

/// Mock VAD 引擎 — 基于简单能量阈值检测
pub struct MockVad {
    /// 能量阈值（RMS），高于此值判定为语音
    threshold: f32,
}

impl MockVad {
    pub fn new() -> Self {
        Self { threshold: 0.01 }
    }

    pub fn with_threshold(threshold: f32) -> Self {
        Self { threshold }
    }
}

impl Default for MockVad {
    fn default() -> Self {
        Self::new()
    }
}

impl VadEngine for MockVad {
    fn detect(&self, samples: &[f32], _sample_rate: u32) -> Result<VadState, VadError> {
        if samples.is_empty() {
            return Ok(VadState::Silence);
        }

        // 计算 RMS 能量
        let rms = (samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32).sqrt();

        Ok(if rms > self.threshold {
            VadState::Speech
        } else {
            VadState::Silence
        })
    }

    fn name(&self) -> &str {
        "mock"
    }
}

/// 创建 VAD 引擎
pub fn create_vad_engine(config: &crate::config::settings::VoiceConfig) -> Box<dyn VadEngine> {
    #[cfg(feature = "sherpa-onnx")]
    {
        let model_path = vad_model_path(config);
        if let Ok(engine) = SileroVad::new(model_path) {
            return Box::new(engine);
        }
    }
    #[cfg(not(feature = "sherpa-onnx"))]
    let _ = config;
    Box::new(MockVad::new())
}

#[cfg(feature = "sherpa-onnx")]
fn vad_model_path(config: &crate::config::settings::VoiceConfig) -> std::path::PathBuf {
    if !config.vad_model.is_empty() && std::path::Path::new(&config.vad_model).exists() {
        return std::path::PathBuf::from(&config.vad_model);
    }
    crate::config::settings::models_dir().join("vad/silero-vad.onnx")
}

/// sherpa-onnx silero-vad 引擎
///
/// 需要下载 silero-vad.onnx 模型文件才能启用（`cargo build --features sherpa-onnx`）。
/// 未启用 feature 时回退到 MockVad（RMS 能量检测）。
///
/// *模型下载指引：https://github.com/k2-fsa/sherpa-onnx/releases*
pub struct SileroVad {
    #[cfg(feature = "sherpa-onnx")]
    inner: Option<std::sync::Mutex<sherpa_onnx::VoiceActivityDetector>>,
    #[cfg(not(feature = "sherpa-onnx"))]
    fallback: MockVad,
}

impl SileroVad {
    #[cfg(feature = "sherpa-onnx")]
    pub fn new(model_path: std::path::PathBuf) -> Result<Self, VadError> {
        use sherpa_onnx::{SileroVadModelConfig, VadModelConfig, VoiceActivityDetector};

        let config = VadModelConfig {
            silero_vad: SileroVadModelConfig {
                model: Some(model_path.to_string_lossy().to_string()),
                threshold: 0.5,
                min_silence_duration: 0.5,
                min_speech_duration: 0.25,
                window_size: 512,
                max_speech_duration: 15.0,
            },
            sample_rate: 16000,
            num_threads: 1,
            provider: Some("cpu".to_string()),
            ..Default::default()
        };

        let vad = VoiceActivityDetector::create(&config, 20.0)
            .ok_or_else(|| VadError::Detection("VAD 初始化失败".to_string()))?;

        Ok(Self {
            inner: Some(std::sync::Mutex::new(vad)),
        })
    }

    #[cfg(not(feature = "sherpa-onnx"))]
    pub fn new(_model_path: std::path::PathBuf) -> Result<Self, VadError> {
        Ok(Self {
            fallback: MockVad::new(),
        })
    }
}

impl VadEngine for SileroVad {
    #[cfg(feature = "sherpa-onnx")]
    fn detect(&self, samples: &[f32], _sample_rate: u32) -> Result<VadState, VadError> {
        let inner = self
            .inner
            .as_ref()
            .ok_or_else(|| VadError::Detection("VAD 引擎未初始化".into()))?;
        let vad = inner
            .lock()
            .map_err(|e| VadError::Detection(format!("锁错误: {}", e)))?;

        vad.accept_waveform(samples);

        Ok(if vad.detected() {
            VadState::Speech
        } else {
            VadState::Silence
        })
    }

    #[cfg(not(feature = "sherpa-onnx"))]
    fn detect(&self, samples: &[f32], sample_rate: u32) -> Result<VadState, VadError> {
        self.fallback.detect(samples, sample_rate)
    }

    fn name(&self) -> &str {
        if cfg!(feature = "sherpa-onnx") {
            "sherpa-onnx-silero-vad"
        } else {
            "mock-vad-fallback"
        }
    }
}
