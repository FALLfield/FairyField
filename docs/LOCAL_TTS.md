# Local TTS Strategy

FairyField needs a local girl voice that is fast, natural, and usable in both Chinese and English. The feature-enabled v1 direction is:

1. **Preferred feature build:** sherpa-onnx Matcha bilingual
   - Chinese: `matcha-icefall-zh-baker`
   - English: `matcha-icefall-en_US-ljspeech`
   - Shared vocoder: `vocos-22khz-univ.onnx`
2. **Fallback:** sherpa-onnx Kokoro multi-lang
3. **Last resort:** macOS `say`

## Actual Launch Modes

| Launch mode | TTS | ASR | VAD |
|---|---|---|---|
| Normal npm run tauri dev | macOS say on macOS, Mock elsewhere | Empty Mock result | Mock engine exists but is not fed audio |
| Dev with sherpa-onnx feature | Matcha when complete, then Kokoro, then platform fallback | Paraformer when model initialization succeeds | Silero can initialize but is not integrated into capture |

The Cargo default feature set is empty, so a standard build does not include the advertised sherpa voice stack. A model directory alone does not enable it.

## Why Not Kokoro First

Kokoro multi-lang is convenient and expressive in English, but it has two problems for FairyField:

- Higher local CPU latency than Matcha.
- Chinese support depends on lexicon/rule files and still sounds less natural than dedicated Chinese models.

Kokoro remains useful as a fallback because it is a single multi-language model and already works through sherpa-onnx.

## Why Matcha Bilingual

Matcha has separate high-speed Chinese and English models. FairyField routes mixed text by script:

- Chinese characters go to the Chinese Baker female model.
- English words go to the English LJSpeech female model.
- Punctuation and whitespace stay with the nearest language segment.

This is not as seamless as a proprietary streaming voice stack, but it gives much lower local latency and better pronunciation than forcing one model to read both languages.

## Latency Design

The local pipeline contains three latency/quality mitigations:

- TTS text is cleaned before synthesis, so markdown, code blocks, URLs, emoji, and internal emotion/tool tags are not read aloud.
- Long replies are split into sentence-sized chunks, so the first sentence can synthesize and play before the full response has been converted to audio.
- Microphone ASR input is captured from the system default device, downmixed to mono, resampled to 16 kHz, and rejected when it is silent or too short. This prevents empty buffers from producing nonsense text.

Future improvement for ChatGPT Voice-like responsiveness:

- Stream LLM tokens into a sentence queue.
- Start TTS as soon as the first sentence is complete.
- Feed generated PCM to the frontend lip-sync channel while audio plays.

## Current Limits

- The v1 path is local-first and turn-based. It starts faster than full-message synthesis, but it is not yet a true duplex streaming voice call like ChatGPT Voice.
- The frontend calls TTS only after the complete agent response returns. Backend chat streaming is simulated after full completion, so sentence chunking does not reduce LLM-to-first-audio latency.
- Recording is one fixed three-second capture. The UI stop action does not cancel backend capture.
- VAD state never changes through the production microphone path because captured samples do not call VAD detection.
- No backend code emits the PCM events expected by the renderer. Audio-driven lip sync is disconnected.
- Simulated lip sync has a state bug that can suppress continued mouth motion and leave the mouth partly open after speech.
- Matcha bilingual routing uses script-based segmentation. Chinese and English pronunciation are much better than one-model Kokoro fallback, but cross-language prosody is still a Phase 7 refinement.
- Japanese is not a first-class route yet. Current routing treats ASCII as English and CJK ideographs as Chinese; kana-specific Japanese handling still needs a real model and tests.
- TTS cleanup removes many markdown/tool artifacts, but it is not yet a strict "Chinese + English + Japanese + safe punctuation only" allowlist.
- Real STT quality is not proven by unit tests. The user has reported Chinese speech producing nonsense text such as "oah"; that requires real microphone diagnostics and ASR model-path verification.
- macOS `say` remains a last-resort fallback only; when it is used, voice quality depends on installed system voices.
- The current macOS fallback always requests the Tingting Chinese voice, including for English text.
- Voice configuration changes do not rebuild the running VoiceState; restart is required.
- The model downloader tolerates component failures and may still print a completion message, so file-level model health checks are required.

## Open Voice Bugs

Do not treat these as solved until they pass manual app testing:

- Chinese microphone input must transcribe Chinese accurately on the user's actual input device.
- English microphone input must transcribe English accurately.
- Mixed Chinese/English replies must not speak markdown, JSON, URLs, command output, tool logs, or other strange symbols.
- First audio should start from the first stable sentence, not after the whole answer is complete.
- TTS playback should drive lip-sync data while the VRM is speaking.
- The app needs a model health check that reports which Matcha, Kokoro, Paraformer, and Silero files are installed or missing.
- The main UI must display ASR/TTS errors instead of silently swallowing them.
- TTS must expose stop/barge-in and emit request-correlated playback state.

## Install

```bash
bash scripts/download-models.sh
```

Optional Kokoro fallback:

```bash
bash scripts/download-models.sh --kokoro
```

Run with real voice engines:

```bash
npm run tauri -- dev --features sherpa-onnx
```
