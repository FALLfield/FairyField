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
- `src-tauri/src/tools/` contains manifests, permissions, execution, built-in tools, MCP transport/OAuth/Composio, and community plugin loading.
- `src-tauri/src/memory/` contains SQLite/FTS memory, 4-layer recall, knowledge graph, mining, embeddings, and shared `AgentMemoryBackend`.
- `src-tauri/src/voice/` contains ASR, TTS, VAD, microphone capture, and Tauri voice commands.
- `src-tauri/src/security/` contains prompt-injection detection, command guarding, and secret/URL redaction.
- `src-tauri/src/gateway/` contains Discord/webhook and cron scheduling.
- `src-tauri/src/config/` contains app config, user config, and secret-store persistence.

## IPC Boundary

The frontend calls Rust through Tauri commands registered in `src-tauri/src/lib.rs`. User-facing config is redacted before it crosses into the frontend; API keys are stored separately under `~/.fairyfield/secrets.json`.

## Published Assets

The default character asset is:

```text
public/models/default/2031903848872972007.glb
```

Large downloaded ASR/TTS/VAD models are not committed. They are fetched with `scripts/download-models.sh` into `~/.fairyfield/models/`.

## Verification

Release verification for v1.0.0:

- `npm run build`
- `npm run test` — 97 frontend tests
- `cargo check`
- `cargo test` — 369 Rust tests
- `cargo clippy -- -D warnings`
- `cargo fmt --check`
