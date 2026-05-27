/**
 * Tauri IPC 调用封装
 *
 * 定义所有前后端通信的 TypeScript 类型和安全封装。
 * 对应 src-tauri/src/ 中的 Tauri Commands。
 */

import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

export function isTauriEnvironment(): boolean {
  if (typeof window === 'undefined') return false;
  const w = window as typeof window & {
    __TAURI_INTERNALS__?: unknown;
    __TAURI__?: unknown;
  };
  return Boolean(w.__TAURI_INTERNALS__ || w.__TAURI__);
}

function browserPreviewEmotion(): EmotionState {
  return {
    current: 'happy',
    intensity: 0.55,
    values: {
      happy: 0.55,
      neutral: 0.45,
    },
  };
}

// ---------------------------------------------------------------------------
// 类型定义
// ---------------------------------------------------------------------------

/** 情绪类型 */
export type Emotion =
  | 'happy'
  | 'sad'
  | 'angry'
  | 'surprised'
  | 'neutral'
  | 'thinking';

/** LLM 配置 */
export interface LlmConfig {
  provider: string;
  api_endpoint: string;
  model: string;
  system_prompt: string;
  max_context_tokens: number;
  temperature: number;
}

/** 语音配置 */
export interface VoiceConfig {
  asr_enabled: boolean;
  tts_enabled: boolean;
  vad_enabled: boolean;
  tts_speaker: string;
  asr_model_path: string;
  tts_model_path: string;
  vad_model_path: string;
}

/** 角色配置 */
export interface CharacterConfig {
  model_path: string;
  default_expression: string;
  name: string;
  soul_path: string;
}

/** 窗口配置 */
export interface WindowConfig {
  width: number;
  height: number;
  transparent: boolean;
  always_on_top: boolean;
  click_through: boolean;
}

/** 记忆系统配置 */
export interface MemoryConfig {
  db_path: string;
  embedding_dim: number;
  wakeup_max_tokens: number;
  dedup_threshold: number;
}

/** 通信网关配置 */
export interface GatewayConfig {
  telegram_enabled: boolean;
  telegram_token: string;
  cron_enabled: boolean;
}

/** UI 配置 */
export interface UiConfig {
  chat_bubble_max_width: number;
  message_font_size: number;
  show_control_panel: boolean;
  accent_color: string;
}

/** 应用配置 */
export interface AppConfig {
  llm: LlmConfig;
  voice: VoiceConfig;
  character: CharacterConfig;
  window: WindowConfig;
  memory: MemoryConfig;
  gateway: GatewayConfig;
  ui: UiConfig;
}

/** 流式回复事件 */
export interface StreamEvent {
  type: 'chunk' | 'done' | 'error';
  content: string;
}

/** 情绪种类（对应 Rust EmotionKind） */
export type EmotionKind =
  | 'happy'
  | 'sad'
  | 'angry'
  | 'surprised'
  | 'neutral'
  | 'thinking'
  | 'excited'
  | 'shy'
  | 'upset';

/** 情绪状态（对应 Rust EmotionState） */
export interface EmotionState {
  /** 当前主情绪 */
  current: EmotionKind;
  /** 情绪强度 0.0 - 1.0 */
  intensity: number;
  /** 各情绪维度的原始值 */
  values: Record<string, number>;
}

/** AI 对话响应（对应 Rust ChatResponse） */
export interface ChatResponse {
  /** AI 回复文本（纯自然语言，不含情绪数据） */
  reply: string;
  /** 情绪状态 */
  emotion: EmotionState;
}

/** LLM 提供商预设 */
export interface ProviderPreset {
  name: string;
  provider_type: string;
  api_endpoint: string;
  model: string;
  api_key: string;
}

/** 首次启动用户配置 */
export interface UserConfig {
  user_name: string;
  call_preference: string;
  fairy_name: string;
  personality: 'warm' | 'playful' | 'quiet' | 'custom';
  custom_personality?: string | null;
  language: string;
  onboarding_completed: boolean;
  created_at?: string | null;
}

// ---------------------------------------------------------------------------
// AI Agent 命令（Phase 2 命名）
// ---------------------------------------------------------------------------

/** 发送聊天消息，返回回复和情绪 */
export async function agentChat(message: string): Promise<ChatResponse> {
  if (!isTauriEnvironment()) {
    return {
      reply: `浏览器预览模式：我收到啦。「${message}」`,
      emotion: browserPreviewEmotion(),
    };
  }
  return invoke<ChatResponse>('agent_chat', { message });
}

/** 发送聊天消息，流式返回回复。返回后端 ChatResponse 含 emotion */
export async function agentChatStream(
  message: string,
  onEvent: (event: StreamEvent) => void,
): Promise<ChatResponse> {
  if (!isTauriEnvironment()) {
    const reply = `浏览器预览模式：我收到啦。「${message}」`;
    for (const chunk of reply.match(/.{1,6}/gu) ?? [reply]) {
      onEvent({ type: 'chunk', content: chunk });
      await new Promise((resolve) => setTimeout(resolve, 10));
    }
    onEvent({ type: 'done', content: '' });
    return {
      reply,
      emotion: browserPreviewEmotion(),
    };
  }

  const unlistenToken = await listen<string>('agent:stream-token', (event) => {
    onEvent({ type: 'chunk', content: event.payload });
  });
  const unlistenDone = await listen<void>('agent:stream-done', () => {
    onEvent({ type: 'done', content: '' });
  });

  try {
    return await invoke<ChatResponse>('agent_chat_stream', { message });
  } finally {
    unlistenToken();
    unlistenDone();
  }
}

/** 获取当前情绪状态 */
export async function agentGetEmotion(): Promise<EmotionState> {
  if (!isTauriEnvironment()) return browserPreviewEmotion();
  return invoke<EmotionState>('agent_get_emotion');
}

// ---------------------------------------------------------------------------
// LLM 提供商管理命令
// ---------------------------------------------------------------------------

/** 列出所有可用的 LLM 提供商预设 */
export async function llmListProviders(): Promise<ProviderPreset[]> {
  if (!isTauriEnvironment()) {
    return [
      {
        name: 'Browser Preview',
        provider_type: 'mock',
        api_endpoint: 'browser-preview',
        model: 'preview',
        api_key: '',
      },
    ];
  }
  return invoke<ProviderPreset[]>('llm_list_providers');
}

/** 切换活跃的 LLM 提供商 */
export async function llmSwitchProvider(name: string): Promise<void> {
  if (!isTauriEnvironment()) {
    if (!name) throw new Error('请选择提供商');
    return;
  }
  return invoke<void>('llm_switch_provider', { name });
}

/** 获取当前活跃的 LLM 提供商 */
export async function llmGetActiveProvider(): Promise<ProviderPreset> {
  if (!isTauriEnvironment()) {
    return {
      name: 'Browser Preview',
      provider_type: 'mock',
      api_endpoint: 'browser-preview',
      model: 'preview',
      api_key: '',
    };
  }
  return invoke<ProviderPreset>('llm_get_active_provider');
}

/** 保存 LLM API Key 到后端密钥存储并重建 Agent */
export async function llmSaveApiKey(
  providerName: string,
  apiKey: string,
): Promise<void> {
  if (!isTauriEnvironment()) {
    if (!providerName || !apiKey) throw new Error('Provider 和 API Key 不能为空');
    return;
  }
  return invoke<void>('llm_save_api_key', { providerName, apiKey });
}

/** 测试 LLM Provider + API Key，不持久化 */
export async function llmTestProvider(
  providerName: string,
  apiKey: string,
): Promise<void> {
  if (!isTauriEnvironment()) {
    if (!providerName || !apiKey) throw new Error('Provider 和 API Key 不能为空');
    return;
  }
  return invoke<void>('llm_test_provider', { providerName, apiKey });
}

/** 加载首次启动用户配置 */
export async function userLoadConfig(): Promise<UserConfig> {
  return invoke<UserConfig>('user_load_config');
}

/** 保存首次启动用户配置 */
export async function userSaveConfig(config: UserConfig): Promise<void> {
  return invoke<void>('user_save_config', { config });
}

// ---------------------------------------------------------------------------
// 语音命令（Phase 2 命名）
// ---------------------------------------------------------------------------

/** 开始语音识别，返回识别到的文本 */
export async function voiceStartAsr(): Promise<string> {
  if (!isTauriEnvironment()) return '';
  return invoke<string>('voice_start_asr');
}

/** 开始语音合成 */
export async function voiceStartTts(text: string): Promise<void> {
  if (!isTauriEnvironment()) {
    if (!text.trim()) throw new Error('TTS 文本不能为空');
    return;
  }
  return invoke<void>('voice_start_tts', { text });
}

/** 获取 VAD（语音活动检测）状态 */
export async function voiceGetVadState(): Promise<boolean> {
  if (!isTauriEnvironment()) return false;
  return invoke<boolean>('voice_get_vad_state');
}

// ---------------------------------------------------------------------------
// 配置命令（Phase 2 命名）
// ---------------------------------------------------------------------------

/** 加载应用配置 */
export async function platformLoadConfig(): Promise<AppConfig> {
  return invoke<AppConfig>('platform_load_config');
}

/** 保存应用配置 */
export async function platformSaveConfig(config: AppConfig): Promise<void> {
  return invoke<void>('platform_save_config', { config });
}

// ---------------------------------------------------------------------------
// 兼容 Phase 0-1 命令别名
// ---------------------------------------------------------------------------

/** @deprecated 使用 agentChat */
export async function chat(message: string): Promise<ChatResponse> {
  return agentChat(message);
}

/** @deprecated 使用 agentGetEmotion */
export async function getEmotionState(): Promise<EmotionState> {
  return agentGetEmotion();
}

/** @deprecated 使用 voiceStartAsr */
export async function startAsr(): Promise<string> {
  return voiceStartAsr();
}

/** @deprecated 使用 voiceStartTts */
export async function startTts(text: string): Promise<number[]> {
  return invoke<number[]>('start_tts', { text });
}

/** @deprecated 使用 voiceGetVadState */
export async function getVadState(): Promise<boolean> {
  return voiceGetVadState();
}

/** @deprecated 使用 platformLoadConfig */
export async function loadConfig(): Promise<AppConfig> {
  return platformLoadConfig();
}

/** @deprecated 使用 platformSaveConfig */
export async function saveConfig(config: AppConfig): Promise<void> {
  return platformSaveConfig(config);
}

// ---------------------------------------------------------------------------
// 窗口命令
// ---------------------------------------------------------------------------

/** 开始窗口拖拽 */
export async function startDrag(): Promise<void> {
  return invoke<void>('start_drag');
}

/** 设置是否忽略鼠标事件（点击穿透） */
export async function setIgnoreCursorEvents(
  ignore: boolean,
): Promise<void> {
  return invoke<void>('set_ignore_cursor_events', { ignore });
}

/** 获取窗口尺寸 */
export async function getWindowSize(): Promise<{
  width: number;
  height: number;
}> {
  return invoke<{ width: number; height: number }>('get_window_size');
}
