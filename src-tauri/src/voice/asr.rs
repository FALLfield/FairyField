//! 语音识别模块 (ASR)
//!
//! 支持 mock 和 sherpa-onnx Paraformer（离线 ASR）。

/// ASR 错误
#[derive(Debug, thiserror::Error)]
pub enum AsrError {
    #[error("ASR 识别失败: {0}")]
    Recognition(String),
    #[error("模型未加载")]
    ModelNotLoaded,
    #[error("配置错误: {0}")]
    Config(String),
}

/// ASR 识别结果
#[derive(Debug, Clone)]
pub struct AsrResult {
    /// 识别文本
    pub text: String,
    /// 置信度 (0.0 - 1.0)
    pub confidence: f32,
}

/// ASR 引擎 trait
pub trait AsrEngine: Send + Sync {
    /// 识别音频数据（PCM f32, 单声道）
    fn recognize(&self, samples: &[f32], sample_rate: u32) -> Result<AsrResult, AsrError>;

    /// 引擎名称
    fn name(&self) -> &str;
}

// ========== Mock ASR ==========

/// Mock ASR 引擎 — 返回固定文本（用于开发测试）
pub struct MockAsr {
    mock_text: String,
}

impl MockAsr {
    pub fn new() -> Self {
        Self {
            mock_text: String::new(),
        }
    }

    pub fn with_text(text: impl Into<String>) -> Self {
        Self {
            mock_text: text.into(),
        }
    }
}

impl Default for MockAsr {
    fn default() -> Self {
        Self::new()
    }
}

impl AsrEngine for MockAsr {
    fn recognize(&self, _samples: &[f32], _sample_rate: u32) -> Result<AsrResult, AsrError> {
        Ok(AsrResult {
            text: self.mock_text.clone(),
            confidence: 0.95,
        })
    }

    fn name(&self) -> &str {
        "mock"
    }
}

/// 创建 ASR 引擎
pub fn create_asr_engine(config: &crate::config::settings::VoiceConfig) -> Box<dyn AsrEngine> {
    #[cfg(feature = "sherpa-onnx")]
    {
        let model_dir = asr_model_dir(config);
        if let Ok(engine) = SherpaOnnxAsr::new(model_dir) {
            return Box::new(engine);
        }
    }
    #[cfg(not(feature = "sherpa-onnx"))]
    let _ = config;
    Box::new(MockAsr::new())
}

#[cfg(feature = "sherpa-onnx")]
fn asr_model_dir(config: &crate::config::settings::VoiceConfig) -> std::path::PathBuf {
    if !config.asr_model.is_empty() && std::path::Path::new(&config.asr_model).exists() {
        return std::path::PathBuf::from(&config.asr_model);
    }
    crate::config::settings::models_dir().join("paraformer")
}

/// sherpa-onnx Paraformer ASR 引擎
///
/// 需要下载 Paraformer 模型文件才能启用（`cargo build --features sherpa-onnx`）。
/// 未启用 feature 时回退到 MockAsr。
///
/// *模型下载指引：https://k2-fsa.github.io/sherpa/onnx/pretrained_models/offline-paraformer/index.html*
pub struct SherpaOnnxAsr {
    #[cfg(feature = "sherpa-onnx")]
    inner: Option<std::sync::Mutex<sherpa_onnx::OfflineRecognizer>>,
    #[cfg(not(feature = "sherpa-onnx"))]
    fallback: MockAsr,
}

impl SherpaOnnxAsr {
    #[cfg(feature = "sherpa-onnx")]
    pub fn new(model_dir: std::path::PathBuf) -> Result<Self, AsrError> {
        use sherpa_onnx::{
            OfflineModelConfig, OfflineParaformerModelConfig, OfflineRecognizer,
            OfflineRecognizerConfig,
        };

        let config = OfflineRecognizerConfig {
            model_config: OfflineModelConfig {
                paraformer: OfflineParaformerModelConfig {
                    model: Some(model_dir.join("model.onnx").to_string_lossy().to_string()),
                },
                tokens: Some(model_dir.join("tokens.txt").to_string_lossy().to_string()),
                provider: Some("cpu".to_string()),
                num_threads: 2,
                ..Default::default()
            },
            decoding_method: Some("greedy_search".to_string()),
            ..Default::default()
        };

        let recognizer = OfflineRecognizer::create(&config)
            .ok_or_else(|| AsrError::Config("ASR 初始化失败".to_string()))?;

        Ok(Self {
            inner: Some(std::sync::Mutex::new(recognizer)),
        })
    }

    #[cfg(not(feature = "sherpa-onnx"))]
    pub fn new(_model_dir: std::path::PathBuf) -> Result<Self, AsrError> {
        Ok(Self {
            fallback: MockAsr::new(),
        })
    }
}

impl AsrEngine for SherpaOnnxAsr {
    #[cfg(feature = "sherpa-onnx")]
    fn recognize(&self, samples: &[f32], sample_rate: u32) -> Result<AsrResult, AsrError> {
        let inner = self
            .inner
            .as_ref()
            .ok_or_else(|| AsrError::Config("ASR 引擎未初始化".into()))?;
        let recognizer = inner
            .lock()
            .map_err(|e| AsrError::Recognition(format!("锁错误: {}", e)))?;

        let stream = recognizer.create_stream();
        stream.accept_waveform(sample_rate as i32, samples);
        recognizer.decode(&stream);

        let text = stream
            .get_result()
            .map(|result| result.text)
            .unwrap_or_default();

        Ok(AsrResult {
            text,
            confidence: 0.85,
        })
    }

    #[cfg(not(feature = "sherpa-onnx"))]
    fn recognize(&self, samples: &[f32], sample_rate: u32) -> Result<AsrResult, AsrError> {
        self.fallback.recognize(samples, sample_rate)
    }

    fn name(&self) -> &str {
        if cfg!(feature = "sherpa-onnx") {
            "sherpa-onnx-paraformer"
        } else {
            "mock-asr-fallback"
        }
    }
}
