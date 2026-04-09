//! 语音活动检测模块 (VAD)
//!
//! 基于 silero-vad 模型的语音活动检测。
//! 当前提供基于能量阈值的 mock 实现。

use super::VoiceError;

/// VAD 引擎 trait
pub trait VadEngine: Send + Sync {
    /// 处理一段音频 PCM 数据，返回是否检测到语音
    fn process(&self, pcm: &[f32], sample_rate: u32) -> Result<bool, VoiceError>;
    /// 重置内部状态
    fn reset(&self);
    /// 是否正在说话
    fn is_speaking(&self) -> bool;
}

/// 基于 silero-vad 的 VAD 引擎（当前为能量阈值 mock）
pub struct SileroVad {
    /// 灵敏度阈值（RMS），默认 0.02
    threshold: std::sync::Mutex<f32>,
    /// 是否正在说话
    speaking: std::sync::Mutex<bool>,
    /// 平滑计数器，避免频繁切换
    frame_count: std::sync::Mutex<u32>,
}

impl SileroVad {
    pub fn new(threshold: f32) -> Result<Self, VoiceError> {
        if threshold < 0.0 || threshold > 1.0 {
            return Err(VoiceError::AudioError(format!(
                "阈值必须在 0-1 之间，当前: {threshold}"
            )));
        }
        Ok(Self {
            threshold: std::sync::Mutex::new(threshold),
            speaking: std::sync::Mutex::new(false),
            frame_count: std::sync::Mutex::new(0),
        })
    }

    fn compute_rms(pcm: &[f32]) -> f32 {
        if pcm.is_empty() {
            return 0.0;
        }
        let sum: f32 = pcm.iter().map(|s| s * s).sum();
        (sum / pcm.len() as f32).sqrt()
    }
}

impl VadEngine for SileroVad {
    fn process(&self, pcm: &[f32], _sample_rate: u32) -> Result<bool, VoiceError> {
        let rms = Self::compute_rms(pcm);
        let threshold = *self.threshold.lock().unwrap();

        let is_voice = rms > threshold;
        let mut count = self.frame_count.lock().unwrap();
        *count += 1;

        if *count >= 3 {
            *self.speaking.lock().unwrap() = is_voice;
            *count = 0;
        }

        Ok(*self.speaking.lock().unwrap())
    }

    fn reset(&self) {
        *self.speaking.lock().unwrap() = false;
        *self.frame_count.lock().unwrap() = 0;
    }

    fn is_speaking(&self) -> bool {
        *self.speaking.lock().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_silero_vad_new() {
        let vad = SileroVad::new(0.02).unwrap();
        assert!(!vad.is_speaking());
    }

    #[test]
    fn test_silero_vad_invalid_threshold() {
        let result = SileroVad::new(-0.1);
        assert!(result.is_err());
    }

    #[test]
    fn test_silero_vad_silence() {
        let vad = SileroVad::new(0.02).unwrap();
        let silence = vec![0.0f32; 1600]; // 100ms 静音
        // 需要累积多帧
        for _ in 0..5 {
            let _ = vad.process(&silence, 16000);
        }
        assert!(!vad.is_speaking());
    }

    #[test]
    fn test_silero_vad_voice() {
        let vad = SileroVad::new(0.01).unwrap();
        // 生成正弦波（模拟语音）
        let voice: Vec<f32> = (0..1600)
            .map(|i| (i as f32 * 2.0 * std::f32::consts::PI * 440.0 / 16000.0).sin() * 0.5)
            .collect();
        for _ in 0..5 {
            let _ = vad.process(&voice, 16000);
        }
        assert!(vad.is_speaking());
    }

    #[test]
    fn test_silero_vad_reset() {
        let vad = SileroVad::new(0.01).unwrap();
        let voice: Vec<f32> = (0..1600)
            .map(|i| (i as f32 * 2.0 * std::f32::consts::PI * 440.0 / 16000.0).sin() * 0.5)
            .collect();
        for _ in 0..5 {
            let _ = vad.process(&voice, 16000);
        }
        vad.reset();
        assert!(!vad.is_speaking());
    }

    #[test]
    fn test_compute_rms() {
        let silence = vec![0.0f32; 100];
        assert_eq!(SileroVad::compute_rms(&silence), 0.0);

        let signal = vec![1.0f32; 100];
        assert!((SileroVad::compute_rms(&signal) - 1.0).abs() < 0.001);
    }
}
