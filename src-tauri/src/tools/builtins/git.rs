//! Git 操作工具模块
//!
//! 封装常用 Git 操作，供 Agent 工具调用。

use crate::tools::executor::{Tool, ToolError};
use serde::Deserialize;
use std::time::Duration;
use tokio::process::Command;

const MAX_OUTPUT_BYTES: usize = 10_000;
const GIT_TIMEOUT_SECS: u64 = 10;
const BLOCKED_READ_OPTIONS: &[&str] = &["--output", "--no-index", "--ext-diff", "--textconv"];

fn is_blocked_read_option(argument: &str) -> bool {
    let option = argument.split_once('=').map_or(argument, |(name, _)| name);
    option.len() > 3
        && option.starts_with("--")
        && BLOCKED_READ_OPTIONS
            .iter()
            .any(|blocked| blocked.starts_with(option))
}

#[derive(Deserialize)]
struct GitParams {
    command: String,
}

fn extract_git_command(input: &str) -> Option<String> {
    serde_json::from_str::<GitParams>(input)
        .map(|params| params.command)
        .ok()
        .or_else(|| {
            let command = input.trim();
            command.starts_with("git ").then(|| command.to_string())
        })
}

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
                "show",
                "stash list",
                "config --list",
            ],
        }
    }

    /// 检查 git 子命令是否允许
    pub fn is_subcommand_allowed(&self, subcmd: &str) -> bool {
        let parts = subcmd.split_whitespace().collect::<Vec<_>>();
        let Some((command, args)) = parts.split_first() else {
            return false;
        };
        if args.iter().any(|arg| is_blocked_read_option(arg)) {
            return false;
        }

        match (*command, args) {
            ("branch", [])
            | ("branch", ["--list"])
            | ("branch", ["--show-current"])
            | ("tag", [])
            | ("tag", ["--list"])
            | ("remote", [])
            | ("remote", ["-v"])
            | ("remote", ["--verbose"]) => return true,
            ("remote", ["get-url", name]) if !name.starts_with('-') => return true,
            _ => {}
        }

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
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "只读 Git 命令，例如 git status --short"
                }
            },
            "required": ["command"]
        })
    }

    fn validate_input(&self, input: &str) -> bool {
        let Some(command) = extract_git_command(input) else {
            return false;
        };
        let Some(subcmd) = command.strip_prefix("git ") else {
            return false;
        };
        self.is_subcommand_allowed(subcmd)
    }

    fn execute(
        &self,
        input: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, ToolError>> + Send + '_>>
    {
        let command = extract_git_command(input);
        Box::pin(async move {
            let command = command.ok_or_else(|| {
                ToolError::ValidationFailed("命令必须是包含 command 字段的 JSON".into())
            })?;
            let subcmd = command
                .strip_prefix("git ")
                .ok_or_else(|| ToolError::ValidationFailed("命令必须以 git 开头".into()))?
                .trim()
                .to_string();
            let args = subcmd
                .split_whitespace()
                .map(str::to_string)
                .collect::<Vec<_>>();

            let mut command = Command::new("git");
            command.args(&args).kill_on_drop(true);
            let output =
                tokio::time::timeout(Duration::from_secs(GIT_TIMEOUT_SECS), command.output())
                    .await
                    .map_err(|_| ToolError::Timeout("git".to_string()))?
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
    fn mutating_branch_tag_and_remote_actions_are_rejected() {
        let tool = GitTool::new();

        for subcommand in [
            "branch -D obsolete",
            "branch --delete obsolete",
            "branch new-branch",
            "tag -d v1.0.0",
            "tag --delete v1.0.0",
            "tag v1.0.0",
            "remote remove origin",
            "remote rename origin upstream",
            "remote add origin https://example.com/repo.git",
            "diff --output=/tmp/fairyfield-diff HEAD",
            "diff --no-index /dev/null /tmp/private-file",
            "diff --no-inde /dev/null /tmp/private-file",
            "diff --ext-diff HEAD~1 HEAD",
            "diff --ext-dif HEAD~1 HEAD",
            "diff --textconv HEAD~1 HEAD",
            "diff --textcon HEAD~1 HEAD",
            "diff --out=/tmp/fairyfield-diff HEAD",
            "log --output /tmp/fairyfield-log",
            "log --ext-diff -1",
            "show --output=/tmp/fairyfield-show HEAD",
            "show --textconv HEAD:file.txt",
        ] {
            assert!(
                !tool.is_subcommand_allowed(subcommand),
                "mutating subcommand was allowed: {subcommand}"
            );
        }
    }

    #[test]
    fn read_only_branch_tag_and_remote_actions_are_allowed() {
        let tool = GitTool::new();

        for subcommand in [
            "branch",
            "branch --list",
            "branch --show-current",
            "tag",
            "tag --list",
            "remote",
            "remote -v",
            "remote get-url origin",
        ] {
            assert!(
                tool.is_subcommand_allowed(subcommand),
                "read-only subcommand was rejected: {subcommand}"
            );
        }
    }

    #[test]
    fn test_validate_input() {
        let tool = GitTool::new();
        assert!(tool.validate_input("git status"));
        assert!(tool.validate_input("git log --oneline"));
        assert!(tool.validate_input(r#"{"command":"git status --short"}"#));
        assert!(!tool.validate_input("git push origin main"));
        assert!(!tool.validate_input(r#"{"command":"git diff --output=/tmp/result HEAD"}"#));
        assert!(!tool.validate_input(r#"{"command":"git diff --no-index /dev/null /tmp/private"}"#));
        assert!(!tool.validate_input("ls"));
        assert!(!tool.validate_input(""));
    }

    #[test]
    fn parameters_schema_exposes_command_for_llm_function_calling() {
        let schema = GitTool::new().parameters_schema();
        assert_eq!(schema["type"], "object");
        assert_eq!(schema["properties"]["command"]["type"], "string");
        assert_eq!(schema["required"][0], "command");
    }

    #[tokio::test]
    async fn execute_status_returns_real_git_output() {
        let tool = GitTool::new();
        let result = tool.execute("git status --short").await.unwrap();
        assert!(!result.contains("Git 执行尚未实现"));
    }
}
