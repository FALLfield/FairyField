//! Coding Agent Manager — manages Claude Code / KiloCode / OpenCode subprocesses
//!
//! Fairy can dispatch coding tasks to external coding agents via CLI subprocess.

use serde::{Deserialize, Serialize};
use std::path::{Component, Path, PathBuf};
use std::process::Stdio;
use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::time::{timeout, Duration};

const MAX_STREAM_BYTES: usize = 256 * 1024;

/// Supported coding agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CodingAgentType {
    #[serde(rename = "claude-code")]
    ClaudeCode,
    #[serde(rename = "kilocode")]
    KiloCode,
    #[serde(rename = "opencode")]
    OpenCode,
}

impl std::fmt::Display for CodingAgentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ClaudeCode => write!(f, "ClaudeCode"),
            Self::KiloCode => write!(f, "KiloCode"),
            Self::OpenCode => write!(f, "OpenCode"),
        }
    }
}

impl CodingAgentType {
    pub fn cli_command(&self) -> &str {
        match self {
            Self::ClaudeCode => "claude",
            Self::KiloCode => "kilo",
            Self::OpenCode => "opencode",
        }
    }
}

/// A task dispatched to a coding agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodingTask {
    pub description: String,
    pub context_files: Vec<String>,
    pub model: Option<String>,
    pub max_turns: Option<u32>,
    pub permission_mode: Option<String>,
}

/// Result from a coding agent execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodingResult {
    pub success: bool,
    pub output: String,
    pub agent: String,
    pub duration_ms: u64,
}

/// Coding Agent Manager — manages subprocess lifecycle
pub struct CodingAgentManager {
    default_timeout: Duration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CommandSpec {
    program: String,
    args: Vec<String>,
    cwd: PathBuf,
    prompt: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CappedOutput {
    bytes: Vec<u8>,
    truncated: bool,
}

impl CodingAgentManager {
    pub fn new() -> Self {
        Self {
            default_timeout: Duration::from_secs(300), // 5 min default
        }
    }

    /// Check if a coding agent CLI is available on the system
    pub async fn check_availability(&self, agent: &CodingAgentType) -> bool {
        let cmd = agent.cli_command();
        tokio::process::Command::new(cmd)
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .await
            .map(|s| s.success())
            .unwrap_or(false)
    }

    /// List available coding agents on this system
    pub async fn list_available(&self) -> Vec<String> {
        let agents = [
            CodingAgentType::ClaudeCode,
            CodingAgentType::KiloCode,
            CodingAgentType::OpenCode,
        ];
        let mut available = Vec::new();
        for agent in &agents {
            if self.check_availability(agent).await {
                available.push(agent.to_string());
            }
        }
        available
    }

    /// Dispatch a task to a coding agent and return the result
    pub async fn dispatch(
        &self,
        agent: &CodingAgentType,
        task: &CodingTask,
        working_dir: Option<&str>,
    ) -> Result<CodingResult, String> {
        let start = std::time::Instant::now();
        let spec = build_command_spec(agent, task, working_dir)?;

        let mut command = tokio::process::Command::new(&spec.program);
        command
            .args(&spec.args)
            .current_dir(&spec.cwd)
            .kill_on_drop(true)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = command
            .spawn()
            .map_err(|e| format!("无法启动 {}: {}。请确认 CLI 已安装并在 PATH 中。", agent, e))?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "Coding agent stdout pipe unavailable".to_string())?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| "Coding agent stderr pipe unavailable".to_string())?;
        let stdout_task = tokio::spawn(read_capped(stdout, MAX_STREAM_BYTES));
        let stderr_task = tokio::spawn(read_capped(stderr, MAX_STREAM_BYTES));

        let status = match timeout(self.default_timeout, child.wait()).await {
            Ok(result) => result.map_err(|e| format!("执行失败: {}", e))?,
            Err(_) => {
                let _ = child.kill().await;
                let _ = child.wait().await;
                let _ = stdout_task.await;
                let _ = stderr_task.await;
                return Err(timeout_error(agent));
            }
        };

        let stdout = stdout_task
            .await
            .map_err(|e| format!("读取 stdout 失败: {}", e))?
            .map_err(|e| format!("读取 stdout 失败: {}", e))?;
        let stderr = stderr_task
            .await
            .map_err(|e| format!("读取 stderr 失败: {}", e))?
            .map_err(|e| format!("读取 stderr 失败: {}", e))?;

        let duration_ms = start.elapsed().as_millis() as u64;
        let stdout = render_capped_output(stdout);
        let stderr = render_capped_output(stderr);

        let combined = if stdout.is_empty() {
            stderr
        } else if stderr.is_empty() {
            stdout
        } else {
            format!("{}\n{}", stdout, stderr)
        };

        Ok(CodingResult {
            success: status.success(),
            output: combined,
            agent: agent.to_string(),
            duration_ms,
        })
    }
}

fn build_command_spec(
    agent: &CodingAgentType,
    task: &CodingTask,
    working_dir: Option<&str>,
) -> Result<CommandSpec, String> {
    let cwd = resolve_working_dir(working_dir)?;
    let permission_mode = validate_permission_mode(task.permission_mode.as_deref())?;
    let context_files = resolve_context_files(&cwd, &task.context_files)?;
    let prompt = build_prompt(task, &context_files, agent);

    let mut args = Vec::new();
    match agent {
        CodingAgentType::ClaudeCode => {
            args.push("-p".to_string());
            if let Some(model) = task.model.as_deref().filter(|m| !m.trim().is_empty()) {
                args.push("--model".to_string());
                args.push(model.trim().to_string());
            }
            if let Some(max_turns) = task.max_turns {
                args.push("--max-turns".to_string());
                args.push(max_turns.to_string());
            }
            if let Some(mode) = permission_mode {
                args.push("--permission-mode".to_string());
                args.push(mode);
            }
            args.push(prompt.clone());
        }
        CodingAgentType::KiloCode => {
            // Kilo's documented CLI uses `kilo run`; model and permission live in config.
            args.push("run".to_string());
            args.push(prompt.clone());
        }
        CodingAgentType::OpenCode => {
            args.push("run".to_string());
            if let Some(model) = task.model.as_deref().filter(|m| !m.trim().is_empty()) {
                args.push("--model".to_string());
                args.push(model.trim().to_string());
            }
            args.push(prompt.clone());
        }
    }

    Ok(CommandSpec {
        program: agent.cli_command().to_string(),
        args,
        cwd,
        prompt,
    })
}

fn resolve_working_dir(working_dir: Option<&str>) -> Result<PathBuf, String> {
    let raw = match working_dir {
        Some(dir) if !dir.trim().is_empty() => PathBuf::from(dir.trim()),
        _ => std::env::current_dir().map_err(|e| format!("无法读取当前目录: {}", e))?,
    };
    let canonical = raw
        .canonicalize()
        .map_err(|e| format!("工作目录不存在或不可访问: {} ({})", raw.display(), e))?;
    if !canonical.is_dir() {
        return Err(format!("工作目录不是目录: {}", canonical.display()));
    }
    if is_root_dir(&canonical) {
        return Err("拒绝将文件系统根目录作为 coding agent 工作目录".to_string());
    }
    if home_dir().as_deref() == Some(canonical.as_path()) {
        return Err("拒绝将用户 Home 目录作为 coding agent 工作目录".to_string());
    }
    Ok(canonical)
}

fn validate_permission_mode(mode: Option<&str>) -> Result<Option<String>, String> {
    let Some(mode) = mode.map(str::trim).filter(|m| !m.is_empty()) else {
        return Ok(None);
    };
    let normalized = mode.to_ascii_lowercase().replace(['-', '_'], "");
    let canonical = match normalized.as_str() {
        "default" => "default",
        "plan" => "plan",
        "strict" => "strict",
        "readonly" => "read-only",
        "ask" => "ask",
        "askedits" | "askbeforeedits" => "askBeforeEdits",
        "acceptedits" => "acceptEdits",
        _ => return Err(format!("拒绝不安全或未知的 permission_mode: {}", mode)),
    };
    Ok(Some(canonical.to_string()))
}

fn resolve_context_files(cwd: &Path, files: &[String]) -> Result<Vec<PathBuf>, String> {
    files
        .iter()
        .map(|file| resolve_context_file(cwd, file))
        .collect()
}

fn resolve_context_file(cwd: &Path, file: &str) -> Result<PathBuf, String> {
    let trimmed = file.trim();
    if trimmed.is_empty() || trimmed.contains('\0') {
        return Err("context_files 包含空路径".to_string());
    }
    let path = Path::new(trimmed);
    if has_parent_traversal(path) {
        return Err(format!("拒绝 context_files 路径穿越: {}", trimmed));
    }

    let candidate = if path.is_absolute() {
        PathBuf::from(path)
    } else {
        cwd.join(path)
    };
    let resolved = candidate.canonicalize().unwrap_or(candidate);
    if !resolved.starts_with(cwd) {
        return Err(format!(
            "拒绝工作目录外的 context file: {}",
            resolved.display()
        ));
    }
    Ok(resolved)
}

fn build_prompt(task: &CodingTask, context_files: &[PathBuf], agent: &CodingAgentType) -> String {
    let mut sections = vec![task.description.trim().to_string()];
    if !context_files.is_empty() {
        let list = context_files
            .iter()
            .map(|path| format!("- {}", path.display()))
            .collect::<Vec<_>>()
            .join("\n");
        sections.push(format!("Context files:\n{}", list));
    }

    let mut hints = Vec::new();
    if !matches!(
        agent,
        CodingAgentType::ClaudeCode | CodingAgentType::OpenCode
    ) {
        if let Some(model) = task.model.as_deref().filter(|m| !m.trim().is_empty()) {
            hints.push(format!("Requested model: {}", model.trim()));
        }
    }
    if !matches!(agent, CodingAgentType::ClaudeCode) {
        if let Some(max_turns) = task.max_turns {
            hints.push(format!("Requested max turns: {}", max_turns));
        }
        if let Ok(Some(mode)) = validate_permission_mode(task.permission_mode.as_deref()) {
            hints.push(format!("Requested permission mode: {}", mode));
        }
    }
    if !hints.is_empty() {
        sections.push(format!("Execution hints:\n{}", hints.join("\n")));
    }

    sections
        .into_iter()
        .filter(|section| !section.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
}

async fn read_capped<R>(mut reader: R, max_bytes: usize) -> std::io::Result<CappedOutput>
where
    R: AsyncRead + Unpin,
{
    let mut output = Vec::with_capacity(max_bytes.min(8192));
    let mut buf = [0u8; 8192];
    let mut truncated = false;

    loop {
        let read = reader.read(&mut buf).await?;
        if read == 0 {
            break;
        }
        let remaining = max_bytes.saturating_sub(output.len());
        if remaining >= read {
            output.extend_from_slice(&buf[..read]);
        } else {
            output.extend_from_slice(&buf[..remaining]);
            truncated = true;
        }
    }

    Ok(CappedOutput {
        bytes: output,
        truncated,
    })
}

fn render_capped_output(output: CappedOutput) -> String {
    let mut rendered = String::from_utf8_lossy(&output.bytes).to_string();
    if output.truncated {
        rendered.push_str("\n[output truncated at 256 KiB]");
    }
    rendered
}

fn timeout_error(agent: &CodingAgentType) -> String {
    format!("{} 执行超时，已终止子进程", agent)
}

fn has_parent_traversal(path: &Path) -> bool {
    path.components()
        .any(|component| matches!(component, Component::ParentDir))
}

fn is_root_dir(path: &Path) -> bool {
    path.parent().is_none()
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

impl Default for CodingAgentManager {
    fn default() -> Self {
        Self::new()
    }
}

// Tauri commands

#[tauri::command]
pub async fn coding_agent_list() -> Result<Vec<String>, String> {
    let manager = CodingAgentManager::new();
    Ok(manager.list_available().await)
}

#[tauri::command]
pub async fn coding_agent_dispatch(
    agent_type: String,
    description: String,
    context_files: Vec<String>,
    model: Option<String>,
    max_turns: Option<u32>,
    permission_mode: Option<String>,
) -> Result<CodingResult, String> {
    let agent = match agent_type.as_str() {
        "claude-code" => CodingAgentType::ClaudeCode,
        "kilocode" => CodingAgentType::KiloCode,
        "opencode" => CodingAgentType::OpenCode,
        other => return Err(format!("未知的 coding agent: {}", other)),
    };

    let task = CodingTask {
        description,
        context_files,
        model,
        max_turns,
        permission_mode,
    };

    let manager = CodingAgentManager::new();
    manager.dispatch(&agent, &task, None).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncWriteExt;

    #[test]
    fn agent_type_cli_commands() {
        assert_eq!(CodingAgentType::ClaudeCode.cli_command(), "claude");
        assert_eq!(CodingAgentType::KiloCode.cli_command(), "kilo");
        assert_eq!(CodingAgentType::OpenCode.cli_command(), "opencode");
    }

    #[test]
    fn agent_type_display() {
        assert_eq!(CodingAgentType::ClaudeCode.to_string(), "ClaudeCode");
        assert_eq!(CodingAgentType::KiloCode.to_string(), "KiloCode");
        assert_eq!(CodingAgentType::OpenCode.to_string(), "OpenCode");
    }

    #[test]
    fn coding_task_serialization() {
        let task = CodingTask {
            description: "Test task".into(),
            context_files: vec!["file.rs".into()],
            model: Some("sonnet".into()),
            max_turns: Some(5),
            permission_mode: Some("acceptEdits".into()),
        };
        let json = serde_json::to_string(&task).unwrap();
        assert!(json.contains("Test task"));
        assert!(json.contains("sonnet"));
    }

    #[test]
    fn coding_result_serialization() {
        let result = CodingResult {
            success: true,
            output: "Done".into(),
            agent: "ClaudeCode".into(),
            duration_ms: 1234,
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("ClaudeCode"));
        assert!(json.contains("1234"));
    }

    #[test]
    fn coding_task_deserialization() {
        let json = r#"{
            "description": "Fix bug",
            "context_files": ["src/main.rs"],
            "model": "haiku",
            "max_turns": 3,
            "permission_mode": "acceptEdits"
        }"#;
        let task: CodingTask = serde_json::from_str(json).unwrap();
        assert_eq!(task.description, "Fix bug");
        assert_eq!(task.context_files.len(), 1);
        assert_eq!(task.model, Some("haiku".into()));
    }

    #[tokio::test]
    async fn manager_list_available_returns_vec() {
        let manager = CodingAgentManager::new();
        let result = manager.list_available().await;
        // In CI/testing, no CLI tools may be available — that's fine
        assert!(result.len() <= 3); // at most 3 agents
    }

    #[test]
    fn validates_safe_permission_modes() {
        assert_eq!(
            validate_permission_mode(Some("acceptEdits")).unwrap(),
            Some("acceptEdits".into())
        );
        assert_eq!(
            validate_permission_mode(Some("read_only")).unwrap(),
            Some("read-only".into())
        );
        assert_eq!(
            validate_permission_mode(Some("ask-before-edits")).unwrap(),
            Some("askBeforeEdits".into())
        );
        assert!(validate_permission_mode(Some("bypassPermissions")).is_err());
        assert!(validate_permission_mode(Some("dangerously-skip-permissions")).is_err());
    }

    #[test]
    fn resolves_project_scoped_working_dir() {
        let dir = tempfile::tempdir().unwrap();
        let resolved = resolve_working_dir(Some(dir.path().to_str().unwrap())).unwrap();
        assert_eq!(resolved, dir.path().canonicalize().unwrap());

        let missing = dir.path().join("missing");
        assert!(resolve_working_dir(Some(missing.to_str().unwrap())).is_err());
        assert!(resolve_working_dir(Some("/")).is_err());
    }

    #[test]
    fn rejects_home_working_dir_when_available() {
        if let Some(home) = home_dir() {
            if home.exists() {
                assert!(resolve_working_dir(Some(home.to_str().unwrap())).is_err());
            }
        }
    }

    #[test]
    fn canonicalizes_context_files_and_rejects_traversal() {
        let dir = tempfile::tempdir().unwrap();
        let nested = dir.path().join("src");
        std::fs::create_dir(&nested).unwrap();
        let file = nested.join("main.rs");
        std::fs::write(&file, "fn main() {}\n").unwrap();

        let cwd = dir.path().canonicalize().unwrap();
        let resolved = resolve_context_files(&cwd, &["src/main.rs".into()]).unwrap();
        assert_eq!(resolved, vec![file.canonicalize().unwrap()]);
        assert!(resolve_context_files(&cwd, &["../secret.txt".into()]).is_err());
        assert!(resolve_context_files(&cwd, &["src/../../secret.txt".into()]).is_err());
    }

    #[test]
    fn injects_context_files_into_prompt() {
        let task = CodingTask {
            description: "Fix the parser".into(),
            context_files: vec![],
            model: Some("sonnet".into()),
            max_turns: Some(4),
            permission_mode: Some("acceptEdits".into()),
        };
        let context = vec![PathBuf::from("/project/src/parser.rs")];
        let prompt = build_prompt(&task, &context, &CodingAgentType::KiloCode);
        assert!(prompt.contains("Fix the parser"));
        assert!(prompt.contains("Context files:"));
        assert!(prompt.contains("- /project/src/parser.rs"));
        assert!(prompt.contains("Requested model: sonnet"));
        assert!(prompt.contains("Requested max turns: 4"));
        assert!(prompt.contains("Requested permission mode: acceptEdits"));
    }

    #[test]
    fn builds_claude_command_with_supported_flags() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("lib.rs");
        std::fs::write(&file, "pub fn x() {}\n").unwrap();
        let task = CodingTask {
            description: "Refactor".into(),
            context_files: vec!["lib.rs".into()],
            model: Some("sonnet".into()),
            max_turns: Some(3),
            permission_mode: Some("plan".into()),
        };

        let spec = build_command_spec(
            &CodingAgentType::ClaudeCode,
            &task,
            Some(dir.path().to_str().unwrap()),
        )
        .unwrap();

        assert_eq!(spec.program, "claude");
        assert_eq!(spec.args[0], "-p");
        assert!(spec.args.windows(2).any(|w| w == ["--model", "sonnet"]));
        assert!(spec.args.windows(2).any(|w| w == ["--max-turns", "3"]));
        assert!(spec
            .args
            .windows(2)
            .any(|w| w == ["--permission-mode", "plan"]));
        assert!(spec.prompt.contains("Context files:"));
        assert!(spec.prompt.contains("lib.rs"));
    }

    #[test]
    fn builds_kilo_command_without_unsupported_flags() {
        let dir = tempfile::tempdir().unwrap();
        let task = CodingTask {
            description: "Implement tests".into(),
            context_files: vec![],
            model: Some("gpt-5".into()),
            max_turns: Some(2),
            permission_mode: Some("ask".into()),
        };

        let spec = build_command_spec(
            &CodingAgentType::KiloCode,
            &task,
            Some(dir.path().to_str().unwrap()),
        )
        .unwrap();

        assert_eq!(spec.program, "kilo");
        assert_eq!(spec.args[0], "run");
        assert!(!spec.args.iter().any(|arg| arg == "--model"));
        assert!(!spec.args.iter().any(|arg| arg == "--permission-mode"));
        assert!(spec.prompt.contains("Requested model: gpt-5"));
        assert!(spec.prompt.contains("Requested permission mode: ask"));
    }

    #[test]
    fn builds_opencode_command_with_model_only() {
        let dir = tempfile::tempdir().unwrap();
        let task = CodingTask {
            description: "Review".into(),
            context_files: vec![],
            model: Some("anthropic/claude-sonnet-4".into()),
            max_turns: Some(9),
            permission_mode: Some("plan".into()),
        };

        let spec = build_command_spec(
            &CodingAgentType::OpenCode,
            &task,
            Some(dir.path().to_str().unwrap()),
        )
        .unwrap();

        assert_eq!(spec.program, "opencode");
        assert_eq!(spec.args[0], "run");
        assert!(spec
            .args
            .windows(2)
            .any(|w| w == ["--model", "anthropic/claude-sonnet-4"]));
        assert!(!spec.args.iter().any(|arg| arg == "--permission-mode"));
        assert!(!spec.args.iter().any(|arg| arg == "--max-turns"));
        assert!(spec.prompt.contains("Requested max turns: 9"));
        assert!(spec.prompt.contains("Requested permission mode: plan"));
    }

    #[tokio::test]
    async fn read_capped_truncates_large_stream() {
        let (mut writer, reader) = tokio::io::duplex(128);
        let input = vec![b'a'; 1024];
        tokio::spawn(async move {
            writer.write_all(&input).await.unwrap();
        });

        let output = read_capped(reader, 256).await.unwrap();
        assert_eq!(output.bytes.len(), 256);
        assert!(output.truncated);
        let rendered = render_capped_output(output);
        assert!(rendered.contains("output truncated"));
    }

    #[test]
    fn timeout_helper_names_terminated_agent() {
        let message = timeout_error(&CodingAgentType::ClaudeCode);
        assert!(message.contains("ClaudeCode"));
        assert!(message.contains("已终止"));
    }
}
