//! Git 操作工具模块
//!
//! 封装常用 Git 操作，供 Agent 工具调用。

use super::executor::{Tool, ToolError};

/// Git 工具
pub struct GitTool {
    /// 允许的 git 子命令白名单
    allowed_commands: Vec<&'static str>,
}

impl GitTool {
    pub fn new() -> Self {
        Self {
            allowed_commands: vec![
                "status",
                "log",
                "diff",
                "branch",
                "remote",
                "show",
                "tag",
                "stash list",
                "config --list",
            ],
        }
    }

    /// 检查 git 子命令是否允许
    pub fn is_subcommand_allowed(&self, subcmd: &str) -> bool {
        self.allowed_commands
            .iter()
            .any(|&allowed| subcmd == allowed || subcmd.starts_with(&format!("{allowed} ")))
    }
}

impl Tool for GitTool {
    fn name(&self) -> &str {
        "git"
    }

    fn description(&self) -> &str {
        "执行只读 Git 操作（status, log, diff, branch 等）"
    }

    fn validate_input(&self, input: &str) -> bool {
        let trimmed = input.trim();
        if !trimmed.starts_with("git ") {
            return false;
        }
        let subcmd = &trimmed[4..];
        self.is_subcommand_allowed(subcmd)
    }

    fn execute(
        &self,
        input: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, ToolError>> + Send + '_>>
    {
        let cmd = input.to_string();
        Box::pin(async move {
            // Phase 3 实现：使用 tokio::process::Command 执行 git 命令
            Err(ToolError::ExecutionFailed(format!(
                "Git 执行尚未实现: {cmd}"
            )))
        })
    }
}

impl Default for GitTool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allowed_commands() {
        let tool = GitTool::new();
        assert!(tool.is_subcommand_allowed("status"));
        assert!(tool.is_subcommand_allowed("log --oneline -5"));
        assert!(tool.is_subcommand_allowed("diff HEAD~1"));
        assert!(!tool.is_subcommand_allowed("push"));
        assert!(!tool.is_subcommand_allowed("reset --hard"));
    }

    #[test]
    fn test_validate_input() {
        let tool = GitTool::new();
        assert!(tool.validate_input("git status"));
        assert!(tool.validate_input("git log --oneline"));
        assert!(!tool.validate_input("git push origin main"));
        assert!(!tool.validate_input("ls"));
        assert!(!tool.validate_input(""));
    }
}
