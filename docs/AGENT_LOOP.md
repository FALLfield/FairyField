# Multi-Agent Development Loop

FairyField can coordinate a bounded development loop for project work. The loop is intentionally evidence-first: it does not report success until coding and testing stages both produce successful evidence and the goal audit finds no unmet requirements.

## Roles

| Role | Responsibility |
|------|----------------|
| ManagerAgent | Converts the user goal into scoped tasks, owners, allowed files, forbidden files, and verification commands. |
| CodingAgent | Dispatches a configured coding CLI (`claude-code`, `kilocode`, or `opencode`) through `CodingAgentManager`. |
| TestingAgent | Runs allowlisted verification commands without a shell and captures capped stdout/stderr evidence. |
| GoalAgent | Compares requirements against coding and testing evidence before declaring success. |

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
- Rust test commands run from `src-tauri` when the loop starts at the project root; npm commands run from the frontend root.
- The loop never runs `npm run tauri build`.
- Dry runs always return `success: false` because they prove only the plan shape.

## Completion Rule

A loop report is successful only when:

1. A CodingAgent stage passes.
2. A TestingAgent stage passes.
3. The GoalAgent finds no unmet requirements.
4. The report includes `AgentResult` evidence for the stages.

If any condition is missing, `unmet_requirements` and `next_actions` explain what still needs work.
