# FairyField Project Structure

This document describes the project structure and organization for the FairyField desktop anime agent application.

## Overview

FairyField is built with:
- **Frontend**: Vue 3 + TypeScript + Vite
- **Backend**: Tauri v2 + Rust
- **Rendering**: PixiJS + pixi-live2d-display

## Directory Structure

```
FairyField/
├── src/                          # Frontend source code
│   ├── renderers/                # Rendering modules
│   │   └── README.md
│   ├── modules/                  # Frontend modules
│   │   └── README.md
│   ├── assets/                   # Static assets
│   ├── App.vue                   # Main Vue component
│   └── main.ts                   # Frontend entry point
│
├── src-tauri/                    # Backend source code
│   ├── src/
│   │   ├── audio/                # Audio pipeline (ASR, TTS)
│   │   │   └── mod.rs
│   │   ├── llm/                  # LLM client module
│   │   │   └── mod.rs
│   │   ├── tools/                # Tool executor module
│   │   │   └── mod.rs
│   │   ├── lib.rs                # Library entry point
│   │   └── main.rs               # Binary entry point
│   ├── Cargo.toml                # Rust dependencies
│   └── tauri.conf.json           # Tauri configuration
│
├── public/                       # Public assets
├── package.json                  # Node.js dependencies
└── vite.config.ts                # Vite configuration
```

## Frontend Modules (Planned)

### Renderers (`src/renderers/`)
- **Live2DRenderer.ts**: Live2D model rendering using pixi.js and pixi-live2d-display
- **Inochi2DRenderer.ts**: Inochi2D format support (future implementation)

### Modules (`src/modules/`)
- **HitTestModule.ts**: Click detection and hit testing for transparent window
- **EyeTrackModule.ts**: Eye tracking and mouse following
- **LipSyncModule.ts**: Lip sync with audio playback
- **HologramController.ts**: Hologram projection mode control
- **AudioRecorder.ts**: Audio recording for voice input

## Backend Modules (Planned)

### Audio Pipeline (`src-tauri/src/audio/`)
- ASR (Automatic Speech Recognition) using OpenAI Whisper API
- TTS (Text-to-Speech) using Edge-TTS or GPT-SoVITS
- Audio recording and playback management

### LLM Client (`src-tauri/src/llm/`)
- Unified interface for multiple LLM providers
- Streaming support via Server-Sent Events (SSE)
- Support for OpenAI, Anthropic Claude, Groq, and local Ollama

### Tool Executor (`src-tauri/src/tools/`)
- Structured tool calling with security boundaries
- Git operations (status, commit)
- File management (read, write)
- Shell command execution (with whitelist)
- Browser automation

## Dependencies

### Frontend Dependencies
- **vue**: ^3.5.13 - Vue.js framework
- **@tauri-apps/api**: ^2 - Tauri API bindings
- **@tauri-apps/plugin-opener**: ^2 - Tauri opener plugin
- **pixi.js**: ^8.0.0 - 2D rendering engine
- **pixi-live2d-display**: ^0.4.0 - Live2D support for PixiJS

### Backend Dependencies
- **tauri**: 2 - Tauri framework
- **serde**: 1 - Serialization/deserialization
- **serde_json**: 1 - JSON support
- **tokio**: 1 - Async runtime (full features)
- **reqwest**: 0.12 - HTTP client (json, stream, multipart features)

## Architecture Alignment

This structure aligns with the design document's architecture:

1. **Frontend Layer**: Vue components + rendering modules
2. **Tauri IPC Layer**: Communication between frontend and backend
3. **Backend Core Layer**: Rust modules for audio, LLM, and tools
4. **External Services**: LLM APIs, Whisper API, TTS services
5. **Local Storage**: Configuration files, memory database, plugins

## Next Steps

1. Implement transparent window configuration (Task 1.3)
2. Implement HitTest module for click-through (Task 2)
3. Implement Live2D rendering system (Task 3)
4. Continue with subsequent tasks as defined in tasks.md
