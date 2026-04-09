//! 语音合成模块 (TTS)
//!
//! 基于 sherpa-onnx Kokoro 模型的离线语音合成。
//! 当前提供 trait 抽象和 mock 实现。

use super::VoiceError;

/// TTS 配置
#[derive(Debug, Clone)]
pub struct TtsConfig {
    /// 模型路径
    pub model_path: String,
    /// 说话人 ID
    pub speaker_id: u32,
    /// 语速（0.5 - 2.0）
    pub speed: f32,
    /// 噪声比例
    pub noise_scale: f32,
}

impl Default for TtsConfig {
    fn default() -> Self {
        Self {
            model_path: String::new(),
            speaker_id: 0,
            speed: 1.0,
            noise_scale: 0.667,
        }
    }
}

/// TTS 引擎 trait
pub trait TtsEngine: Send + Sync {
    /// 合成语音，返回 PCM 数据
    fn synthesize(&self, text: &str) -> Result<Vec<f32>, VoiceError>;
    /// 获取输出采样率
    fn sample_rate(&self) -> u32;
    /// 设置说话人
    fn set_speaker(&self, id: u32);
}

/// Kokoro TTS 引擎（当前为 mock 实现）
pub struct KokoroTts {
    config: std::sync::RwLock<TtsConfig>,
}

impl KokoroTts {
    pub fn new(config: TtsConfig) -> Result<Self, VoiceError> {
        if config.speed < 0.5 || config.speed > 2.0 {
            return Err(VoiceError::AudioError(format!(
                "语速必须在 0.5-2.0 之间，当前: {}",
                config.speed
            )));
        }
        Ok(Self {
            config: std::sync::RwLock::new(config),
        })
    }

    pub fn config(&self) -> std::sync::RwLockReadGuard<'_, TtsConfig> {
        self.config.read().unwrap()
    }
}

impl TtsEngine for KokoroTts {
    fn synthesize(&self, text: &str) -> Result<Vec<f32>, VoiceError> {
        let config = self.config.read().unwrap();
        if config.model_path.is_empty() {
            return Err(VoiceError::ModelLoadFailed(
                "TTS 模型路径未配置".to_string(),
            ));
        }
        // Mock: 返回 1 秒静音（16000 采样率 * 1 秒）
        let duration_sec = text.len() as f32 / 5.0; // 粗略估算时长
        let samples = (duration_sec * config.speed * 16000.0) as usize;
        Ok(vec![0.0f32; samples.max(1600)])
    }

    fn sample_rate(&self) -> u32 {
        16000
    }

    fn set_speaker(&self, id: u32) {
        self.config.write().unwrap().speaker_id = id;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> TtsConfig {
        TtsConfig {
            model_path: "/tmp/test_tts".to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn test_kokoro_new() {
        let tts = KokoroTts::new(test_config()).unwrap();
        assert_eq!(tts.sample_rate(), 16000);
        assert_eq!(tts.config().speaker_id, 0);
    }

    #[test]
    fn test_kokoro_invalid_speed() {
        let result = KokoroTts::new(TtsConfig {
            speed: 3.0,
            ..Default::default()
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_kokoro_synthesize() {
        let tts = KokoroTts::new(test_config()).unwrap();
        let pcm = tts.synthesize("你好世界").unwrap();
        assert!(!pcm.is_empty());
        assert!(pcm.iter().all(|&s| s == 0.0)); // mock 返回静音
    }

    #[test]
    fn test_kokoro_synthesize_no_model() {
        let tts = KokoroTts::new(TtsConfig::default()).unwrap();
        let result = tts.synthesize("你好");
        assert!(result.is_err());
    }

    #[test]
    fn test_kokoro_set_speaker() {
        let tts = KokoroTts::new(test_config()).unwrap();
        tts.set_speaker(42);
        assert_eq!(tts.config().speaker_id, 42);
    }
}
