export {
  // Phase 2 命令
  agentChat,
  agentChatStream,
  agentGetEmotion,
  voiceStartAsr,
  voiceStartTts,
  voiceGetVadState,
  platformLoadConfig,
  platformSaveConfig,
  // Phase 0-1 兼容
  chat,
  getEmotionState,
  startAsr,
  startTts,
  getVadState,
  loadConfig,
  saveConfig,
  // 窗口命令
  startDrag,
  setIgnoreCursorEvents,
  getWindowSize,
} from './tauri-commands';

export type {
  LlmConfig,
  VoiceConfig,
  CharacterConfig,
  WindowConfig,
  MemoryConfig,
  GatewayConfig,
  UiConfig,
  AppConfig,
  StreamEvent,
} from './tauri-commands';

export { createEmotionEngine } from './emotion-engine';
export type { Emotion, EmotionState, EmotionWeights, EmotionEngine, UpperFaceBlendShapes, MouthInfluence } from './emotion-engine';
