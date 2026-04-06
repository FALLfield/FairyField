//! LLM 提供商抽象模块
//!
//! 统一 OpenAI / Claude 等不同 LLM 提供商的接口。

/// LLM 提供商 trait
///
/// Phase 3 实现：定义统一的 LLM 调用接口，
/// 支持不同提供商的适配（OpenAI、Claude 等）。
#[allow(dead_code)]
pub struct LlmProvider {
    // TODO: 提供商配置
    // TODO: HTTP 客户端
}
