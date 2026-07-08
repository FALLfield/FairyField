//! 子 Agent 委派系统
//!
//! Primary Agent 可以将任务委派给专门的子 Agent 执行。
//! 每个 SubAgent 有独立的 system prompt 和工具集。

use super::toolset::Toolset;
use serde::{Deserialize, Serialize};

/// 子 Agent 类型
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SubAgentKind {
    ToolExecutor,
    Searcher,
    Coder,
}

impl std::fmt::Display for SubAgentKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SubAgentKind::ToolExecutor => write!(f, "ToolExecutor"),
            SubAgentKind::Searcher => write!(f, "Searcher"),
            SubAgentKind::Coder => write!(f, "Coder"),
        }
    }
}

/// 子 Agent 定义
#[derive(Debug, Clone)]
pub struct SubAgent {
    kind: SubAgentKind,
    name: String,
    system_prompt: String,
    allowed_tools: Vec<String>,
    max_rounds: u32,
}

impl SubAgent {
    pub fn new(kind: SubAgentKind, name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            system_prompt: default_system_prompt(&kind),
            allowed_tools: default_tools(&kind),
            kind,
            name,
            max_rounds: 5,
        }
    }

    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = prompt.into();
        self
    }

    pub fn with_allowed_tools(mut self, tools: Vec<String>) -> Self {
        self.allowed_tools = tools;
        self
    }

    pub fn with_max_rounds(mut self, rounds: u32) -> Self {
        self.max_rounds = rounds;
        self
    }

    pub fn kind(&self) -> &SubAgentKind {
        &self.kind
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn system_prompt(&self) -> &str {
        &self.system_prompt
    }
    pub fn allowed_tools(&self) -> &[String] {
        &self.allowed_tools
    }
    pub fn max_rounds(&self) -> u32 {
        self.max_rounds
    }
}

/// 委派结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationResult {
    pub success: bool,
    pub output: String,
    pub rounds_used: u32,
    pub tools_called: Vec<String>,
}

/// 委派管理器
pub struct DelegationManager {
    agents: Vec<SubAgent>,
}

impl DelegationManager {
    pub fn new() -> Self {
        Self { agents: Vec::new() }
    }

    /// 创建带默认 3 个子 Agent 的管理器
    pub fn with_default_agents() -> Self {
        let mut mgr = Self::new();
        mgr.register(SubAgent::new(SubAgentKind::ToolExecutor, "ToolExecutor"));
        mgr.register(SubAgent::new(SubAgentKind::Searcher, "Searcher"));
        mgr.register(SubAgent::new(SubAgentKind::Coder, "Coder"));
        mgr
    }

    pub fn register(&mut self, agent: SubAgent) {
        self.agents.retain(|a| a.kind != agent.kind);
        self.agents.push(agent);
    }

    /// 根据任务描述选择子 Agent（关键词匹配）
    pub fn select_agent(&self, task_description: &str) -> Option<&SubAgent> {
        let lower = task_description.to_lowercase();

        for kw in &["搜索", "search", "查找", "检索", "query", "信息"] {
            if lower.contains(kw) {
                return self.find_by_kind(&SubAgentKind::Searcher);
            }
        }
        for kw in &["代码", "code", "编程", "开发", "函数", "function", "debug"] {
            if lower.contains(kw) {
                return self.find_by_kind(&SubAgentKind::Coder);
            }
        }
        self.find_by_kind(&SubAgentKind::ToolExecutor)
    }

    /// 委派任务（当前 mock 实现）
    pub async fn delegate(
        &self,
        agent_kind: SubAgentKind,
        task: &str,
        context: &str,
    ) -> Result<DelegationResult, String> {
        match self.find_by_kind(&agent_kind) {
            Some(a) => Ok(DelegationResult {
                success: true,
                output: format!(
                    "[SubAgent {}] executed: {}\nContext: {}",
                    a.name, task, context
                ),
                rounds_used: 1,
                tools_called: Vec::new(),
            }),
            None => Err(format!("no agent registered for kind {:?}", agent_kind)),
        }
    }

    /// 使用工具集执行任务（连接真实工具执行管道）
    ///
    /// 根据任务描述选择合适的子 Agent，然后通过 Toolset 执行工具调用。
    /// 真正的 LLM 决策（决定调用哪个工具）将在后续集成中实现。
    pub async fn delegate_with_tools(
        &self,
        task: &str,
        toolset: &Toolset,
    ) -> Result<DelegationResult, String> {
        let agent = self.select_agent(task);
        let agent_name = agent.map(|a| a.name.as_str()).unwrap_or("default");
        let _agent_kind = agent
            .map(|a| a.kind.clone())
            .unwrap_or(SubAgentKind::ToolExecutor);

        // 列出可用工具（供日志和调试）
        let available = toolset.list_tools();
        let tool_names: Vec<String> = available.iter().map(|t| t.name.clone()).collect();

        // 未来：通过 LLM 决定调用哪个工具
        // 当前：返回任务接收确认 + 可用工具列表
        Ok(DelegationResult {
            success: true,
            output: format!(
                "[SubAgent {}] received task: '{}'\nAvailable tools: {}",
                agent_name,
                task,
                tool_names.join(", ")
            ),
            rounds_used: 1,
            tools_called: Vec::new(),
        })
    }

    pub fn list_agents(&self) -> &[SubAgent] {
        &self.agents
    }

    fn find_by_kind(&self, kind: &SubAgentKind) -> Option<&SubAgent> {
        self.agents.iter().find(|a| &a.kind == kind)
    }
}

impl Default for DelegationManager {
    fn default() -> Self {
        Self::with_default_agents()
    }
}

fn default_system_prompt(kind: &SubAgentKind) -> String {
    match kind {
        SubAgentKind::ToolExecutor => {
            "你是一个工具执行 Agent。准确、安全地执行指定的工具调用。".into()
        }
        SubAgentKind::Searcher => "你是一个搜索 Agent。擅长信息检索和整理。".into(),
        SubAgentKind::Coder => "你是一个编码 Agent。擅长代码编写、调试和重构。".into(),
    }
}

fn default_tools(kind: &SubAgentKind) -> Vec<String> {
    match kind {
        SubAgentKind::ToolExecutor => vec![
            "web_search".into(),
            "web_fetch".into(),
            "file_read".into(),
            "file_write".into(),
        ],
        SubAgentKind::Searcher => vec![
            "web_search".into(),
            "web_fetch".into(),
            "memory_search".into(),
        ],
        SubAgentKind::Coder => vec!["file_read".into(), "file_write".into(), "terminal".into()],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_manager_has_three_agents() {
        assert_eq!(
            DelegationManager::with_default_agents().list_agents().len(),
            3
        );
    }

    #[test]
    fn select_agent_search() {
        let mgr = DelegationManager::with_default_agents();
        assert_eq!(
            mgr.select_agent("搜索今天的天气").unwrap().kind(),
            &SubAgentKind::Searcher
        );
        assert_eq!(
            mgr.select_agent("search for rust").unwrap().kind(),
            &SubAgentKind::Searcher
        );
    }

    #[test]
    fn select_agent_code() {
        let mgr = DelegationManager::with_default_agents();
        assert_eq!(
            mgr.select_agent("写一段代码").unwrap().kind(),
            &SubAgentKind::Coder
        );
    }

    #[test]
    fn select_agent_defaults_to_tool() {
        let mgr = DelegationManager::with_default_agents();
        assert_eq!(
            mgr.select_agent("帮我完成任务").unwrap().kind(),
            &SubAgentKind::ToolExecutor
        );
    }

    #[test]
    fn default_agents_use_registered_file_tool_names() {
        let mgr = DelegationManager::with_default_agents();
        let tool_agent = mgr.find_by_kind(&SubAgentKind::ToolExecutor).unwrap();
        assert!(tool_agent.allowed_tools().contains(&"file_read".into()));
        assert!(tool_agent.allowed_tools().contains(&"file_write".into()));
        assert!(!tool_agent.allowed_tools().contains(&"read_file".into()));
        assert!(!tool_agent.allowed_tools().contains(&"write_file".into()));

        let coder = mgr.find_by_kind(&SubAgentKind::Coder).unwrap();
        assert!(coder.allowed_tools().contains(&"file_read".into()));
        assert!(coder.allowed_tools().contains(&"file_write".into()));
        assert!(!coder.allowed_tools().contains(&"read_file".into()));
        assert!(!coder.allowed_tools().contains(&"write_file".into()));
    }

    #[tokio::test]
    async fn delegate_mock_result() {
        let mgr = DelegationManager::with_default_agents();
        let result = mgr
            .delegate(SubAgentKind::Coder, "write a function", "rust")
            .await
            .unwrap();
        assert!(result.success);
        assert!(result.output.contains("Coder"));
    }

    #[tokio::test]
    async fn delegate_missing_agent() {
        let mgr = DelegationManager::new();
        let result = mgr
            .delegate(SubAgentKind::ToolExecutor, "do something", "")
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn delegate_with_tools_returns_success() {
        use crate::security::guard::CommandGuard;
        use crate::security::injection::InjectionDetector;
        use crate::tools::executor::ToolExecutor;
        use std::sync::Arc;

        let executor = Arc::new(ToolExecutor::new());
        let toolset = Toolset::new(
            executor,
            Arc::new(CommandGuard::new()),
            Arc::new(InjectionDetector::new()),
        );

        let mgr = DelegationManager::with_default_agents();
        let result = mgr.delegate_with_tools("搜索天气信息", &toolset).await;
        assert!(result.is_ok());
        let res = result.unwrap();
        assert!(res.success);
        assert!(res.output.contains("Searcher"));
    }
}
