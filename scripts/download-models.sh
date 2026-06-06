#!/bin/bash
# =============================================================================
# FairyField sherpa-onnx 模型下载脚本
# =============================================================================
# 下载离线语音处理所需的 ONNX 模型文件：
#   - Matcha TTS（低延迟中英双语女声）~200MB
#   - Kokoro TTS（多语言兜底）        ~350MB
#   - Paraformer ASR（中文流式识别）  ~200MB
#   - silero-vad（语音活动检测）      ~2MB
# 默认下载 Matcha + ASR + VAD，约 400MB；传 --kokoro 可额外下载 Kokoro。
# =============================================================================

set -euo pipefail

MODEL_DIR="${FAIRYFIELD_MODEL_DIR:-$HOME/.fairyfield/models}"
MATCHA_DIR="$MODEL_DIR/matcha"
KOKORO_DIR="$MODEL_DIR/kokoro"
PARAFORMER_DIR="$MODEL_DIR/paraformer"
VAD_DIR="$MODEL_DIR/vad"
DOWNLOAD_KOKORO=0

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log()  { echo -e "${GREEN}[INFO]${NC} $*"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $*"; }
err()  { echo -e "${RED}[ERROR]${NC} $*"; }

parse_args() {
    for arg in "$@"; do
        case "$arg" in
            --kokoro)
                DOWNLOAD_KOKORO=1
                ;;
            --all)
                DOWNLOAD_KOKORO=1
                ;;
            -h|--help)
                echo "Usage: $0 [--kokoro|--all]"
                echo "  默认下载低延迟 Matcha 双语 TTS + Paraformer + VAD"
                echo "  --kokoro / --all 额外下载 Kokoro multi-lang 兜底模型"
                exit 0
                ;;
            *)
                warn "忽略未知参数: $arg"
                ;;
        esac
    done
}

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

download_file() {
    local url="$1"
    local output="$2"
    local part="${output}.part"

    mkdir -p "$(dirname "$output")"
    if [ -f "$output" ]; then
        warn "文件已存在，跳过 ($output)"
        return 0
    fi

    if curl -L -f -C - --progress-bar "$url" -o "$part"; then
        mv "$part" "$output"
        return 0
    fi

    warn "下载未完成，已保留可续传文件: $part"
    return 1
}

download_and_extract() {
    local url="$1"
    local archive="$2"
    local dest="$3"
    mkdir -p "$dest"

    download_file "$url" "$archive" || return 1
    tar -xjf "$archive" -C "$dest"
}

# ---- Download Matcha bilingual TTS ----
download_matcha() {
    log "下载 Matcha 双语 TTS 模型 (~200MB)..."
    mkdir -p "$MATCHA_DIR"

    local zh_url="https://github.com/k2-fsa/sherpa-onnx/releases/download/tts-models/matcha-icefall-zh-baker.tar.bz2"
    local en_url="https://github.com/k2-fsa/sherpa-onnx/releases/download/tts-models/matcha-icefall-en_US-ljspeech.tar.bz2"
    local vocoder_url="https://github.com/k2-fsa/sherpa-onnx/releases/download/vocoder-models/vocos-22khz-univ.onnx"

    if [ -f "$MATCHA_DIR/matcha-icefall-zh-baker/model-steps-3.onnx" ] \
       && [ -f "$MATCHA_DIR/matcha-icefall-en_US-ljspeech/model-steps-3.onnx" ] \
       && [ -f "$MATCHA_DIR/vocos-22khz-univ.onnx" ]; then
        warn "Matcha 双语模型已存在，跳过 ($MATCHA_DIR)"
        return 0
    fi

    if [ ! -f "$MATCHA_DIR/matcha-icefall-zh-baker/model-steps-3.onnx" ]; then
        if ! download_and_extract "$zh_url" "$MATCHA_DIR/matcha-icefall-zh-baker.tar.bz2" "$MATCHA_DIR"; then
            err "Matcha 中文模型下载失败"
            return 1
        fi
    fi

    if [ ! -f "$MATCHA_DIR/matcha-icefall-en_US-ljspeech/model-steps-3.onnx" ]; then
        if ! download_and_extract "$en_url" "$MATCHA_DIR/matcha-icefall-en_US-ljspeech.tar.bz2" "$MATCHA_DIR"; then
            err "Matcha 英文模型下载失败"
            return 1
        fi
    fi

    if [ ! -f "$MATCHA_DIR/vocos-22khz-univ.onnx" ]; then
        if ! download_file "$vocoder_url" "$MATCHA_DIR/vocos-22khz-univ.onnx"; then
            err "Matcha vocoder 下载失败"
            return 1
        fi
    fi

    log "Matcha 双语 TTS 模型下载完成"
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

    local archive="$KOKORO_DIR/kokoro-multi-lang-v1_0.tar.bz2"
    if download_file "$kokoro_url" "$archive"; then
        tar -xjf "$archive" -C "$KOKORO_DIR" --strip-components=1 2>/dev/null || \
        tar -xjf "$archive" -C "$KOKORO_DIR"
        log "Kokoro TTS 模型下载完成"
    else
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

    local archive="$PARAFORMER_DIR/sherpa-onnx-paraformer-zh-2024-03-09.tar.bz2"
    if download_file "$asr_url" "$archive"; then
        tar -xjf "$archive" -C "$PARAFORMER_DIR" --strip-components=1 2>/dev/null || \
        tar -xjf "$archive" -C "$PARAFORMER_DIR"
        log "Paraformer ASR 模型下载完成"
    else
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

    if download_file "$vad_url" "$VAD_DIR/silero-vad.onnx"; then
        log "Silero VAD 模型下载完成"
    else
        err "Silero VAD 下载失败"
        err "手动下载: $vad_url"
        return 1
    fi
}

# ---- Main ----
main() {
    parse_args "$@"

    echo ""
    echo "  ✨ FairyField sherpa-onnx 模型下载"
    echo "  目标目录: $MODEL_DIR"
    echo ""

    check_deps

    download_matcha || true
    if [ "$DOWNLOAD_KOKORO" -eq 1 ]; then
        download_kokoro || true
    else
        warn "跳过 Kokoro 兜底模型；如需安装请运行: bash scripts/download-models.sh --kokoro"
    fi
    download_paraformer || true
    download_vad || true

    echo ""
    log "下载完成！"
    log "模型文件位于: $MODEL_DIR"
    log ""
    log "编译时启用真实语音引擎:"
    log "  cd src-tauri && cargo build --features sherpa-onnx"
    log "运行时默认优先使用 Matcha 双语 TTS；缺失时回退 Kokoro/macOS say。"
    echo ""

    # Summary
    local total_size=$(du -sh "$MODEL_DIR" 2>/dev/null | cut -f1)
    log "模型总大小: ${total_size:-未知}"
}

main "$@"
