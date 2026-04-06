//! 工具 Agent 模块
//!
//! 专门执行工具调用的子 Agent，受安全边界约束。

/// 工具执行 Agent
///
/// Phase 3 实现：专门负责执行工具调用（文件操作、Git、Shell 等），
/// 在严格的安全沙箱内运行。
#[allow(dead_code)]
pub struct ToolAgent {
    // TODO: 可用工具列表
    // TODO: 安全策略
}
