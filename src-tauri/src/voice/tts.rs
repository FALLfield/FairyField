//! 语音合成模块 (TTS)
//!
//! 支持 mock（开发模式）和 sherpa-onnx Kokoro（离线 TTS）。

use crate::voice::audio::{AudioError, AudioOutput};

/// TTS 错误
#[derive(Debug, thiserror::Error)]
pub enum TtsError {
    #[error("TTS 合成失败: {0}")]
    Synthesis(String),
    #[error("音频播放失败: {0}")]
    Audio(#[from] AudioError),
    #[error("模型未加载")]
    ModelNotLoaded,
    #[error("配置错误: {0}")]
    Config(String),
}

/// TTS 合成结果
#[derive(Debug, Clone)]
pub struct TtsResult {
    /// PCM f32 采样数据（单声道）
    pub samples: Vec<f32>,
    /// 采样率
    pub sample_rate: u32,
}

/// TTS 引擎 trait — 所有 TTS 实现必须满足
pub trait TtsEngine: Send + Sync {
    /// 合成文本为 PCM 音频
    fn synthesize(&self, text: &str) -> Result<TtsResult, TtsError>;

    /// 获取采样率
    fn sample_rate(&self) -> u32;

    /// 引擎名称
    fn name(&self) -> &str;
}

// ========== Mock TTS（开发模式） ==========

/// Mock TTS 引擎 — 返回静音 PCM（用于开发和测试）
pub struct MockTts {
    sample_rate: u32,
    duration_secs: f32,
}

impl MockTts {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate,
            duration_secs: 0.5,
        }
    }

    /// 根据文本长度生成对应时长的静音
    pub fn with_text_proportional_duration(mut self) -> Self {
        self.duration_secs = -1.0; // 标记：按文本长度计算
        self
    }
}

impl TtsEngine for MockTts {
    fn synthesize(&self, text: &str) -> Result<TtsResult, TtsError> {
        let duration = if self.duration_secs < 0.0 {
            // 按文本长度估算：平均每个字符 0.08 秒
            (text.len() as f32 * 0.08).clamp(0.3, 10.0)
        } else {
            self.duration_secs
        };

        let num_samples = (self.sample_rate as f32 * duration) as usize;
        // 生成一个柔和的提示音（短促正弦波），而不是纯静音
        let mut samples = Vec::with_capacity(num_samples);
        let freq = 440.0; // A4 音高
        for i in 0..num_samples {
            let t = i as f32 / self.sample_rate as f32;
            // 渐入渐出的正弦波
            let envelope = if t < 0.05 {
                t / 0.05
            } else if t > duration - 0.05 {
                (duration - t) / 0.05
            } else {
                1.0
            };
            let sample = (2.0 * std::f32::consts::PI * freq * t).sin() * envelope * 0.1;
            samples.push(sample);
        }

        Ok(TtsResult {
            samples,
            sample_rate: self.sample_rate,
        })
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn name(&self) -> &str {
        "mock"
    }
}

// ========== macOS `say` 命令 TTS（真实语音，无需额外依赖） ==========

/// macOS `say` 命令 TTS — 使用系统自带语音合成
///
/// 通过 `std::process::Command` 调用 macOS 内置 `say` 命令，
/// 直接播放到扬声器。返回空 samples 以跳过 cpal 播放路径。
pub struct MacSayTts;

/// 清理 TTS 输入文本：去除 markdown 语法、emoji 等非语音内容
fn sanitize_tts_text(text: &str) -> String {
    let mut result = text.to_string();

    // 移除 markdown 粗体/斜体标记
    result = result.replace("**", "");
    result = result.replace("__", "");
    // 移除 markdown 标题标记
    let heading_re = regex_lite::Regex::new(r"^#{1,6}\s*").unwrap();
    result = heading_re.replace_all(&result, "").to_string();

    // 移除 markdown 链接，保留文本
    let link_re = regex_lite::Regex::new(r"\[([^\]]*)\]\([^)]*\)").unwrap();
    result = link_re.replace_all(&result, "$1").to_string();

    // 移除 markdown 列表标记
    let list_re = regex_lite::Regex::new(r"^\s*[-*+]\s").unwrap();
    result = list_re.replace_all(&result, "").to_string();

    // 移除 emoji（Unicode 范围）
    let emoji_re = regex_lite::Regex::new(
        r"[\x{1F600}-\x{1F64F}\x{1F300}-\x{1F5FF}\x{1F680}-\x{1F6FF}\x{1F1E0}-\x{1F1FF}\x{2600}-\x{26FF}\x{2700}-\x{27BF}\x{FE00}-\x{FE0F}\x{1F900}-\x{1F9FF}\x{1FA00}-\x{1FA6F}\x{1FA70}-\x{1FAFF}]"
    ).unwrap();
    result = emoji_re.replace_all(&result, "").to_string();

    // 移除剩余的特殊符号（保留中日韩文字、标点、字母、数字）
    let special_re = regex_lite::Regex::new(r"[→←↑↓★☆♦♢♠♣♥■□●○◆◇△▽▷◁]").unwrap();
    result = special_re.replace_all(&result, "").to_string();

    // 合并多余空白
    let whitespace_re = regex_lite::Regex::new(r"\s+").unwrap();
    result = whitespace_re.replace_all(&result, " ").to_string();

    result.trim().to_string()
}

impl MacSayTts {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MacSayTts {
    fn default() -> Self {
        Self::new()
    }
}

impl TtsEngine for MacSayTts {
    fn synthesize(&self, text: &str) -> Result<TtsResult, TtsError> {
        // 清理文本：去除 markdown、emoji 等非语音内容
        let clean_text = sanitize_tts_text(text);
        if clean_text.is_empty() {
            return Ok(TtsResult {
                samples: vec![],
                sample_rate: 22050,
            });
        }

        // 使用 macOS say 命令直接播放到扬声器
        // 必须指定中文语音（Tingting），否则默认英文语音会把中文读成乱码
        let output = std::process::Command::new("say")
            .arg("-v")
            .arg("Tingting") // 中文女声
            .arg("-r")
            .arg("200") // 语速
            .arg(&clean_text)
            .output()
            .map_err(|e| TtsError::Synthesis(format!("say 命令失败: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(TtsError::Synthesis(format!("say 失败: {}", stderr)));
        }

        // say 直接播放，返回空 samples（TtsManager 检测到空 samples 会跳过 cpal）
        Ok(TtsResult {
            samples: vec![],
            sample_rate: 22050,
        })
    }

    fn sample_rate(&self) -> u32 {
        22050
    }

    fn name(&self) -> &str {
        "mac-say"
    }
}

// ========== sherpa-onnx Kokoro TTS（离线，feature-gated） ==========

#[cfg(feature = "sherpa-onnx")]
pub mod kokoro {
    use super::{TtsEngine, TtsError, TtsResult};
    use std::path::PathBuf;
    use std::sync::Mutex;

    /// Kokoro TTS 配置
    #[derive(Clone)]
    pub struct KokoroConfig {
        pub model_path: PathBuf,
        pub voices_path: PathBuf,
        pub tokens_path: PathBuf,
        pub data_dir: PathBuf,
        pub speaker_id: i32,
        pub speed: f32,
    }

    impl Default for KokoroConfig {
        fn default() -> Self {
            let base = crate::config::settings::models_dir().join("kokoro");
            Self {
                model_path: base.join("model.onnx"),
                voices_path: base.join("voices.bin"),
                tokens_path: base.join("tokens.txt"),
                data_dir: base.join("espeak-ng-data"),
                speaker_id: 0,
                speed: 1.0,
            }
        }
    }

    /// Kokoro TTS 引擎（通过 sherpa-onnx OfflineTts）
    pub struct KokoroTts {
        tts: Mutex<sherpa_onnx::offline_tts::OfflineTts>,
        sample_rate: u32,
        #[allow(dead_code)]
        config: KokoroConfig,
    }

    impl KokoroTts {
        pub fn new(config: KokoroConfig) -> Result<Self, TtsError> {
            use sherpa_onnx::offline_tts::{
                OfflineTts, OfflineTtsConfig, OfflineTtsKokoroModelConfig, OfflineTtsModelConfig,
            };

            let tts_config = OfflineTtsConfig {
                model: OfflineTtsModelConfig {
                    kokoro: OfflineTtsKokoroModelConfig {
                        model: config.model_path.to_string_lossy().to_string(),
                        voices: config.voices_path.to_string_lossy().to_string(),
                        tokens: config.tokens_path.to_string_lossy().to_string(),
                        data_dir: config.data_dir.to_string_lossy().to_string(),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            };

            let tts = OfflineTts::new(tts_config)
                .map_err(|e| TtsError::Synthesis(format!("Kokoro 初始化失败: {}", e)))?;

            Ok(Self {
                tts: Mutex::new(tts),
                sample_rate: 24000,
                config,
            })
        }
    }

    impl TtsEngine for KokoroTts {
        fn synthesize(&self, text: &str) -> Result<TtsResult, TtsError> {
            let tts = self
                .tts
                .lock()
                .map_err(|e| TtsError::Synthesis(format!("锁错误: {}", e)))?;
            let audio = tts
                .generate(text, self.config.speaker_id, self.config.speed)
                .map_err(|e| TtsError::Synthesis(format!("Kokoro 合成失败: {}", e)))?;

            Ok(TtsResult {
                samples: audio.samples,
                sample_rate: audio.sample_rate as u32,
            })
        }

        fn sample_rate(&self) -> u32 {
            self.sample_rate
        }

        fn name(&self) -> &str {
            "kokoro"
        }
    }
}

/// 创建 TTS 引擎（根据配置和平台自动选择）
pub fn create_tts_engine(config: &crate::config::settings::VoiceConfig) -> Box<dyn TtsEngine> {
    // 如果启用了 sherpa-onnx feature，尝试创建 Kokoro 引擎
    #[cfg(feature = "sherpa-onnx")]
    {
        let _ = config; // 使用 config 避免未使用警告
        let model_dir = dirs_next_or_home(config);
        let kokoro_config = kokoro::KokoroConfig {
            model_path: model_dir.join("kokoro/model.onnx"),
            voices_path: model_dir.join("kokoro/voices.bin"),
            tokens_path: model_dir.join("kokoro/tokens.txt"),
            data_dir: model_dir.join("kokoro/espeak-ng-data"),
            speaker_id: config.tts_speaker_id,
            speed: config.tts_speed,
        };
        if let Ok(engine) = kokoro::KokoroTts::new(kokoro_config) {
            return Box::new(engine);
        }
        // 加载失败时回退到平台默认
    }

    #[cfg(not(feature = "sherpa-onnx"))]
    let _ = config;

    // macOS 上使用 say 命令获得真实语音
    #[cfg(target_os = "macos")]
    {
        Box::new(MacSayTts::new())
    }

    // 其他平台 fallback 到 MockTts
    #[cfg(not(target_os = "macos"))]
    {
        Box::new(MockTts::new(24000).with_text_proportional_duration())
    }
}

/// 获取模型目录（优先环境变量，否则 ~/.fairyfield/models/）
#[cfg(feature = "sherpa-onnx")]
fn dirs_next_or_home(config: &crate::config::settings::VoiceConfig) -> std::path::PathBuf {
    if !config.tts_model.is_empty() {
        if let Some(parent) = std::path::Path::new(&config.tts_model).parent() {
            return parent.to_path_buf();
        }
    }
    crate::config::settings::models_dir()
}

/// TTS 管理器 — 合成并播放音频
pub struct TtsManager {
    engine: Box<dyn TtsEngine>,
    audio: AudioOutput,
}

impl TtsManager {
    /// 创建 TTS 管理器
    pub fn new(engine: Box<dyn TtsEngine>) -> Result<Self, TtsError> {
        let audio = AudioOutput::new()?;
        Ok(Self { engine, audio })
    }

    /// 从配置创建
    pub fn from_config(config: &crate::config::settings::VoiceConfig) -> Result<Self, TtsError> {
        let engine = create_tts_engine(config);
        Self::new(engine)
    }

    /// 合成文本并播放到扬声器
    ///
    /// 如果引擎已经自行播放了音频（如 MacSayTts 通过 `say` 命令），
    /// samples 会为空，此时跳过 cpal 播放。
    pub fn speak(&self, text: &str) -> Result<TtsResult, TtsError> {
        let result = self.engine.synthesize(text)?;
        if !result.samples.is_empty() {
            self.audio
                .play(result.samples.clone(), result.sample_rate)?;
        }
        Ok(result)
    }

    /// 仅合成，不播放（用于获取 PCM 数据发送给前端做口型同步）
    pub fn synthesize_only(&self, text: &str) -> Result<TtsResult, TtsError> {
        self.engine.synthesize(text)
    }

    /// 获取采样率
    pub fn sample_rate(&self) -> u32 {
        self.engine.sample_rate()
    }

    /// 停止播放
    pub fn stop(&self) {
        self.audio.stop();
    }
}
