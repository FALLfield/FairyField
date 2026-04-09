//! 语音识别模块 (ASR)
//!
//! 基于 sherpa-onnx Paraformer 模型的离线语音识别。
//! 当前提供 trait 抽象和 mock 实现。

use super::VoiceError;

/// ASR 配置
#[derive(Debug, Clone)]
pub struct AsrConfig {
    /// 模型路径
    pub model_path: String,
    /// 语言代码（如 "zh", "en"）
    pub language: String,
    /// 采样率
    pub sample_rate: u32,
    /// 是否启用标点
    pub enable_punctuation: bool,
}

impl Default for AsrConfig {
    fn default() -> Self {
        Self {
            model_path: String::new(),
            language: "zh".to_string(),
            sample_rate: 16000,
            enable_punctuation: true,
        }
    }
}

/// ASR 引擎 trait
pub trait AsrEngine: Send + Sync {
    /// 开始识别
    fn start(&self) -> Result<(), VoiceError>;
    /// 停止识别
    fn stop(&self) -> Result<(), VoiceError>;
    /// 喂入音频数据，返回识别到的文本（如果有完整句子）
    fn feed_audio(&self, pcm: &[f32], sample_rate: u32) -> Result<Option<String>, VoiceError>;
    /// 是否正在运行
    fn is_running(&self) -> bool;
}

/// Paraformer ASR 引擎（当前为 mock 实现）
pub struct ParaformerAsr {
    config: AsrConfig,
    running: std::sync::atomic::AtomicBool,
}

impl ParaformerAsr {
    pub fn new(config: AsrConfig) -> Result<Self, VoiceError> {
        Ok(Self {
            config,
            running: std::sync::atomic::AtomicBool::new(false),
        })
    }

    pub fn config(&self) -> &AsrConfig {
        &self.config
    }
}

impl AsrEngine for ParaformerAsr {
    fn start(&self) -> Result<(), VoiceError> {
        if self.config.model_path.is_empty() {
            return Err(VoiceError::ModelLoadFailed(
                "模型路径未配置".to_string(),
            ));
        }
        self.running.store(true, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }

    fn stop(&self) -> Result<(), VoiceError> {
        self.running.store(false, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }

    fn feed_audio(&self, _pcm: &[f32], _sample_rate: u32) -> Result<Option<String>, VoiceError> {
        if !self.running.load(std::sync::atomic::Ordering::Relaxed) {
            return Err(VoiceError::NotInitialized);
        }
        // Mock: 不返回识别结果，真实实现需要 sherpa-onnx
        Ok(None)
    }

    fn is_running(&self) -> bool {
        self.running.load(std::sync::atomic::Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> AsrConfig {
        AsrConfig {
            model_path: "/tmp/test_model".to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn test_paraformer_new() {
        let asr = ParaformerAsr::new(test_config()).unwrap();
        assert!(!asr.is_running());
        assert_eq!(asr.config().language, "zh");
    }

    #[test]
    fn test_paraformer_start_stop() {
        let asr = ParaformerAsr::new(test_config()).unwrap();
        asr.start().unwrap();
        assert!(asr.is_running());
        asr.stop().unwrap();
        assert!(!asr.is_running());
    }

    #[test]
    fn test_paraformer_start_no_model() {
        let asr = ParaformerAsr::new(AsrConfig::default()).unwrap();
        let result = asr.start();
        assert!(result.is_err());
    }

    #[test]
    fn test_paraformer_feed_audio_not_running() {
        let asr = ParaformerAsr::new(test_config()).unwrap();
        let result = asr.feed_audio(&[0.0f32; 1600], 16000);
        assert!(result.is_err());
    }

    #[test]
    fn test_paraformer_feed_audio_mock() {
        let asr = ParaformerAsr::new(test_config()).unwrap();
        asr.start().unwrap();
        let result = asr.feed_audio(&[0.0f32; 1600], 16000);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none()); // mock 不返回结果
    }
}
