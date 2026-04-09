//! 工具执行器模块
//!
//! 管理工具注册、选择和执行的生命周期。

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

/// 工具执行错误
#[derive(Debug)]
pub enum ToolError {
    NotFound(String),
    ValidationFailed(String),
    ExecutionFailed(String),
    Timeout(String),
}

impl std::fmt::Display for ToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToolError::NotFound(name) => write!(f, "工具未找到: {name}"),
            ToolError::ValidationFailed(msg) => write!(f, "参数验证失败: {msg}"),
            ToolError::ExecutionFailed(msg) => write!(f, "执行失败: {msg}"),
            ToolError::Timeout(name) => write!(f, "工具执行超时: {name}"),
        }
    }
}

impl std::error::Error for ToolError {}

/// 工具 trait
pub trait Tool: Send + Sync {
    /// 工具名称
    fn name(&self) -> &str;
    /// 工具描述
    fn description(&self) -> &str;
    /// 验证输入
    fn validate_input(&self, input: &str) -> bool;
    /// 执行工具
    fn execute(
        &self,
        input: &str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<String, ToolError>> + Send + '_>,
    >;
}

/// 工具执行器
pub struct ToolExecutor {
    tools: std::sync::Mutex<HashMap<String, Arc<dyn Tool>>>,
    timeout: Duration,
}

impl ToolExecutor {
    pub fn new() -> Self {
        Self {
            tools: std::sync::Mutex::new(HashMap::new()),
            timeout: Duration::from_secs(30),
        }
    }

    pub fn with_timeout(timeout: Duration) -> Self {
        Self {
            tools: std::sync::Mutex::new(HashMap::new()),
            timeout,
        }
    }

    /// 注册工具
    pub fn register_tool(&self, tool: Arc<dyn Tool>) {
        let name = tool.name().to_string();
        self.tools.lock().unwrap().insert(name, tool);
    }

    /// 执行工具
    pub async fn execute(&self, name: &str, input: &str) -> Result<String, ToolError> {
        let tools = self.tools.lock().unwrap();
        let tool = tools
            .get(name)
            .ok_or_else(|| ToolError::NotFound(name.to_string()))?
            .clone();
        drop(tools);

        if !tool.validate_input(input) {
            return Err(ToolError::ValidationFailed(format!(
                "输入验证失败: {input}"
            )));
        }

        tokio::time::timeout(self.timeout, tool.execute(input))
            .await
            .map_err(|_| ToolError::Timeout(name.to_string()))?
    }

    /// 列出所有已注册工具
    pub fn list_tools(&self) -> Vec<(String, String)> {
        let tools = self.tools.lock().unwrap();
        tools.values().map(|t| (t.name().to_string(), t.description().to_string())).collect()
    }
}

impl Default for ToolExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct EchoTool;

    impl Tool for EchoTool {
        fn name(&self) -> &str { "echo" }
        fn description(&self) -> &str { "回显输入" }
        fn validate_input(&self, input: &str) -> bool { !input.is_empty() }
        fn execute(&self, input: &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, ToolError>> + Send + '_>> {
            let output = input.to_string();
            Box::pin(async move { Ok(output) })
        }
    }

    #[tokio::test]
    async fn test_register_and_execute() {
        let executor = ToolExecutor::new();
        executor.register_tool(Arc::new(EchoTool));
        let result = executor.execute("echo", "hello").await;
        assert_eq!(result.unwrap(), "hello");
    }

    #[tokio::test]
    async fn test_tool_not_found() {
        let executor = ToolExecutor::new();
        let result = executor.execute("nonexistent", "test").await;
        assert!(matches!(result, Err(ToolError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_validation_failed() {
        let executor = ToolExecutor::new();
        executor.register_tool(Arc::new(EchoTool));
        let result = executor.execute("echo", "").await;
        assert!(matches!(result, Err(ToolError::ValidationFailed(_))));
    }

    #[test]
    fn test_list_tools() {
        let executor = ToolExecutor::new();
        executor.register_tool(Arc::new(EchoTool));
        let tools = executor.list_tools();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].0, "echo");
    }
}
