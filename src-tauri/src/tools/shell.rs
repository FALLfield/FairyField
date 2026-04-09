//! Shell 命令工具模块
//!
//! 封装受限的 Shell 命令执行，支持命令白名单。

use super::executor::{Tool, ToolError};
use std::collections::HashSet;

/// Shell 工具
pub struct ShellTool {
    /// 命令白名单
    whitelist: HashSet<String>,
}

impl ShellTool {
    pub fn new() -> Self {
        let mut whitelist = HashSet::new();
        // 默认安全命令
        whitelist.insert("ls".to_string());
        whitelist.insert("pwd".to_string());
        whitelist.insert("cat".to_string());
        whitelist.insert("echo".to_string());
        whitelist.insert("date".to_string());
        whitelist.insert("whoami".to_string());
        whitelist.insert("uname".to_string());
        whitelist.insert("node".to_string());
        whitelist.insert("npm".to_string());
        whitelist.insert("cargo".to_string());
        whitelist.insert("rustc".to_string());
        Self { whitelist }
    }

    pub fn with_whitelist(commands: Vec<&str>) -> Self {
        let whitelist = commands.iter().map(|s| s.to_string()).collect();
        Self { whitelist }
    }

    /// 检查命令是否在白名单中
    pub fn is_allowed(&self, command: &str) -> bool {
        let base_cmd = command.split_whitespace().next().unwrap_or("");
        self.whitelist.contains(base_cmd)
    }

    /// 检查命令注入
    pub fn check_injection(command: &str) -> bool {
        let dangerous = [";", "&&", "||", "|", "`", "$(", ">", "<", "\n", "\r"];
        dangerous.iter().any(|pattern| command.contains(pattern))
    }
}

impl Tool for ShellTool {
    fn name(&self) -> &str {
        "shell"
    }

    fn description(&self) -> &str {
        "执行白名单内的 Shell 命令"
    }

    fn validate_input(&self, input: &str) -> bool {
        if input.trim().is_empty() {
            return false;
        }
        self.is_allowed(input) && !Self::check_injection(input)
    }

    fn execute(&self, input: &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, ToolError>> + Send + '_>> {
        let cmd = input.to_string();
        Box::pin(async move {
            // Phase 3 实现：使用 tokio::process::Command 执行
            Err(ToolError::ExecutionFailed(format!("Shell 执行尚未实现: {cmd}")))
        })
    }
}

impl Default for ShellTool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_whitelist() {
        let tool = ShellTool::new();
        assert!(tool.is_allowed("ls -la"));
        assert!(tool.is_allowed("cargo build"));
        assert!(!tool.is_allowed("rm -rf /"));
    }

    #[test]
    fn test_custom_whitelist() {
        let tool = ShellTool::with_whitelist(vec!["git", "docker"]);
        assert!(tool.is_allowed("git status"));
        assert!(!tool.is_allowed("ls"));
    }

    #[test]
    fn test_injection_detection() {
        assert!(ShellTool::check_injection("ls; rm -rf /"));
        assert!(ShellTool::check_injection("cat /etc/passwd && echo done"));
        assert!(ShellTool::check_injection("echo $(whoami)"));
        assert!(!ShellTool::check_injection("ls -la"));
        assert!(!ShellTool::check_injection("cargo build"));
    }

    #[test]
    fn test_validate_input() {
        let tool = ShellTool::new();
        assert!(tool.validate_input("ls"));
        assert!(!tool.validate_input(""));
        assert!(!tool.validate_input("rm -rf /"));
        assert!(!tool.validate_input("ls; cat /etc/passwd"));
    }
}
