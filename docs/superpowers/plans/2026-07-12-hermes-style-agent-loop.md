# Hermes-Style Primary Agent Loop Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make Fairy reliably continue from research and other tool observations to requested side effects and verified completion.

**Architecture:** Keep `PrimaryAgent::run_agent_loop` as the orchestration boundary and adopt Hermes's central invariant: every tool result returns to the model and only a visible no-tool response completes the turn. Add bounded retries, duplicate-call protection, budget pressure, Unicode-safe observation caps, and a tool-free finalization call at the hard limit.

**Tech Stack:** Rust 2021, Tokio, Tauri v2, serde_json, existing `LlmProvider` and `Toolset` traits.

---

### Task 1: Multi-Step Loop Regression Harness

**Files:**
- Modify: `src-tauri/src/agent/primary.rs`

- [x] **Step 1: Add a scripted provider to the existing test module**

Implement a test-only `ScriptedProvider` with a `VecDeque<LlmResponse>`, recorded message batches, a tool-free final reply, and `LlmProvider::chat_with_tools` that pops one response per iteration.

- [x] **Step 2: Add the failing research-to-artifact test**

Script `web_search -> file_write -> Text("completed")`, register recording tools through a real `Toolset`, execute `run_agent_loop`, and assert both tools ran in order and the final response is `completed`.

- [x] **Step 3: Run the focused test and confirm the current direct-return behavior fails**

Run:

```bash
cd src-tauri
cargo test agent::primary::tests::agent_loop_continues_from_research_to_artifact -- --exact
```

Expected: FAIL because only `web_search` executes and raw search output is returned.

- [x] **Step 4: Add failing tests for empty follow-up, duplicate calls, tool-error recovery, and hard-limit finalization**

Each test must assert externally visible behavior and recorded tool/provider calls, not private helper implementation.

### Task 2: Hermes-Style Loop Controller

**Files:**
- Modify: `src-tauri/src/agent/primary.rs`

- [x] **Step 1: Remove direct success-tool termination**

Delete `should_return_tool_results_directly`. Always append each tool result and continue to the next model iteration.

- [x] **Step 2: Preserve assistant text and cap observations**

Store `LlmResponse::ToolCalls.text` in the assistant message and as a fallback. Truncate every tool observation with the existing character-safe `truncate_chars` helper before adding it to model context.

- [x] **Step 3: Bound repeated calls and expose recoverable failures**

Canonicalize JSON arguments with `serde_json`, count `(name, arguments)` invocations, execute a pair at most twice, and append a `Tool error:` observation for later duplicates. Continue after both execution and security errors.

- [x] **Step 4: Add empty-response continuation and budget pressure**

Allow two empty no-tool responses after tool work before fallback. Append consolidation guidance to the final tool observation at 75% and urgent final-answer guidance at 90% of the iteration budget.

- [x] **Step 5: Make limit finalization truthful and tool-free**

Append a system instruction stating that the tool budget is exhausted, then call `provider.chat` without tool definitions. Require it to summarize completed actions, failures, and uncompleted requirements without claiming success.

- [x] **Step 6: Raise the bounded default budget**

Set `AgentConfig::default().max_tool_rounds` to 12 and the outer loop timeout to 300 seconds. Keep the limits explicit and tested.

- [x] **Step 7: Run focused tests until green**

Run:

```bash
cd src-tauri
cargo test agent::primary::tests
```

Expected: all primary-agent tests pass.

### Task 3: Execution Prompt Contract

**Files:**
- Modify: `src-tauri/src/agent/primary.rs`

- [x] **Step 1: Add a failing prompt-construction test**

Build messages for an agent with tools and assert a system message requires requested side effects and artifact verification before completion.

- [x] **Step 2: Add the execution policy**

Inject a separate system message only when a toolset exists. The policy must require completing all requested actions, using observations for subsequent work, recovering from errors, verifying artifacts with read tools, and reporting partial completion truthfully.

- [x] **Step 3: Run the prompt and loop tests**

Run:

```bash
cd src-tauri
cargo test agent::primary::tests
```

Expected: PASS.

### Task 4: Independent Review And Documentation

**Files:**
- Modify: `docs/AGENT_LOOP.md`
- Modify: `docs/REALITY_CHECK.md`
- Modify: `CHANGELOG.md`
- Modify: `CLAUDE.md`

- [x] **Step 1: Run spec-compliance review**

Confirm tool results never terminate the loop directly, all regression cases are covered, and deferred features remain documented as deferred.

- [x] **Step 2: Run code-quality review**

Check provider message validity, Unicode safety, loop bounds, security-path preservation, error truthfulness, and accidental changes outside the assigned files.

- [x] **Step 3: Update documentation with measured claims**

Replace the documented premature-return defect with the tested behavior and retain explicit limitations: no true token streaming, request cancellation, parallel tool batches, semantic goal verifier, provider failover, or complete Hermes parity.

- [x] **Step 4: Run full verification without a Tauri release build**

```bash
npm run test
npm run build
cd src-tauri
cargo fmt --check
cargo check
cargo test
cargo clippy -- -D warnings
cargo check --features sherpa-onnx
cargo test --features sherpa-onnx
cargo clippy --features sherpa-onnx -- -D warnings
```

Expected: every command passes. Do not run `npm run tauri build`.

- [x] **Step 5: Create a scoped local checkpoint**

Review `git diff --check`, tracked-secret scan, and `git status`; commit only the loop implementation, its tests, and reconciled documentation after verification.
