# FairyField V1 Reality Check And Handover

Updated: 2026-07-12

This is the source of truth for the next maintainer or model. Code and fresh verification take precedence over older phase checklists, optimistic completion labels, and reference-project plans.

## Audit Baseline

The following Git facts describe the start of the 2026-07-10 handover audit, before these documentation edits are committed:

- The publishable Git repository is /Users/fallfield/Desktop/Projects/FairyField/FairyField.
- The parent directory contains planning documents plus the Hermes Agent and MemPalace reference checkouts. Those files are useful research material, but they are not part of the FairyField Git repository.
- Parent-level AGENTS.md, CLAUDE.md, PROPOSAL.md, PROMPT_FOR_CODEX.md, and TEST_MANUAL.md contain older phase claims. Use them for vision and workflow context, not current completion status.
- main and origin/main pointed to commit 60aac49 at audit start.
- Package, Cargo, and Tauri metadata all say 1.0.0.
- The v1.0.0 tag was five commits behind that baseline commit.
- The pre-handover checkout described itself as v1.0.0-5-g60aac49-dirty. The dirty state was documentation-only.
- Existing backup branches include backup/pre-loop-20260708-223520 and earlier v1 backup points.

## Fresh Automated Verification

The following commands passed on 2026-07-10:

- npm run test: 102 frontend tests.
- npm run build: passed, with the existing large Three.js chunk warning.
- cargo fmt --check: passed.
- cargo check: passed.
- cargo test: 409 Rust tests.
- cargo clippy -- -D warnings: passed.
- cargo check --features sherpa-onnx: passed.
- cargo test --features sherpa-onnx: 411 Rust tests.
- cargo clippy --features sherpa-onnx -- -D warnings: passed.
- A tracked-file secret-pattern scan found only deliberate redaction test fixtures.

These checks prove compilation and unit-level behavior. They do not prove the real microphone, installed voice models, audible latency, lip sync, external OAuth services, MCP protocol interoperability, packaging, signing, notarization, or every user workflow.

The project owner has explicitly asked that a Tauri release build not be used as routine verification. Release packaging must be a separate, deliberate step.

Handover incident: a delegated read-only release audit nevertheless ran the default npm run tauri build command. It generated ignored local macOS app and DMG artifacts, did not enable sherpa-onnx, did not publish anything, and did not modify tracked source. Treat this as an audit process error, not as release acceptance evidence.

Post-handover verification on 2026-07-12, after the Hermes-style loop and Git safety changes:

- npm run test: 102 frontend tests.
- npm run build: passed with the existing large Three.js chunk warning.
- cargo fmt --check, cargo check, and cargo clippy -- -D warnings: passed.
- cargo test: 425 Rust tests.
- cargo check/test/clippy with sherpa-onnx: passed; 427 Rust tests.
- No Tauri release build was run for this change.

## Capability Map

| Area | Current status | What is actually true |
|---|---|---|
| Desktop shell | Real | Tauri, Vue, Three.js, bundled character, chat, onboarding, config panel, and developer panels exist. |
| Text chat | Partial | LLM providers and a tool-calling loop exist, but displayed streaming is simulated after the full backend answer completes. |
| Agent tools | Partial | 21 runtime tool definitions are registered. Core web, weather, file, GitHub, Git, shell, memory, and Obsidian paths contain real logic; many service tools are configuration or metadata scaffolds. |
| Multi-step completion | Improved | The primary loop now continues after tool observations and regression coverage proves search, write, read-back verification, and final response. Semantic requirement verification is still model-driven. |
| Development loop | Partial | A deterministic Manager, Coding, Testing, and Goal pipeline exists, but it is not the normal chat path, is not exposed in the UI, and does not semantically verify every requirement. |
| Coding CLI bridge | Partial | Claude Code, KiloCode, and OpenCode adapters exist. At handover time Codex was installed on this Mac but was not supported by the bridge. |
| TTS | Partial | Normal builds use macOS say on macOS. Real Matcha or Kokoro requires the sherpa-onnx feature and installed models. |
| STT | Partial | Feature builds can capture, downmix, resample, reject silence, and run Paraformer. Capture is fixed to three seconds and has no usable stop, partial transcript, or VAD endpointing. |
| VAD | Disconnected | VAD engines compile, but microphone samples never call the detection path, so the exposed state does not become useful. |
| Lip sync | Broken/partial | TTS start and finish events exist, but no PCM event reaches the renderer. The simulated mouth path can update once and remain partly open. |
| Body animation | Real but basic | Breathing, sway, head motion, arm pose, and random nod, lean, shoulder, and hand gestures run. They are not speech or emotion aware. |
| Memory storage | Real but risky | SQLite drawers, FTS5, Jieba tokenization, wake-up context, and duplicate checks exist. |
| Semantic memory | Nonfunctional placeholder | Embeddings are deterministic byte-derived vectors, the index is process-only, and production saves never populate it. |
| Knowledge graph | Isolated CRUD | Temporal facts and invalidation work through commands, but chat mining and recall do not use the graph. |
| Growth and skills | Manual CRUD | Experience and skill tables exist, but the normal agent and UI do not run an autonomous growth loop. Skills are not TOML plus Markdown at runtime. |
| MCP | IPC facade only | The so-called MCP server exposes Tauri commands. There is no stdio, SSE, or HTTP MCP protocol server for external Codex or Claude clients. |
| Discord and Cron | Disconnected scaffold | Webhook client and in-memory Cron CRUD exist, but startup leaves Discord unconfigured and no scheduler loop dispatches messages. |
| Onboarding personalization | Persisted but unused | Name, preferred address, personality, and language are saved, but PrimaryAgent does not consume UserConfig or soul/SOUL.md. |
| Release | Source candidate | Version metadata is aligned, but the tag lags main and a signed, notarized, tested distributable is not established by this handover. |

## Critical Bugs And Risks

### 1. Premature tool-result completion was fixed on 2026-07-12

PrimaryAgent no longer directly returns web_search, web_fetch, weather, github, or git results. Every tool result returns to the model as an observation. The loop now has a 12-iteration budget, duplicate-call limits, recoverable tool errors, Unicode-safe observation caps, budget pressure, and a truthful tool-free finalization call.

Automated coverage now proves:

- web search followed by Markdown write, read-back verification, and final response;
- same-batch sequential tool execution;
- provider/tool failure handling without false success;
- canonical duplicate-call blocking;
- a blocked Git mutation returning through the real Toolset path.

Remaining limitation: completion is still decided by the model under an execution policy. There is no separate semantic Goal agent proving every requirement, no request cancellation, and no Hermes-style tool-pair-aware context summarization or provider failover.

### 2. Security controls are not on one trustworthy execution path

Current problems:

- User messages and recalled memory can enter the model without ingress injection checks.
- Tool output is returned to the LLM without automatic redaction or untrusted-data isolation.
- Stored user text is later injected as a system message, creating persistent prompt-poisoning risk.
- Relative file paths can reach sensitive project files, and path containment does not fully defend symlink cases.
- Security IPC and Toolset use different CommandGuard instances, so an IPC approval does not approve the command the agent later executes.
- The Git tool now rejects branch/tag/remote mutations, output-writing flags, `--no-index` arbitrary-file reads, and external diff/textconv helpers including abbreviated long options. It exposes a valid function-calling schema and uses a kill-on-timeout Tokio child. Broader command and filesystem policy still requires continued adversarial review.
- API keys and OAuth tokens are kept in plaintext JSON files with ordinary filesystem permissions.

Required outcome:

- Use one shared guard and approval store.
- Treat user input, files, web pages, tool output, and memory as untrusted data with provenance.
- Redact secrets automatically before model and UI boundaries.
- Canonicalize and contain every file path.
- Replace Git prefix matching with exact read-only operations.
- Store secrets with platform credential storage or at minimum strict file permissions.

### 3. Chat streaming, abort, and clear are not real end-to-end operations

- Rust waits for the complete non-streaming agent loop, then emits two-character chunks.
- Frontend events are global and not correlated to a request.
- Abort only clears local Vue state; the backend request keeps running.
- A later request can receive events from an aborted request.
- Clear chat only clears the frontend. Backend history and persisted memory remain.
- OpenAI-compatible request construction ignores configured temperature, max tokens, and top-p.
- Model routing, context compression, ToolAgent, and delegation modules are not connected to PrimaryAgent.

Required outcome:

- Use provider streaming during the agent loop.
- Add request IDs to every stream event.
- Add backend cancellation and TTS interruption.
- Invoke backend history clearing from the UI.
- Provide separate, explicit controls for conversation history and persisted memory.

### 4. Voice is not a continuous ChatGPT Voice-like system

- Normal builds have no real ASR because sherpa-onnx is disabled by default.
- Real voice requires a feature-enabled build and complete local models.
- Recording is fixed to three seconds and cannot be stopped by the current UI.
- VAD is not fed microphone data.
- TTS begins only after the full agent answer is complete.
- TTS synthesis and playback are synchronous and not interruptible.
- Chinese and English Matcha routing exists, but Japanese kana has no route and Japanese kanji is treated as Chinese.
- macOS fallback always selects the Tingting Chinese voice, including for English.
- Voice errors are not visibly surfaced in the main UI.
- The downloader can swallow component failures and still print a completion message.

Required outcome:

- Add model-health reporting and fail loudly on incomplete models.
- Use continuous capture with VAD endpointing and partial transcripts.
- Stream LLM text into a first-stable-sentence TTS queue.
- Add barge-in, cancellation, and latency telemetry.
- Add real Chinese, English, mixed-language, and Japanese acceptance tests.

### 5. Avatar speech evidence is incomplete

Rust emits speaking start and finish events but does not emit the PCM event expected by the renderer. Two frontend consumers also listen to different PCM event names. The fallback mouth simulation has a state bug that can suppress continued movement and prevent a clean close.

Body idle animation is implemented and tested, but it is independent of speech and emotion.

### 6. Memory is useful storage, not yet MemPalace parity

- FTS-backed drawer storage is real.
- L1 is a short list of high-importance drawers, not a generated recent-conversation summary.
- Mining is keyword-based.
- Save failures are logged instead of shown to the user.
- Recalled memory is injected as trusted system content.
- The vector command starts with an empty process-only index and is never populated by production writes.
- Knowledge graph, growth, skills, and AgentMemoryBackend are not integrated into normal conversation.
- Clear and correction controls are absent.
- Configured database path, embedding size, wake-up budget, and dedup threshold are not all honored by runtime wiring.

### 7. Most advertised integrations are scaffolds

The runtime registry contains 21 tools, not 135 independently executable tools.

Real or substantially implemented paths include:

- web_search, web_fetch, weather;
- file_read and file_write;
- terminal with a narrow allowlist;
- git and GitHub;
- memory search and save;
- Obsidian when a vault is configured.

Scaffold behavior includes:

- Notion, Linear, Gmail, Calendar, Slack, Discord, Translate, and Image returning ready with network_call skipped when configured;
- Browser and Cron returning queued without performing the side effect;
- Composio catalog discovery without execution;
- community plugin manifest loading without runtime registration and execution;
- manifest permissions, enablement, OAuth, rate limits, and per-tool timeout metadata not controlling Toolset execution;
- Obsidian always being registered without a vault path in the production registry;
- Discord webhook configuration not being represented in AppConfig or used to initialize GatewayState;
- Cron jobs remaining in memory without a background scheduling and dispatch loop.

### 8. The development loop is not yet the requested Codex-based loop

- Manager and Goal are deterministic Rust stages, not independent reasoning agents.
- Goal checks tests and changed-file evidence, not each requirement against code behavior.
- Any scoped edit plus unrelated passing tests can be reported as success.
- Changes to files that were already dirty before the loop can escape its changed-file set-difference check.
- Codex CLI is installed but the adapter does not support it.
- There is no worktree isolation, progress UI, cancellation, resume, PTY monitoring, or routing from ordinary chat.
- The availability list returns display names that do not round-trip to the dispatch parser.

### 9. Personalization and soul files are disconnected

The onboarding wizard persists user name, preferred address, Fairy name, personality, and language. PrimaryAgent still uses an embedded default system prompt. UserConfig, soul/SOUL.md, soul/identity.txt, model routing, context compression, delegation, and ToolAgent are not wired into the normal path.

Onboarding marks user.json complete before API-key setup finishes, so a key-save failure can leave the next launch skipping onboarding. Saving a key rebuilds the agent but does not select that provider. The normal ControlPanel can switch providers but has no API-key entry flow for users who skipped setup.

### 10. Release engineering is incomplete

- v1.0.0 points to an older commit than current main.
- As of 2026-07-10, the public [FairyField v1.0.0 GitHub release](https://github.com/FALLfield/FairyField/releases/tag/v1.0.0) is source-only and has zero uploaded assets.
- The latest GitHub CI run for main at 60aac49 is red. Frontend passed, while backend pins Rust 1.85 even though current locked dependencies require Rust 1.87 or 1.88, and the security audit is not pointed at src-tauri/Cargo.lock.
- Standard release commands do not enable sherpa-onnx, so packaged speech behavior differs from the advertised local voice stack.
- macOS signing, notarization, Gatekeeper acceptance, Windows support, Linux support, and upgrade behavior remain release gates.
- README clone instructions must match the actual repository root.
- package.json still declares a nested repository directory even though this checkout is the repository root.

## Next Execution Order

1. Commit this truthful handover as a backup point after review.
2. Fix shared security state, Git mutation holes, path containment, secret permissions, and memory prompt poisoning.
3. Implement real request-correlated streaming, backend abort, and backend clear.
4. Add Codex support, route long coding tasks, and make goal verification requirement-aware.
5. Add tool-pair-aware context compaction, provider retry/failover, and safe independent read batching.
6. Wire UserConfig and soul/SOUL.md into PrimaryAgent.
7. Repair the voice path: model health, VAD capture, partial ASR, first-sentence TTS, cancellation, PCM lip sync, and visible errors.
8. Replace or remove placeholder vector and MCP claims.
9. Turn high-value scaffold tools into real operations one at a time, with permission and integration tests.
10. Run automated tests, Tauri development smoke tests, and a recorded manual acceptance matrix.
11. Only then prepare a new release tag, signed artifacts, and publication notes.

## Definition Of Done For A Public V1

- Search plus Desktop Markdown creates and verifies the file.
- Weather and web fetch handle Chinese text without panic or multi-minute hangs.
- Unsafe Git, shell, file, web, memory, and recalled-content inputs are blocked or isolated.
- Streaming is genuinely incremental and request-correlated.
- Abort stops backend generation and TTS; clear removes the intended history.
- Chinese and English microphone input work on the real device.
- Mixed Chinese and English TTS does not read code, URLs, JSON, logs, or strange symbols.
- First audio begins before a long answer finishes.
- VRM mouth movement varies during audible speech and closes afterward.
- Random body motion remains subtle and does not clip badly.
- A preference can be remembered, corrected, invalidated, sourced, and deleted.
- External MCP connectivity passes an actual protocol handshake and tools/call test, or MCP claims are removed.
- A coding task can run through a Codex-supported, isolated, cancellable loop whose Goal stage checks every requirement.
- CI is green from a clean checkout.
- Signed release artifacts install and run on every advertised operating system.

## Do Not Overclaim

- Hermes-inspired is accurate. Hermes-equivalent is not.
- MemPalace-inspired FTS memory is accurate. Production semantic memory is not.
- Tauri IPC memory facade is accurate. External MCP server is not.
- Local voice code exists is accurate. Default fully local ChatGPT Voice-quality conversation is not.
- Body idle animation exists is accurate. Audio-driven lip sync is not.
- 527 default automated checks pass is accurate. Bug-free, release-ready, and every tool works are not.
