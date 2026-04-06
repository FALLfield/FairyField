//! 语音活动检测模块 (VAD)
//!
//! 基于 sherpa-onnx 的语音活动检测（silero-vad 模型）。

/// VAD 引擎
///
/// Phase 2 实现：使用 sherpa-onnx silero-vad 模型
/// 检测用户是否在说话，用于自动触发 ASR。
#[allow(dead_code)]
pub struct VadEngine {
    // TODO: sherpa-onnx VAD 实例
    // TODO: 灵敏度阈值
}
