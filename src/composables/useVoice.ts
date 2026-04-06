import { ref } from 'vue';

/**
 * 语音管道 composable（stub）
 *
 * 提供 ASR（语音识别）和 TTS（语音合成）的接口。
 * Phase 2 实现时接入 sherpa-onnx 后端。
 */

export interface VoiceState {
  isListening: boolean;
  isSpeaking: boolean;
  lastRecognized: string;
}

export function useVoice() {
  const isListening = ref(false);
  const isSpeaking = ref(false);
  const lastRecognized = ref('');
  const error = ref<string | null>(null);

  /** 开始语音识别（ASR） */
  async function startListening(): Promise<void> {
    // TODO: Phase 2 — 调用 tauriCommands.startAsr()
    isListening.value = true;
    error.value = null;
  }

  /** 停止语音识别 */
  async function stopListening(): Promise<void> {
    // TODO: Phase 2 — 调用 Tauri IPC 停止 ASR
    isListening.value = false;
  }

  /** 语音合成（TTS） */
  async function speak(text: string): Promise<void> {
    // TODO: Phase 2 — 调用 tauriCommands.startTts(text)
    void text;
    isSpeaking.value = true;
  }

  /** 获取 VAD（语音活动检测）状态 */
  async function getVadState(): Promise<boolean> {
    // TODO: Phase 2 — 调用 tauriCommands.getVadState()
    return false;
  }

  return {
    isListening,
    isSpeaking,
    lastRecognized,
    error,
    startListening,
    stopListening,
    speak,
    getVadState,
  };
}
