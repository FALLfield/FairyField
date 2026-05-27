# Contributing to FairyField

Thanks for your interest in contributing! FairyField is an open-source desktop AI companion project.

## Development Setup

```bash
git clone https://github.com/fallfield/FairyField.git
cd FairyField/FairyField
npm install
```

## Branch Naming

- `feature/short-description` — new features
- `fix/short-description` — bug fixes
- `docs/short-description` — documentation only

## Code Conventions

See `CLAUDE.md` for full conventions. Quick summary:

- **Frontend**: TypeScript strict, Vue 3 Composition API, 2-space indent
- **Backend**: Rust 2021, `rustfmt` + `clippy`, 4-space indent
- **Comments**: Chinese or English, API naming in English
- **Tests**: Vitest (frontend), `#[test]` (backend)
- **Target**: 80%+ test coverage

## Before Submitting

```bash
# Frontend
npm run test          # all frontend tests must pass
npm run build         # no errors

# Backend
cd src-tauri
cargo test            # all Rust tests must pass
cargo clippy -- -D warnings
cargo fmt --check
```

## Pull Request Process

1. Fork and create your branch from `main`
2. Write or update tests as needed
3. Ensure all tests pass
4. Update documentation if needed
5. Submit PR with clear description

## Architecture

The project follows a multi-agent architecture (see `CLAUDE.md`):
- `src/` — Vue 3 frontend (3D rendering, chat UI)
- `src-tauri/src/` — Rust backend (agent, voice, memory, security, tools)

## Getting Help

- Open an issue for bugs or feature requests
- Check `docs/USAGE_GUIDE.md` for setup help
- See `ARCHITECTURE.md` and `PROJECT_STRUCTURE.md` for technical architecture details
