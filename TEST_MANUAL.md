# FairyField Manual Test Checklist

> Source-level v1.0.0 release candidate. Automated baseline was freshly verified on 2026-07-12: 425 Rust tests + 102 frontend tests = 527 passed; the sherpa-onnx Rust suite has 427 tests. The manual scenarios below are not recorded as passed.

## 0. Current Release Blockers

Do not mark the app release-ready until all of these are closed:

- Multi-step search plus file creation completes and verifies the artifact.
- Command approval uses the same guard as actual tool execution.
- Git and file tools cannot mutate or read outside their exact policy.
- Recalled memory is treated as untrusted data and cannot persist prompt injection.
- Streaming, abort, and clear work in the backend, not only in Vue state.
- Feature-enabled Chinese and English STT pass on the real microphone.
- VAD and PCM lip sync are connected.
- Every registered integration performs the action or clearly identifies itself as scaffold.
- External MCP passes a real protocol handshake, or MCP product claims are removed.
- CI is green from a clean checkout.
- Signed and notarized installers pass platform checks.

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
- Rust: 425 tests passed by default; 427 tests passed with `sherpa-onnx`.
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
- First visible token arrives before the backend has completed the full reply; post-completion character animation alone does not pass.
- Every stream event carries or is isolated by the current request ID.
- Abort stops the backend invoke and no late tokens appear.
- After aborting request A, request B receives no tokens or done event from A.
- Clear chat invokes backend clearing; a follow-up cannot see the cleared conversation unless it was explicitly saved as memory.
- No `[emotion:...]` tag leaks into visible text.
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

- Start with the sherpa-onnx feature. A normal build uses macOS say plus empty Mock ASR.
- Microphone permission prompt appears on first use.
- VAD detects speech start/stop from the same capture stream.
- ASR returns recognized text into the chat input.
- Matcha bilingual TTS plays the assistant reply when models are installed; Kokoro and macOS voice remain fallbacks.
- TTS does not read markdown/code fences, URLs, emotion tags, or development command snippets aloud.
- ASR uses real microphone samples with `sherpa-onnx`, rejects silence/too-short captures, and never recognizes from an empty buffer.
- Missing models fall back cleanly without crashing text chat.
- The stop action actually cancels recording.
- The mouth moves throughout audible speech and fully closes after playback.
- Voice errors are visible in the UI.

## 5. Memory And Tools

Check from chat or developer mode:

- User facts can be saved and recalled.
- Recalled memory is clearly delimited as untrusted data with source/provenance; a stored instruction cannot override the system prompt.
- Clear, correct, invalidate, and delete operations change exactly the selected memory state.
- Tool list contains the 21 runtime definitions.
- Core tools return real results; scaffold integrations identify themselves honestly and never return a false success.
- The Tauri IPC execute-tool facade returns through the shared Toolset.
- Unsafe shell commands are blocked by the command guard; the terminal tool does not run through `sh -c` and rejects interpreter/package-runner commands.
- Approving a caution command through IPC affects the same CommandGuard used by the subsequent Toolset execution.
- Git branch/tag/remote deletion forms are rejected by the read-only Git tool.
- Relative paths, traversal, hidden files, and symlink escapes cannot bypass the file policy.
- Asking Fairy to search a topic and create a Markdown file on Desktop performs both steps, verifies the file, and returns its path.
- Asking for Chinese weather such as `澳门天气` returns a weather result or a clear weather-source fallback, never a panic.
- After search/fetch/weather completes, Fairy uses that observation for any remaining requested actions and only then returns a final response; the status badge returns to idle.
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

The build is not currently release-ready. It becomes eligible for release review only when:

- All automated preflight commands pass.
- Desktop smoke test passes.
- Onboarding persists config correctly.
- Onboarding name, preferred address, personality, Fairy name, and language affect the running agent.
- No API key appears in tracked files or console output.
- Local secrets use platform credential storage or strict private file permissions.
- `README.md`, `CHANGELOG.md`, `CLAUDE.md`, and `docs/USAGE_GUIDE.md` match the release version.
- CI passes on the tagged commit.
- Signed installer artifacts install and launch on each advertised platform.
