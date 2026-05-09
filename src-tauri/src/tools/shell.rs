//! Shell 命令工具模块
//!
//! 带白名单和超时控制的终端命令执行。

use super::executor::{Tool, ToolError};
use serde::Deserialize;
use std::pin::Pin;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const DEFAULT_TIMEOUT_SECS: u64 = 10;

/// 输出内容截断上限（字节）
const MAX_OUTPUT_BYTES: usize = 10_000;

const ALLOWED_COMMANDS: &[&str] = &[
    "ls", "cat", "pwd", "echo", "date", "which", "whoami", "env", "find", "grep", "head", "tail",
    "wc", "sort", "uniq", "git", "cargo", "rustc", "node", "npm", "pnpm", "python3",
];

#[derive(Deserialize)]
struct ShellParams {
    command: String,
    #[serde(default = "default_timeout")]
    timeout: u64,
}

fn default_timeout() -> u64 {
    DEFAULT_TIMEOUT_SECS
}

fn extract_cmd_name(command: &str) -> String {
    command
        .trim()
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_string()
}

pub struct ShellTool {
    allowed: Vec<String>,
}

impl ShellTool {
    pub fn new() -> Self {
        Self {
            allowed: ALLOWED_COMMANDS.iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl Tool for ShellTool {
    fn name(&self) -> &str {
        "terminal"
    }
    fn description(&self) -> &str {
        "执行终端命令（白名单限制）"
    }
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "command": { "type": "string", "description": "要执行的命令" },
                "timeout": { "type": "number", "description": "超时时间（秒）" }
            },
            "required": ["command"]
        })
    }
    fn validate_input(&self, input: &str) -> bool {
        if let Ok(params) = serde_json::from_str::<ShellParams>(input) {
            let cmd_name = extract_cmd_name(&params.command);
            return self.allowed.contains(&cmd_name);
        }
        false
    }
    fn execute(
        &self,
        input: &str,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<String, ToolError>> + Send + '_>> {
        let result = match serde_json::from_str::<ShellParams>(input) {
            Ok(params) => {
                let cmd_name = extract_cmd_name(&params.command);
                if !self.allowed.contains(&cmd_name) {
                    return Box::pin(async move {
                        Err(ToolError::ValidationFailed(format!(
                            "命令 '{}' 不在白名单中",
                            cmd_name
                        )))
                    });
                }
                // 同步执行带超时
                match wait_with_timeout(&params.command, params.timeout) {
                    Ok(output) => Ok(output),
                    Err(e) => Err(ToolError::ExecutionFailed(e)),
                }
            }
            Err(e) => Err(ToolError::ValidationFailed(e.to_string())),
        };
        Box::pin(async move { result })
    }
}

/// 将字符串截断到 max_bytes 字节，保证不切割多字节 UTF-8 字符。
/// 兼容稳定 Rust（无需 floor_char_boundary nightly 特性）。
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

/// 执行 shell 命令并真正强制超时。
/// spawn 子进程后轮询状态，超时时 kill。
fn wait_with_timeout(command: &str, timeout_secs: u64) -> Result<String, String> {
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(command)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;

    let start = Instant::now();
    let timeout = Duration::from_secs(timeout_secs.max(1));

    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let output = child.wait_with_output().map_err(|e| e.to_string())?;
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                // CRITICAL: 截断到字节边界，避免在多字节字符中间切割
                let result = truncate_to_bytes(&stdout, MAX_OUTPUT_BYTES).to_string();
                if !status.success() {
                    return Err(if stderr.is_empty() {
                        result
                    } else {
                        format!("{}\n{}", result, stderr)
                    });
                }
                return Ok(result);
            }
            Ok(None) => {
                if start.elapsed() >= timeout {
                    let _ = child.kill();
                    return Err(format!("命令执行超时（{}秒）", timeout_secs));
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(e) => return Err(e.to_string()),
        }
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
    fn validate_allowed_command() {
        let tool = ShellTool::new();
        assert!(tool.validate_input(r#"{"command": "ls -la"}"#));
        assert!(tool.validate_input(r#"{"command": "echo hello"}"#));
    }

    #[test]
    fn reject_disallowed_command() {
        let tool = ShellTool::new();
        assert!(!tool.validate_input(r#"{"command": "rm -rf /"}"#));
        assert!(!tool.validate_input(r#"{"command": "sudo apt install"}"#));
    }

    #[tokio::test]
    async fn execute_echo() {
        let tool = ShellTool::new();
        let result = tool.execute(r#"{"command": "echo hello world"}"#).await;
        assert!(result.unwrap().contains("hello world"));
    }

    #[test]
    fn extract_cmd() {
        assert_eq!(extract_cmd_name("ls -la"), "ls");
        assert_eq!(extract_cmd_name("git status"), "git");
    }

    #[tokio::test]
    async fn timeout_kills_long_command() {
        let tool = ShellTool::new();
        // `find /` 会遍历整个文件系统，1 秒内极大概率超时
        let result = tool.execute(r#"{"command": "find /", "timeout": 1}"#).await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("超时"), "expected timeout error, got: {}", err);
    }

    #[tokio::test]
    async fn truncates_multibyte_output() {
        let tool = ShellTool::new();
        // Generate CJK output exceeding MAX_OUTPUT_BYTES
        let result = tool
            .execute(r#"{"command": "python3 -c \"print('你' * 5000)\""}"#)
            .await;
        // python3 is in the allowed list; skip if not installed
        if let Ok(output) = result {
            assert!(output.len() <= MAX_OUTPUT_BYTES + 3); // at most one char over
            assert!(output.chars().all(|c| c == '你' || c == '\n'));
        }
    }
}
