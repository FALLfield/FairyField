# Changelog

All notable changes to FairyField will be documented in this file.

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
