# FairyField Phase 7 Roadmap

> Version 1 is a strong repository baseline and release candidate, not a fully polished public product yet. Phase 7 must close the real task, voice, memory, and tool gaps before the app should be marketed as a complete companion.

Reality checkpoint, 2026-07-10: v1 has a strong automated-test baseline, but the user's latest manual findings show that three areas must be treated as unfinished before adding more surface area: task completion after tool use, real voice/STT quality, and memory/tool parity with Hermes and MemPalace. See `docs/REALITY_CHECK.md` for the current bug list.

## 0A. Close Security And State Semantics

**Goal:** no tool, memory, or UI flow may claim safety while using a disconnected guard or trusting untrusted content.

- Share one CommandGuard between IPC approval and real Toolset execution.
- Replace Git prefix matching with exact read-only operations.
- Canonicalize file paths and defend symlink containment.
- Treat user input, web pages, tool output, and recalled memory as untrusted data with provenance.
- Add automatic secret redaction and private local credential permissions.
- Implement request-correlated streaming, backend abort, backend clear, and explicit persisted-memory controls.

**Acceptance target:** unsafe Git/file/memory prompts are blocked, approval affects the actual command, abort stops backend work, and clear removes exactly the selected state.

## 0B. Close The Actual Task Loop

**Goal:** Fairy must finish the user's requested action, not merely return the first useful tool result.

**Core loop update (2026-07-12):** PrimaryAgent now treats every tool result as an observation and continues until a no-tool response or bounded finalization. Regression coverage proves search, write, read-back verification, error recovery, duplicate blocking, and truthful incomplete fallbacks.

- Add an independent semantic Goal verifier instead of relying only on the model's completion decision.
- Add request cancellation, provider retry/failover, and tool-pair-aware context compaction.
- Add safe parallel execution for proven-independent read operations.
- Route long coding/project tasks into the Manager/Coding/Testing/Goal development loop.

**Acceptance target:** "Search the web for X and create a markdown file on my Desktop" creates the file, verifies it exists, and returns the Desktop path.

## 1. Streaming Voice Input And Output

**Goal:** make conversation feel close to real-time instead of "wait, then speak".

- Stream ASR partial text from microphone input into the chat composer.
- Stream LLM tokens into a sentence queue instead of waiting for the full reply.
- Start TTS when the first stable sentence is complete.
- Send playback amplitude/viseme hints to `LipSyncModule` while audio is playing.
- Measure latency with lightweight telemetry: mic stop to first token, first token to first audio, first audio to mouth movement.

**Acceptance target:** first audible response starts within 800-1500 ms for short replies on a normal Mac when the LLM provider is responsive and local TTS models are installed.

## 2. Voice Quality And Multilingual Reliability

**Goal:** keep a sweet local girl voice while supporting Chinese and English naturally.

- Finish Matcha bilingual model download reliability and resumable checksums.
- Add a model health command that reports missing Matcha/Kokoro/Paraformer/Silero files.
- Add TTS normalization tests for markdown, URLs, tool tags, emoji, Chinese punctuation, and mixed-language text.
- Add a voice settings UI for engine, speaking rate, pitch, and fallback order.
- Keep Kokoro optional, not default, unless future local multilingual quality improves.

**Acceptance target:** Chinese, English, and mixed Chinese-English replies do not read internal symbols aloud and do not crash when one model is missing.

## 3. Hermes-Scale Tools

**Goal:** grow from a broad built-in tool surface to a practical everyday tool system.

- Prioritize the tools people will actually use daily: browser/page reading, web search, weather, files, Git, Obsidian, reminders, email/calendar, and coding agents.
- Add a tool test harness that runs configured, unconfigured, offline, and unsafe-input scenarios for every tool.
- Replace false-ready scaffold responses with real execution or explicit unsupported status.
- Make tool results visible immediately in chat when they are already user-facing.
- Expand MCP import/export so Fairy can use external tools and external agents can use Fairy memory.
- Add per-tool permission UI: read-only, write-local, network, shell, OAuth, and destructive.

**Acceptance target:** every registered tool either executes, returns a clear configuration-required message, or is blocked by security policy within the configured timeout.

This acceptance target must not treat network_call skipped or queued-without-side-effect as execution.

## 3A. Release Engineering

**Goal:** convert the source candidate into a reproducible, installable release.

- Fix CI toolchain and Cargo.lock audit paths, then require a green tagged commit.
- Decide whether official builds include sherpa-onnx and make documentation match the package.
- Add model-health reporting and a supported model installation path.
- Produce signed and notarized macOS artifacts.
- Either verify Windows/Linux behavior or remove those platforms from v1 claims.
- Test clean install, first run, upgrade, uninstall, and Gatekeeper behavior.

**Acceptance target:** the tagged commit has green CI and signed artifacts that install and launch on every advertised platform.

## 4. UI Refinement

**Goal:** make Fairy feel like a companion living on the desktop, not a web panel floating over a model.

- Add compact and expanded chat layouts.
- Add a quiet voice-call mode where chat history can collapse while Fairy speaks.
- Add better first-run model status: text ready, voice ready, memory ready, tools ready.
- Add motion-safe settings for idle animation intensity, expression strength, and reduced animation.
- Improve empty/error states without explanatory clutter inside the main UI.

**Acceptance target:** the first screen remains the actual companion experience, with clear status and controls available but not visually competing with the character.

## 5. XR And Holographic Prototype Path

**Goal:** prepare a physical projection path without forcing it into v1 desktop stability.

- Phase 7A: camera-based head tracking for desktop parallax.
- Phase 7B: Pepper's ghost / acrylic prism prototype using the existing Three.js render output.
- Phase 7C: multi-view or pseudo-volumetric display experiment only after stable head tracking.
- Keep XR output as a renderer mode, not a fork of the app.
- Document hardware BOM, calibration steps, latency budget, and safety constraints.

**Acceptance target:** Fairy can run in a hologram preview mode that reuses the same character, animation, emotion, and lip-sync state as desktop mode.

## 6. Market Position

The AI companion category is growing quickly but is crowded. Appfigures data reported by TechCrunch said AI companion apps had 337 active revenue-generating apps worldwide, 60 million downloads in H1 2025, and a projected $120M+ 2025 revenue run rate. Market research also points to continued growth in text, voice, and multimodal companion segments.

FairyField should not compete as another mobile roleplay app. The sharper position is:

- local-first desktop companion
- visible embodied avatar, not only chat
- private memory on the user's machine
- coding/work assistant tools through MCP and local agents
- Chinese-English voice and desktop workflow support
- future XR/holographic embodiment

**Phase 7 business test:** publish an open-source v1, collect GitHub stars/issues/manual installs, then test whether users care most about local voice, persistent memory, desktop overlay, coding-agent integration, or XR.

## 7. HCI Research Directions

The strongest HCI directions for FairyField are not generic chatbot evaluation. They are about long-lived, embodied, memory-bearing agents.

- **Relational memory:** how persistent local memory changes trust, attachment, and repair after mistakes.
- **Embodied presence:** how avatar expression, gaze, idle motion, and lip-sync change perceived companionship.
- **Human-agent collaboration:** how a companion should ask, interrupt, delegate, and recover during real work.
- **Agency boundaries:** how to make autonomous tools useful without making the agent feel unsafe or intrusive.
- **Identity and personhood:** how users describe a persistent companion that remembers them but remains software.
- **XR transition:** how perceived presence changes from desktop overlay to projected/holographic embodiment.

Possible study sequence:

1. Two-week diary study comparing text-only, voice-only, and embodied desktop modes.
2. Lab study on response latency, lip-sync, and trust/comfort.
3. Field study on memory recall quality and user correction workflows.
4. XR prototype study comparing desktop parallax and physical projection.

## References

- TechCrunch / Appfigures: [AI companion apps on track to pull in $120M in 2025](https://techcrunch.com/2025/08/12/ai-companion-apps-on-track-to-pull-in-120m-in-2025/)
- Grand View Research: [United States AI Companion Market Size & Outlook](https://www.grandviewresearch.com/horizon/outlook/ai-companion-market/united-states)
- ACM CHI 2026: [Accepted Workshops](https://chi2026.acm.org/workshops/accepted/)
- ACM CHI 2026: [Accepted Panels](https://chi2026.acm.org/accepted-panels/)
- HCI International 2026: [AI in HCI Conference](https://2026.hci.international/ai-hci)
- arXiv 2026: [From Human-Human Collaboration to Human-Agent Collaboration](https://arxiv.org/abs/2602.05987)
- arXiv 2025: [Reframing Human-Robot Interaction Through Extended Reality](https://arxiv.org/abs/2512.02569)
