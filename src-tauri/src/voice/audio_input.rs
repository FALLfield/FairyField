//! 麦克风音频捕获模块
//!
//! 使用 cpal 从默认输入设备捕获音频，提供线程安全的环形缓冲区。

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
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

impl AudioInput {
    /// 创建新的音频输入，从默认麦克风开始捕获
    pub fn new() -> Result<Self, AudioInputError> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or(AudioInputError::NoDevice)?;
        let config = device
            .default_input_config()
            .map_err(|e| AudioInputError::Config(e.to_string()))?;

        let sample_rate = config.sample_rate().0;
        let buffer = Arc::new(Mutex::new(Vec::new()));
        let buf_clone = buffer.clone();

        let stream = device
            .build_input_stream(
                &cpal::StreamConfig {
                    channels: 1,
                    sample_rate: cpal::SampleRate(sample_rate),
                    buffer_size: cpal::BufferSize::Default,
                },
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    if let Ok(mut buf) = buf_clone.lock() {
                        // 限制缓冲区大小防止内存泄漏（最多保留 60 秒 16kHz 单声道音频）
                        if buf.len() < (sample_rate as usize * 60) {
                            buf.extend_from_slice(data);
                        }
                    }
                },
                |err| eprintln!("音频输入错误: {}", err),
                None,
            )
            .map_err(|e| AudioInputError::Stream(e.to_string()))?;

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
