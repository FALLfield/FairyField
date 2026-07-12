# FairyField Architecture

FairyField v1.0.0 is a Tauri v2 desktop AI companion built with a Vue/Three.js frontend and a Rust backend. The repository has no Python runtime dependency.

## Frontend

- `src/components/CharacterCanvas.vue` hosts the Three.js canvas and VRM/GLB character lifecycle.
- `src/renderers/VRMRenderer.ts` loads and renders the model with `@pixiv/three-vrm`.
- `src/modules/` contains expression, eye tracking, lip sync, idle animation, and hit-test modules.
- `src/components/ChatPanel.vue`, `ChatHistory.vue`, `ChatBubble.vue`, and `ChatInput.vue` provide the chat surface.
- `src/components/ControlPanel.vue` owns provider/config controls and stays separate from the chat UI.
- `src/components/OnboardingWizard.vue` handles first-run setup.

## Backend

- `src-tauri/src/agent/` contains the primary companion agent, toolset bridge, delegation, compression, and Coding Agent manager.
- `src-tauri/src/llm/` contains provider adapters, streaming parsing, model routing, and prompt caching.
- `src-tauri/src/tools/` contains 21 registered tool definitions plus manifests and scaffold modules for permissions, OAuth, Composio, and community plugins. Several service tools do not yet perform network or side-effect operations.
- `src-tauri/src/memory/` contains real SQLite/FTS memory, partial 4-layer recall, isolated knowledge-graph CRUD, keyword mining, placeholder embeddings, and an unconnected `AgentMemoryBackend`.
- `src-tauri/src/text.rs` centralizes UTF-8 safe truncation used by tool logs, web fetches, memory fallbacks, and LLM diagnostics.
- `src-tauri/src/voice/` contains ASR, TTS, VAD, microphone capture, and Tauri voice commands.
- `src-tauri/src/security/` contains prompt-injection detection, command guarding, and secret/URL redaction.
- `src-tauri/src/gateway/` contains Discord/webhook and cron scheduling.
- `src-tauri/src/config/` contains app config, user config, and secret-store persistence.

## IPC Boundary

The frontend calls Rust through Tauri commands registered in `src-tauri/src/lib.rs`. User-facing config is redacted before it crosses into the frontend; API keys are stored separately under `~/.fairyfield/secrets.json`.

Agent tool calls, `tools_execute`, and the Tauri IPC `fairy_execute_tool` facade share the same `Toolset` execution path. Tool-input checks are shared, but security is not end-to-end: chat ingress, recalled memory, and tool output are not isolated or redacted automatically, and the IPC approval state uses a different CommandGuard instance.

## Reference Integration

FairyField v1 borrows the shape of Hermes Agent and MemPalace without carrying their Python runtime into the app.

- Hermes-inspired structures exist through `Tool`, `ToolExecutor`, `ToolRegistry`, manifests, transport/OAuth metadata, command guards, and Coding Agent subprocess control. This is not Hermes feature parity.
- The web tools have concrete hardening: generic search has reqwest + curl-backed upstream fallback, weather queries use dedicated weather sources before generic search, tool output is UTF-8 safe, and `web_fetch` has DNS timeouts, manual redirect validation, SSRF guards, and response-size caps.
- PrimaryAgent now follows the Hermes invariant that tool results are observations and only a no-tool model response completes a normal turn. Its 12-iteration loop has duplicate protection, recoverable errors, Unicode caps, budget pressure, artifact-verification policy, and tool-free limit finalization. Semantic Goal verification, cancellation, failover, context summarization, and safe parallelism remain open.
- The development loop records Git-detected changed files and test output, but Manager and Goal are deterministic stages. Goal does not yet verify each requirement semantically, and edits to files already dirty before the loop can escape its delta check.
- MemPalace-inspired storage exists through wake-up context, FTS5 with Chinese tokenization, temporal knowledge-graph commands, duplicate prevention, and verbatim drawers. Embeddings are placeholder, the vector index is empty in production, and no external MCP protocol server exists.
- Remaining work is product closure: semantic completion verification, shared security state, real streaming/abort/clear, Codex support, persistent embeddings, secured memory recall, live integrations, real MCP transport, voice completion, and release engineering.

## Published Assets

The default character asset is:

```text
public/models/default/2031903848872972007.glb
```

Large downloaded ASR/TTS/VAD models are not committed. They are fetched with `scripts/download-models.sh` into `~/.fairyfield/models/`.

## Verification

Fresh verification on 2026-07-12 covered compile/test/lint checks. It did not establish real microphone quality, installed-model latency, PCM lip sync, OAuth providers, MCP interoperability, CI health, package signing, notarization, or installer behavior.

- `npm run build`
- `npm run test` — 102 frontend tests
- `cargo check`
- `cargo test` — 425 Rust tests
- `cargo clippy -- -D warnings`
- `cargo check --features sherpa-onnx`
- `cargo test --features sherpa-onnx` — 427 Rust tests
- `cargo clippy --features sherpa-onnx -- -D warnings`
- `cargo fmt --check`
