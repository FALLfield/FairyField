//! 工具 Agent 模块
//!
//! 专门执行工具调用的子 Agent，受安全边界约束。

use serde::{Deserialize, Serialize};

/// 工具调用请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub tool_name: String,
    pub arguments: serde_json::Value,
}

/// 工具调用结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_name: String,
    pub success: bool,
    pub output: String,
    pub execution_time_ms: u64,
}

/// 工具 Agent 执行计划
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub calls: Vec<ToolCall>,
    pub max_parallel: usize,
}

/// 工具 Agent
pub struct ToolAgent {
    available_tools: Vec<String>,
    max_execution_time: u64,
}

impl ToolAgent {
    pub fn new(available_tools: Vec<String>) -> Self {
        Self {
            available_tools,
            max_execution_time: 30,
        }
    }

    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.max_execution_time = seconds;
        self
    }

    pub fn available_tools(&self) -> &[String] {
        &self.available_tools
    }
    pub fn max_execution_time(&self) -> u64 {
        self.max_execution_time
    }

    /// 验证工具调用是否在允许列表中
    pub fn validate_call(&self, call: &ToolCall) -> Result<(), String> {
        if !self.available_tools.contains(&call.tool_name) {
            return Err(format!("tool '{}' not in available tools", call.tool_name));
        }
        Ok(())
    }

    /// 创建执行计划（顺序执行）
    pub fn plan(&self, calls: Vec<ToolCall>) -> ExecutionPlan {
        ExecutionPlan {
            calls,
            max_parallel: 1,
        }
    }

    /// 创建安全执行计划（过滤无效调用）
    pub fn safe_plan(&self, calls: Vec<ToolCall>) -> ExecutionPlan {
        let valid: Vec<ToolCall> = calls
            .into_iter()
            .filter(|c| self.validate_call(c).is_ok())
            .collect();
        ExecutionPlan {
            calls: valid,
            max_parallel: 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sample_tools() -> Vec<String> {
        vec!["web_search".into(), "read_file".into(), "write_file".into()]
    }

    #[test]
    fn validate_known_tool() {
        let agent = ToolAgent::new(sample_tools());
        assert!(agent
            .validate_call(&ToolCall {
                tool_name: "web_search".into(),
                arguments: json!({})
            })
            .is_ok());
    }

    #[test]
    fn validate_unknown_tool() {
        let agent = ToolAgent::new(sample_tools());
        assert!(agent
            .validate_call(&ToolCall {
                tool_name: "delete_everything".into(),
                arguments: json!({})
            })
            .is_err());
    }

    #[test]
    fn safe_plan_filters_invalid() {
        let agent = ToolAgent::new(sample_tools());
        let plan = agent.safe_plan(vec![
            ToolCall {
                tool_name: "web_search".into(),
                arguments: json!({}),
            },
            ToolCall {
                tool_name: "dangerous".into(),
                arguments: json!({}),
            },
            ToolCall {
                tool_name: "read_file".into(),
                arguments: json!({}),
            },
        ]);
        assert_eq!(plan.calls.len(), 2);
    }
}
