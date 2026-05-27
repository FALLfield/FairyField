# Changelog

All notable changes to FairyField will be documented in this file.

## [1.0.0] — 2026-05-26

### Added
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
- 117 new Rust tests (369 total), tools/mcp/rate_limit token bucket, and full built-in tool smoke coverage
- User configuration system (config/user.rs, ~/.fairyfield/user.json)
- Coding Agent Manager (Claude Code/KiloCode/OpenCode CLI subprocess support)
- GitHub API integration tool
- MCP Server expansion: fairy_execute_tool + fairy_get_context
- OnboardingWizard: 4-step first-time setup (name, call preference, personality, LLM config)
- Shared AgentMemory backend for external agents and MCP adapters
- Community plugin manifest loader, Composio catalog scaffolding, and Phase 6 integration tool surfaces

### Changed
- Emotion buttons removed from ControlPanel (now AI-driven + hotkeys)
- ControlPanel z-index: 200-300, position: bottom-right to top-left
- ChatHistory gap: 6px-10px, consecutive same-role messages grouped
- Tool directory: flat tools/ to tools/builtins/ + tools/mcp/ + tools/community/
- MCP Server: 3 tools to 5 tools (added execute_tool + get_context)
- Release metadata aligned for GitHub v1 publication
- Release verification now covers 466 automated tests (369 Rust + 97 Frontend)

### Fixed
- `autoResize()` in ChatInput now properly recalculates on input event
- ChatBubble relativeTime handles undefined timestamp gracefully
- Onboarding now checks `~/.fairyfield/user.json` in Tauri before localStorage fallback
- LLM API keys saved during onboarding persist across restart outside public `config.json`
- Character click-through no longer steals chat/control input clicks
- Grouped user chat bubbles align to the right
- Web search now uses a DuckDuckGo Lite fallback plus encoded search links when the Instant Answer API has no results
- Git tool now executes real read-only git commands instead of returning a placeholder response
- Agent status badge now returns to idle after stream completion or error
- Browser preview no longer crashes when Tauri event listeners are unavailable
- VRM loading now uses `combineSkeletons()` instead of deprecated `removeUnnecessaryJoints()`

### Security
- Rate limiting on tool execution (token bucket algorithm)
- OAuth token config structure for secure third-party integrations
- Coding Agent subprocess execution has cwd validation, permission-mode allowlist, timeout kill, and output caps

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
