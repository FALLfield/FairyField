//! Development loop manager
//!
//! Orchestrates manager/coding/testing/goal agents for real project work.

use super::coding_agent::{CodingAgentManager, CodingAgentType, CodingTask};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::process::Stdio;
use tokio::io::AsyncReadExt;
use tokio::time::{timeout, Duration};

const DEFAULT_MAX_ITERATIONS: u8 = 1;
const MAX_ITERATIONS: u8 = 3;
const TEST_TIMEOUT_SECS: u64 = 180;
const MAX_TEST_OUTPUT_BYTES: usize = 128 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LoopAgentRole {
    ManagerAgent,
    CodingAgent,
    TestingAgent,
    GoalAgent,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LoopStageStatus {
    Planned,
    Skipped,
    Passed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AgentTaskStatus {
    Queued,
    Scoping,
    Assigned,
    Coding,
    Testing,
    Integration,
    Blocked,
    Complete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevelopmentLoopRequest {
    pub objective: String,
    #[serde(default)]
    pub requirements: Vec<String>,
    #[serde(default)]
    pub context_files: Vec<String>,
    #[serde(default)]
    pub coding_agent: Option<String>,
    #[serde(default)]
    pub working_dir: Option<String>,
    #[serde(default)]
    pub test_commands: Vec<String>,
    #[serde(default)]
    pub max_iterations: Option<u8>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub permission_mode: Option<String>,
    #[serde(default)]
    pub dry_run: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopAgentPlan {
    pub role: LoopAgentRole,
    pub responsibility: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTask {
    pub id: String,
    pub goal_id: String,
    pub role: LoopAgentRole,
    pub owner: String,
    pub scope: String,
    pub files_allowed: Vec<String>,
    pub files_forbidden: Vec<String>,
    pub context_files: Vec<String>,
    pub acceptance: Vec<String>,
    pub status: AgentTaskStatus,
    pub dependencies: Vec<String>,
    pub risk: String,
    pub logs: Vec<String>,
    pub artifacts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevelopmentLoopPlan {
    pub objective: String,
    pub requirements: Vec<String>,
    pub context_files: Vec<String>,
    pub test_commands: Vec<String>,
    pub max_iterations: u8,
    pub agents: Vec<LoopAgentPlan>,
    pub tasks: Vec<AgentTask>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopStageResult {
    pub role: LoopAgentRole,
    pub status: LoopStageStatus,
    pub summary: String,
    pub artifacts: Vec<String>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResult {
    pub task_id: String,
    pub role: LoopAgentRole,
    pub status: LoopStageStatus,
    pub summary: String,
    pub changed_files: Vec<String>,
    pub artifacts: Vec<String>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevelopmentLoopReport {
    pub success: bool,
    pub dry_run: bool,
    pub iterations: u8,
    pub plan: DevelopmentLoopPlan,
    pub stages: Vec<LoopStageResult>,
    pub agent_results: Vec<AgentResult>,
    pub unmet_requirements: Vec<String>,
    pub next_actions: Vec<String>,
}

pub struct DevelopmentLoopManager {
    coding_manager: CodingAgentManager,
}

impl DevelopmentLoopManager {
    pub fn new() -> Self {
        Self {
            coding_manager: CodingAgentManager::new(),
        }
    }

    pub fn plan(&self, request: &DevelopmentLoopRequest) -> Result<DevelopmentLoopPlan, String> {
        build_loop_plan(request)
    }

    pub async fn run(
        &self,
        request: DevelopmentLoopRequest,
    ) -> Result<DevelopmentLoopReport, String> {
        let plan = self.plan(&request)?;
        let cwd = resolve_loop_working_dir(request.working_dir.as_deref())?;
        let mut stages = Vec::new();
        let mut unmet_requirements = Vec::new();
        let mut next_actions = Vec::new();
        let baseline_changed_files = git_changed_files(&cwd)
            .unwrap_or_default()
            .into_iter()
            .collect::<BTreeSet<_>>();
        let mut previous_test_feedback = Vec::new();

        stages.push(LoopStageResult {
            role: LoopAgentRole::ManagerAgent,
            status: LoopStageStatus::Passed,
            summary: format!(
                "Loop planned with {} requirement(s), {} context file(s), and {} test command(s).",
                plan.requirements.len(),
                plan.context_files.len(),
                plan.test_commands.len()
            ),
            artifacts: vec![cwd.display().to_string()],
            duration_ms: 0,
        });

        if request.dry_run {
            stages.push(LoopStageResult {
                role: LoopAgentRole::CodingAgent,
                status: LoopStageStatus::Skipped,
                summary: "Dry run: coding agent dispatch was not executed.".into(),
                artifacts: vec![coding_prompt(&plan)],
                duration_ms: 0,
            });
            stages.push(LoopStageResult {
                role: LoopAgentRole::TestingAgent,
                status: LoopStageStatus::Skipped,
                summary: "Dry run: test commands were validated but not executed.".into(),
                artifacts: plan.test_commands.clone(),
                duration_ms: 0,
            });
            unmet_requirements.push(
                "Dry run only proves the loop plan shape; implementation and tests were not executed."
                    .into(),
            );
            next_actions
                .push("Run again with dry_run=false after selecting a coding_agent.".into());
            stages.push(goal_stage(
                &plan,
                &stages,
                &unmet_requirements,
                &next_actions,
            ));
            return Ok(DevelopmentLoopReport {
                success: false,
                dry_run: true,
                iterations: 0,
                agent_results: agent_results_from_stages(&plan, &stages),
                plan,
                stages,
                unmet_requirements,
                next_actions,
            });
        }

        let mut iterations = 0;
        let mut coding_was_executed = false;
        let max_iterations = plan.max_iterations;
        for iteration in 1..=max_iterations {
            iterations = iteration;
            if let Some(agent_name) = request.coding_agent.as_deref() {
                let agent = parse_coding_agent(agent_name)?;
                let task = CodingTask {
                    description: coding_prompt_for_iteration(
                        &plan,
                        &previous_test_feedback,
                        iteration,
                    ),
                    context_files: plan.context_files.clone(),
                    model: request.model.clone(),
                    max_turns: Some(6),
                    permission_mode: request.permission_mode.clone().or(Some("ask".into())),
                };
                let start = std::time::Instant::now();
                let cwd_string = cwd.to_string_lossy().to_string();
                match self
                    .coding_manager
                    .dispatch(&agent, &task, Some(cwd_string.as_str()))
                    .await
                {
                    Ok(result) if result.success => {
                        coding_was_executed = true;
                        let changed_files =
                            git_changed_delta(&cwd, &baseline_changed_files).unwrap_or_default();
                        let violations = scope_violations(&plan, &changed_files);
                        let artifacts = coding_artifacts(&result.output, &changed_files);
                        if !violations.is_empty() {
                            stages.push(LoopStageResult {
                                role: LoopAgentRole::CodingAgent,
                                status: LoopStageStatus::Failed,
                                summary: format!(
                                    "{} changed file(s) outside the assigned scope on iteration {}.",
                                    violations.len(),
                                    iteration
                                ),
                                artifacts,
                                duration_ms: result.duration_ms,
                            });
                            unmet_requirements.push(format!(
                                "Coding agent changed files outside scope: {}",
                                violations.join(", ")
                            ));
                            break;
                        }
                        stages.push(LoopStageResult {
                            role: LoopAgentRole::CodingAgent,
                            status: LoopStageStatus::Passed,
                            summary: format!("{} completed iteration {}.", result.agent, iteration),
                            artifacts,
                            duration_ms: result.duration_ms,
                        });
                    }
                    Ok(result) => {
                        stages.push(LoopStageResult {
                            role: LoopAgentRole::CodingAgent,
                            status: LoopStageStatus::Failed,
                            summary: format!("{} failed iteration {}.", result.agent, iteration),
                            artifacts: vec![truncate_artifact(&result.output)],
                            duration_ms: result.duration_ms,
                        });
                        unmet_requirements.push(format!(
                            "Coding agent failed before tests on iteration {}.",
                            iteration
                        ));
                        break;
                    }
                    Err(error) => {
                        stages.push(LoopStageResult {
                            role: LoopAgentRole::CodingAgent,
                            status: LoopStageStatus::Failed,
                            summary: format!("Coding agent could not run: {error}"),
                            artifacts: vec![agent.to_string()],
                            duration_ms: start.elapsed().as_millis() as u64,
                        });
                        unmet_requirements.push("Coding agent execution failed.".into());
                        next_actions.push("Install/configure the selected coding agent CLI or choose another one.".into());
                        break;
                    }
                }
            } else {
                stages.push(LoopStageResult {
                    role: LoopAgentRole::CodingAgent,
                    status: LoopStageStatus::Skipped,
                    summary: "No coding_agent was selected, so no code edits were attempted."
                        .into(),
                    artifacts: vec![coding_prompt(&plan)],
                    duration_ms: 0,
                });
                unmet_requirements
                    .push("No coding agent was selected to operate the real task.".into());
                break;
            }

            let test_result = run_test_commands(&cwd, &plan.test_commands).await;
            let tests_passed = matches!(test_result.status, LoopStageStatus::Passed);
            if !tests_passed {
                previous_test_feedback = test_result.artifacts.clone();
            }
            stages.push(test_result);
            if tests_passed {
                break;
            }
            if iteration == max_iterations {
                unmet_requirements.push(format!(
                    "Tests still failed after {} iteration(s).",
                    max_iterations
                ));
            }
        }

        if !coding_was_executed && !unmet_requirements.iter().any(|u| u.contains("coding")) {
            unmet_requirements.push("No coding agent completed a task.".into());
        }

        unmet_requirements.extend(goal_unmet_requirements(&plan, &stages));
        next_actions.extend(goal_next_actions(&unmet_requirements));
        let goal = goal_stage(&plan, &stages, &unmet_requirements, &next_actions);
        stages.push(goal);
        let success = unmet_requirements.is_empty()
            && stages.iter().any(|stage| {
                stage.role == LoopAgentRole::CodingAgent && stage.status == LoopStageStatus::Passed
            })
            && stages.iter().any(|stage| {
                stage.role == LoopAgentRole::TestingAgent && stage.status == LoopStageStatus::Passed
            });

        Ok(DevelopmentLoopReport {
            success,
            dry_run: false,
            iterations,
            agent_results: agent_results_from_stages(&plan, &stages),
            plan,
            stages,
            unmet_requirements,
            next_actions,
        })
    }
}

impl Default for DevelopmentLoopManager {
    fn default() -> Self {
        Self::new()
    }
}

fn build_loop_plan(request: &DevelopmentLoopRequest) -> Result<DevelopmentLoopPlan, String> {
    let objective = request.objective.trim();
    if objective.is_empty() {
        return Err("development loop objective cannot be empty".into());
    }

    let requirements = normalize_requirements(&request.requirements, objective);
    let context_files = normalize_context_files(&request.context_files)?;
    let test_commands = normalize_test_commands(&request.test_commands)?;
    let max_iterations = request
        .max_iterations
        .unwrap_or(DEFAULT_MAX_ITERATIONS)
        .clamp(1, MAX_ITERATIONS);
    let goal_id = goal_id_for(objective);
    let owner = infer_owner(&context_files);

    Ok(DevelopmentLoopPlan {
        objective: objective.to_string(),
        max_iterations,
        agents: vec![
            LoopAgentPlan {
                role: LoopAgentRole::ManagerAgent,
                responsibility: "Plan the loop, assign stages, preserve scope, and stop unsafe work."
                    .into(),
                inputs: vec!["objective".into(), "requirements".into(), "context_files".into()],
                outputs: vec!["DevelopmentLoopPlan".into()],
            },
            LoopAgentPlan {
                role: LoopAgentRole::CodingAgent,
                responsibility:
                    "Operate the real task through a configured coding CLI and report changed files."
                        .into(),
                inputs: vec!["CodingTask".into(), "working_dir".into()],
                outputs: vec!["CodingResult".into()],
            },
            LoopAgentPlan {
                role: LoopAgentRole::TestingAgent,
                responsibility:
                    "Run allowlisted verification commands without shell metacharacters."
                        .into(),
                inputs: vec!["test_commands".into()],
                outputs: vec!["test outputs".into()],
            },
            LoopAgentPlan {
                role: LoopAgentRole::GoalAgent,
                responsibility:
                    "Compare evidence against requirements and return unmet requirements."
                        .into(),
                inputs: vec!["stage results".into(), "requirements".into()],
                outputs: vec!["goal verdict".into()],
            },
        ],
        tasks: build_agent_tasks(
            &goal_id,
            objective,
            &owner,
            &context_files,
            &requirements,
            &test_commands,
        ),
        requirements,
        context_files,
        test_commands,
    })
}

fn build_agent_tasks(
    goal_id: &str,
    objective: &str,
    owner: &str,
    context_files: &[String],
    requirements: &[String],
    test_commands: &[String],
) -> Vec<AgentTask> {
    let allowed = files_allowed(owner, context_files);
    let forbidden = files_forbidden();
    vec![
        AgentTask {
            id: "manager-scope".into(),
            goal_id: goal_id.into(),
            role: LoopAgentRole::ManagerAgent,
            owner: "manager".into(),
            scope: "Convert the objective into bounded coding, testing, and goal-audit work."
                .into(),
            files_allowed: vec!["planning metadata only".into()],
            files_forbidden: forbidden.clone(),
            context_files: context_files.to_vec(),
            acceptance: vec![
                "Plan contains manager, coding, testing, and goal agents.".into(),
                "Plan records owner, file scope, requirements, and verification commands.".into(),
            ],
            status: AgentTaskStatus::Scoping,
            dependencies: vec![],
            risk: "low".into(),
            logs: vec![format!("objective: {objective}")],
            artifacts: vec!["DevelopmentLoopPlan".into()],
        },
        AgentTask {
            id: format!("coding-{owner}"),
            goal_id: goal_id.into(),
            role: LoopAgentRole::CodingAgent,
            owner: owner.into(),
            scope: "Make the smallest project-scoped code changes needed to satisfy the goal."
                .into(),
            files_allowed: allowed,
            files_forbidden: forbidden.clone(),
            context_files: context_files.to_vec(),
            acceptance: requirements.to_vec(),
            status: AgentTaskStatus::Queued,
            dependencies: vec!["manager-scope".into()],
            risk: if owner == "integration" {
                "medium".into()
            } else {
                "low".into()
            },
            logs: vec![],
            artifacts: vec!["CodingResult".into()],
        },
        AgentTask {
            id: "testing-regression".into(),
            goal_id: goal_id.into(),
            role: LoopAgentRole::TestingAgent,
            owner: "testing".into(),
            scope: "Run allowlisted verification commands and capture capped output evidence."
                .into(),
            files_allowed: vec!["read-only test execution".into()],
            files_forbidden: forbidden.clone(),
            context_files: context_files.to_vec(),
            acceptance: test_commands.to_vec(),
            status: AgentTaskStatus::Queued,
            dependencies: vec![format!("coding-{owner}")],
            risk: "low".into(),
            logs: vec![],
            artifacts: vec!["test outputs".into()],
        },
        AgentTask {
            id: "goal-audit".into(),
            goal_id: goal_id.into(),
            role: LoopAgentRole::GoalAgent,
            owner: "goal".into(),
            scope:
                "Compare requirements with coding and testing evidence before declaring success."
                    .into(),
            files_allowed: vec!["read-only evidence review".into()],
            files_forbidden: forbidden,
            context_files: context_files.to_vec(),
            acceptance: requirements.to_vec(),
            status: AgentTaskStatus::Queued,
            dependencies: vec!["testing-regression".into()],
            risk: "low".into(),
            logs: vec![],
            artifacts: vec!["goal verdict".into()],
        },
    ]
}

fn normalize_requirements(input: &[String], objective: &str) -> Vec<String> {
    let mut requirements = input
        .iter()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    if requirements.is_empty() {
        requirements.push(objective.to_string());
    }
    requirements
}

fn normalize_context_files(files: &[String]) -> Result<Vec<String>, String> {
    files
        .iter()
        .map(|file| {
            let trimmed = file.trim();
            if trimmed.is_empty() || trimmed.contains('\0') {
                return Err("context_files contains an empty path".into());
            }
            let path = Path::new(trimmed);
            if path.is_absolute() {
                return Err(format!(
                    "context_files must be project-relative paths: {trimmed}"
                ));
            }
            if path
                .components()
                .any(|component| matches!(component, Component::ParentDir))
            {
                return Err(format!("context_files path traversal rejected: {trimmed}"));
            }
            Ok(trimmed.to_string())
        })
        .collect()
}

fn normalize_test_commands(commands: &[String]) -> Result<Vec<String>, String> {
    let commands = commands
        .iter()
        .map(|command| command.trim())
        .filter(|command| !command.is_empty())
        .collect::<Vec<_>>();

    if commands.is_empty() {
        return Ok(vec![
            "npm run test".into(),
            "npm run build".into(),
            "cargo test".into(),
        ]);
    }

    commands
        .into_iter()
        .map(|command| {
            validate_test_command(command)?;
            Ok(command.to_string())
        })
        .collect()
}

fn validate_test_command(command: &str) -> Result<(), String> {
    if command
        .chars()
        .any(|ch| matches!(ch, ';' | '|' | '&' | '`' | '$' | '<' | '>' | '\n' | '\r'))
    {
        return Err(format!(
            "test command contains unsafe shell syntax: {command}"
        ));
    }

    let allowed = [
        "npm run test",
        "npm run build",
        "cargo fmt --check",
        "cargo check",
        "cargo test",
        "cargo clippy -- -D warnings",
        "cargo check --features sherpa-onnx",
        "cargo test --features sherpa-onnx",
        "cargo clippy --features sherpa-onnx -- -D warnings",
    ];
    if allowed.contains(&command) {
        Ok(())
    } else {
        Err(format!("test command is not allowlisted: {command}"))
    }
}

fn parse_coding_agent(agent_type: &str) -> Result<CodingAgentType, String> {
    match agent_type.trim() {
        "claude-code" => Ok(CodingAgentType::ClaudeCode),
        "kilocode" => Ok(CodingAgentType::KiloCode),
        "opencode" => Ok(CodingAgentType::OpenCode),
        other => Err(format!("unknown coding agent: {other}")),
    }
}

fn resolve_loop_working_dir(working_dir: Option<&str>) -> Result<PathBuf, String> {
    let raw = match working_dir {
        Some(dir) if !dir.trim().is_empty() => PathBuf::from(dir.trim()),
        _ => std::env::current_dir().map_err(|e| format!("cannot read current dir: {e}"))?,
    };
    let canonical = raw
        .canonicalize()
        .map_err(|e| format!("working_dir is not accessible: {} ({e})", raw.display()))?;
    if !canonical.is_dir() {
        return Err(format!(
            "working_dir is not a directory: {}",
            canonical.display()
        ));
    }
    if canonical.parent().is_none() {
        return Err("refusing to run a development loop at filesystem root".into());
    }
    Ok(canonical)
}

fn coding_prompt(plan: &DevelopmentLoopPlan) -> String {
    format!(
        "You are the CodingAgent in FairyField's development loop.\n\nObjective:\n{}\n\nRequirements:\n{}\n\nContext files:\n{}\n\nRules:\n- Make scoped edits only.\n- Do not revert unrelated user changes.\n- Report changed files and verification evidence.\n",
        plan.objective,
        plan.requirements
            .iter()
            .map(|requirement| format!("- {requirement}"))
            .collect::<Vec<_>>()
            .join("\n"),
        if plan.context_files.is_empty() {
            "- none".into()
        } else {
            plan.context_files
                .iter()
                .map(|file| format!("- {file}"))
                .collect::<Vec<_>>()
                .join("\n")
        }
    )
}

fn coding_prompt_for_iteration(
    plan: &DevelopmentLoopPlan,
    previous_test_feedback: &[String],
    iteration: u8,
) -> String {
    let mut prompt = coding_prompt(plan);
    if !previous_test_feedback.is_empty() {
        prompt.push_str(&format!(
            "\n\nPrevious test failure from iteration {}:\n{}\n\nUse this feedback to fix the same task, then report changed files and verification evidence.",
            iteration.saturating_sub(1),
            previous_test_feedback
                .iter()
                .map(|artifact| truncate_artifact(artifact))
                .collect::<Vec<_>>()
                .join("\n\n")
        ));
    }
    prompt
}

async fn run_test_commands(cwd: &Path, commands: &[String]) -> LoopStageResult {
    let start = std::time::Instant::now();
    if commands.is_empty() {
        return LoopStageResult {
            role: LoopAgentRole::TestingAgent,
            status: LoopStageStatus::Skipped,
            summary: "No test commands were configured.".into(),
            artifacts: vec![],
            duration_ms: 0,
        };
    }

    let mut outputs = Vec::new();
    for command in commands {
        if let Err(error) = validate_test_command(command) {
            return LoopStageResult {
                role: LoopAgentRole::TestingAgent,
                status: LoopStageStatus::Failed,
                summary: error,
                artifacts: outputs,
                duration_ms: start.elapsed().as_millis() as u64,
            };
        }
        match run_one_test_command(cwd, command).await {
            Ok(output) => outputs.push(format!("{command}\n{output}")),
            Err(error) => {
                outputs.push(format!("{command}\n{error}"));
                return LoopStageResult {
                    role: LoopAgentRole::TestingAgent,
                    status: LoopStageStatus::Failed,
                    summary: format!("Test command failed: {command}"),
                    artifacts: outputs.into_iter().map(|o| truncate_artifact(&o)).collect(),
                    duration_ms: start.elapsed().as_millis() as u64,
                };
            }
        }
    }

    LoopStageResult {
        role: LoopAgentRole::TestingAgent,
        status: LoopStageStatus::Passed,
        summary: format!("{} test command(s) passed.", commands.len()),
        artifacts: outputs.into_iter().map(|o| truncate_artifact(&o)).collect(),
        duration_ms: start.elapsed().as_millis() as u64,
    }
}

async fn run_one_test_command(cwd: &Path, command: &str) -> Result<String, String> {
    let mut parts = command.split_whitespace();
    let program = parts
        .next()
        .ok_or_else(|| "test command cannot be empty".to_string())?;
    let args = parts.collect::<Vec<_>>();
    let command_dir = command_working_dir(cwd, command);

    let mut child = tokio::process::Command::new(program)
        .args(args)
        .current_dir(command_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| format!("failed to start test command: {e}"))?;

    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| "stdout pipe unavailable".to_string())?;
    let mut stderr = child
        .stderr
        .take()
        .ok_or_else(|| "stderr pipe unavailable".to_string())?;
    let stdout_task = tokio::spawn(async move { read_limited(&mut stdout).await });
    let stderr_task = tokio::spawn(async move { read_limited(&mut stderr).await });

    let status = match timeout(Duration::from_secs(TEST_TIMEOUT_SECS), child.wait()).await {
        Ok(result) => result.map_err(|e| format!("failed to wait for command: {e}"))?,
        Err(_) => {
            let _ = child.kill().await;
            let _ = child.wait().await;
            return Err(format!("test command timed out after {TEST_TIMEOUT_SECS}s"));
        }
    };

    let stdout = stdout_task
        .await
        .map_err(|e| format!("failed to join stdout task: {e}"))?
        .map_err(|e| format!("failed to read stdout: {e}"))?;
    let stderr = stderr_task
        .await
        .map_err(|e| format!("failed to join stderr task: {e}"))?
        .map_err(|e| format!("failed to read stderr: {e}"))?;
    let output = if stderr.trim().is_empty() {
        stdout
    } else if stdout.trim().is_empty() {
        stderr
    } else {
        format!("{stdout}\n{stderr}")
    };

    if status.success() {
        Ok(output)
    } else {
        Err(output)
    }
}

async fn read_limited<R>(reader: &mut R) -> std::io::Result<String>
where
    R: tokio::io::AsyncRead + Unpin,
{
    let mut output = Vec::new();
    let mut buf = [0u8; 8192];
    loop {
        let read = reader.read(&mut buf).await?;
        if read == 0 {
            break;
        }
        let remaining = MAX_TEST_OUTPUT_BYTES.saturating_sub(output.len());
        if remaining == 0 {
            break;
        }
        output.extend_from_slice(&buf[..read.min(remaining)]);
    }
    Ok(String::from_utf8_lossy(&output).to_string())
}

fn command_working_dir(cwd: &Path, command: &str) -> PathBuf {
    if command.starts_with("cargo ")
        && !cwd.join("Cargo.toml").exists()
        && cwd.join("src-tauri").join("Cargo.toml").exists()
    {
        cwd.join("src-tauri")
    } else {
        cwd.to_path_buf()
    }
}

fn git_changed_delta(cwd: &Path, baseline: &BTreeSet<String>) -> Result<Vec<String>, String> {
    Ok(git_changed_files(cwd)?
        .into_iter()
        .filter(|file| !baseline.contains(file))
        .collect())
}

fn git_changed_files(cwd: &Path) -> Result<Vec<String>, String> {
    let output = Command::new("git")
        .args(["status", "--short", "--untracked-files=all"])
        .current_dir(cwd)
        .output()
        .map_err(|e| format!("failed to run git status: {e}"))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }
    Ok(parse_git_status_paths(&String::from_utf8_lossy(
        &output.stdout,
    )))
}

fn parse_git_status_paths(output: &str) -> Vec<String> {
    let mut files = BTreeSet::new();
    for line in output.lines() {
        let Some(path_part) = line.get(3..) else {
            continue;
        };
        for path in path_part.split(" -> ") {
            let cleaned = path.trim().trim_matches('"');
            if looks_like_git_status_path(cleaned) {
                files.insert(cleaned.to_string());
            }
        }
    }
    files.into_iter().collect()
}

fn looks_like_git_status_path(value: &str) -> bool {
    !value.is_empty()
        && !value.contains('\0')
        && !value.contains("..")
        && !Path::new(value).is_absolute()
}

fn coding_artifacts(output: &str, changed_files: &[String]) -> Vec<String> {
    let mut artifacts = vec![truncate_artifact(output)];
    let changed = if changed_files.is_empty() {
        "Changed files:\n- none detected by git status".into()
    } else {
        format!(
            "Changed files:\n{}",
            changed_files
                .iter()
                .map(|file| format!("- {file}"))
                .collect::<Vec<_>>()
                .join("\n")
        )
    };
    artifacts.push(changed);
    artifacts
}

fn goal_unmet_requirements(plan: &DevelopmentLoopPlan, stages: &[LoopStageResult]) -> Vec<String> {
    let mut unmet = Vec::new();
    if !stages.iter().any(|stage| {
        stage.role == LoopAgentRole::CodingAgent && stage.status == LoopStageStatus::Passed
    }) {
        unmet.push("No successful CodingAgent stage is available as evidence.".into());
    }
    if !stages.iter().any(|stage| {
        stage.role == LoopAgentRole::TestingAgent && stage.status == LoopStageStatus::Passed
    }) {
        unmet.push("No successful TestingAgent stage is available as evidence.".into());
    }
    if plan.requirements.is_empty() {
        unmet.push("No explicit requirement was available for the GoalAgent to verify.".into());
    }
    if stages.iter().any(|stage| {
        stage.role == LoopAgentRole::CodingAgent && stage.status == LoopStageStatus::Passed
    }) {
        let changed_files = changed_files_from_stages(stages);
        if changed_files.is_empty() {
            unmet.push(
                "No Git-detected changed files are available as CodingAgent evidence.".into(),
            );
        }
        let violations = scope_violations(plan, &changed_files);
        if !violations.is_empty() {
            unmet.push(format!(
                "CodingAgent evidence includes files outside scope: {}",
                violations.join(", ")
            ));
        }
    }
    unmet
}

fn changed_files_from_stages(stages: &[LoopStageResult]) -> Vec<String> {
    stages
        .iter()
        .filter(|stage| stage.role == LoopAgentRole::CodingAgent)
        .flat_map(|stage| extract_changed_files(&stage.artifacts))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn scope_violations(plan: &DevelopmentLoopPlan, files: &[String]) -> Vec<String> {
    let allowed = plan
        .tasks
        .iter()
        .find(|task| task.role == LoopAgentRole::CodingAgent)
        .map(|task| task.files_allowed.as_slice())
        .unwrap_or(&[]);

    files
        .iter()
        .filter(|file| !file_allowed_by_scope(file, allowed))
        .cloned()
        .collect()
}

fn file_allowed_by_scope(file: &str, allowed: &[String]) -> bool {
    if !looks_like_project_file(file) {
        return false;
    }
    allowed.iter().any(|scope| {
        scope == "project-scoped files required by objective"
            || file == scope
            || scope.ends_with('/') && file.starts_with(scope)
    })
}

fn agent_results_from_stages(
    plan: &DevelopmentLoopPlan,
    stages: &[LoopStageResult],
) -> Vec<AgentResult> {
    stages
        .iter()
        .map(|stage| {
            let task_id = plan
                .tasks
                .iter()
                .find(|task| task.role == stage.role)
                .map(|task| task.id.clone())
                .unwrap_or_else(|| format!("{:?}", stage.role));
            AgentResult {
                task_id,
                role: stage.role.clone(),
                status: stage.status.clone(),
                summary: stage.summary.clone(),
                changed_files: extract_changed_files(&stage.artifacts),
                artifacts: stage.artifacts.clone(),
                duration_ms: stage.duration_ms,
            }
        })
        .collect()
}

fn extract_changed_files(artifacts: &[String]) -> Vec<String> {
    let mut files = BTreeSet::new();
    for artifact in artifacts {
        for line in artifact.lines() {
            let trimmed = line
                .trim()
                .trim_start_matches(['-', '*', '`', ' '])
                .trim_end_matches('`');
            if looks_like_project_file(trimmed) {
                files.insert(trimmed.to_string());
            }
        }
    }
    files.into_iter().collect()
}

fn looks_like_project_file(value: &str) -> bool {
    let allowed_roots = [
        "src/",
        "src-tauri/",
        "docs/",
        "scripts/",
        "public/",
        ".github/",
        "README.md",
        "CHANGELOG.md",
        "CLAUDE.md",
        "TEST_MANUAL.md",
    ];
    !value.contains('\0')
        && !value.contains("..")
        && allowed_roots.iter().any(|root| value.starts_with(root))
}

fn goal_id_for(objective: &str) -> String {
    let normalized = objective
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>();
    let slug = normalized
        .split('-')
        .filter(|part| !part.is_empty())
        .take(6)
        .collect::<Vec<_>>()
        .join("-");
    if slug.is_empty() {
        "goal-development-loop".into()
    } else {
        format!("goal-{slug}")
    }
}

fn infer_owner(context_files: &[String]) -> String {
    let owners = context_files
        .iter()
        .map(|file| owner_for_file(file))
        .collect::<BTreeSet<_>>();
    match owners.len() {
        0 => "integration".into(),
        1 => owners.into_iter().next().unwrap_or("integration").into(),
        _ => "integration".into(),
    }
}

fn owner_for_file(file: &str) -> &'static str {
    if file.starts_with("src/renderers/")
        || file.starts_with("src/modules/")
        || file == "src/components/CharacterCanvas.vue"
    {
        "renderer"
    } else if file.starts_with("src-tauri/src/voice/") {
        "voice"
    } else if file.starts_with("src-tauri/src/agent/")
        || file.starts_with("src-tauri/src/llm/")
        || file.starts_with("soul/")
    {
        "soul"
    } else if file.starts_with("src-tauri/src/tools/") {
        "tools"
    } else if file.starts_with("src-tauri/src/memory/")
        || file.starts_with("src-tauri/src/growth/")
        || file.starts_with("src-tauri/src/mcp/")
    {
        "memory"
    } else if file.starts_with("src-tauri/src/gateway/") {
        "gateway"
    } else if file.starts_with("src-tauri/src/security/") {
        "security"
    } else {
        "platform"
    }
}

fn files_allowed(owner: &str, context_files: &[String]) -> Vec<String> {
    if !context_files.is_empty() {
        return context_files.to_vec();
    }
    match owner {
        "renderer" => vec!["src/renderers/".into(), "src/modules/".into()],
        "voice" => vec!["src-tauri/src/voice/".into()],
        "soul" => vec!["src-tauri/src/agent/".into(), "src-tauri/src/llm/".into()],
        "tools" => vec!["src-tauri/src/tools/".into()],
        "memory" => vec!["src-tauri/src/memory/".into(), "src-tauri/src/mcp/".into()],
        "gateway" => vec!["src-tauri/src/gateway/".into()],
        "security" => vec!["src-tauri/src/security/".into()],
        "platform" => vec!["src-tauri/src/lib.rs".into(), "src/lib/".into()],
        _ => vec!["project-scoped files required by objective".into()],
    }
}

fn files_forbidden() -> Vec<String> {
    vec![
        "files outside the selected working_dir".into(),
        "unrelated user changes".into(),
        "tracked secrets, API keys, or local model files".into(),
        "Tauri release build artifacts".into(),
    ]
}

fn goal_next_actions(unmet_requirements: &[String]) -> Vec<String> {
    if unmet_requirements.is_empty() {
        vec!["Prepare the branch for review or release.".into()]
    } else {
        unmet_requirements
            .iter()
            .map(|item| format!("Resolve: {item}"))
            .collect()
    }
}

fn goal_stage(
    plan: &DevelopmentLoopPlan,
    stages: &[LoopStageResult],
    unmet_requirements: &[String],
    next_actions: &[String],
) -> LoopStageResult {
    let passed = unmet_requirements.is_empty();
    LoopStageResult {
        role: LoopAgentRole::GoalAgent,
        status: if passed {
            LoopStageStatus::Passed
        } else {
            LoopStageStatus::Failed
        },
        summary: if passed {
            format!(
                "GoalAgent verified {} requirement(s) against coding and test evidence.",
                plan.requirements.len()
            )
        } else {
            format!(
                "GoalAgent found {} unmet requirement(s) across {} stage(s).",
                unmet_requirements.len(),
                stages.len()
            )
        },
        artifacts: if passed {
            next_actions.to_vec()
        } else {
            unmet_requirements.to_vec()
        },
        duration_ms: 0,
    }
}

fn truncate_artifact(text: &str) -> String {
    let mut output = text.chars().take(4_000).collect::<String>();
    if text.chars().count() > 4_000 {
        output.push_str("\n[artifact truncated]");
    }
    output
}

#[tauri::command]
pub async fn development_loop_plan(
    objective: String,
    requirements: Vec<String>,
    context_files: Vec<String>,
    test_commands: Vec<String>,
    max_iterations: Option<u8>,
) -> Result<DevelopmentLoopPlan, String> {
    let request = DevelopmentLoopRequest {
        objective,
        requirements,
        context_files,
        coding_agent: None,
        working_dir: None,
        test_commands,
        max_iterations,
        model: None,
        permission_mode: None,
        dry_run: true,
    };
    DevelopmentLoopManager::new().plan(&request)
}

#[tauri::command]
pub async fn development_loop_run(
    request: DevelopmentLoopRequest,
) -> Result<DevelopmentLoopReport, String> {
    DevelopmentLoopManager::new().run(request).await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> DevelopmentLoopRequest {
        DevelopmentLoopRequest {
            objective: "Fix the flaky web search tool".into(),
            requirements: vec![
                "Coding agent must operate on scoped files".into(),
                "Testing agent must run regression tests".into(),
                "Goal agent must report unmet requirements".into(),
            ],
            context_files: vec!["src-tauri/src/tools/builtins/web.rs".into()],
            coding_agent: Some("claude-code".into()),
            working_dir: None,
            test_commands: vec!["cargo test".into()],
            max_iterations: Some(2),
            model: Some("sonnet".into()),
            permission_mode: Some("ask".into()),
            dry_run: true,
        }
    }

    #[test]
    fn builds_four_agent_plan() {
        let manager = DevelopmentLoopManager::new();
        let plan = manager.plan(&request()).unwrap();

        assert_eq!(plan.max_iterations, 2);
        assert_eq!(plan.agents.len(), 4);
        assert_eq!(plan.tasks.len(), 4);
        assert!(plan
            .tasks
            .iter()
            .any(|task| task.role == LoopAgentRole::CodingAgent && task.owner == "tools"));
        assert!(plan
            .agents
            .iter()
            .any(|agent| agent.role == LoopAgentRole::ManagerAgent));
        assert!(plan
            .agents
            .iter()
            .any(|agent| agent.role == LoopAgentRole::CodingAgent));
        assert!(plan
            .agents
            .iter()
            .any(|agent| agent.role == LoopAgentRole::TestingAgent));
        assert!(plan
            .agents
            .iter()
            .any(|agent| agent.role == LoopAgentRole::GoalAgent));
    }

    #[test]
    fn rejects_unsafe_context_and_test_commands() {
        let mut req = request();
        req.context_files = vec!["../secret.txt".into()];
        assert!(DevelopmentLoopManager::new().plan(&req).is_err());

        let mut req = request();
        req.context_files = vec!["/tmp/secret.txt".into()];
        assert!(DevelopmentLoopManager::new().plan(&req).is_err());

        let mut req = request();
        req.test_commands = vec!["cargo test; rm -rf /".into()];
        assert!(DevelopmentLoopManager::new().plan(&req).is_err());

        let mut req = request();
        req.test_commands = vec!["rm -rf target".into()];
        assert!(DevelopmentLoopManager::new().plan(&req).is_err());
    }

    #[tokio::test]
    async fn dry_run_reports_unproven_goal() {
        let report = DevelopmentLoopManager::new().run(request()).await.unwrap();

        assert!(!report.success);
        assert!(report.dry_run);
        assert_eq!(report.iterations, 0);
        assert!(report
            .stages
            .iter()
            .any(|stage| stage.role == LoopAgentRole::GoalAgent
                && stage.status == LoopStageStatus::Failed));
        assert!(report
            .unmet_requirements
            .iter()
            .any(|item| item.contains("Dry run")));
    }

    #[test]
    fn clamps_iteration_count() {
        let mut req = request();
        req.max_iterations = Some(99);
        let plan = DevelopmentLoopManager::new().plan(&req).unwrap();
        assert_eq!(plan.max_iterations, MAX_ITERATIONS);
    }

    #[test]
    fn default_tests_are_release_relevant() {
        let mut req = request();
        req.test_commands.clear();
        let plan = DevelopmentLoopManager::new().plan(&req).unwrap();
        assert!(plan.test_commands.contains(&"npm run test".into()));
        assert!(plan.test_commands.contains(&"npm run build".into()));
        assert!(plan.test_commands.contains(&"cargo test".into()));
    }

    #[test]
    fn routes_command_working_dirs_by_toolchain() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("src-tauri has project parent");
        assert!(command_working_dir(root, "cargo test").ends_with("src-tauri"));
        assert_eq!(command_working_dir(root, "npm run test"), root);
    }

    #[test]
    fn extracts_project_files_from_agent_artifacts() {
        let files = extract_changed_files(&[
            "- src-tauri/src/agent/development_loop.rs".into(),
            "`docs/AGENT_LOOP.md`".into(),
            "/tmp/secret.txt".into(),
            "../outside.md".into(),
        ]);
        assert_eq!(
            files,
            vec![
                "docs/AGENT_LOOP.md".to_string(),
                "src-tauri/src/agent/development_loop.rs".to_string()
            ]
        );
    }

    #[test]
    fn prompt_carries_failed_test_feedback_to_next_iteration() {
        let plan = DevelopmentLoopManager::new().plan(&request()).unwrap();
        let prompt = coding_prompt_for_iteration(
            &plan,
            &["cargo test\nthread failed at web.rs:42".into()],
            2,
        );

        assert!(prompt.contains("Previous test failure from iteration 1"));
        assert!(prompt.contains("thread failed at web.rs:42"));
        assert!(prompt.contains("Use this feedback"));
    }

    #[test]
    fn parses_git_status_paths_as_loop_evidence() {
        let output = concat!(
            " M src-tauri/src/agent/development_loop.rs\n",
            "?? docs/AGENT_LOOP.md\n",
            "R  src/old.ts -> src/new.ts\n",
            "?? .env\n",
            "?? /tmp/secret.txt\n"
        );
        let files = parse_git_status_paths(output);

        assert_eq!(
            files,
            vec![
                ".env".to_string(),
                "docs/AGENT_LOOP.md".to_string(),
                "src-tauri/src/agent/development_loop.rs".to_string(),
                "src/new.ts".to_string(),
                "src/old.ts".to_string()
            ]
        );
    }

    #[test]
    fn detects_loop_scope_violations() {
        let plan = DevelopmentLoopManager::new().plan(&request()).unwrap();
        let violations = scope_violations(
            &plan,
            &[
                "src-tauri/src/tools/builtins/web.rs".into(),
                "src-tauri/src/voice/tts.rs".into(),
            ],
        );

        assert_eq!(violations, vec!["src-tauri/src/voice/tts.rs".to_string()]);
    }

    #[test]
    fn goal_requires_git_changed_file_evidence() {
        let plan = DevelopmentLoopManager::new().plan(&request()).unwrap();
        let stages = vec![
            LoopStageResult {
                role: LoopAgentRole::CodingAgent,
                status: LoopStageStatus::Passed,
                summary: "agent claimed success".into(),
                artifacts: vec!["all good".into()],
                duration_ms: 1,
            },
            LoopStageResult {
                role: LoopAgentRole::TestingAgent,
                status: LoopStageStatus::Passed,
                summary: "tests passed".into(),
                artifacts: vec!["cargo test ok".into()],
                duration_ms: 1,
            },
        ];

        let unmet = goal_unmet_requirements(&plan, &stages);
        assert!(unmet
            .iter()
            .any(|item| item.contains("No Git-detected changed files")));
    }

    #[test]
    fn goal_accepts_scoped_changed_file_evidence() {
        let plan = DevelopmentLoopManager::new().plan(&request()).unwrap();
        let stages = vec![
            LoopStageResult {
                role: LoopAgentRole::CodingAgent,
                status: LoopStageStatus::Passed,
                summary: "agent completed".into(),
                artifacts: vec!["Changed files:\n- src-tauri/src/tools/builtins/web.rs".into()],
                duration_ms: 1,
            },
            LoopStageResult {
                role: LoopAgentRole::TestingAgent,
                status: LoopStageStatus::Passed,
                summary: "tests passed".into(),
                artifacts: vec!["cargo test ok".into()],
                duration_ms: 1,
            },
        ];

        assert!(goal_unmet_requirements(&plan, &stages).is_empty());
    }
}
