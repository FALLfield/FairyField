# Changelog

All notable changes to FairyField will be documented in this file.

## [Unreleased]

### Changed
- Reworked `PrimaryAgent` into a bounded Hermes-style execution loop: tool results always return to the model, the default budget is 12 iterations, observations are Unicode-capped, repeated canonical calls are bounded, and budget exhaustion receives a tool-free finalization turn.
- Added an explicit execution policy requiring requested side effects and artifact read-back before reporting success.
- Added `docs/REALITY_CHECK.md` as the source of truth for verified functionality, user-reported failures, incomplete scaffolding, and next-model handoff.
- Updated voice, roadmap, architecture, development-loop, usage, and manual-test docs to distinguish implemented code from runtime scaffolding and unverified product behavior.
- Updated verification to 527 default automated tests (425 Rust + 102 frontend) and 427 Rust tests under `sherpa-onnx` after the Hermes-style loop regressions.
- Recorded that the runtime registry exposes 21 tools, external MCP is not implemented, semantic vector memory is placeholder, and the normal build does not include real sherpa ASR/TTS/VAD.

### Fixed
- Fixed multi-step requests stopping after web, weather, GitHub, or Git output. Regression coverage now proves `web_search -> file_write -> file_read -> final response` and tool-error recovery.
- Git function calling now has a real JSON schema, rejects branch/tag/remote mutations, output writes, `--no-index` file reads, and external diff/textconv helpers including abbreviated long options, and kills timed-out Git children.
- Documented the Desktop file-tool path fix from `60aac49`: Desktop-relative, home-relative, `$HOME`-relative, and absolute Desktop paths now resolve for user-facing file writes while non-Desktop paths remain blocked.

### Known Issues
- Multi-step execution still relies on the model to decide semantic completion; no independent requirement verifier, request cancellation, provider failover, or tool-pair-aware context summarizer is connected to normal chat.
- Chat streaming is simulated after full completion; abort and clear affect only frontend state.
- Security IPC and Toolset do not share CommandGuard state, file policy needs symlink/canonical containment review, and recalled memory can become trusted system content.
- Voice is not ChatGPT Voice-like: default ASR is empty Mock, VAD is disconnected, recording is fixed-length, PCM lip sync is disconnected, simulated mouth motion is buggy, and multilingual behavior is incomplete.
- Memory FTS storage exists, but persistent semantic vectors, integrated knowledge-graph recall, safe memory provenance, correction/deletion controls, and production MCP access are incomplete.
- Onboarding personality data and soul files are persisted but not consumed by PrimaryAgent.
- The development loop lacks Codex support, UI routing, worktree isolation, cancellation, and requirement-level Goal verification.
- Third-party tools, Browser, Cron, Composio, and community plugins include non-executing scaffold paths.
- The v1.0.0 tag lags main; latest main CI is red because the backend toolchain is pinned below locked dependency requirements and the audit job does not target src-tauri/Cargo.lock. Signed release artifacts also require a separate pass.

## [1.0.0] — 2026-05-28

### Added
- Low-latency Matcha bilingual TTS path: Chinese `matcha-icefall-zh-baker` + English `matcha-icefall-en_US-ljspeech`
- Universal TTS text cleaner for markdown/code/emoji/URL/tool tags before any engine synthesizes
- Sentence-level TTS chunking so long replies can start speaking from the first short sentence instead of waiting for full-message synthesis
- Emotion/Chat UI separation: ControlPanel moved to top-left, default collapsed
- Expression hotkey system: Ctrl+1~8 for manual expression triggers
- ChatPanel autoScroll: pauses when user scrolls up, resumes at bottom
- ChatBubble message grouping: consecutive same-role messages merge visually
- ChatInput auto-height: textarea grows 1-5 rows based on content
- CharacterCanvas 0.5s smooth emotion transitions + thinking pose animation
- ToolManifest declaration system (13 categories, 9 permissions, OAuth config)
- MCP transport layer (stdio/sse/streamable HTTP, JSON-RPC messages)
- Token compressor (TokenJuice-style: HTML to text, URL shortening, truncation)
- Tools directory restructured: builtins/ (web, file_ops, shell, git, memory, obsidian)
- Obsidian vault integration tool (search notes, read/write, walkdir traversal)
- 157 new Rust-side tests since the MVP baseline; default Rust suite is now 409 tests, with 411 tests under `sherpa-onnx`
- User configuration system (config/user.rs, ~/.fairyfield/user.json)
- Coding Agent Manager (Claude Code/KiloCode/OpenCode CLI subprocess support)
- Multi-agent development loop with ManagerAgent, CodingAgent, TestingAgent, and GoalAgent evidence gates
- GitHub API integration tool
- MCP Server expansion: fairy_execute_tool + fairy_get_context
- OnboardingWizard: 4-step first-time setup (name, call preference, personality, LLM config)
- Shared AgentMemory backend for external agents and MCP adapters
- Community plugin manifest loader, Composio catalog scaffolding, and Phase 6 integration tool surfaces
- Phase 7 roadmap covering streaming voice, multilingual TTS reliability, Hermes-scale tools, UI refinement, XR prototype path, market position, and HCI research directions

### Changed
- Default offline TTS preference changed from Kokoro-only to Matcha bilingual, with Kokoro and macOS `say` as fallbacks
- `scripts/download-models.sh` now downloads Matcha bilingual models by default and makes Kokoro optional via `--kokoro`
- Emotion buttons removed from ControlPanel (now AI-driven + hotkeys)
- ControlPanel z-index: 200-300, position: bottom-right to top-left
- ChatHistory gap: 6px-10px, consecutive same-role messages grouped
- Tool directory: flat tools/ to tools/builtins/ + tools/mcp/ + tools/community/
- MCP Server: 3 tools to 5 tools (added execute_tool + get_context)
- Release metadata aligned for GitHub v1 publication
- Release verification now covers 511 default automated tests (409 Rust + 102 Frontend), plus 411 Rust tests under `sherpa-onnx`
- Memory wake-up now derives L1 summaries and L2 palace indexes from stored drawers before falling back to static metadata
- Memory saves now skip normalized exact duplicates in the same wing/room, and fallback conversation mining stores user text verbatim instead of truncating it
- Development loop retries now feed the previous failed test output back into the CodingAgent prompt instead of starting the next iteration blind
- Tools IPC and MCP `fairy_execute_tool` now execute through the same Agent `Toolset` path used by the primary agent

### Fixed
- `autoResize()` in ChatInput now properly recalculates on input event
- ChatBubble relativeTime handles undefined timestamp gracefully
- Onboarding now checks `~/.fairyfield/user.json` in Tauri before localStorage fallback
- LLM API keys saved during onboarding persist across restart outside public `config.json`
- Character click-through no longer steals chat/control input clicks
- Grouped user chat bubbles align to the right
- Web search now uses DuckDuckGo Instant Answer, curl-backed fallback, DuckDuckGo Lite fallback, and encoded search links when upstream sources are unavailable
- Web/weather search no longer panics on Chinese output; all tool-log truncation is UTF-8 safe
- Weather queries now use dedicated weather providers before generic web search, with a short-timeout curl fallback for environments where reqwest cannot reach Open-Meteo
- Web fetch now has manual safe redirects, async DNS timeouts, private-network blocking, response-size caps, and clear timeout/error messages
- Tool IPC execution now enforces a 30 second timeout instead of allowing long-running tools to hang the UI
- Search/fetch/weather tool results now return directly to chat when the tool output is already user-facing, so slow post-tool LLM summarization cannot hide the result
- Agent status badges are reset on mount and after send/abort so stale `executing_tool` states do not linger in the UI
- Git tool now executes real read-only git commands instead of returning a placeholder response
- Agent status badge now returns to idle after stream completion or error
- Browser preview no longer crashes when Tauri event listeners are unavailable
- VRM loading now uses `combineSkeletons()` instead of deprecated `removeUnnecessaryJoints()`
- Voice ASR now captures real microphone audio for `voice_start_asr`, downmixes to mono, resamples to the ASR target sample rate, rejects silent/too-short recordings, and emits `voice:asr_result`
- TTS cleanup strips developer artifacts such as command/file snippets before any engine speaks, while preserving natural Chinese/English text
- `web_fetch` response caps now decode only valid UTF-8 boundaries, preventing replacement characters or byte-boundary panics
- Web fetch SSRF protection now blocks IPv6 link-local addresses in addition to localhost, loopback, private, and unique-local ranges
- Weather is no longer config-gated; it delegates to the hardened weather/web path and can return live weather without an API key when public sources are reachable
- MCP dotted IDs now normalize to real registered tools such as `obsidian_search`, `notion`, `github`, and `terminal`

### Security
- Rate limiting on tool execution (token bucket algorithm)
- OAuth token config structure for secure third-party integrations
- Coding Agent subprocess execution has cwd validation, permission-mode allowlist, timeout kill, and output caps
- Development-loop test execution uses allowlisted commands, no shell metacharacters, capped output, scoped context files, and explicit dry-run failure
- Terminal tool no longer uses `sh -c`; it executes allowlisted inspection commands directly and rejects interpreter/package-runner entry points such as Python, Node, npm, pnpm, and Cargo
- Development-loop GoalAgent now requires Git-detected changed-file evidence and fails the loop when a CodingAgent edits outside its assigned file scope

## [0.1.0] — 2026-05-14

### Added
- 3D VRM anime character rendering (Three.js + @pixiv/three-vrm)
- Transparent always-on-top desktop window (Tauri v2)
- AI soul with Agent Loop + LLM Function Calling (OpenAI/Claude/DeepSeek)
- Multi-provider LLM switching (OpenAI, Claude, DeepSeek, GLM)
- Streaming chat with emotion-colored bubbles and typing animation
- Offline voice pipeline: sherpa-onnx Paraformer ASR + Kokoro TTS + Silero VAD
- Microphone input with recording indicator and voice-to-text
- 4-layer memory stack (SQLite + FTS5 + jieba Chinese tokenization)
- Temporal knowledge graph with valid_from/to triples
- Vector semantic search (sqlite-vec + ONNX embedding)
- Plugin system with JSON dynamic loading
- Discord Webhook notification gateway
- Autonomous growth engine + skill management (TOML + Markdown)
- Security: prompt injection detection, command whitelist, secret redaction
- Claude Code MCP Server integration
- Developer mode panel (Ctrl+Shift+D)
- Emotion system with natural decay and expression mapping
- Lip-sync and eye-tracking animation modules
- macOS microphone permission handling
- Comprehensive .gitignore and open-source packaging

### Fixed
- Shell command injection via newline/metacharacter bypass
- Async blocking sleep in shell executor
- ChatPanel component split (612 → 75 lines)
- Vue error boundary to prevent full app crash
- LLM provider fallback on empty API key
- Config serde deserialization with partial fields
- mine_conversation missing .await
- emotion tag leaking into streamed chat messages
- useAgent concurrent send race condition

### Security
- Block 9 shell metacharacters (newline, semicolon, pipe, backtick, etc.)
- Shell commands execute on dedicated blocking thread
- API keys never serialized to disk
- Config save failures logged instead of silently dropped

### Infrastructure
- Graphify knowledge graph auto-rebuild on commit (1111 nodes, 1780 edges)
- Conventional commits with Co-Authored-By trailer
- 345 automated tests (252 Rust + 93 Frontend)
- Comprehensive audit report with severity matrix
