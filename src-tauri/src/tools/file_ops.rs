//! 文件操作工具模块
//!
//! 封装安全的文件读写操作。

use super::executor::{Tool, ToolError};

/// 文件操作工具
pub struct FileOpsTool {
    /// 允许的根目录
    allowed_roots: Vec<String>,
}

impl FileOpsTool {
    pub fn new() -> Self {
        Self {
            allowed_roots: vec![],
        }
    }

    pub fn with_allowed_roots(roots: Vec<String>) -> Self {
        Self { allowed_roots: roots }
    }

    /// 检查路径是否安全（无路径遍历）
    pub fn is_path_safe(path: &str) -> bool {
        // 拒绝空路径
        if path.trim().is_empty() {
            return false;
        }
        // 拒绝路径遍历
        if path.contains("..") {
            return false;
        }
        // 拒绝绝对路径（除非在允许列表中）
        if path.starts_with('/') || path.starts_with('~') {
            return false;
        }
        true
    }

    /// 检查路径是否在允许的根目录内
    pub fn is_within_allowed_roots(&self, path: &str) -> bool {
        if self.allowed_roots.is_empty() {
            return true; // 没有设置限制时允许所有
        }
        self.allowed_roots.iter().any(|root| path.starts_with(root))
    }
}

impl Tool for FileOpsTool {
    fn name(&self) -> &str {
        "file_ops"
    }

    fn description(&self) -> &str {
        "安全的文件读写操作（read, write, list）"
    }

    fn validate_input(&self, input: &str) -> bool {
        Self::is_path_safe(input) && self.is_within_allowed_roots(input)
    }

    fn execute(&self, input: &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, ToolError>> + Send + '_>> {
        let cmd = input.to_string();
        Box::pin(async move {
            // Phase 3 实现：解析操作类型并执行
            Err(ToolError::ExecutionFailed(format!("文件操作尚未实现: {cmd}")))
        })
    }
}

impl Default for FileOpsTool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_safe() {
        assert!(FileOpsTool::is_path_safe("config.json"));
        assert!(FileOpsTool::is_path_safe("src/main.rs"));
        assert!(!FileOpsTool::is_path_safe(""));
        assert!(!FileOpsTool::is_path_safe("../etc/passwd"));
        assert!(!FileOpsTool::is_path_safe("/etc/passwd"));
        assert!(!FileOpsTool::is_path_safe("~/.ssh/id_rsa"));
    }

    #[test]
    fn test_allowed_roots() {
        let tool = FileOpsTool::with_allowed_roots(vec!["src/".to_string()]);
        assert!(tool.is_within_allowed_roots("src/main.rs"));
        assert!(!tool.is_within_allowed_roots("tests/test.rs"));
    }

    #[test]
    fn test_no_roots_restriction() {
        let tool = FileOpsTool::new();
        assert!(tool.is_within_allowed_roots("anything/file.txt"));
    }

    #[test]
    fn test_validate_input() {
        let tool = FileOpsTool::new();
        assert!(tool.validate_input("config.json"));
        assert!(!tool.validate_input("../secret"));
    }
}
