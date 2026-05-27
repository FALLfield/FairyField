<script setup lang="ts">
import { ref, onErrorCaptured } from 'vue';
import CharacterCanvas from './components/CharacterCanvas.vue';
import ChatPanel from './components/ChatPanel.vue';
import ControlPanel from './components/ControlPanel.vue';
import MemoryPanel from './components/MemoryPanel.vue';
import ToolsPanel from './components/ToolsPanel.vue';
import AgentStatusBadge from './components/AgentStatusBadge.vue';
import AgentLogPanel from './components/AgentLogPanel.vue';
import OnboardingWizard from './components/OnboardingWizard.vue';
import { useAgent } from './composables/useAgent';
import { useVoice } from './composables/useVoice';
import { useDevMode } from './composables/useDevMode';
import { useAgentStatus } from './composables/useAgentStatus';
import { useOnboarding } from './composables/useOnboarding';
import type { OnboardingData } from './composables/useOnboarding';

const characterCanvasRef = ref<InstanceType<typeof CharacterCanvas> | null>(null);
const appError = ref<string | null>(null);

// Vue 错误边界 — 防止单个组件异常导致整个应用卸载
onErrorCaptured((err: Error, _instance, info: string) => {
  console.error('[FairyField] 组件错误:', info, err.message);
  appError.value = `发生了意外错误: ${err.message}`;
  setTimeout(() => { appError.value = null; }, 5000);
  return false; // 阻止错误继续传播
});

const fps = ref(0);
const modelLoaded = ref(false);
const errorMessage = ref('');

const {
  messages,
  isStreaming,
  currentReply,
  error: agentError,
  fairyEmotion,
  send,
  abort,
  clearHistory,
} = useAgent();

const {
  speak,
  startListening,
} = useVoice();

const { devMode, onAvatarTripleClick } = useDevMode();
const { status: agentStatus, toolLog, resetStatus: resetAgentStatus, setStatus: setAgentStatus } = useAgentStatus();
const { showOnboarding, isLoading: onboardingLoading, completeOnboarding } = useOnboarding();

// --- 录音状态 ---
const isRecording = ref(false);
const recordingText = ref('');

/** 发送消息并在 AI 回复完成后自动触发 TTS */
async function sendWithTts(text: string): Promise<void> {
  resetAgentStatus();
  try {
    await send(text);
    // 获取最后一条消息（send 完成后 assistant 消息已入列）
    const lastMsg = messages.value[messages.value.length - 1];
    if (lastMsg?.role === 'assistant' && lastMsg.content) {
      speak(lastMsg.content); // 不 await，异步播放不阻塞
    }
  } finally {
    setAgentStatus('idle');
  }
}

function abortAgentReply(): void {
  abort();
  setAgentStatus('idle');
}

/** 开始/停止语音识别 */
async function toggleRecording(): Promise<void> {
  if (isRecording.value) {
    // 停止录音
    isRecording.value = false;
    const text = recordingText.value.trim();
    if (text) {
      await sendWithTts(text);
    }
    recordingText.value = '';
  } else {
    // 开始录音
    isRecording.value = true;
    recordingText.value = '';
    try {
      const text = await startListening();
      if (text && isRecording.value) {
        recordingText.value = text;
        isRecording.value = false;
        await sendWithTts(text);
      }
    } catch (e) {
      if (import.meta.env.DEV) console.error('ASR 失败:', e);
    } finally {
      isRecording.value = false;
      recordingText.value = '';
    }
  }
}

function onModelLoaded(): void {
  modelLoaded.value = true;
  errorMessage.value = '';
}

function onModelError(message: string): void {
  modelLoaded.value = false;
  errorMessage.value = message;
}

function onFps(value: number): void {
  fps.value = value;
}

/** Handle onboarding wizard completion */
async function handleOnboardingComplete(data: OnboardingData): Promise<void> {
  await completeOnboarding(data);
}
</script>

<template>
  <!-- 首次启动引导向导 -->
  <OnboardingWizard
    v-if="showOnboarding && !onboardingLoading"
    :show="showOnboarding && !onboardingLoading"
    @complete="handleOnboardingComplete"
  />

  <!-- 加载状态 -->
  <div v-else-if="onboardingLoading" class="app-loading" aria-label="加载中">
    <div class="loading-spinner" />
  </div>

  <!-- 主界面 -->
  <main v-else class="app-root">
    <!-- 全局错误边界提示 -->
    <div v-if="appError" class="status-overlay status-error">
      <p>{{ appError }}</p>
    </div>

    <!-- 加载/错误提示 -->
    <div v-if="errorMessage" class="status-overlay status-error">
      <p>{{ errorMessage }}</p>
    </div>

    <!-- 3D 角色画布 -->
    <div @click="onAvatarTripleClick">
      <CharacterCanvas
        ref="characterCanvasRef"
        :emotion="fairyEmotion"
        @loaded="onModelLoaded"
        @error="onModelError"
        @fps="onFps"
      />
    </div>

    <!-- 角色下方的对话面板 -->
    <div class="chat-anchor">
      <ChatPanel
        :messages="messages"
        :current-emotion="fairyEmotion"
        :is-streaming="isStreaming"
        :current-reply="currentReply"
        :error="agentError"
        :is-recording="isRecording"
        :recording-text="recordingText"
        @send="sendWithTts"
        @abort="abortAgentReply"
        @clear="clearHistory"
        @toggle-mic="toggleRecording"
      />
    </div>

    <!-- 调试控制面板 -->
    <ControlPanel
      :fps="fps"
      :model-loaded="modelLoaded"
      :expression-module="characterCanvasRef?.getExpressionModule() ?? null"
    />

    <!-- 开发者面板（Phase 3 Wave 2） -->
    <MemoryPanel v-if="devMode" />
    <ToolsPanel v-if="devMode" />
    <AgentLogPanel v-if="devMode" :logs="toolLog" @clear="resetAgentStatus" />

    <!-- Agent 状态指示（始终渲染，idle 时自动隐藏） -->
    <AgentStatusBadge :status="agentStatus" />
  </main>
</template>

<style scoped>
.app-root {
  position: fixed;
  inset: 0;
  width: 100vw;
  height: 100vh;
  overflow: hidden;
  /* 透明背景 — Tauri 透明窗口需要 */
  background: transparent;
}

.status-overlay {
  position: fixed;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  z-index: 1000;
  padding: 1rem 1.5rem;
  border-radius: 8px;
  font-size: 0.875rem;
  text-align: center;
  pointer-events: none;
}

.status-error {
  background: rgba(220, 38, 38, 0.85);
  color: #fff;
}

/* 对话面板锚点：固定在底部中央，角色脚下 */
.chat-anchor {
  position: fixed;
  bottom: 24px;
  left: 50%;
  transform: translateX(-50%);
  z-index: 250;
  pointer-events: none;
  display: flex;
  justify-content: center;
}

/* 首次加载状态 */
.app-loading {
  position: fixed;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(10, 10, 18, 0.95);
  z-index: 1000;
}

.loading-spinner {
  width: 32px;
  height: 32px;
  border: 3px solid rgba(255, 255, 255, 0.1);
  border-top-color: rgba(180, 160, 255, 0.7);
  border-radius: 50%;
  animation: app-spin 0.7s linear infinite;
}

@keyframes app-spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
