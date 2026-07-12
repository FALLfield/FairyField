# Hermes-Style Primary Agent Loop

## Goal

Make Fairy complete multi-step tool tasks instead of stopping after the first useful result. The acceptance workflow is: research a topic, create the requested Markdown file, verify the file, and only then report completion.

This design borrows the execution semantics of the bundled Hermes Agent while retaining FairyField's Rust, Tauri, security, memory, and tool abstractions.

## Behavioral Contract

1. A tool call is always an intermediate step. Tool output is appended to the conversation and returned to the model for the next decision.
2. A turn finishes only when the model returns visible text without tool calls, the user cancels, or the iteration/time budget is exhausted.
3. Tool failures are structured observations, not terminal responses. The model may correct arguments or choose another tool.
4. Repeated identical calls are bounded so a model cannot spin forever.
5. Near the iteration limit, the model receives budget pressure and must consolidate or finish.
6. At the hard limit, Fairy performs one tool-free finalization call that truthfully summarizes completed work and unresolved items.
7. Assistant text emitted alongside tool calls is retained as a fallback but is not treated as completion when substantive tools remain.

## Architecture

`PrimaryAgent::run_agent_loop` remains the single orchestration boundary. Each iteration performs:

1. bounded-iteration check;
2. Unicode-safe per-observation size cap;
3. `chat_with_tools` provider call;
4. assistant/tool-call message persistence;
5. sequential tool execution through `Toolset`, preserving existing security checks;
6. structured result append and next iteration.

Sequential execution is intentional for this milestone because file writes, reads, searches, and shell calls may depend on earlier results. Safe parallel read-only batching can be added later once dependency analysis is explicit.

## Reliability Rules

- Default budget increases from 5 to 12 model iterations, enough for common research and artifact workflows without copying Hermes's CLI-oriented 90-turn default.
- Identical `(tool name, canonical JSON arguments)` calls execute at most twice per user turn. Later duplicates become a tool error observation.
- Empty model replies after tools retry within the remaining loop budget.
- Provider failure before any tool result remains an error. Provider failure after work has begun triggers a truthful fallback containing completed tool evidence.
- Tool results are Unicode-safe and capped before entering model context.
- Budget warnings are appended to the latest tool result at 75% and 90% usage.

## Prompt Contract

The system prompt will explicitly require Fairy to:

- complete every requested side effect;
- use tool results as evidence for subsequent actions;
- verify created or modified files before claiming success;
- inspect tool errors and recover when possible;
- never claim an artifact exists solely because research succeeded.

## Tests

Regression tests use a scripted provider and real `Toolset` test tools:

- web search result is followed by file write and final response;
- read-only search output is not returned directly;
- tool errors are fed back so the model can recover;
- duplicate identical calls are blocked after the allowed count;
- empty follow-up responses continue rather than returning raw tool output;
- iteration exhaustion makes a tool-free finalization request;
- existing security checks remain in the execution path.

Full acceptance requires targeted Rust tests, all Rust tests with and without `sherpa-onnx`, frontend tests/build, formatting, and strict Clippy. No Tauri release build is permitted.

## Deferred Work

True token streaming during tool orchestration, request-scoped cancellation, parallel tool batches, provider failover, semantic goal verification, and full Hermes-style context summarization remain separate milestones. This change establishes the correct execution semantics they depend on.
