//! 麦克风音频捕获模块
//!
//! 使用 cpal 从默认输入设备捕获音频，提供线程安全的环形缓冲区。

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample, SampleFormat, SizedSample};
use std::sync::{Arc, Mutex};

/// 音频输入错误
#[derive(Debug, thiserror::Error)]
pub enum AudioInputError {
    #[error("未找到麦克风设备")]
    NoDevice,
    #[error("音频配置错误: {0}")]
    Config(String),
    #[error("音频流错误: {0}")]
    Stream(String),
    #[error("录音太短或没有检测到有效语音")]
    NoSpeech,
}

/// 麦克风音频输入
///
/// 创建一个后台音频输入流，持续将捕获的音频数据追加到内部缓冲区。
/// 调用 `drain()` 取出累积的音频样本。
pub struct AudioInput {
    #[allow(dead_code)]
    stream: Option<cpal::Stream>,
    buffer: Arc<Mutex<Vec<f32>>>,
    sample_rate: u32,
}

pub struct PreparedAsrCapture {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
}

impl AudioInput {
    /// 创建新的音频输入，从默认麦克风开始捕获
    pub fn new() -> Result<Self, AudioInputError> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or(AudioInputError::NoDevice)?;
        let supported_config = device
            .default_input_config()
            .map_err(|e| AudioInputError::Config(e.to_string()))?;
        let sample_format = supported_config.sample_format();
        let sample_rate = supported_config.sample_rate().0;
        let channels = supported_config.channels().max(1) as usize;
        let config: cpal::StreamConfig = supported_config.into();

        let buffer = Arc::new(Mutex::new(Vec::new()));
        let stream = build_mono_input_stream(
            &device,
            &config,
            sample_format,
            channels,
            sample_rate,
            buffer.clone(),
        )?;

        stream
            .play()
            .map_err(|e| AudioInputError::Stream(e.to_string()))?;

        Ok(Self {
            stream: Some(stream),
            buffer,
            sample_rate,
        })
    }

    /// 取出所有累积的音频样本（清空缓冲区）
    pub fn drain(&self) -> Vec<f32> {
        if let Ok(mut buf) = self.buffer.lock() {
            std::mem::take(&mut *buf)
        } else {
            Vec::new()
        }
    }

    /// 采样率
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// 停止音频捕获
    pub fn stop(&mut self) {
        self.stream = None;
    }
}

fn build_mono_input_stream(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    sample_format: SampleFormat,
    channels: usize,
    sample_rate: u32,
    buffer: Arc<Mutex<Vec<f32>>>,
) -> Result<cpal::Stream, AudioInputError> {
    match sample_format {
        SampleFormat::F32 => {
            build_typed_input_stream::<f32>(device, config, channels, sample_rate, buffer)
        }
        SampleFormat::F64 => {
            build_typed_input_stream::<f64>(device, config, channels, sample_rate, buffer)
        }
        SampleFormat::I8 => {
            build_typed_input_stream::<i8>(device, config, channels, sample_rate, buffer)
        }
        SampleFormat::I16 => {
            build_typed_input_stream::<i16>(device, config, channels, sample_rate, buffer)
        }
        SampleFormat::I32 => {
            build_typed_input_stream::<i32>(device, config, channels, sample_rate, buffer)
        }
        SampleFormat::I64 => {
            build_typed_input_stream::<i64>(device, config, channels, sample_rate, buffer)
        }
        SampleFormat::U8 => {
            build_typed_input_stream::<u8>(device, config, channels, sample_rate, buffer)
        }
        SampleFormat::U16 => {
            build_typed_input_stream::<u16>(device, config, channels, sample_rate, buffer)
        }
        SampleFormat::U32 => {
            build_typed_input_stream::<u32>(device, config, channels, sample_rate, buffer)
        }
        SampleFormat::U64 => {
            build_typed_input_stream::<u64>(device, config, channels, sample_rate, buffer)
        }
        other => Err(AudioInputError::Config(format!(
            "不支持的麦克风采样格式: {other:?}"
        ))),
    }
}

fn build_typed_input_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    channels: usize,
    sample_rate: u32,
    buffer: Arc<Mutex<Vec<f32>>>,
) -> Result<cpal::Stream, AudioInputError>
where
    T: Sample + SizedSample + Send + 'static,
    f32: FromSample<T>,
{
    device
        .build_input_stream(
            config,
            move |data: &[T], _: &cpal::InputCallbackInfo| {
                if let Ok(mut buf) = buffer.lock() {
                    if buf.len() >= sample_rate as usize * 60 {
                        return;
                    }
                    append_mono_frames(&mut buf, data, channels);
                }
            },
            |err| eprintln!("音频输入错误: {}", err),
            None,
        )
        .map_err(|e| AudioInputError::Stream(e.to_string()))
}

fn append_mono_frames<T>(out: &mut Vec<f32>, input: &[T], channels: usize)
where
    T: Sample + Copy,
    f32: FromSample<T>,
{
    let channels = channels.max(1);
    for frame in input.chunks(channels) {
        let sum: f32 = frame.iter().copied().map(f32::from_sample).sum();
        out.push(sum / frame.len().max(1) as f32);
    }
}

pub fn validate_asr_capture(samples: &[f32], sample_rate: u32) -> Result<(), AudioInputError> {
    let min_samples = (sample_rate as f32 * 0.25) as usize;
    if samples.len() < min_samples {
        return Err(AudioInputError::NoSpeech);
    }

    let rms =
        (samples.iter().map(|sample| sample * sample).sum::<f32>() / samples.len() as f32).sqrt();
    if rms < 0.003 {
        return Err(AudioInputError::NoSpeech);
    }
    Ok(())
}

pub fn prepare_asr_samples(
    samples: &[f32],
    sample_rate: u32,
    target_sample_rate: u32,
) -> Result<PreparedAsrCapture, AudioInputError> {
    if sample_rate == 0 || target_sample_rate == 0 {
        return Err(AudioInputError::Config("采样率不能为 0".into()));
    }

    let prepared = if sample_rate == target_sample_rate {
        samples.to_vec()
    } else {
        resample_linear(samples, sample_rate, target_sample_rate)
    };
    validate_asr_capture(&prepared, target_sample_rate)?;
    Ok(PreparedAsrCapture {
        samples: prepared,
        sample_rate: target_sample_rate,
    })
}

fn resample_linear(samples: &[f32], source_rate: u32, target_rate: u32) -> Vec<f32> {
    if samples.is_empty() {
        return Vec::new();
    }

    let output_len = ((samples.len() as f64 * target_rate as f64) / source_rate as f64)
        .round()
        .max(1.0) as usize;
    let step = source_rate as f64 / target_rate as f64;
    let mut output = Vec::with_capacity(output_len);

    for index in 0..output_len {
        let pos = index as f64 * step;
        let left = pos.floor() as usize;
        if left + 1 >= samples.len() {
            output.push(*samples.last().unwrap_or(&0.0));
            continue;
        }
        let frac = (pos - left as f64) as f32;
        let a = samples[left];
        let b = samples[left + 1];
        output.push(a + (b - a) * frac);
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn downmixes_interleaved_stereo_to_mono() {
        let mut out = Vec::new();
        append_mono_frames(&mut out, &[0.5f32, -0.5, 0.25, 0.75], 2);
        assert_eq!(out, vec![0.0, 0.5]);
    }

    #[test]
    fn rejects_empty_short_and_silent_asr_capture() {
        assert!(validate_asr_capture(&[], 16_000).is_err());
        assert!(validate_asr_capture(&vec![0.0; 16_000], 16_000).is_err());
        assert!(validate_asr_capture(&vec![0.001; 16_000], 16_000).is_err());
    }

    #[test]
    fn accepts_non_silent_asr_capture() {
        let samples = (0..16_000)
            .map(|index| if index % 2 == 0 { 0.03 } else { -0.03 })
            .collect::<Vec<_>>();
        assert!(validate_asr_capture(&samples, 16_000).is_ok());
    }

    #[test]
    fn prepares_asr_samples_at_target_sample_rate() {
        let samples = (0..48_000)
            .map(|index| index as f32 / 48_000.0)
            .collect::<Vec<_>>();
        let prepared = prepare_asr_samples(&samples, 48_000, 16_000).unwrap();

        assert_eq!(prepared.sample_rate, 16_000);
        assert_eq!(prepared.samples.len(), 16_000);
        assert!((prepared.samples[8_000] - 0.5).abs() < 0.01);
    }
}
