//! SSE 流式响应模块
//!
//! 处理 LLM 的流式输出，支持逐 token 返回。

/// 流式响应处理器
///
/// Phase 3 实现：管理 SSE 连接，逐 token 解析 LLM 响应，
/// 通过 Tauri 事件系统推送到前端。
#[allow(dead_code)]
pub struct StreamHandler {
    // TODO: SSE 解析器
    // TODO: 事件发送通道
}
