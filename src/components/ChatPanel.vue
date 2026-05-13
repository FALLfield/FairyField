<script setup lang="ts">
/**
 * ChatPanel — 角色下方的对话面板容器
 *
 * 组合 ChatHistory（折叠/展开气泡列表）和 ChatInput（输入区域）。
 */
import { ref, nextTick, watch, useTemplateRef } from 'vue';
import ChatHistory from './ChatHistory.vue';
import ChatInput from './ChatInput.vue';
import type { AgentMessage } from '../composables/useAgent';
import type { Emotion } from '../lib/emotion-engine';

const props = defineProps<{
  messages: AgentMessage[];
  currentEmotion: Emotion;
  isStreaming: boolean;
  currentReply: string;
  error: string | null;
  isRecording: boolean;
  recordingText: string;
}>();

const emit = defineEmits<{
  (e: 'send', text: string): void;
  (e: 'abort'): void;
  (e: 'clear'): void;
  (e: 'toggle-mic'): void;
}>();

const isExpanded = ref(false);
const inputText = ref('');
const historyListRef = useTemplateRef<HTMLDivElement>('historyList');

function scrollToBottom(): void {
  nextTick(() => {
    if (historyListRef.value) historyListRef.value.scrollTop = historyListRef.value.scrollHeight;
  });
}

watch(() => props.messages.length, () => scrollToBottom());
watch(() => props.currentReply, () => scrollToBottom());

function handleSend(): void {
  const text = inputText.value.trim();
  if (!text || props.isStreaming) return;
  inputText.value = '';
  emit('send', text);
}
</script>

<template>
  <div class="chat-panel" :class="{ 'is-expanded': isExpanded }">
    <ChatHistory
      :messages="messages"
      :current-emotion="currentEmotion"
      :is-streaming="isStreaming"
      :current-reply="currentReply"
      :is-expanded="isExpanded"
      @expand="isExpanded = true; scrollToBottom()"
      @collapse="isExpanded = false"
      @clear="emit('clear')"
    />

    <div v-if="error" class="chat-error">{{ error }}</div>

    <ChatInput
      :input-text="inputText"
      :is-streaming="isStreaming"
      :is-recording="isRecording"
      :recording-text="recordingText"
      @update:input-text="inputText = $event"
      @send="handleSend"
      @abort="emit('abort')"
      @toggle-mic="emit('toggle-mic')"
    />
  </div>
</template>

<style scoped>
.chat-panel {
  display: flex; flex-direction: column;
  width: 380px; max-width: 90vw;
  background: rgba(10, 10, 10, 0.65);
  backdrop-filter: blur(20px) saturate(1.4);
  -webkit-backdrop-filter: blur(20px) saturate(1.4);
  border-radius: 16px;
  border: 1px solid rgba(255, 255, 255, 0.07);
  pointer-events: auto; overflow: hidden;
  transition: max-height 0.35s cubic-bezier(0.16, 1, 0.3, 1), background-color 0.3s ease;
  animation: panel-enter 0.4s cubic-bezier(0.16, 1, 0.3, 1) both;
}
.chat-panel:not(.is-expanded) { max-height: 180px; background: rgba(10, 10, 10, 0.45); border-color: rgba(255, 255, 255, 0.04); }
.chat-panel.is-expanded { max-height: 50vh; }

.chat-error {
  padding: 6px 12px; font-size: 0.75rem;
  color: #f87171; background: rgba(248, 113, 113, 0.06);
}

@keyframes panel-enter {
  from { opacity: 0; transform: translateY(12px) scale(0.97); }
  to { opacity: 1; transform: translateY(0) scale(1); }
}

@media (prefers-reduced-motion: reduce) {
  .chat-panel {
    animation: none;
    transition: none;
  }
}
</style>
