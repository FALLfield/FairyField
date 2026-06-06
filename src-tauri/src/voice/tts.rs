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

#[cfg(any(feature = "sherpa-onnx", test))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TtsLang {
    Zh,
    En,
}

#[cfg(any(feature = "sherpa-onnx", test))]
#[derive(Debug, Clone, PartialEq, Eq)]
struct TtsSegment {
    lang: TtsLang,
    text: String,
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

/// 清理 TTS 输入文本：去除 markdown、代码块、emoji、URL 等非语音内容。
fn clean_tts_text(text: &str) -> String {
    let mut lines = Vec::new();
    let mut in_code_block = false;
    for line in text.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_code_block = !in_code_block;
            continue;
        }
        if !in_code_block {
            lines.push(line);
        }
    }

    let mut result = lines.join(" ");

    // 移除内部控制标签和工具标签
    let emotion_re = regex_lite::Regex::new(r"\[emotion:[^\]]+\]").unwrap();
    result = emotion_re.replace_all(&result, "").to_string();
    let bracket_tag_re = regex_lite::Regex::new(r"\[[a-zA-Z_ -]{1,24}:[^\]]+\]").unwrap();
    result = bracket_tag_re.replace_all(&result, "").to_string();

    // 移除 URL 和 inline code
    let url_re = regex_lite::Regex::new(r"https?://\S+").unwrap();
    result = url_re.replace_all(&result, "").to_string();
    let inline_code_re = regex_lite::Regex::new(r"`[^`]*`").unwrap();
    result = inline_code_re.replace_all(&result, "").to_string();

    // 移除 markdown 粗体/斜体标记
    result = result.replace("**", "");
    result = result.replace("__", "");
    result = result.replace('*', "");
    result = result.replace('_', "");

    // 移除 markdown 链接，保留文本
    let link_re = regex_lite::Regex::new(r"\[([^\]]*)\]\([^)]*\)").unwrap();
    result = link_re.replace_all(&result, "$1").to_string();

    let cleaned_lines: Vec<String> = result
        .lines()
        .map(|line| {
            let mut line = line.trim().to_string();
            while line.starts_with('#') || line.starts_with('>') {
                line = line[1..].trim_start().to_string();
            }
            if line.starts_with("- ") || line.starts_with("+ ") {
                line = line[2..].trim_start().to_string();
            }
            line
        })
        .collect();
    result = cleaned_lines.join(" ");

    // 移除 emoji（Unicode 范围）
    let emoji_re = regex_lite::Regex::new(
        r"[\x{1F600}-\x{1F64F}\x{1F300}-\x{1F5FF}\x{1F680}-\x{1F6FF}\x{1F1E0}-\x{1F1FF}\x{2600}-\x{26FF}\x{2700}-\x{27BF}\x{FE00}-\x{FE0F}\x{1F900}-\x{1F9FF}\x{1FA00}-\x{1FA6F}\x{1FA70}-\x{1FAFF}]"
    ).unwrap();
    result = emoji_re.replace_all(&result, "").to_string();

    // 移除剩余的特殊符号（保留中日韩文字、标点、字母、数字）
    let special_re = regex_lite::Regex::new(r"[→←↑↓★☆♦♢♠♣♥■□●○◆◇△▽▷◁|$]").unwrap();
    result = special_re.replace_all(&result, "").to_string();
    let markdown_re = regex_lite::Regex::new(r"[#~=<>{}\[\]\\]").unwrap();
    result = markdown_re.replace_all(&result, "").to_string();

    // 合并多余空白
    let whitespace_re = regex_lite::Regex::new(r"\s+").unwrap();
    result = whitespace_re.replace_all(&result, " ").to_string();

    result.trim().to_string()
}

fn split_tts_utterances(text: &str) -> Vec<String> {
    const MAX_CHARS: usize = 72;
    let mut chunks = Vec::new();
    let mut current = String::new();

    for ch in text.chars() {
        current.push(ch);
        let count = current.chars().count();
        let is_sentence_end =
            matches!(ch, '。' | '！' | '？' | '.' | '!' | '?' | '；' | ';' | '\n');
        if is_sentence_end || count >= MAX_CHARS {
            push_tts_chunk(&mut chunks, &mut current);
        }
    }
    push_tts_chunk(&mut chunks, &mut current);
    chunks
}

#[cfg(any(feature = "sherpa-onnx", test))]
fn split_language_segments(text: &str) -> Vec<TtsSegment> {
    let mut segments = Vec::new();
    let mut current = String::new();
    let mut current_lang: Option<TtsLang> = None;

    for ch in text.chars() {
        let ch_lang = char_tts_lang(ch);
        if let Some(lang) = ch_lang {
            if let Some(active) = current_lang {
                if active != lang {
                    push_language_segment(&mut segments, active, &mut current);
                }
            }
            current_lang = Some(lang);
            current.push(ch);
        } else if current_lang.is_some() {
            current.push(ch);
        }
    }

    if let Some(lang) = current_lang {
        push_language_segment(&mut segments, lang, &mut current);
    }

    segments
}

#[cfg(any(feature = "sherpa-onnx", test))]
fn push_language_segment(segments: &mut Vec<TtsSegment>, lang: TtsLang, current: &mut String) {
    let text = current.trim();
    if !text.is_empty() {
        segments.push(TtsSegment {
            lang,
            text: text.to_string(),
        });
    }
    current.clear();
}

#[cfg(any(feature = "sherpa-onnx", test))]
fn char_tts_lang(ch: char) -> Option<TtsLang> {
    if ch.is_ascii_alphabetic() {
        Some(TtsLang::En)
    } else if is_cjk(ch) {
        Some(TtsLang::Zh)
    } else {
        None
    }
}

#[cfg(any(feature = "sherpa-onnx", test))]
fn is_cjk(ch: char) -> bool {
    matches!(
        ch as u32,
        0x3400..=0x4DBF
            | 0x4E00..=0x9FFF
            | 0xF900..=0xFAFF
            | 0x20000..=0x2A6DF
            | 0x2A700..=0x2B73F
            | 0x2B740..=0x2B81F
            | 0x2B820..=0x2CEAF
    )
}

fn push_tts_chunk(chunks: &mut Vec<String>, current: &mut String) {
    let chunk = current.trim();
    if !chunk.is_empty() {
        chunks.push(chunk.to_string());
    }
    current.clear();
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
        let clean_text = clean_tts_text(text);
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

// ========== sherpa-onnx Matcha 双语 TTS（低延迟，feature-gated） ==========

#[cfg(feature = "sherpa-onnx")]
pub mod matcha {
    use super::{split_language_segments, TtsEngine, TtsError, TtsLang, TtsResult};
    use std::path::PathBuf;
    use std::sync::Mutex;

    #[derive(Clone)]
    pub struct MatchaConfig {
        pub acoustic_model: PathBuf,
        pub vocoder: PathBuf,
        pub tokens: PathBuf,
        pub data_dir: Option<PathBuf>,
        pub dict_dir: Option<PathBuf>,
        pub lexicon: Option<String>,
        pub rule_fsts: Option<String>,
        pub speed: f32,
        pub name: &'static str,
    }

    pub struct MatchaTts {
        tts: Mutex<sherpa_onnx::OfflineTts>,
        sample_rate: u32,
        speed: f32,
        name: &'static str,
    }

    impl MatchaTts {
        pub fn new(config: MatchaConfig) -> Result<Self, TtsError> {
            use sherpa_onnx::{
                OfflineTts, OfflineTtsConfig, OfflineTtsMatchaModelConfig, OfflineTtsModelConfig,
            };

            let tts_config = OfflineTtsConfig {
                model: OfflineTtsModelConfig {
                    matcha: OfflineTtsMatchaModelConfig {
                        acoustic_model: Some(config.acoustic_model.to_string_lossy().to_string()),
                        vocoder: Some(config.vocoder.to_string_lossy().to_string()),
                        tokens: Some(config.tokens.to_string_lossy().to_string()),
                        data_dir: config
                            .data_dir
                            .as_ref()
                            .map(|path| path.to_string_lossy().to_string()),
                        dict_dir: config
                            .dict_dir
                            .as_ref()
                            .map(|path| path.to_string_lossy().to_string()),
                        lexicon: config.lexicon.clone(),
                        length_scale: (1.0 / config.speed.clamp(0.5, 2.0)).clamp(0.5, 2.0),
                        ..Default::default()
                    },
                    provider: Some("cpu".to_string()),
                    num_threads: super::tts_thread_count(),
                    ..Default::default()
                },
                rule_fsts: config.rule_fsts.clone(),
                max_num_sentences: 1,
                silence_scale: 0.15,
                ..Default::default()
            };

            let tts = OfflineTts::create(&tts_config)
                .ok_or_else(|| TtsError::Synthesis(format!("{} 初始化失败", config.name)))?;
            let sample_rate = tts.sample_rate() as u32;
            Ok(Self {
                tts: Mutex::new(tts),
                sample_rate,
                speed: config.speed,
                name: config.name,
            })
        }
    }

    impl TtsEngine for MatchaTts {
        fn synthesize(&self, text: &str) -> Result<TtsResult, TtsError> {
            let tts = self
                .tts
                .lock()
                .map_err(|e| TtsError::Synthesis(format!("锁错误: {}", e)))?;
            let generation_config = sherpa_onnx::GenerationConfig {
                speed: self.speed,
                ..Default::default()
            };
            let audio = tts
                .generate_with_config::<fn(&[f32], f32) -> bool>(text, &generation_config, None)
                .ok_or_else(|| TtsError::Synthesis(format!("{} 合成失败", self.name)))?;
            Ok(TtsResult {
                samples: audio.samples().to_vec(),
                sample_rate: audio.sample_rate() as u32,
            })
        }

        fn sample_rate(&self) -> u32 {
            self.sample_rate
        }

        fn name(&self) -> &str {
            self.name
        }
    }

    pub struct MatchaBilingualTts {
        zh: MatchaTts,
        en: MatchaTts,
        sample_rate: u32,
    }

    impl MatchaBilingualTts {
        pub fn new(zh: MatchaConfig, en: MatchaConfig) -> Result<Self, TtsError> {
            let zh = MatchaTts::new(zh)?;
            let en = MatchaTts::new(en)?;
            if zh.sample_rate() != en.sample_rate() {
                return Err(TtsError::Config(format!(
                    "Matcha 中英采样率不一致: zh={} en={}",
                    zh.sample_rate(),
                    en.sample_rate()
                )));
            }
            let sample_rate = zh.sample_rate();
            Ok(Self {
                zh,
                en,
                sample_rate,
            })
        }
    }

    impl TtsEngine for MatchaBilingualTts {
        fn synthesize(&self, text: &str) -> Result<TtsResult, TtsError> {
            let segments = split_language_segments(text);
            if segments.is_empty() {
                return Ok(TtsResult {
                    samples: vec![],
                    sample_rate: self.sample_rate,
                });
            }

            let mut samples = Vec::new();
            let pause = vec![0.0; (self.sample_rate as f32 * 0.035) as usize];
            for (index, segment) in segments.iter().enumerate() {
                let result = match segment.lang {
                    TtsLang::Zh => self.zh.synthesize(&segment.text)?,
                    TtsLang::En => self.en.synthesize(&segment.text)?,
                };
                if result.sample_rate != self.sample_rate {
                    return Err(TtsError::Config(format!(
                        "Matcha 片段采样率不一致: expected={} got={}",
                        self.sample_rate, result.sample_rate
                    )));
                }
                if index > 0 && !samples.is_empty() {
                    samples.extend_from_slice(&pause);
                }
                samples.extend(result.samples);
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
            "matcha-bilingual"
        }
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
        pub dict_dir: Option<PathBuf>,
        pub lexicon: Option<String>,
        pub rule_fsts: Option<String>,
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
                dict_dir: Some(base.join("dict")),
                lexicon: None,
                rule_fsts: None,
                speaker_id: 0,
                speed: 1.0,
            }
        }
    }

    /// Kokoro TTS 引擎（通过 sherpa-onnx OfflineTts）
    pub struct KokoroTts {
        tts: Mutex<sherpa_onnx::OfflineTts>,
        sample_rate: u32,
        #[allow(dead_code)]
        config: KokoroConfig,
    }

    impl KokoroTts {
        pub fn new(config: KokoroConfig) -> Result<Self, TtsError> {
            use sherpa_onnx::{
                OfflineTts, OfflineTtsConfig, OfflineTtsKokoroModelConfig, OfflineTtsModelConfig,
            };

            let tts_config = OfflineTtsConfig {
                model: OfflineTtsModelConfig {
                    kokoro: OfflineTtsKokoroModelConfig {
                        model: Some(config.model_path.to_string_lossy().to_string()),
                        voices: Some(config.voices_path.to_string_lossy().to_string()),
                        tokens: Some(config.tokens_path.to_string_lossy().to_string()),
                        data_dir: Some(config.data_dir.to_string_lossy().to_string()),
                        dict_dir: config
                            .dict_dir
                            .as_ref()
                            .map(|path| path.to_string_lossy().to_string()),
                        lexicon: config.lexicon.clone(),
                        ..Default::default()
                    },
                    provider: Some("cpu".to_string()),
                    num_threads: 2,
                    ..Default::default()
                },
                rule_fsts: config.rule_fsts.clone(),
                max_num_sentences: 1,
                silence_scale: 0.2,
                ..Default::default()
            };

            let tts = OfflineTts::create(&tts_config)
                .ok_or_else(|| TtsError::Synthesis("Kokoro 初始化失败".to_string()))?;
            let sample_rate = tts.sample_rate() as u32;

            Ok(Self {
                tts: Mutex::new(tts),
                sample_rate,
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
            let generation_config = sherpa_onnx::GenerationConfig {
                sid: self.config.speaker_id,
                speed: self.config.speed,
                ..Default::default()
            };
            let audio = tts
                .generate_with_config::<fn(&[f32], f32) -> bool>(text, &generation_config, None)
                .ok_or_else(|| TtsError::Synthesis("Kokoro 合成失败".to_string()))?;

            Ok(TtsResult {
                samples: audio.samples().to_vec(),
                sample_rate: audio.sample_rate() as u32,
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
    #[cfg(feature = "sherpa-onnx")]
    {
        let _ = config; // 使用 config 避免未使用警告
        if !config.tts_model.eq_ignore_ascii_case("kokoro") {
            if let Some(engine) = create_matcha_bilingual_engine(config) {
                return engine;
            }
        }

        let model_dir = kokoro_dir(config);
        let kokoro_config = kokoro::KokoroConfig {
            model_path: model_dir.join("model.onnx"),
            voices_path: model_dir.join("voices.bin"),
            tokens_path: model_dir.join("tokens.txt"),
            data_dir: model_dir.join("espeak-ng-data"),
            dict_dir: existing_dir(&model_dir.join("dict")),
            lexicon: join_existing_files(&model_dir, &["lexicon-us-en.txt", "lexicon-zh.txt"]),
            rule_fsts: join_existing_files(
                &model_dir,
                &["date-zh.fst", "phone-zh.fst", "number-zh.fst"],
            ),
            speaker_id: config.tts_speaker_id,
            speed: config.tts_speed,
        };
        if let Ok(engine) = kokoro::KokoroTts::new(kokoro_config) {
            tracing::info!("Voice TTS: 使用 Kokoro 离线引擎 ({})", model_dir.display());
            return Box::new(engine);
        }
        tracing::warn!(
            "Voice TTS: Kokoro 初始化失败，回退到平台默认 TTS ({})",
            model_dir.display()
        );
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

#[cfg(feature = "sherpa-onnx")]
fn create_matcha_bilingual_engine(
    config: &crate::config::settings::VoiceConfig,
) -> Option<Box<dyn TtsEngine>> {
    let root = matcha_root(config);
    let zh_dir = root.join("matcha-icefall-zh-baker");
    let en_dir = root.join("matcha-icefall-en_US-ljspeech");
    let vocoder = root.join("vocos-22khz-univ.onnx");

    let required = [
        zh_dir.join("model-steps-3.onnx"),
        zh_dir.join("tokens.txt"),
        zh_dir.join("lexicon.txt"),
        en_dir.join("model-steps-3.onnx"),
        en_dir.join("tokens.txt"),
        en_dir.join("espeak-ng-data"),
        vocoder.clone(),
    ];
    if !required.iter().all(|path| path.exists()) {
        tracing::warn!(
            "Voice TTS: Matcha 双语模型未完整安装，跳过 ({})",
            root.display()
        );
        return None;
    }

    let zh = matcha::MatchaConfig {
        acoustic_model: zh_dir.join("model-steps-3.onnx"),
        vocoder: vocoder.clone(),
        tokens: zh_dir.join("tokens.txt"),
        data_dir: None,
        dict_dir: existing_dir(&zh_dir.join("dict")),
        lexicon: Some(zh_dir.join("lexicon.txt").to_string_lossy().to_string()),
        rule_fsts: join_existing_files(
            &zh_dir,
            &["phone.fst", "date.fst", "number.fst", "new_heteronym.fst"],
        ),
        speed: config.tts_speed,
        name: "matcha-zh-baker",
    };
    let en = matcha::MatchaConfig {
        acoustic_model: en_dir.join("model-steps-3.onnx"),
        vocoder,
        tokens: en_dir.join("tokens.txt"),
        data_dir: existing_dir(&en_dir.join("espeak-ng-data")),
        dict_dir: None,
        lexicon: None,
        rule_fsts: None,
        speed: config.tts_speed,
        name: "matcha-en-ljspeech",
    };

    match matcha::MatchaBilingualTts::new(zh, en) {
        Ok(engine) => {
            tracing::info!("Voice TTS: 使用 Matcha 双语低延迟引擎 ({})", root.display());
            Some(Box::new(engine))
        }
        Err(err) => {
            tracing::warn!("Voice TTS: Matcha 双语初始化失败，继续尝试 Kokoro: {}", err);
            None
        }
    }
}

#[cfg(feature = "sherpa-onnx")]
fn matcha_root(config: &crate::config::settings::VoiceConfig) -> std::path::PathBuf {
    let value = config.tts_model.trim();
    if !value.is_empty()
        && !value.eq_ignore_ascii_case("matcha")
        && !value.eq_ignore_ascii_case("matcha-bilingual")
        && !value.eq_ignore_ascii_case("kokoro")
    {
        let path = std::path::Path::new(value);
        if path.is_dir() {
            if path.join("matcha-icefall-zh-baker").is_dir() {
                return path.to_path_buf();
            }
            if let Some(parent) = path.parent() {
                if parent.join("matcha-icefall-zh-baker").is_dir() {
                    return parent.to_path_buf();
                }
            }
        }
    }
    crate::config::settings::models_dir().join("matcha")
}

/// 获取模型目录（优先环境变量，否则 ~/.fairyfield/models/）
#[cfg(feature = "sherpa-onnx")]
fn kokoro_dir(config: &crate::config::settings::VoiceConfig) -> std::path::PathBuf {
    if !config.tts_model.is_empty() {
        let path = std::path::Path::new(&config.tts_model);
        if path.is_file() {
            if let Some(parent) = path.parent() {
                return parent.to_path_buf();
            }
        } else if path.is_dir() {
            if path.join("model.onnx").is_file() {
                return path.to_path_buf();
            }
            let nested = path.join("kokoro");
            if nested.is_dir() {
                return nested;
            }
        } else if path.is_absolute() && path.extension().is_some() {
            if let Some(parent) = path.parent() {
                return parent.to_path_buf();
            }
        }
    }
    crate::config::settings::models_dir().join("kokoro")
}

#[cfg(feature = "sherpa-onnx")]
fn existing_dir(path: &std::path::Path) -> Option<std::path::PathBuf> {
    path.is_dir().then(|| path.to_path_buf())
}

#[cfg(feature = "sherpa-onnx")]
fn join_existing_files(model_dir: &std::path::Path, names: &[&str]) -> Option<String> {
    let files: Vec<String> = names
        .iter()
        .map(|name| model_dir.join(name))
        .filter(|path| path.is_file())
        .map(|path| path.to_string_lossy().to_string())
        .collect();
    (!files.is_empty()).then(|| files.join(","))
}

#[cfg(feature = "sherpa-onnx")]
fn tts_thread_count() -> i32 {
    std::thread::available_parallelism()
        .map(|n| n.get().clamp(1, 4) as i32)
        .unwrap_or(2)
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
        let clean_text = clean_tts_text(text);
        if clean_text.is_empty() {
            return Ok(TtsResult {
                samples: vec![],
                sample_rate: self.engine.sample_rate(),
            });
        }

        let chunks = split_tts_utterances(&clean_text);
        let mut all_samples = Vec::new();
        let mut sample_rate = self.engine.sample_rate();
        for chunk in chunks {
            let result = self.engine.synthesize(&chunk)?;
            sample_rate = result.sample_rate;
            if !result.samples.is_empty() {
                self.audio
                    .play(result.samples.clone(), result.sample_rate)?;
                all_samples.extend(result.samples);
            }
        }
        Ok(TtsResult {
            samples: all_samples,
            sample_rate,
        })
    }

    /// 仅合成，不播放（用于获取 PCM 数据发送给前端做口型同步）
    pub fn synthesize_only(&self, text: &str) -> Result<TtsResult, TtsError> {
        let clean_text = clean_tts_text(text);
        if clean_text.is_empty() {
            return Ok(TtsResult {
                samples: vec![],
                sample_rate: self.engine.sample_rate(),
            });
        }
        self.engine.synthesize(&clean_text)
    }

    /// 获取采样率
    pub fn sample_rate(&self) -> u32 {
        self.engine.sample_rate()
    }

    /// 当前 TTS 引擎名称，用于启动日志和诊断。
    pub fn engine_name(&self) -> &str {
        self.engine.name()
    }

    /// 停止播放
    pub fn stop(&self) {
        self.audio.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::{TtsEngine, TtsError, TtsManager, TtsResult};
    use std::sync::{Arc, Mutex};

    struct RecordingTts {
        calls: Arc<Mutex<Vec<String>>>,
    }

    impl TtsEngine for RecordingTts {
        fn synthesize(&self, text: &str) -> Result<TtsResult, TtsError> {
            self.calls.lock().unwrap().push(text.to_string());
            Ok(TtsResult {
                samples: vec![],
                sample_rate: 22050,
            })
        }

        fn sample_rate(&self) -> u32 {
            22050
        }

        fn name(&self) -> &str {
            "recording"
        }
    }

    #[test]
    fn manager_sanitizes_text_for_every_engine() {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let manager = TtsManager::new(Box::new(RecordingTts {
            calls: Arc::clone(&calls),
        }))
        .expect("create manager");

        manager
            .speak("[emotion:happy] **你好** 😊\n```rust\nprintln!(\"secret\");\n```\nhttps://example.com")
            .expect("speak");

        let calls = calls.lock().unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0], "你好");
    }

    #[test]
    fn split_tts_utterances_keeps_first_audio_chunk_short() {
        let chunks = super::split_tts_utterances(
            "第一句很短。第二句也很短！This one is short. 这一句后面没有那么短但是应该仍然可以单独合成。",
        );
        assert!(chunks.len() >= 3);
        assert_eq!(chunks[0], "第一句很短。");
        assert!(chunks[0].chars().count() < 16);
    }

    #[test]
    fn split_language_segments_routes_mixed_chinese_and_english() {
        let segments = super::split_language_segments("你好 Natasha, how are you? 今天 OK.");
        let langs: Vec<_> = segments.iter().map(|segment| segment.lang).collect();
        assert_eq!(
            langs,
            vec![
                super::TtsLang::Zh,
                super::TtsLang::En,
                super::TtsLang::Zh,
                super::TtsLang::En
            ]
        );
        assert_eq!(segments[0].text.trim(), "你好");
        assert!(segments[1].text.contains("Natasha"));
        assert_eq!(segments[2].text.trim(), "今天");
        assert!(segments[3].text.contains("OK"));
    }

    #[cfg(feature = "sherpa-onnx")]
    #[test]
    fn create_tts_engine_prefers_kokoro_when_models_are_installed() {
        let model_dir = crate::config::settings::models_dir().join("kokoro");
        let required = [
            model_dir.join("model.onnx"),
            model_dir.join("voices.bin"),
            model_dir.join("tokens.txt"),
            model_dir.join("espeak-ng-data"),
        ];
        if !required.iter().all(|path| path.exists()) {
            eprintln!(
                "跳过 Kokoro 引擎测试：模型目录未完整安装 ({})",
                model_dir.display()
            );
            return;
        }

        let mut config = crate::config::settings::default_config().voice;
        config.tts_model = String::new();
        config.tts_speaker_id = 47;

        let engine = super::create_tts_engine(&config);
        assert_eq!(engine.name(), "kokoro");

        let result = engine.synthesize("你好").expect("Kokoro 应能合成中文语音");
        assert!(!result.samples.is_empty());
        assert!(result.sample_rate > 0);
    }

    #[cfg(feature = "sherpa-onnx")]
    #[test]
    fn create_tts_engine_prefers_matcha_bilingual_when_models_are_installed() {
        let root = crate::config::settings::models_dir().join("matcha");
        let required = [
            root.join("matcha-icefall-zh-baker/model-steps-3.onnx"),
            root.join("matcha-icefall-zh-baker/tokens.txt"),
            root.join("matcha-icefall-zh-baker/lexicon.txt"),
            root.join("matcha-icefall-en_US-ljspeech/model-steps-3.onnx"),
            root.join("matcha-icefall-en_US-ljspeech/tokens.txt"),
            root.join("matcha-icefall-en_US-ljspeech/espeak-ng-data"),
            root.join("vocos-22khz-univ.onnx"),
        ];
        if !required.iter().all(|path| path.exists()) {
            eprintln!(
                "跳过 Matcha 双语引擎测试：模型目录未完整安装 ({})",
                root.display()
            );
            return;
        }

        let mut config = crate::config::settings::default_config().voice;
        config.tts_model = "matcha".to_string();

        let engine = super::create_tts_engine(&config);
        assert_eq!(engine.name(), "matcha-bilingual");

        let result = engine
            .synthesize("你好 Natasha, welcome back.")
            .expect("Matcha 应能合成中英混合语音");
        assert!(!result.samples.is_empty());
        assert!(result.sample_rate > 0);
    }
}
