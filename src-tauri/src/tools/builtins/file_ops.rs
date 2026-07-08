//! 文件操作工具模块
//!
//! file_read 和 file_write 工具，带路径遍历防护。

use crate::tools::executor::{Tool, ToolError};
use serde::Deserialize;
use std::ffi::OsStr;
use std::path::{Component, Path, PathBuf};
use std::pin::Pin;

/// 输出内容截断上限（字节）
const MAX_OUTPUT_BYTES: usize = 10_000;

/// 安全检查：允许项目相对路径，以及用户桌面路径。
///
/// 绝对路径只允许指向 `~/Desktop`，避免 LLM 将文件写入任意系统目录。
fn is_path_safe(path: &str) -> bool {
    resolve_user_file_path(path).is_ok()
}

fn resolve_user_file_path(path: &str) -> Result<PathBuf, ToolError> {
    match home_dir() {
        Some(home) => resolve_user_file_path_with_home(path, &home),
        None => resolve_user_file_path_inner(path, None),
    }
}

fn resolve_user_file_path_with_home(path: &str, home: &Path) -> Result<PathBuf, ToolError> {
    resolve_user_file_path_inner(path, Some(home))
}

fn resolve_user_file_path_inner(path: &str, home: Option<&Path>) -> Result<PathBuf, ToolError> {
    if path.is_empty() {
        return Err(invalid_path("路径不能为空"));
    }
    if path.contains('\0') {
        return Err(invalid_path("路径包含空字节"));
    }
    if looks_like_windows_absolute_path(path) {
        return Err(invalid_path("拒绝 Windows 绝对路径"));
    }

    let raw = Path::new(path);
    if has_parent_component(raw) {
        return Err(invalid_path("路径包含非法的上级目录引用"));
    }

    if let Some(rest) = path.strip_prefix("~/") {
        let home = require_home(home)?;
        return resolve_desktop_scoped_path(home.join(rest), home);
    }
    if let Some(rest) = path.strip_prefix("$HOME/") {
        let home = require_home(home)?;
        return resolve_desktop_scoped_path(home.join(rest), home);
    }
    if path == "~" || path == "$HOME" {
        return Err(invalid_path("用户目录根路径不可作为文件工具目标"));
    }

    if raw.is_absolute() {
        let home = require_home(home)?;
        return resolve_desktop_scoped_path(raw.to_path_buf(), home);
    }

    if first_component_is_desktop(raw) {
        let home = require_home(home)?;
        return resolve_desktop_scoped_path(home.join(raw), home);
    }

    Ok(raw.to_path_buf())
}

fn resolve_desktop_scoped_path(candidate: PathBuf, home: &Path) -> Result<PathBuf, ToolError> {
    if has_parent_component(&candidate) {
        return Err(invalid_path("路径包含非法的上级目录引用"));
    }

    let desktop = home.join("Desktop");
    if !candidate.starts_with(&desktop) {
        return Err(invalid_path("绝对路径和 Home 路径只允许访问用户桌面目录"));
    }
    Ok(candidate)
}

fn require_home(home: Option<&Path>) -> Result<&Path, ToolError> {
    home.ok_or_else(|| invalid_path("无法解析用户 Home 目录"))
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

fn has_parent_component(path: &Path) -> bool {
    path.components()
        .any(|component| matches!(component, Component::ParentDir | Component::Prefix(_)))
}

fn first_component_is_desktop(path: &Path) -> bool {
    path.components().next().is_some_and(
        |component| matches!(component, Component::Normal(name) if name == OsStr::new("Desktop")),
    )
}

fn looks_like_windows_absolute_path(path: &str) -> bool {
    path.len() >= 2 && path.as_bytes()[1] == b':'
}

fn invalid_path(message: impl Into<String>) -> ToolError {
    ToolError::ValidationFailed(message.into())
}

fn read_resolved_file(path: &Path) -> Result<String, ToolError> {
    match std::fs::read_to_string(path) {
        Ok(content) => {
            let truncated = truncate_to_bytes(&content, MAX_OUTPUT_BYTES);
            Ok(truncated.to_string())
        }
        Err(e) => Err(ToolError::ExecutionFailed(format!(
            "{}: {}",
            path.display(),
            e
        ))),
    }
}

fn write_resolved_file(path: &Path, content: &str) -> Result<String, ToolError> {
    let dir_result = match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => std::fs::create_dir_all(parent)
            .map_err(|e| ToolError::ExecutionFailed(format!("无法创建目录 {:?}: {}", parent, e))),
        _ => Ok(()),
    };

    match dir_result {
        Ok(()) => match std::fs::write(path, content) {
            Ok(()) => Ok(format!(
                "成功写入 {} 字节到 {}",
                content.len(),
                path.display()
            )),
            Err(e) => Err(ToolError::ExecutionFailed(e.to_string())),
        },
        Err(e) => Err(e),
    }
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
            "properties": { "path": { "type": "string", "description": "文件路径（项目相对路径，或 Desktop/foo.md、~/Desktop/foo.md、/Users/you/Desktop/foo.md）" } },
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
            Ok(params) => {
                resolve_user_file_path(&params.path).and_then(|path| read_resolved_file(&path))
            }
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
                "path": { "type": "string", "description": "文件路径（项目相对路径，或 Desktop/foo.md、~/Desktop/foo.md、/Users/you/Desktop/foo.md）" },
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
            Ok(params) => {
                let path = match resolve_user_file_path(&params.path) {
                    Ok(path) => path,
                    Err(e) => return Box::pin(async move { Err(e) }),
                };
                write_resolved_file(&path, &params.content)
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
    fn resolves_desktop_user_paths_without_touching_real_home() {
        let home = tempfile::tempdir().unwrap();
        let desktop_file = home.path().join("Desktop").join("fairy-note.md");

        assert_eq!(
            resolve_user_file_path_with_home("~/Desktop/fairy-note.md", home.path()).unwrap(),
            desktop_file
        );
        assert_eq!(
            resolve_user_file_path_with_home("$HOME/Desktop/fairy-note.md", home.path()).unwrap(),
            desktop_file
        );
        assert_eq!(
            resolve_user_file_path_with_home("Desktop/fairy-note.md", home.path()).unwrap(),
            desktop_file
        );

        let absolute_desktop_file = desktop_file.to_string_lossy().to_string();
        assert_eq!(
            resolve_user_file_path_with_home(&absolute_desktop_file, home.path()).unwrap(),
            desktop_file
        );
    }

    #[test]
    fn rejects_unsafe_desktop_path_expansions() {
        let home = tempfile::tempdir().unwrap();
        let outside_home = home.path().join("Documents").join("secret.md");
        let outside_home = outside_home.to_string_lossy().to_string();

        assert!(resolve_user_file_path_with_home("~/Desktop/../secret.md", home.path()).is_err());
        assert!(resolve_user_file_path_with_home("Desktop/../secret.md", home.path()).is_err());
        assert!(resolve_user_file_path_with_home("~/Documents/secret.md", home.path()).is_err());
        assert!(
            resolve_user_file_path_with_home("$HOME/Documents/secret.md", home.path()).is_err()
        );
        assert!(resolve_user_file_path_with_home(&outside_home, home.path()).is_err());
        assert!(resolve_user_file_path_with_home("/etc/passwd", home.path()).is_err());
    }

    #[test]
    fn write_and_read_desktop_markdown_path_with_test_home() {
        let home = tempfile::tempdir().unwrap();
        let path = resolve_user_file_path_with_home("Desktop/fairy-note.md", home.path()).unwrap();

        let write = write_resolved_file(&path, "# Fairy note\nhello desktop");
        assert!(write.is_ok(), "desktop write failed: {:?}", write);

        let read = read_resolved_file(&path);
        assert_eq!(read.unwrap(), "# Fairy note\nhello desktop");
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
