import { ref } from 'vue';
import * as tauriCommands from '../lib/tauri-commands';

/**
 * 语音管道 composable
 *
 * 提供 ASR（语音识别）和 TTS（语音合成）的接口。
 * 通过 Tauri IPC 调用 sherpa-onnx 后端。
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
    error.value = null;
    try {
      await tauriCommands.startAsr();
      isListening.value = true;
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : String(e);
    }
  }

  /** 停止语音识别 */
  async function stopListening(): Promise<void> {
    isListening.value = false;
  }

  /** 语音合成（TTS） */
  async function speak(text: string): Promise<void> {
    error.value = null;
    try {
      isSpeaking.value = true;
      await tauriCommands.startTts(text);
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : String(e);
    } finally {
      isSpeaking.value = false;
    }
  }

  /** 获取 VAD（语音活动检测）状态 */
  async function getVadState(): Promise<boolean> {
    return tauriCommands.getVadState();
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
