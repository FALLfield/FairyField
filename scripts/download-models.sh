#!/bin/bash
# =============================================================================
# FairyField sherpa-onnx 模型下载脚本
# =============================================================================
# 下载离线语音处理所需的 ONNX 模型文件：
#   - Kokoro TTS（多语言语音合成）    ~350MB
#   - Paraformer ASR（中文流式识别）  ~200MB
#   - silero-vad（语音活动检测）      ~2MB
# 总计约 550MB
# =============================================================================

set -euo pipefail

MODEL_DIR="${FAIRYFIELD_MODEL_DIR:-$HOME/.fairyfield/models}"
KOKORO_DIR="$MODEL_DIR/kokoro"
PARAFORMER_DIR="$MODEL_DIR/paraformer"
VAD_DIR="$MODEL_DIR/vad"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log()  { echo -e "${GREEN}[INFO]${NC} $*"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $*"; }
err()  { echo -e "${RED}[ERROR]${NC} $*"; }

# ---- Check prerequisites ----
check_deps() {
    if ! command -v curl &>/dev/null; then
        err "需要 curl，请先安装"
        exit 1
    fi
    if ! command -v tar &>/dev/null; then
        err "需要 tar，请先安装"
        exit 1
    fi
}

# ---- Download Kokoro TTS ----
download_kokoro() {
    log "下载 Kokoro TTS 模型 (~350MB)..."
    mkdir -p "$KOKORO_DIR"

    local kokoro_url="https://github.com/k2-fsa/sherpa-onnx/releases/download/tts-models/kokoro-multi-lang-v1_0.tar.bz2"

    if [ -f "$KOKORO_DIR/model.onnx" ] && [ -f "$KOKORO_DIR/voices.bin" ]; then
        warn "Kokoro 模型已存在，跳过 ($KOKORO_DIR)"
        return 0
    fi

    local tmp_file="$(mktemp)"
    if curl -L --progress-bar "$kokoro_url" -o "$tmp_file"; then
        tar -xjf "$tmp_file" -C "$KOKORO_DIR" --strip-components=1 2>/dev/null || \
        tar -xjf "$tmp_file" -C "$KOKORO_DIR"
        rm -f "$tmp_file"
        log "Kokoro TTS 模型下载完成"
    else
        rm -f "$tmp_file"
        err "Kokoro TTS 下载失败"
        err "手动下载: $kokoro_url"
        return 1
    fi
}

# ---- Download Paraformer ASR ----
download_paraformer() {
    log "下载 Paraformer ASR 模型 (~200MB)..."
    mkdir -p "$PARAFORMER_DIR"

    local asr_url="https://github.com/k2-fsa/sherpa-onnx/releases/download/asr-models/sherpa-onnx-paraformer-zh-2024-03-09.tar.bz2"

    if [ -f "$PARAFORMER_DIR/model.onnx" ]; then
        warn "Paraformer 模型已存在，跳过 ($PARAFORMER_DIR)"
        return 0
    fi

    local tmp_file="$(mktemp)"
    if curl -L --progress-bar "$asr_url" -o "$tmp_file"; then
        tar -xjf "$tmp_file" -C "$PARAFORMER_DIR" --strip-components=1 2>/dev/null || \
        tar -xjf "$tmp_file" -C "$PARAFORMER_DIR"
        rm -f "$tmp_file"
        log "Paraformer ASR 模型下载完成"
    else
        rm -f "$tmp_file"
        err "Paraformer ASR 下载失败"
        err "手动下载: $asr_url"
        return 1
    fi
}

# ---- Download Silero VAD ----
download_vad() {
    log "下载 Silero VAD 模型 (~2MB)..."
    mkdir -p "$VAD_DIR"

    local vad_url="https://github.com/k2-fsa/sherpa-onnx/releases/download/asr-models/silero_vad.onnx"

    if [ -f "$VAD_DIR/silero-vad.onnx" ]; then
        warn "Silero VAD 模型已存在，跳过 ($VAD_DIR)"
        return 0
    fi

    if curl -L --progress-bar "$vad_url" -o "$VAD_DIR/silero-vad.onnx"; then
        log "Silero VAD 模型下载完成"
    else
        err "Silero VAD 下载失败"
        err "手动下载: $vad_url"
        return 1
    fi
}

# ---- Main ----
main() {
    echo ""
    echo "  ✨ FairyField sherpa-onnx 模型下载"
    echo "  目标目录: $MODEL_DIR"
    echo ""

    check_deps

    download_kokoro || true
    download_paraformer || true
    download_vad || true

    echo ""
    log "下载完成！"
    log "模型文件位于: $MODEL_DIR"
    log ""
    log "编译时启用真实语音引擎:"
    log "  cd src-tauri && cargo build --features sherpa-onnx"
    echo ""

    # Summary
    local total_size=$(du -sh "$MODEL_DIR" 2>/dev/null | cut -f1)
    log "模型总大小: ${total_size:-未知}"
}

main "$@"
