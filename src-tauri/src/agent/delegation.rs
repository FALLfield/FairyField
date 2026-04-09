//! 委托管理模块
//!
//! 负责意图识别和任务路由。

/// 意图类型
#[derive(Debug, Clone, PartialEq)]
pub enum Intent {
    /// 普通对话
    Chat,
    /// Git 操作
    RunGit,
    /// Shell 命令
    RunShell,
    /// 文件操作
    FileOperation,
    /// 未知意图
    Unknown,
}

/// 委托管理器
pub struct DelegationManager;

impl DelegationManager {
    pub fn new() -> Self {
        Self
    }

    /// 简单关键词意图识别
    pub fn classify_intent(message: &str) -> Intent {
        let msg = message.to_lowercase();

        // Git 操作关键词
        if msg.starts_with("git ")
            || msg.contains("commit")
            || msg.contains("push")
            || msg.contains("pull")
            || msg.contains("branch")
            || msg.contains("merge")
        {
            return Intent::RunGit;
        }

        // Shell 命令关键词
        if msg.starts_with("run ")
            || msg.starts_with("exec ")
            || msg.starts_with("cmd ")
            || msg.starts_with("$ ")
        {
            return Intent::RunShell;
        }

        // 文件操作关键词
        if msg.starts_with("file/")
            || msg.starts_with("read ")
            || msg.starts_with("write ")
            || msg.starts_with("create file")
            || msg.starts_with("delete file")
        {
            return Intent::FileOperation;
        }

        Intent::Chat
    }
}

impl Default for DelegationManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_git_intent() {
        assert_eq!(DelegationManager::classify_intent("git status"), Intent::RunGit);
        assert_eq!(DelegationManager::classify_intent("commit the changes"), Intent::RunGit);
        assert_eq!(DelegationManager::classify_intent("push to remote"), Intent::RunGit);
    }

    #[test]
    fn test_classify_shell_intent() {
        assert_eq!(DelegationManager::classify_intent("run ls -la"), Intent::RunShell);
        assert_eq!(DelegationManager::classify_intent("exec npm test"), Intent::RunShell);
    }

    #[test]
    fn test_classify_file_intent() {
        assert_eq!(DelegationManager::classify_intent("file/read config.json"), Intent::FileOperation);
        assert_eq!(DelegationManager::classify_intent("read the file"), Intent::FileOperation);
    }

    #[test]
    fn test_classify_chat_intent() {
        assert_eq!(DelegationManager::classify_intent("你好"), Intent::Chat);
        assert_eq!(DelegationManager::classify_intent("今天天气怎么样"), Intent::Chat);
    }

    #[test]
    fn test_classify_case_insensitive() {
        assert_eq!(DelegationManager::classify_intent("GIT STATUS"), Intent::RunGit);
    }
}
