//! 音频输出模块 — 使用 cpal 播放 PCM 数据到扬声器
//!
//! 音频播放在独立线程中执行，避免 cpal::Stream 的 !Send 问题。

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream, StreamConfig};
use std::sync::mpsc::{self, Receiver, Sender};

/// 音频播放错误
#[derive(Debug, thiserror::Error)]
pub enum AudioError {
    #[error("找不到音频输出设备")]
    NoDevice,
    #[error("音频流创建失败: {0}")]
    StreamCreate(String),
    #[error("音频播放失败: {0}")]
    Playback(String),
}

/// 音频播放命令（发送到音频线程）
enum AudioCommand {
    /// 播放 PCM 数据
    Play { samples: Vec<f32>, sample_rate: u32 },
    /// 停止播放
    Stop,
    /// 关闭音频线程
    Shutdown,
}

/// 音频播放结果（从音频线程返回）
enum AudioResult {
    /// 播放完成
    Done,
    /// 播放出错
    Error(String),
}

/// 音频输出器 — 通过后台线程播放 PCM 音频
///
/// cpal::Stream 不是 Send，所以不能跨线程传递。
/// 解决方案：在独立的音频线程中创建和管理 Stream，
/// 主线程通过 channel 发送播放命令。
pub struct AudioOutput {
    cmd_tx: Sender<AudioCommand>,
    result_rx: Receiver<AudioResult>,
}

impl AudioOutput {
    /// 创建新的音频输出（启动后台音频线程）
    pub fn new() -> Result<Self, AudioError> {
        let (cmd_tx, cmd_rx) = mpsc::channel();
        let (result_tx, result_rx) = mpsc::channel();

        std::thread::Builder::new()
            .name("fairy-audio".to_string())
            .spawn(move || {
                audio_thread(cmd_rx, result_tx);
            })
            .map_err(|e| AudioError::StreamCreate(format!("创建音频线程失败: {}", e)))?;

        Ok(Self { cmd_tx, result_rx })
    }

    /// 播放 f32 PCM 采样数据（阻塞直到播放完成）
    pub fn play(&self, samples: Vec<f32>, sample_rate: u32) -> Result<(), AudioError> {
        if samples.is_empty() {
            return Ok(());
        }

        self.cmd_tx
            .send(AudioCommand::Play {
                samples,
                sample_rate,
            })
            .map_err(|e| AudioError::Playback(format!("发送播放命令失败: {}", e)))?;

        // 等待播放完成
        match self
            .result_rx
            .recv_timeout(std::time::Duration::from_secs(30))
        {
            Ok(AudioResult::Done) => Ok(()),
            Ok(AudioResult::Error(msg)) => Err(AudioError::Playback(msg)),
            Err(_) => Err(AudioError::Playback("播放超时".to_string())),
        }
    }

    /// 停止当前播放
    pub fn stop(&self) {
        let _ = self.cmd_tx.send(AudioCommand::Stop);
        // 消费掉可能的结果
        let _ = self
            .result_rx
            .recv_timeout(std::time::Duration::from_secs(1));
    }
}

impl Drop for AudioOutput {
    fn drop(&mut self) {
        let _ = self.cmd_tx.send(AudioCommand::Shutdown);
    }
}

/// 音频线程主循环
fn audio_thread(cmd_rx: Receiver<AudioCommand>, result_tx: Sender<AudioResult>) {
    let mut current_stream: Option<Stream> = None;

    loop {
        match cmd_rx.recv() {
            Ok(AudioCommand::Play {
                samples,
                sample_rate,
            }) => {
                // 停止之前的播放
                current_stream = None;

                match play_samples(&samples, sample_rate) {
                    Ok(stream) => {
                        current_stream = Some(stream);
                        let _ = result_tx.send(AudioResult::Done);
                    }
                    Err(e) => {
                        let _ = result_tx.send(AudioResult::Error(e.to_string()));
                    }
                }
            }
            Ok(AudioCommand::Stop) => {
                current_stream = None;
            }
            Ok(AudioCommand::Shutdown) | Err(_) => {
                break;
            }
        }
    }
}

/// 在当前线程播放 PCM 采样（返回 Stream 保持播放）
fn play_samples(samples: &[f32], sample_rate: u32) -> Result<Stream, AudioError> {
    let host = cpal::default_host();
    let device = host.default_output_device().ok_or(AudioError::NoDevice)?;

    let supported_config = device
        .default_output_config()
        .map_err(|e| AudioError::StreamCreate(format!("获取配置失败: {}", e)))?;

    let sample_format = supported_config.sample_format();
    let config = StreamConfig {
        channels: 1,
        sample_rate: cpal::SampleRate(sample_rate),
        buffer_size: cpal::BufferSize::Default,
    };

    let buffer: std::sync::Arc<std::sync::Mutex<std::collections::VecDeque<f32>>> =
        std::sync::Arc::new(std::sync::Mutex::new(samples.iter().copied().collect()));
    let buffer_clone = buffer.clone();

    let stream = match sample_format {
        SampleFormat::F32 => build_stream::<f32>(&device, &config, buffer_clone),
        SampleFormat::I16 => build_stream::<i16>(&device, &config, buffer_clone),
        SampleFormat::U16 => build_stream::<u16>(&device, &config, buffer_clone),
        _ => {
            return Err(AudioError::StreamCreate(format!(
                "不支持的采样格式: {:?}",
                sample_format
            )))
        }
    }?;

    stream
        .play()
        .map_err(|e| AudioError::Playback(format!("启动播放失败: {}", e)))?;

    // 等待所有采样播放完毕
    let timeout = std::time::Duration::from_secs(30);
    let start = std::time::Instant::now();
    loop {
        let remaining = {
            let b = buffer.lock().unwrap();
            b.len()
        };
        if remaining == 0 {
            break;
        }
        if start.elapsed() > timeout {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }

    Ok(stream)
}

fn build_stream<T>(
    device: &cpal::Device,
    config: &StreamConfig,
    buffer: std::sync::Arc<std::sync::Mutex<std::collections::VecDeque<f32>>>,
) -> Result<Stream, AudioError>
where
    T: cpal::Sample + cpal::FromSample<f32> + cpal::SizedSample,
{
    let err_fn = |err: cpal::StreamError| {
        eprintln!("音频流错误: {}", err);
    };

    device
        .build_output_stream(
            config,
            move |output: &mut [T], _: &cpal::OutputCallbackInfo| {
                let mut buf = buffer.lock().unwrap();
                for sample in output.iter_mut() {
                    let value = buf.pop_front().unwrap_or(0.0);
                    *sample = T::from_sample(value);
                }
            },
            err_fn,
            None,
        )
        .map_err(|e| AudioError::StreamCreate(format!("创建流失败: {}", e)))
}
