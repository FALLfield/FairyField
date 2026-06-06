# Local TTS Strategy

FairyField needs a local girl voice that is fast, natural, and usable in both Chinese and English. The v1 direction is:

1. **Default:** sherpa-onnx Matcha bilingual
   - Chinese: `matcha-icefall-zh-baker`
   - English: `matcha-icefall-en_US-ljspeech`
   - Shared vocoder: `vocos-22khz-univ.onnx`
2. **Fallback:** sherpa-onnx Kokoro multi-lang
3. **Last resort:** macOS `say`

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

The local pipeline now reduces latency in two ways:

- TTS text is cleaned before synthesis, so markdown, code blocks, URLs, emoji, and internal emotion/tool tags are not read aloud.
- Long replies are split into sentence-sized chunks, so the first sentence can synthesize and play before the full response has been converted to audio.

Future improvement for ChatGPT Voice-like responsiveness:

- Stream LLM tokens into a sentence queue.
- Start TTS as soon as the first sentence is complete.
- Feed generated PCM to the frontend lip-sync channel while audio plays.

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
