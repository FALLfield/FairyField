//! 文件操作工具模块
//!
//! file_read 和 file_write 工具，带路径遍历防护。

use super::executor::{Tool, ToolError};
use serde::Deserialize;
use std::path::Path;
use std::pin::Pin;

/// 输出内容截断上限（字节）
const MAX_OUTPUT_BYTES: usize = 10_000;

/// 安全检查：路径不能包含 `..`、绝对路径、空字节
fn is_path_safe(path: &str) -> bool {
    if path.contains("..") || path.starts_with('/') || path.contains('\0') {
        return false;
    }
    // Reject absolute Windows paths (e.g. C:\...)
    if path.len() >= 2 && path.as_bytes()[1] == b':' {
        return false;
    }
    true
    // TODO(Phase 3 Wave 2): Full canonicalization check against a configured base directory.
    // The current check prevents obvious traversal but does not resolve symlinks.
}

/// 将字符串截断到 max_bytes 字节，保证不切割多字节 UTF-8 字符。
/// 兼容稳定 Rust（无需 floor_char_boundary nightly 特性）。
fn truncate_to_bytes(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }
    let mut end = max_bytes;
    // 向后回退直到找到有效的 UTF-8 边界（最多 3 字节）
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

#[derive(Deserialize)]
struct ReadParams {
    path: String,
}

pub struct FileReadTool;

impl Tool for FileReadTool {
    fn name(&self) -> &str {
        "file_read"
    }
    fn description(&self) -> &str {
        "读取文件内容"
    }
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": { "path": { "type": "string", "description": "文件路径（相对路径）" } },
            "required": ["path"]
        })
    }
    fn validate_input(&self, input: &str) -> bool {
        if let Ok(params) = serde_json::from_str::<ReadParams>(input) {
            return is_path_safe(&params.path);
        }
        false
    }
    fn execute(
        &self,
        input: &str,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<String, ToolError>> + Send + '_>> {
        let result = match serde_json::from_str::<ReadParams>(input) {
            Ok(params) if !is_path_safe(&params.path) => {
                Err(ToolError::ValidationFailed("路径包含非法字符".into()))
            }
            Ok(params) => match std::fs::read_to_string(&params.path) {
                Ok(content) => {
                    let truncated = truncate_to_bytes(&content, MAX_OUTPUT_BYTES);
                    Ok(truncated.to_string())
                }
                Err(e) => Err(ToolError::ExecutionFailed(e.to_string())),
            },
            Err(e) => Err(ToolError::ValidationFailed(e.to_string())),
        };
        Box::pin(async move { result })
    }
}

#[derive(Deserialize)]
struct WriteParams {
    path: String,
    content: String,
}

pub struct FileWriteTool;

impl Tool for FileWriteTool {
    fn name(&self) -> &str {
        "file_write"
    }
    fn description(&self) -> &str {
        "写入文件内容"
    }
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "文件路径（相对路径）" },
                "content": { "type": "string", "description": "文件内容" }
            },
            "required": ["path", "content"]
        })
    }
    fn validate_input(&self, input: &str) -> bool {
        if let Ok(params) = serde_json::from_str::<WriteParams>(input) {
            return is_path_safe(&params.path);
        }
        false
    }
    fn execute(
        &self,
        input: &str,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<String, ToolError>> + Send + '_>> {
        let result = match serde_json::from_str::<WriteParams>(input) {
            Ok(params) if !is_path_safe(&params.path) => {
                Err(ToolError::ValidationFailed("路径包含非法字符".into()))
            }
            Ok(params) => {
                // 确保父目录存在，正确传播错误
                let dir_result = match Path::new(&params.path).parent() {
                    Some(parent) if !parent.as_os_str().is_empty() => {
                        std::fs::create_dir_all(parent).map_err(|e| {
                            ToolError::ExecutionFailed(format!("无法创建目录 {:?}: {}", parent, e))
                        })
                    }
                    _ => Ok(()),
                };
                match dir_result {
                    Ok(()) => match std::fs::write(&params.path, &params.content) {
                        Ok(()) => Ok(format!(
                            "成功写入 {} 字节到 {}",
                            params.content.len(),
                            params.path
                        )),
                        Err(e) => Err(ToolError::ExecutionFailed(e.to_string())),
                    },
                    Err(e) => Err(e),
                }
            }
            Err(e) => Err(ToolError::ValidationFailed(e.to_string())),
        };
        Box::pin(async move { result })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_safety() {
        assert!(is_path_safe("src/main.rs"));
        assert!(!is_path_safe("../../etc/passwd"));
        assert!(!is_path_safe("/etc/passwd"));
        assert!(!is_path_safe("foo/../../../bar"));
        // Windows absolute paths
        assert!(!is_path_safe("C:\\Windows\\System32"));
        // Null bytes
        assert!(!is_path_safe("foo\0bar"));
    }

    #[test]
    fn validate_read() {
        let tool = FileReadTool;
        assert!(tool.validate_input(r#"{"path": "src/main.rs"}"#));
        assert!(!tool.validate_input(r#"{"path": "../../etc/passwd"}"#));
    }

    #[test]
    fn validate_write() {
        let tool = FileWriteTool;
        assert!(tool.validate_input(r#"{"path": "test.txt", "content": "hello"}"#));
        assert!(!tool.validate_input(r#"{"path": "/etc/passwd", "content": "bad"}"#));
    }

    #[tokio::test]
    async fn read_nonexistent_file() {
        let tool = FileReadTool;
        let result = tool
            .execute(r#"{"path": "nonexistent_file_xyz.txt"}"#)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn write_and_read_file() {
        let rel_path = "fairyfield_test_write.txt";
        let write_tool = FileWriteTool;
        let read_tool = FileReadTool;

        let w = write_tool
            .execute(&format!(
                r#"{{"path": "{}", "content": "hello test"}}"#,
                rel_path
            ))
            .await;
        assert!(w.is_ok(), "write failed: {:?}", w);

        let r = read_tool
            .execute(&format!(r#"{{"path": "{}"}}"#, rel_path))
            .await;
        assert_eq!(r.unwrap(), "hello test");

        let _ = std::fs::remove_file(rel_path);
    }

    #[tokio::test]
    async fn read_truncates_multibyte_safe() {
        let rel_path = "fairyfield_test_cjk.txt";
        // Create a file with CJK content exceeding MAX_OUTPUT_BYTES
        let cjk_char = "你"; // 3 bytes in UTF-8
        let large_content = cjk_char.repeat(5000); // 15000 bytes
        std::fs::write(rel_path, &large_content).unwrap();

        let read_tool = FileReadTool;
        let result = read_tool
            .execute(&format!(r#"{{"path": "{}"}}"#, rel_path))
            .await;
        let output = result.unwrap();
        // Output should be truncated but never panic on UTF-8 boundary
        assert!(output.len() <= MAX_OUTPUT_BYTES);
        assert!(output.chars().all(|c| c == '你'));

        let _ = std::fs::remove_file(rel_path);
    }

    #[test]
    fn truncate_multibyte_boundary() {
        let s = "你好世界".repeat(1000); // 12 bytes per repeat = 12000 bytes
        let truncated = truncate_to_bytes(&s, MAX_OUTPUT_BYTES);
        assert!(truncated.len() <= MAX_OUTPUT_BYTES);
        // Must be valid UTF-8
        assert!(truncated
            .chars()
            .all(|c| c == '你' || c == '好' || c == '世' || c == '界'));
    }
}
