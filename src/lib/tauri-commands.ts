/**
 * Tauri IPC 调用封装
 *
 * 定义所有前后端通信的 TypeScript 类型和安全封装。
 * 对应 src-tauri/src/ 中的 Tauri Commands。
 */

import { invoke } from '@tauri-apps/api/core';

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
}

/** 窗口配置 */
export interface WindowConfig {
  width: number;
  height: number;
  transparent: boolean;
  always_on_top: boolean;
  click_through: boolean;
}

/** 应用配置 */
export interface AppConfig {
  llm: LlmConfig;
  voice: VoiceConfig;
  character: CharacterConfig;
  window: WindowConfig;
}

/** 流式回复事件 */
export interface StreamEvent {
  type: 'chunk' | 'done' | 'error';
  content: string;
}

// ---------------------------------------------------------------------------
// AI Agent 命令
// ---------------------------------------------------------------------------

/** 发送聊天消息，返回完整回复 */
export async function chat(message: string): Promise<string> {
  return invoke<string>('chat', { message });
}

/** 发送聊天消息，返回流式回复（SSE） */
export async function chatStream(
  message: string,
  onEvent: (event: StreamEvent) => void,
): Promise<void> {
  // TODO: Phase 3 — 使用 Tauri 事件监听实现 SSE
  void message;
  void onEvent;
}

/** 获取当前情绪状态 */
export async function getEmotionState(): Promise<Emotion> {
  return invoke<Emotion>('get_emotion_state');
}

// ---------------------------------------------------------------------------
// 语音命令
// ---------------------------------------------------------------------------

/** 开始语音识别 */
export async function startAsr(): Promise<void> {
  return invoke<void>('start_asr');
}

/** 开始语音合成，返回 PCM 音频数据 */
export async function startTts(text: string): Promise<number[]> {
  return invoke<number[]>('start_tts', { text });
}

/** 获取 VAD（语音活动检测）状态 */
export async function getVadState(): Promise<boolean> {
  return invoke<boolean>('get_vad_state');
}

// ---------------------------------------------------------------------------
// 配置命令
// ---------------------------------------------------------------------------

/** 加载应用配置 */
export async function loadConfig(): Promise<AppConfig> {
  return invoke<AppConfig>('load_config');
}

/** 保存应用配置 */
export async function saveConfig(config: AppConfig): Promise<void> {
  return invoke<void>('save_config', { config });
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
  return invoke<void>('set_ignore_cursor', { ignore });
}

/** 获取窗口尺寸 */
export async function getWindowSize(): Promise<{
  width: number;
  height: number;
}> {
  return invoke<{ width: number; height: number }>('get_window_size');
}
