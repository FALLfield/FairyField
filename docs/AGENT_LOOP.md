# Multi-Agent Development Loop

FairyField contains a bounded development-loop backend for project work. It gates success on coding, changed-file, and test evidence, but this is not yet a full multi-agent reasoning system and its current Goal stage does not prove semantic requirement completion.

## Primary Agent Execution Loop

Normal Fairy chat uses the primary agent loop in `src-tauri/src/agent/primary.rs`. On 2026-07-12 this loop adopted the central Hermes execution invariant: a tool result is an observation for the next model turn, not an automatic final response.

The bounded loop now:

- allows up to 12 model/tool iterations and five minutes per user turn;
- continues after web, weather, GitHub, Git, file, and other tool results;
- retains assistant text emitted alongside tool calls without treating it as completion;
- exposes tool failures to the model so it can correct arguments or choose another safe tool;
- caps Unicode observations before returning them to model context;
- blocks a third identical canonical tool call to stop repetition loops;
- adds budget guidance at 75% and 90% usage;
- performs a tool-free, truthful finalization call when the budget is exhausted;
- explicitly requires side-effect completion and artifact read-back before success.

Regression coverage proves `web_search -> file_write -> file_read -> final response`, same-batch sequential execution, tool-error recovery, provider failure before and after tool work, duplicate blocking, Unicode caps, budget thresholds, and a rejected Git mutation flowing through the real `PrimaryAgent -> Toolset -> ToolExecutor` path.

This is improved Hermes-style execution, not complete Hermes parity. The loop still lacks request-scoped cancellation, true incremental token streaming, safe parallel tool batches, provider failover, tool-pair-aware context summarization, and a semantic verifier that independently proves every user requirement.

## Development Loop Gap

The separate Manager/Coding/Testing/Goal development loop is real, but it is not yet the default route for coding requests from ordinary chat. Long coding tasks still need UI routing, Codex support, worktree isolation, cancellation, and requirement-aware Goal verification.

## Roles

| Role | Responsibility |
|------|----------------|
| ManagerAgent | Deterministic Rust planner that converts fields in the request into scoped task records. |
| CodingAgent | Dispatches a configured coding CLI (`claude-code`, `kilocode`, or `opencode`) through `CodingAgentManager`. |
| TestingAgent | Runs allowlisted verification commands without a shell and captures capped stdout/stderr evidence. |
| GoalAgent | Deterministic Rust audit of stage status, changed files, and tests. It does not inspect the final code against each requirement. |

Codex CLI is not currently supported even when it is installed. The coding-agent availability command returns display names such as ClaudeCode, while dispatch parsing expects identifiers such as claude-code, so the list and dispatch interfaces do not round-trip cleanly.

The frontend has typed wrappers for the plan and run commands, but no normal user interface invokes them.

## Task Schema

Each loop plan contains four `AgentTask` records:

```json
{
  "id": "coding-tools",
  "goal_id": "goal-fix-web-search",
  "role": "coding_agent",
  "owner": "tools",
  "scope": "Make the smallest project-scoped code changes needed to satisfy the goal.",
  "files_allowed": ["src-tauri/src/tools/builtins/web.rs"],
  "files_forbidden": [
    "files outside the selected working_dir",
    "unrelated user changes",
    "tracked secrets, API keys, or local model files",
    "Tauri release build artifacts"
  ],
  "context_files": ["src-tauri/src/tools/builtins/web.rs"],
  "acceptance": ["Web search handles Chinese output without byte-index panic"],
  "status": "queued",
  "dependencies": ["manager-scope"],
  "risk": "low",
  "logs": [],
  "artifacts": ["CodingResult"]
}
```

Owners are inferred from context files using the project module map: `renderer`, `voice`, `soul`, `tools`, `memory`, `gateway`, `security`, `platform`, or `integration`.

## Commands

The backend exposes two Tauri commands:

| Command | Purpose |
|---------|---------|
| `development_loop_plan` | Validate and preview the four-agent plan. |
| `development_loop_run` | Execute ManagerAgent, CodingAgent, TestingAgent, and GoalAgent stages. |

Frontend wrappers live in `src/lib/tauri-commands.ts`:

```ts
await developmentLoopPlan(
  'Fix web search UTF-8 truncation',
  ['No byte-index panic on Chinese search results'],
  ['src-tauri/src/tools/builtins/web.rs'],
  ['cargo test'],
  2,
);
```

## Safety Gates

- Test commands are allowlisted and executed directly, not through a shell.
- Shell metacharacters such as `;`, `|`, `&`, backticks, redirects, and newlines are rejected.
- Context files reject `..` path traversal.
- Coding agents inherit the existing `CodingAgentManager` cwd validation, permission-mode allowlist, timeout kill, and output caps.
- CodingAgent output is not trusted by itself. The loop records changed files from `git status --short` and adds them to the stage artifacts.
- If a CodingAgent changes files outside the assigned `files_allowed` scope, the CodingAgent stage fails and the loop stops before tests.
- If tests fail and another iteration is available, the next CodingAgent prompt includes the capped failed test output.
- Rust test commands run from `src-tauri` when the loop starts at the project root; npm commands run from the frontend root.
- The loop never runs `npm run tauri build`.
- Dry runs always return `success: false` because they prove only the plan shape.

## Known Safety And Correctness Gaps

- The changed-file gate snapshots path names, not file contents. Further edits to a file that was already dirty before the loop can be missed.
- GoalAgent can pass after any allowed file changed and unrelated tests passed; it does not map each requirement to behavioral evidence.
- Out-of-scope changes cause a failed report but are not automatically reverted.
- The loop has no worktree isolation, background process monitoring, progress UI, cancel, resume, or rollback.
- At the 2026-07-10 handover, Claude Code was the only installed supported adapter. Codex was installed but unsupported; KiloCode and OpenCode were absent.
- The normal PrimaryAgent never routes long coding requests into this loop.
- CommandGuard approval used by tool execution is separate from the security IPC approval state.

## Completion Rule

A loop report is structurally successful only when:

1. A CodingAgent stage passes.
2. A TestingAgent stage passes.
3. The GoalAgent derives no unmet requirement from stage status and changed-file evidence.
4. Git-detected changed files exist for the CodingAgent stage.
5. Git-detected changed files are inside the assigned scope.
6. The report includes `AgentResult` evidence for the stages.

If any condition is missing, `unmet_requirements` and `next_actions` explain what still needs work. Even when all conditions pass, a human or stronger reviewer must still check the requested behavior against the resulting code.
