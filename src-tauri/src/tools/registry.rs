//! 工具注册表
//!
//! 自注册模式——所有内置工具在 init 时自动注册到 ToolExecutor。

use super::builtins::memory_tool::{MemorySaveTool, MemorySearchTool};
use super::executor::{Tool, ToolError, ToolExecutor, ToolInfo};
use crate::memory::MemoryLayers;
use std::sync::{Arc, Mutex};

/// 工具注册表
pub struct ToolRegistry {
    executor: ToolExecutor,
    tool_infos: Mutex<Vec<ToolInfo>>,
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
            Arc::new(super::builtins::web::WebSearchTool),
            Arc::new(super::builtins::web::WebFetchTool),
            Arc::new(super::builtins::file_ops::FileReadTool),
            Arc::new(super::builtins::file_ops::FileWriteTool),
            Arc::new(super::builtins::shell::ShellTool::new()),
            search_tool,
            save_tool,
            Arc::new(super::builtins::git::GitTool::new()),
            Arc::new(super::builtins::github::GitHubTool::new()),
            Arc::new(super::builtins::obsidian::ObsidianTool::new()),
            Arc::new(super::builtins::browser::BrowserTool::new()),
            Arc::new(super::builtins::cron::CronTool::new()),
            Arc::new(super::builtins::weather::WeatherTool::new()),
            Arc::new(super::builtins::translate::TranslateTool::new()),
            Arc::new(super::builtins::notion::NotionTool::new()),
            Arc::new(super::builtins::linear::LinearTool::new()),
            Arc::new(super::builtins::gmail::GmailTool::new()),
            Arc::new(super::builtins::calendar::CalendarTool::new()),
            Arc::new(super::builtins::slack::SlackTool::new()),
            Arc::new(super::builtins::discord::DiscordTool::new()),
            Arc::new(super::builtins::image::ImageTool::new()),
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
            tool_infos: Mutex::new(infos),
        }
    }

    /// 动态注册额外工具
    pub fn register(&self, tool: Arc<dyn Tool>) {
        let info = ToolInfo {
            name: tool.name().to_string(),
            description: tool.description().to_string(),
            parameters_schema: tool.parameters_schema(),
        };
        self.executor.register_tool(tool);
        let mut infos = self.tool_infos.lock().unwrap();
        if let Some(existing) = infos.iter_mut().find(|t| t.name == info.name) {
            *existing = info;
        } else {
            infos.push(info);
        }
    }

    /// 列出所有工具信息
    pub fn tool_descriptions(&self) -> Vec<ToolInfo> {
        self.tool_infos.lock().unwrap().clone()
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

    struct DynamicTestTool;

    impl Tool for DynamicTestTool {
        fn name(&self) -> &str {
            "dynamic_test"
        }

        fn description(&self) -> &str {
            "Dynamic test tool"
        }

        fn parameters_schema(&self) -> serde_json::Value {
            serde_json::json!({
                "type": "object",
                "properties": {
                    "message": {"type": "string"}
                }
            })
        }

        fn validate_input(&self, input: &str) -> bool {
            serde_json::from_str::<serde_json::Value>(input).is_ok()
        }

        fn execute(
            &self,
            _input: &str,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<String, ToolError>> + Send + '_>,
        > {
            Box::pin(async { Ok("ok".into()) })
        }
    }

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

    #[test]
    fn dynamic_register_updates_tool_infos() {
        let reg = ToolRegistry::new();
        assert!(!reg
            .tool_descriptions()
            .iter()
            .any(|t| t.name == "dynamic_test"));

        reg.register(Arc::new(DynamicTestTool));

        assert!(reg
            .list_tools()
            .iter()
            .any(|(name, _)| name == "dynamic_test"));
        assert!(reg
            .tool_descriptions()
            .iter()
            .any(|t| t.name == "dynamic_test"
                && t.description == "Dynamic test tool"
                && t.parameters_schema["properties"]["message"]["type"] == "string"));
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

    fn contains_any(haystack: &str, needles: &[&str]) -> bool {
        needles.is_empty() || needles.iter().any(|needle| haystack.contains(needle))
    }

    #[tokio::test]
    async fn builtin_tools_smoke_matrix_execute_or_report_configuration() {
        let reg = ToolRegistry::new();
        std::fs::create_dir_all("target/tool-smoke").unwrap();

        let cases: Vec<(&str, &str, &[&str], &[&str])> = vec![
            (
                "web_search",
                r#"{"query":"Rust programming language","limit":2}"#,
                &["Rust", "搜索结果", "Google"],
                &["网络限制", "解析搜索结果失败"],
            ),
            (
                "web_fetch",
                r#"{"url":"https://example.com"}"#,
                &["Example Domain"],
                &["请求失败", "DNS lookup failed", "DNS lookup timed out"],
            ),
            (
                "file_write",
                r#"{"path":"target/tool-smoke/matrix.txt","content":"fairyfield tool smoke"}"#,
                &["成功写入"],
                &[],
            ),
            (
                "file_read",
                r#"{"path":"target/tool-smoke/matrix.txt"}"#,
                &["fairyfield tool smoke"],
                &[],
            ),
            (
                "terminal",
                r#"{"command":"echo fairyfield_tool_smoke","timeout":2}"#,
                &["fairyfield_tool_smoke"],
                &[],
            ),
            (
                "memory_save",
                r#"{"content":"user likes rust programming","category":"preference"}"#,
                &["memory_save mock"],
                &[],
            ),
            (
                "memory_search",
                r#"{"query":"rust","limit":1}"#,
                &["memory_search mock"],
                &[],
            ),
            ("git", "git status --short", &[], &[]),
            (
                "github",
                r#"{"action":"get_issue","owner":"rust-lang","repo":"rust","issue_number":1}"#,
                &["\"number\"", "\"title\""],
                &["GitHub API request failed", "GitHub API error"],
            ),
            (
                "obsidian_search",
                r#"{"query":"rust","max_results":1}"#,
                &[],
                &["Obsidian vault path not configured"],
            ),
            (
                "browser",
                r#"{"action":"open_url","url":"https://example.com"}"#,
                &["queued"],
                &[],
            ),
            (
                "cron",
                r#"{"action":"schedule_reminder","schedule":"in 10 minutes","message":"stretch"}"#,
                &["queued"],
                &[],
            ),
            (
                "weather",
                r#"{"city":"Tokyo","days":1}"#,
                &["天气", "weather", "Tokyo", "东京"],
                &[],
            ),
            (
                "translate",
                r#"{"text":"hello","target_lang":"zh"}"#,
                &["configuration_required", "ready"],
                &[],
            ),
            (
                "notion",
                r#"{"action":"search","query":"notes"}"#,
                &["configuration_required", "ready"],
                &[],
            ),
            (
                "linear",
                r#"{"action":"search_issues","query":"bug"}"#,
                &["configuration_required", "ready"],
                &[],
            ),
            (
                "gmail",
                r#"{"action":"search_messages","query":"invoice"}"#,
                &["configuration_required", "ready"],
                &[],
            ),
            (
                "calendar",
                r#"{"action":"list_events"}"#,
                &["configuration_required", "ready"],
                &[],
            ),
            (
                "slack",
                r#"{"action":"list_channels"}"#,
                &["configuration_required", "ready"],
                &[],
            ),
            (
                "discord",
                r#"{"action":"list_guilds"}"#,
                &["configuration_required", "ready"],
                &[],
            ),
            (
                "image",
                r#"{"action":"generate","prompt":"a tiny desk fairy"}"#,
                &["configuration_required", "ready"],
                &[],
            ),
        ];

        let listed = reg
            .list_tools()
            .into_iter()
            .map(|(name, _)| name)
            .collect::<std::collections::HashSet<_>>();

        for (name, input, ok_contains, err_contains) in cases {
            assert!(listed.contains(name), "tool is not registered: {name}");
            let tool = reg.find_tool(name).expect("registered tool should exist");
            assert!(
                tool.validate_input(input),
                "tool rejected smoke input: {name}"
            );

            match tool.execute(input).await {
                Ok(output) => {
                    eprintln!(
                        "tool-smoke {name}: ok {}",
                        output.chars().take(120).collect::<String>()
                    );
                    assert!(
                        contains_any(&output, ok_contains),
                        "unexpected output for {name}: {output}"
                    );
                    assert!(
                        !output.contains("尚未实现") && !output.contains("not implemented"),
                        "tool is still a stub: {name}: {output}"
                    );
                }
                Err(err) => {
                    let text = err.to_string();
                    eprintln!("tool-smoke {name}: expected-error {text}");
                    assert!(
                        contains_any(&text, err_contains),
                        "unexpected error for {name}: {text}"
                    );
                }
            }
        }
    }
}
