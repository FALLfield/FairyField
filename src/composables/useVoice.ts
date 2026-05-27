import { ref, readonly, onMounted, onUnmounted } from 'vue';
import * as tauriCommands from '../lib/tauri-commands';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

/**
 * 语音管道 composable
 *
 * 提供 ASR（语音识别）和 TTS（语音合成）的接口。
 * 通过 Tauri IPC 调用 sherpa-onnx 后端。
 * 监听语音事件，支持将识别文本直接转发给 Agent。
 */

export interface VoiceState {
  isListening: boolean;
  isSpeaking: boolean;
  lastRecognized: string;
}

/** TTS 音频 chunk 事件（用于 LipSyncModule） */
export interface TtsChunkEvent {
  /** PCM 音频数据 */
  samples: number[];
  /** 采样率 */
  sample_rate: number;
}

/** ASR 识别结果事件 */
export interface AsrResultEvent {
  /** 识别文本 */
  text: string;
  /** 是否为最终结果（vs 中间结果） */
  is_final: boolean;
}

export function useVoice() {
  const isListening = ref(false);
  const isSpeaking = ref(false);
  const isChatting = ref(false);
  const lastRecognized = ref('');
  const error = ref<string | null>(null);

  let asrUnlisten: UnlistenFn | null = null;
  let ttsUnlisten: UnlistenFn | null = null;

  /** TTS 音频数据回调（由外部 LipSyncModule 注册） */
  let onTtsChunk: ((event: TtsChunkEvent) => void) | null = null;

  /** ASR 识别结果回调（由外部注册，用于自动发送给 Agent） */
  let onAsrResult: ((text: string) => void) | null = null;

  /** 设置 TTS 音频数据回调 */
  function setTtsChunkHandler(handler: (event: TtsChunkEvent) => void): void {
    onTtsChunk = handler;
  }

  /** 设置 ASR 识别结果回调 */
  function setAsrResultHandler(handler: (text: string) => void): void {
    onAsrResult = handler;
  }

  /** 开始语音识别（ASR），返回识别到的文本 */
  async function startListening(): Promise<string> {
    error.value = null;
    try {
      isListening.value = true;
      const text = await tauriCommands.voiceStartAsr();
      lastRecognized.value = text;
      return text;
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : String(e);
      return '';
    } finally {
      isListening.value = false;
    }
  }

  /** 停止语音识别 */
  function stopListening(): void {
    isListening.value = false;
    lastRecognized.value = '';
  }

  /** 语音合成（TTS） */
  async function speak(text: string): Promise<void> {
    error.value = null;
    try {
      isSpeaking.value = true;
      await tauriCommands.voiceStartTts(text);
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : String(e);
    } finally {
      isSpeaking.value = false;
    }
  }

  /** 获取 VAD（语音活动检测）状态 */
  async function getVadState(): Promise<boolean> {
    return tauriCommands.voiceGetVadState();
  }

  /**
   * 语音对话：ASR → Agent → TTS 全流程
   *
   * @param agentSend - useAgent 的 send 函数，用于将文本发给 AI
   * @returns 识别到的用户文本，失败返回空字符串
   */
  async function voiceChat(agentSend: (message: string) => Promise<void>): Promise<string> {
    error.value = null;
    isChatting.value = true;

    try {
      // 1. ASR：获取语音识别文本
      const userText = await startListening();
      if (!userText) {
        return '';
      }

      // 2. Agent：将文本发给 AI 获取回复
      await agentSend(userText);

      // 3. TTS：获取最后一条 assistant 消息并播放语音
      //    agentSend 完成后，useAgent.messages 已经更新，
      //    但 useVoice 不直接依赖 useAgent 的内部状态，
      //    所以回复的语音播放由调用方通过 speak() 触发，
      //    或者通过 onAsrResult 回调链处理。
      return userText;
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : String(e);
      return '';
    } finally {
      isChatting.value = false;
    }
  }

  /** 注册 Tauri 事件监听 */
  onMounted(async () => {
    if (!tauriCommands.isTauriEnvironment()) return;

    // 监听 ASR 识别结果
    asrUnlisten = await listen<AsrResultEvent>('voice:asr_result', (event) => {
      lastRecognized.value = event.payload.text;
      if (event.payload.is_final && onAsrResult) {
        onAsrResult(event.payload.text);
      }
    });

    // 监听 TTS 音频 chunk
    ttsUnlisten = await listen<TtsChunkEvent>('voice:tts_chunk', (event) => {
      if (onTtsChunk) {
        onTtsChunk(event.payload);
      }
    });
  });

  /** 清理事件监听 */
  onUnmounted(() => {
    if (asrUnlisten) {
      asrUnlisten();
      asrUnlisten = null;
    }
    if (ttsUnlisten) {
      ttsUnlisten();
      ttsUnlisten = null;
    }
  });

  return {
    isListening: readonly(isListening),
    isSpeaking: readonly(isSpeaking),
    isChatting: readonly(isChatting),
    lastRecognized: readonly(lastRecognized),
    error: readonly(error),
    startListening,
    stopListening,
    speak,
    getVadState,
    voiceChat,
    setTtsChunkHandler,
    setAsrResultHandler,
  };
}
