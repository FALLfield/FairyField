# FairyField Project Structure

```text
FairyField/
├── src/                         # Vue 3 + TypeScript frontend
│   ├── acceptance/              # Frontend acceptance tests
│   ├── components/              # Chat, control, onboarding, canvas UI
│   ├── composables/             # Vue composables for agent/onboarding/hotkeys
│   ├── lib/                     # Frontend helpers and Tauri command wrappers
│   ├── modules/                 # VRM expression, lip sync, eye tracking, hit test
│   ├── renderers/               # Three.js + three-vrm renderer
│   ├── App.vue
│   └── main.ts
├── src-tauri/                   # Rust backend and Tauri shell
│   ├── src/
│   │   ├── agent/               # Primary agent, toolset, delegation, coding agent
│   │   ├── config/              # App config, user profile, secret store
│   │   ├── gateway/             # Discord/webhook and cron gateway
│   │   ├── growth/              # Growth engine and skills
│   │   ├── llm/                 # Provider adapters, streaming, routing, caching
│   │   ├── mcp/                 # Fairy MCP server
│   │   ├── memory/              # SQLite memory, KG, embeddings, shared backend
│   │   ├── plugins/             # JSON plugin loader
│   │   ├── security/            # Injection guard, command guard, redaction
│   │   ├── tools/               # Tool registry, executor, manifests, builtins, MCP
│   │   ├── voice/               # ASR/TTS/VAD/audio input
│   │   ├── text.rs              # UTF-8 safe text truncation helpers
│   │   ├── lib.rs
│   │   └── main.rs
│   ├── Cargo.toml
│   ├── Cargo.lock
│   └── tauri.conf.json
├── public/
│   └── models/default/          # Default GLB character asset
├── config/default.json          # Public default config template
├── docs/                        # User-facing setup guides
├── plugins/                     # Example plugin manifests
├── scripts/                     # Model download helpers
├── soul/                        # Fairy personality and identity prompts
├── README.md
├── CHANGELOG.md
├── CLAUDE.md
├── CONTRIBUTING.md
├── package.json
└── package-lock.json
```

## Notes

- Runtime state lives under `~/.fairyfield/`.
- API keys are kept out of public config and stored in the secret store.
- Downloaded voice models are intentionally outside the repository.
- The frontend and backend communicate only through typed Tauri command wrappers in `src/lib/tauri-commands.ts`.
