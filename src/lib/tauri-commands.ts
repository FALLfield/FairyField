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

export type LoopAgentRole =
  | 'manager_agent'
  | 'coding_agent'
  | 'testing_agent'
  | 'goal_agent';

export type LoopStageStatus = 'planned' | 'skipped' | 'passed' | 'failed';
export type AgentTaskStatus =
  | 'queued'
  | 'scoping'
  | 'assigned'
  | 'coding'
  | 'testing'
  | 'integration'
  | 'blocked'
  | 'complete';

export interface LoopAgentPlan {
  role: LoopAgentRole;
  responsibility: string;
  inputs: string[];
  outputs: string[];
}

export interface AgentTask {
  id: string;
  goal_id: string;
  role: LoopAgentRole;
  owner: string;
  scope: string;
  files_allowed: string[];
  files_forbidden: string[];
  context_files: string[];
  acceptance: string[];
  status: AgentTaskStatus;
  dependencies: string[];
  risk: string;
  logs: string[];
  artifacts: string[];
}

export interface DevelopmentLoopPlan {
  objective: string;
  requirements: string[];
  context_files: string[];
  test_commands: string[];
  max_iterations: number;
  agents: LoopAgentPlan[];
  tasks: AgentTask[];
}

export interface DevelopmentLoopRequest {
  objective: string;
  requirements: string[];
  context_files: string[];
  coding_agent?: string | null;
  working_dir?: string | null;
  test_commands: string[];
  max_iterations?: number | null;
  model?: string | null;
  permission_mode?: string | null;
  dry_run: boolean;
}

export interface LoopStageResult {
  role: LoopAgentRole;
  status: LoopStageStatus;
  summary: string;
  artifacts: string[];
  duration_ms: number;
}

export interface AgentResult {
  task_id: string;
  role: LoopAgentRole;
  status: LoopStageStatus;
  summary: string;
  changed_files: string[];
  artifacts: string[];
  duration_ms: number;
}

export interface DevelopmentLoopReport {
  success: boolean;
  dry_run: boolean;
  iterations: number;
  plan: DevelopmentLoopPlan;
  stages: LoopStageResult[];
  agent_results: AgentResult[];
  unmet_requirements: string[];
  next_actions: string[];
}

function buildPreviewDevelopmentLoopPlan(
  objective: string,
  requirements: string[],
  contextFiles: string[],
  testCommands: string[],
  maxIterations?: number | null,
): DevelopmentLoopPlan {
  const normalizedRequirements = requirements.length > 0 ? requirements : [objective];
  const normalizedTests =
    testCommands.length > 0
      ? testCommands
      : ['npm run test', 'npm run build', 'cargo test'];
  const owner = contextFiles.some((file) => file.startsWith('src-tauri/src/tools/'))
    ? 'tools'
    : contextFiles.some((file) => file.startsWith('src-tauri/src/voice/'))
      ? 'voice'
      : contextFiles.some((file) => file.startsWith('src/renderers/'))
        ? 'renderer'
        : contextFiles.length > 0
          ? 'platform'
          : 'integration';
  const goalId = `goal-${objective
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-|-$/g, '')
    .split('-')
    .filter(Boolean)
    .slice(0, 6)
    .join('-') || 'development-loop'}`;
  const forbidden = [
    'files outside the selected working_dir',
    'unrelated user changes',
    'tracked secrets, API keys, or local model files',
    'Tauri release build artifacts',
  ];
  return {
    objective,
    requirements: normalizedRequirements,
    context_files: contextFiles,
    test_commands: normalizedTests,
    max_iterations: Math.min(Math.max(maxIterations ?? 1, 1), 3),
    agents: [
      {
        role: 'manager_agent',
        responsibility: 'Plan, assign, and coordinate the development loop.',
        inputs: ['objective', 'requirements', 'context_files'],
        outputs: ['DevelopmentLoopPlan'],
      },
      {
        role: 'coding_agent',
        responsibility: 'Operate the scoped coding task through a configured CLI.',
        inputs: ['CodingTask'],
        outputs: ['CodingResult'],
      },
      {
        role: 'testing_agent',
        responsibility: 'Run allowlisted verification commands and collect evidence.',
        inputs: ['test_commands'],
        outputs: ['test outputs'],
      },
      {
        role: 'goal_agent',
        responsibility: 'Audit requirements against coding and testing evidence.',
        inputs: ['stage results', 'requirements'],
        outputs: ['goal verdict'],
      },
    ],
    tasks: [
      {
        id: 'manager-scope',
        goal_id: goalId,
        role: 'manager_agent',
        owner: 'manager',
        scope: 'Convert the objective into bounded coding, testing, and goal-audit work.',
        files_allowed: ['planning metadata only'],
        files_forbidden: forbidden,
        context_files: contextFiles,
        acceptance: [
          'Plan contains manager, coding, testing, and goal agents.',
          'Plan records owner, file scope, requirements, and verification commands.',
        ],
        status: 'scoping',
        dependencies: [],
        risk: 'low',
        logs: [`objective: ${objective}`],
        artifacts: ['DevelopmentLoopPlan'],
      },
      {
        id: `coding-${owner}`,
        goal_id: goalId,
        role: 'coding_agent',
        owner,
        scope: 'Make the smallest project-scoped code changes needed to satisfy the goal.',
        files_allowed:
          contextFiles.length > 0
            ? contextFiles
            : ['project-scoped files required by objective'],
        files_forbidden: forbidden,
        context_files: contextFiles,
        acceptance: normalizedRequirements,
        status: 'queued',
        dependencies: ['manager-scope'],
        risk: owner === 'integration' ? 'medium' : 'low',
        logs: [],
        artifacts: ['CodingResult'],
      },
      {
        id: 'testing-regression',
        goal_id: goalId,
        role: 'testing_agent',
        owner: 'testing',
        scope: 'Run allowlisted verification commands and capture capped output evidence.',
        files_allowed: ['read-only test execution'],
        files_forbidden: forbidden,
        context_files: contextFiles,
        acceptance: normalizedTests,
        status: 'queued',
        dependencies: [`coding-${owner}`],
        risk: 'low',
        logs: [],
        artifacts: ['test outputs'],
      },
      {
        id: 'goal-audit',
        goal_id: goalId,
        role: 'goal_agent',
        owner: 'goal',
        scope: 'Compare requirements with coding and testing evidence before declaring success.',
        files_allowed: ['read-only evidence review'],
        files_forbidden: forbidden,
        context_files: contextFiles,
        acceptance: normalizedRequirements,
        status: 'queued',
        dependencies: ['testing-regression'],
        risk: 'low',
        logs: [],
        artifacts: ['goal verdict'],
      },
    ],
  };
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

export async function developmentLoopPlan(
  objective: string,
  requirements: string[] = [],
  contextFiles: string[] = [],
  testCommands: string[] = [],
  maxIterations?: number | null,
): Promise<DevelopmentLoopPlan> {
  if (!isTauriEnvironment()) {
    return buildPreviewDevelopmentLoopPlan(
      objective,
      requirements,
      contextFiles,
      testCommands,
      maxIterations,
    );
  }
  return invoke<DevelopmentLoopPlan>('development_loop_plan', {
    objective,
    requirements,
    contextFiles,
    testCommands,
    maxIterations,
  });
}

export async function developmentLoopRun(
  request: DevelopmentLoopRequest,
): Promise<DevelopmentLoopReport> {
  if (!isTauriEnvironment()) {
    const plan = buildPreviewDevelopmentLoopPlan(
      request.objective,
      request.requirements,
      request.context_files,
      request.test_commands,
      request.max_iterations,
    );
    return {
      success: false,
      dry_run: true,
      iterations: 0,
      plan,
      stages: [
        {
          role: 'manager_agent',
          status: 'passed',
          summary: 'Browser preview planned the development loop.',
          artifacts: [],
          duration_ms: 0,
        },
        {
          role: 'goal_agent',
          status: 'failed',
          summary: 'Browser preview cannot execute coding or testing agents.',
          artifacts: ['Run inside the Tauri app to execute the real loop.'],
          duration_ms: 0,
        },
      ],
      agent_results: [
        {
          task_id: 'manager-scope',
          role: 'manager_agent',
          status: 'passed',
          summary: 'Browser preview planned the development loop.',
          changed_files: [],
          artifacts: [],
          duration_ms: 0,
        },
        {
          task_id: 'goal-audit',
          role: 'goal_agent',
          status: 'failed',
          summary: 'Browser preview cannot execute coding or testing agents.',
          changed_files: [],
          artifacts: ['Run inside the Tauri app to execute the real loop.'],
          duration_ms: 0,
        },
      ],
      unmet_requirements: ['Browser preview did not execute coding or tests.'],
      next_actions: ['Open FairyField in Tauri and run the loop again.'],
    };
  }
  return invoke<DevelopmentLoopReport>('development_loop_run', { request });
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
