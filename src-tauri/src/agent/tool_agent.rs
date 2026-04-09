//! 工具 Agent 模块
//!
//! 专门执行工具调用的子 Agent，受安全边界约束。
//!
//! 当前仅作为类型占位，实际工具执行逻辑在 tools/executor.rs 中。

/// 工具 Agent
pub struct ToolAgent {
    /// 可用工具列表
    #[allow(dead_code)]
    tools: Vec<String>,
}

impl ToolAgent {
    pub fn new(tools: Vec<String>) -> Self {
        Self { tools }
    }

    pub fn available_tools(&self) -> &[String] {
        &self.tools
    }
}
