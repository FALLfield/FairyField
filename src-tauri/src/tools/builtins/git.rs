//! Git 操作工具模块
//!
//! 封装常用 Git 操作，供 Agent 工具调用。

use crate::tools::executor::{Tool, ToolError};
use std::process::Command;

const MAX_OUTPUT_BYTES: usize = 10_000;

fn truncate_to_bytes(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }
    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

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
            let subcmd = cmd
                .strip_prefix("git ")
                .ok_or_else(|| ToolError::ValidationFailed("命令必须以 git 开头".into()))?
                .trim()
                .to_string();
            let args = subcmd
                .split_whitespace()
                .map(str::to_string)
                .collect::<Vec<_>>();

            tokio::task::spawn_blocking(move || {
                let output = Command::new("git")
                    .args(&args)
                    .output()
                    .map_err(|e| ToolError::ExecutionFailed(format!("Git 启动失败: {e}")))?;

                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                let combined = if stderr.trim().is_empty() {
                    stdout.to_string()
                } else if stdout.trim().is_empty() {
                    stderr.to_string()
                } else {
                    format!("{}\n{}", stdout, stderr)
                };
                let truncated = truncate_to_bytes(&combined, MAX_OUTPUT_BYTES).to_string();

                if output.status.success() {
                    Ok(truncated)
                } else {
                    Err(ToolError::ExecutionFailed(truncated))
                }
            })
            .await
            .unwrap_or_else(|e| Err(ToolError::ExecutionFailed(format!("Git 执行线程崩溃: {e}"))))
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

    #[tokio::test]
    async fn execute_status_returns_real_git_output() {
        let tool = GitTool::new();
        let result = tool.execute("git status --short").await.unwrap();
        assert!(!result.contains("Git 执行尚未实现"));
    }
}
