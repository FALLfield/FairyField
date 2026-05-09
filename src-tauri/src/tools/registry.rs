//! 工具注册表
//!
//! 自注册模式——所有内置工具在 init 时自动注册到 ToolExecutor。

use super::executor::{Tool, ToolError, ToolExecutor, ToolInfo};
use super::memory_tool::{MemorySaveTool, MemorySearchTool};
use crate::memory::MemoryLayers;
use std::sync::{Arc, Mutex};

/// 工具注册表
pub struct ToolRegistry {
    executor: ToolExecutor,
    tool_infos: Vec<ToolInfo>,
}

impl ToolRegistry {
    /// 创建注册表（mock 记忆工具，用于测试和未初始化场景）
    pub fn new() -> Self {
        Self::build_registry(None)
    }

    /// 创建注册表并注入真实 MemoryLayers
    ///
    /// 当 `layers` 为 `Some` 时，memory_search 和 memory_save
    /// 使用真实 FTS5 + SQLite 后端；否则回退到 mock 响应。
    pub fn new_with_memory(layers: Option<Arc<Mutex<MemoryLayers>>>) -> Self {
        Self::build_registry(layers)
    }

    /// 消费注册表，返回内部的 ToolExecutor（用于创建 Toolset）
    pub fn into_executor(self) -> ToolExecutor {
        self.executor
    }

    /// 内部：构建工具列表
    fn build_registry(memory_layers: Option<Arc<Mutex<MemoryLayers>>>) -> Self {
        let executor = ToolExecutor::new();
        let mut infos = Vec::new();

        // 根据是否传入 MemoryLayers 决定使用 mock 还是真实实现
        let search_tool: Arc<dyn Tool> = match &memory_layers {
            Some(layers) => Arc::new(MemorySearchTool::with_layers(Arc::clone(layers))),
            None => Arc::new(MemorySearchTool::new()),
        };
        let save_tool: Arc<dyn Tool> = match &memory_layers {
            Some(layers) => Arc::new(MemorySaveTool::with_layers(Arc::clone(layers))),
            None => Arc::new(MemorySaveTool::new()),
        };

        let tools: Vec<Arc<dyn Tool>> = vec![
            Arc::new(super::web::WebSearchTool),
            Arc::new(super::web::WebFetchTool),
            Arc::new(super::file_ops::FileReadTool),
            Arc::new(super::file_ops::FileWriteTool),
            Arc::new(super::shell::ShellTool::new()),
            search_tool,
            save_tool,
            Arc::new(super::git::GitTool::new()),
        ];

        for tool in tools {
            infos.push(ToolInfo {
                name: tool.name().to_string(),
                description: tool.description().to_string(),
                parameters_schema: tool.parameters_schema(),
            });
            executor.register_tool(tool);
        }

        Self {
            executor,
            tool_infos: infos,
        }
    }

    /// 动态注册额外工具
    pub fn register(&self, tool: Arc<dyn Tool>) {
        self.executor.register_tool(tool);
    }

    /// 列出所有工具信息
    pub fn tool_descriptions(&self) -> &[ToolInfo] {
        &self.tool_infos
    }

    /// 执行工具
    pub async fn execute(&self, name: &str, input: &str) -> Result<String, ToolError> {
        self.executor.execute(name, input).await
    }

    /// 列出所有工具名称和描述
    pub fn list_tools(&self) -> Vec<(String, String)> {
        self.executor.list_tools()
    }

    /// 查找工具的 Arc 引用（用于跨 await 执行）
    pub fn find_tool(&self, name: &str) -> Option<std::sync::Arc<dyn Tool>> {
        self.executor.get_tool(name)
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_creates_with_builtin_tools() {
        let reg = ToolRegistry::new();
        assert!(!reg.tool_descriptions().is_empty());
        assert!(reg
            .tool_descriptions()
            .iter()
            .any(|t| t.name == "web_search"));
        assert!(reg
            .tool_descriptions()
            .iter()
            .any(|t| t.name == "file_read"));
    }

    #[tokio::test]
    async fn execute_unknown_tool() {
        let reg = ToolRegistry::new();
        let result = reg.execute("nonexistent", "{}").await;
        assert!(result.is_err());
    }

    #[test]
    fn registry_with_memory_creates_tools() {
        let store = crate::memory::MemoryStore::new(":memory:").unwrap();
        let layers = Arc::new(Mutex::new(MemoryLayers::new(store)));
        let reg = ToolRegistry::new_with_memory(Some(layers));
        assert!(reg
            .tool_descriptions()
            .iter()
            .any(|t| t.name == "memory_search"));
        assert!(reg
            .tool_descriptions()
            .iter()
            .any(|t| t.name == "memory_save"));
    }

    #[tokio::test]
    async fn registry_with_memory_search_roundtrip() {
        let store = crate::memory::MemoryStore::new(":memory:").unwrap();
        let layers = Arc::new(Mutex::new(MemoryLayers::new(store)));
        let reg = ToolRegistry::new_with_memory(Some(layers));

        // 搜索空库应返回 []
        let result = reg
            .execute("memory_search", r#"{"query": "test"}"#)
            .await
            .unwrap();
        assert_eq!(result, "[]");
    }
}
