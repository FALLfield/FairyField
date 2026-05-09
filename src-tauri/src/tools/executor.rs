//! 工具执行器模块
//!
//! 管理工具注册、选择和执行的生命周期。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

/// 工具信息（供 LLM 工具调用参考）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
    pub parameters_schema: serde_json::Value,
}

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
    /// JSON Schema 描述参数格式（供 LLM 工具调用参考）
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({"type": "object", "properties": {}})
    }
    /// 验证输入
    fn validate_input(&self, input: &str) -> bool;
    /// 执行工具
    fn execute(
        &self,
        input: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, ToolError>> + Send + '_>>;
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
    ///
    /// 使用 `get_tool()` 提取 Arc 后再异步执行，避免 `MutexGuard` 跨 `.await`
    /// 导致 future 不满足 `Send` 约束。
    pub async fn execute(&self, name: &str, input: &str) -> Result<String, ToolError> {
        let tool = self
            .get_tool(name)
            .ok_or_else(|| ToolError::NotFound(name.to_string()))?;

        if !tool.validate_input(input) {
            return Err(ToolError::ValidationFailed(format!(
                "输入验证失败: {input}"
            )));
        }

        tokio::time::timeout(self.timeout, tool.execute(input))
            .await
            .map_err(|_| ToolError::Timeout(name.to_string()))?
    }

    /// 获取工具的 Arc 引用（用于跨 await 执行）
    pub fn get_tool(&self, name: &str) -> Option<Arc<dyn Tool>> {
        let tools = self.tools.lock().unwrap();
        tools.get(name).cloned()
    }

    /// 列出所有已注册工具（名称 + 描述）
    pub fn list_tools(&self) -> Vec<(String, String)> {
        let tools = self.tools.lock().unwrap();
        tools
            .values()
            .map(|t| (t.name().to_string(), t.description().to_string()))
            .collect()
    }

    /// 列出所有已注册工具的完整信息（含参数 schema）
    pub fn list_tool_infos(&self) -> Vec<ToolInfo> {
        let tools = self.tools.lock().unwrap();
        tools
            .values()
            .map(|t| ToolInfo {
                name: t.name().to_string(),
                description: t.description().to_string(),
                parameters_schema: t.parameters_schema(),
            })
            .collect()
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
        fn name(&self) -> &str {
            "echo"
        }
        fn description(&self) -> &str {
            "回显输入"
        }
        fn validate_input(&self, input: &str) -> bool {
            !input.is_empty()
        }
        fn execute(
            &self,
            input: &str,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<String, ToolError>> + Send + '_>,
        > {
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

    #[test]
    fn test_list_tool_infos() {
        let executor = ToolExecutor::new();
        executor.register_tool(Arc::new(EchoTool));
        let infos = executor.list_tool_infos();
        assert_eq!(infos.len(), 1);
        assert_eq!(infos[0].name, "echo");
        assert_eq!(infos[0].description, "回显输入");
        // Default parameters_schema is {"type": "object", "properties": {}}
        assert!(infos[0].parameters_schema.is_object());
    }
}
