# FairyField Manual Test Checklist

> v1.0.0 release candidate, verified 2026-07-08. Automated baseline: 409 Rust tests + 102 frontend tests = 511 passed. With `sherpa-onnx`, the Rust suite has 411 tests.

## 1. Automated Preflight

Run from `FairyField/`:

```bash
npm run build
npm run test
```

Run from `FairyField/src-tauri/`:

```bash
cargo check
cargo test
cargo clippy -- -D warnings
cargo fmt --check
cargo check --features sherpa-onnx
cargo test --features sherpa-onnx
cargo clippy --features sherpa-onnx -- -D warnings
```

Expected:

- Frontend: 9 files, 102 tests passed.
- Rust: 409 tests passed by default; 411 tests passed with `sherpa-onnx`.
- No clippy warnings in default or `sherpa-onnx` builds.

## 2. Desktop Smoke Test

Run:

```bash
npm run tauri dev
```

Check:

- Transparent always-on-top window opens.
- Default 3D character loads from `public/models/default/2031903848872972007.glb`.
- Chat input accepts text and sends a message.
- Streaming reply appears without leaking `[emotion:...]` tags.
- ControlPanel is top-left and collapsed by default.
- ChatPanel remains usable while the canvas is visible.
- Ctrl+Shift+D toggles developer mode.
- Ctrl+1 through Ctrl+8 trigger expression debug hotkeys.

## 3. Onboarding

On a fresh profile or after temporarily moving `~/.fairyfield/user.json`:

- First launch shows the onboarding wizard.
- Step 1 stores user name.
- Step 2 stores call preference.
- Step 3 stores personality preference.
- Step 4 can skip LLM config or save/test a provider key.
- `~/.fairyfield/user.json` is created.
- API keys are not written to public `config/default.json`.

## 4. Voice

With models installed under `~/.fairyfield/models/`:

- Microphone permission prompt appears on first use.
- VAD detects speech start/stop.
- ASR returns recognized text into the chat input.
- Matcha bilingual TTS plays the assistant reply when models are installed; Kokoro and macOS voice remain fallbacks.
- TTS does not read markdown/code fences, URLs, emotion tags, or development command snippets aloud.
- ASR uses real microphone samples with `sherpa-onnx`, rejects silence/too-short captures, and never recognizes from an empty buffer.
- Missing models fall back cleanly without crashing text chat.

## 5. Memory And Tools

Check from chat or developer mode:

- User facts can be saved and recalled.
- Memory search wraps recalled context in `<memory-context>`.
- Tool list includes builtins, MCP-related tools, and community plugin surfaces.
- `fairy.execute_tool` returns a real tool result rather than a stub.
- Unsafe shell commands are blocked by the command guard; the terminal tool does not run through `sh -c` and rejects interpreter/package-runner commands.
- Asking for Chinese weather such as `澳门天气` returns a weather result or a clear weather-source fallback, never a panic.
- After search/fetch/weather completes, the chat shows the tool result directly and the status badge returns to idle.
- Asking `web_fetch` for a public page returns text within the tool timeout.
- `web_fetch` rejects localhost, private IPs, unsupported schemes, and unsafe redirects.
- Large web pages are capped and marked as truncated instead of blocking the agent loop.

## 6. Development Loop

Run a dry-run plan from the frontend wrapper or IPC command:

- `development_loop_plan` returns ManagerAgent, CodingAgent, TestingAgent, and GoalAgent tasks.
- Dry runs return `success: false` with an unmet requirement explaining that no code/tests ran.
- Non-dry-run loops include Git-detected changed files in CodingAgent artifacts.
- If tests fail and iterations remain, the next CodingAgent prompt includes the previous failed test output.
- GoalAgent fails when no Git-detected changed files exist or when changed files are outside `files_allowed`.

## 7. Release Gate

The build is release-ready when:

- All automated preflight commands pass.
- Desktop smoke test passes.
- Onboarding persists config correctly.
- No API key appears in tracked files or console output.
- `README.md`, `CHANGELOG.md`, `CLAUDE.md`, and `docs/USAGE_GUIDE.md` match the release version.
