//! 命令守卫
//!
//! 对终端命令进行三级安全分类：
//! - Safe: 安全命令（ls, cat, pwd 等），直接放行
//! - Caution: 谨慎命令（git, curl, find 等），首次需确认
//! - Dangerous: 危险命令（rm, sudo, mkfs 等），每次需确认

use serde::Serialize;
use std::sync::Mutex;

/// 命令危险等级
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DangerLevel {
    /// 安全：ls, cat, pwd, echo, date, which, whoami, env 等
    Safe,
    /// 谨慎：git, find, grep, head, tail, wc, curl, wget 等
    Caution,
    /// 危险：rm, mv, cp, chmod, chown, sudo, mkfs, dd, kill, shutdown 等
    Dangerous,
}

/// 命令守卫
///
/// 维护安全/危险命令列表和当前会话的批准状态。
/// 线程安全：内部使用 Mutex 保护已批准命令列表。
pub struct CommandGuard {
    /// 允许的安全命令（无需确认）
    safe_commands: Vec<String>,
    /// 需要谨慎对待的命令（首次确认后放行）
    caution_commands: Vec<String>,
    /// 危险命令（每次都需要确认）
    dangerous_commands: Vec<String>,
    /// 当前会话已批准的命令列表
    approved: Mutex<Vec<String>>,
}

/// 命令检查结果
#[must_use = "GuardResult should be checked to determine if a command is safe to execute"]
#[derive(Debug, Clone, Serialize)]
pub struct GuardResult {
    /// 命令的危险等级
    pub level: DangerLevel,
    /// 分类原因说明
    pub reason: String,
    /// 是否已被批准（session 级别）
    pub is_approved: bool,
}

impl CommandGuard {
    /// 创建命令守卫，使用默认的命令分类列表
    pub fn new() -> Self {
        let safe_commands: Vec<String> = [
            "ls", "cat", "pwd", "echo", "date", "which", "whoami", "env", "uname", "hostname",
            "id", "true", "false", "test", "printf", "seq", "wc", "head", "tail", "sort", "uniq",
            "tee", "yes", "basename", "dirname", "realpath", "readlink", "stat",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();

        let caution_commands: Vec<String> = [
            "git", "find", "grep", "egrep", "fgrep", "awk", "sed", "curl", "wget", "ssh", "scp",
            "rsync", "tar", "gzip", "gunzip", "zip", "unzip", "docker", "npm", "yarn", "pnpm",
            "cargo", "rustc", "rustup", "make", "cmake", "node", "python3", "python", "ruby", "go",
            "java", "javac",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();

        let dangerous_commands: Vec<String> = [
            "rm",
            "rmdir",
            "mv",
            "cp",
            "chmod",
            "chown",
            "chgrp",
            "sudo",
            "su",
            "mkfs",
            "dd",
            "kill",
            "killall",
            "pkill",
            "shutdown",
            "reboot",
            "halt",
            "poweroff",
            "format",
            "fdisk",
            "parted",
            "mkswap",
            "mount",
            "umount",
            "iptables",
            "systemctl",
            "service",
            "launchctl",
            "defaults",
            "dscl",
            "nvram",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();

        Self {
            safe_commands,
            caution_commands,
            dangerous_commands,
            approved: Mutex::new(Vec::new()),
        }
    }

    /// 检查命令的危险等级
    ///
    /// 解析命令字符串的第一个词作为命令名进行分类。
    /// 如果命令已被 session 批准，标记为 is_approved=true。
    pub fn check_command(&self, command: &str) -> GuardResult {
        let cmd_name = Self::extract_command_name(command);

        // 检查是否已被批准
        let is_approved = self.is_approved(&cmd_name);

        // 按优先级检查：先检查危险（最严格），再谨慎，最后安全
        // 同时检查前缀匹配（如 mkfs.ext4 匹配 mkfs）
        let is_dangerous = self.dangerous_commands.contains(&cmd_name)
            || self
                .dangerous_commands
                .iter()
                .any(|dc| cmd_name.starts_with(&format!("{}.", dc)));
        if is_dangerous {
            return GuardResult {
                level: DangerLevel::Dangerous,
                reason: format!("命令 '{}' 属于危险操作，每次执行都需要确认", cmd_name),
                is_approved,
            };
        }

        if self.caution_commands.contains(&cmd_name) {
            return GuardResult {
                level: DangerLevel::Caution,
                reason: format!("命令 '{}' 属于谨慎操作，首次执行需要确认", cmd_name),
                is_approved,
            };
        }

        if self.safe_commands.contains(&cmd_name) {
            return GuardResult {
                level: DangerLevel::Safe,
                reason: format!("命令 '{}' 属于安全操作，可以直接执行", cmd_name),
                is_approved: true, // Safe 命令始终视为已批准
            };
        }

        // 未知命令默认为 Caution
        GuardResult {
            level: DangerLevel::Caution,
            reason: format!("命令 '{}' 不在已知命令列表中，默认需要确认", cmd_name),
            is_approved,
        }
    }

    /// 批准一个命令（session 级别）
    ///
    /// 当 session_only=true 时，命令仅在当前会话内有效。
    /// Dangerous 命令即使被批准，下次仍需确认（仅当次有效）。
    pub fn approve(&self, command: &str, session_only: bool) {
        if !session_only {
            return;
        }

        let cmd_name = Self::extract_command_name(command);

        // Dangerous 命令不记录 session 批准（每次都需要确认）
        if self.dangerous_commands.contains(&cmd_name) {
            return;
        }

        let mut approved = match self.approved.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        if !approved.contains(&cmd_name) {
            approved.push(cmd_name);
        }
    }

    /// 检查命令是否已被批准（session 级别）
    pub fn is_approved(&self, command: &str) -> bool {
        let cmd_name = if command.contains(' ') {
            Self::extract_command_name(command)
        } else {
            command.to_string()
        };

        let approved = match self.approved.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        approved.contains(&cmd_name)
    }

    /// 清除所有 session 级别的批准
    pub fn clear_session(&self) {
        let mut approved = match self.approved.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        approved.clear();
    }

    /// 从命令字符串中提取命令名
    ///
    /// 处理以下情况：
    /// - 带参数的命令：`ls -la /tmp` → `ls`
    /// - 带路径的命令：`/usr/bin/rm` → `rm`
    /// - 管道命令取第一个：`cat file | grep foo` → `cat`
    /// - 环境变量前缀：`VAR=value cmd` → `cmd`
    fn extract_command_name(command: &str) -> String {
        // 去除前后空白
        let trimmed = command.trim();

        if trimmed.is_empty() {
            return String::new();
        }

        // 处理管道，只取第一个命令
        let first_segment = trimmed.split('|').next().unwrap_or(trimmed).trim();

        // 处理环境变量赋值前缀（如 VAR=value cmd）
        let mut cmd_part = first_segment;
        while let Some(space_pos) = cmd_part.find(' ') {
            let before_space = &cmd_part[..space_pos];
            if before_space.contains('=') && !before_space.starts_with('-') {
                cmd_part = cmd_part[space_pos..].trim();
            } else {
                break;
            }
        }

        // 取第一个 token
        let first_token = cmd_part.split_whitespace().next().unwrap_or("");

        // 去除路径前缀，只保留命令名
        if let Some(slash_pos) = first_token.rfind('/') {
            first_token[(slash_pos + 1)..].to_string()
        } else {
            first_token.to_string()
        }
    }
}

impl Default for CommandGuard {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_commands() {
        let guard = CommandGuard::new();

        let safe_cases = vec![
            "ls",
            "ls -la",
            "cat /etc/hosts",
            "pwd",
            "echo hello",
            "date",
            "which cargo",
            "whoami",
            "env",
            "uname -a",
            "hostname",
            "id",
            "true",
            "false",
            "seq 1 10",
            "wc -l file.txt",
            "head -n 5 file",
            "tail -f log.txt",
            "sort file.txt",
            "uniq -c",
            "basename /path/to/file",
            "dirname /path/to/file",
            "stat file.txt",
            "realpath ../some/path",
        ];
        for cmd in safe_cases {
            let result = guard.check_command(cmd);
            assert_eq!(
                result.level,
                DangerLevel::Safe,
                "Expected '{}' to be Safe, got {:?}: {}",
                cmd,
                result.level,
                result.reason
            );
        }
    }

    #[test]
    fn test_caution_commands() {
        let guard = CommandGuard::new();

        let caution_cases = vec![
            "git status",
            "git commit -m 'msg'",
            "find . -name '*.rs'",
            "grep -r 'pattern'",
            "curl https://example.com",
            "wget https://example.com/file",
            "cargo build",
            "cargo test",
            "npm install",
            "pnpm add pkg",
            "node app.js",
            "python3 script.py",
            "go run main.go",
            "docker ps",
            "rsync -av src/ dst/",
            "tar -czf archive.tar.gz dir/",
            "gzip file.txt",
            "make",
            "cmake ..",
        ];
        for cmd in caution_cases {
            let result = guard.check_command(cmd);
            assert_eq!(
                result.level,
                DangerLevel::Caution,
                "Expected '{}' to be Caution, got {:?}: {}",
                cmd,
                result.level,
                result.reason
            );
        }
    }

    #[test]
    fn test_dangerous_commands() {
        let guard = CommandGuard::new();

        let dangerous_cases = vec![
            "rm -rf /",
            "rm file.txt",
            "rmdir empty_dir",
            "mv old new",
            "cp src dst",
            "chmod 777 file",
            "chown user file",
            "chgrp group file",
            "sudo apt install",
            "su root",
            "mkfs.ext4 /dev/sda1",
            "dd if=/dev/zero of=/dev/sda",
            "kill -9 1234",
            "killall process",
            "pkill -f pattern",
            "shutdown -h now",
            "reboot",
            "halt",
            "poweroff",
            "fdisk /dev/sda",
            "parted /dev/sda",
            "mount /dev/sda1 /mnt",
            "umount /mnt",
            "iptables -A INPUT -j DROP",
            "systemctl stop service",
            "launchctl load -w /Library/LaunchDaemons/com.example.plist",
            "defaults write com.apple.dock",
            "dscl . -create",
            "nvram boot-args",
        ];
        for cmd in dangerous_cases {
            let result = guard.check_command(cmd);
            assert_eq!(
                result.level,
                DangerLevel::Dangerous,
                "Expected '{}' to be Dangerous, got {:?}: {}",
                cmd,
                result.level,
                result.reason
            );
        }
    }

    #[test]
    fn test_unknown_command_is_caution() {
        let guard = CommandGuard::new();
        let result = guard.check_command("my_custom_tool --flag");
        assert_eq!(result.level, DangerLevel::Caution);
        assert!(!result.is_approved);
    }

    #[test]
    fn test_approve_caution_command() {
        let guard = CommandGuard::new();

        // 首次检查应该未批准
        let result = guard.check_command("git status");
        assert!(!result.is_approved);

        // 批准后应该标记为已批准
        guard.approve("git status", true);
        assert!(guard.is_approved("git"));
    }

    #[test]
    fn test_approve_non_session_ignored() {
        let guard = CommandGuard::new();
        guard.approve("git", false);
        assert!(!guard.is_approved("git"));
    }

    #[test]
    fn test_dangerous_command_not_session_approved() {
        let guard = CommandGuard::new();

        // Dangerous 命令即使调用 approve 也不会 session 批准
        guard.approve("rm", true);
        assert!(!guard.is_approved("rm"));
    }

    #[test]
    fn test_safe_command_always_approved() {
        let guard = CommandGuard::new();
        let result = guard.check_command("ls");
        assert!(result.is_approved);
    }

    #[test]
    fn test_clear_session() {
        let guard = CommandGuard::new();

        guard.approve("git", true);
        assert!(guard.is_approved("git"));

        guard.clear_session();
        assert!(!guard.is_approved("git"));
    }

    #[test]
    fn test_extract_command_name_simple() {
        assert_eq!(CommandGuard::extract_command_name("ls"), "ls");
        assert_eq!(CommandGuard::extract_command_name("ls -la"), "ls");
        assert_eq!(
            CommandGuard::extract_command_name("  cat  file.txt  "),
            "cat"
        );
    }

    #[test]
    fn test_extract_command_name_with_path() {
        assert_eq!(CommandGuard::extract_command_name("/usr/bin/rm"), "rm");
        assert_eq!(
            CommandGuard::extract_command_name("/usr/local/bin/node app.js"),
            "node"
        );
    }

    #[test]
    fn test_extract_command_name_pipe() {
        assert_eq!(
            CommandGuard::extract_command_name("cat file | grep foo"),
            "cat"
        );
        assert_eq!(
            CommandGuard::extract_command_name("echo hello | wc -l"),
            "echo"
        );
    }

    #[test]
    fn test_extract_command_name_env_prefix() {
        assert_eq!(
            CommandGuard::extract_command_name("VAR=value cargo build"),
            "cargo"
        );
        assert_eq!(CommandGuard::extract_command_name("HOME=/tmp ls -la"), "ls");
        // 多个环境变量
        assert_eq!(
            CommandGuard::extract_command_name("A=1 B=2 node script.js"),
            "node"
        );
    }

    #[test]
    fn test_extract_command_name_empty() {
        assert_eq!(CommandGuard::extract_command_name(""), "");
        assert_eq!(CommandGuard::extract_command_name("   "), "");
    }

    #[test]
    fn test_default_impl() {
        let guard = CommandGuard::default();
        let result = guard.check_command("ls");
        assert_eq!(result.level, DangerLevel::Safe);
    }
}
