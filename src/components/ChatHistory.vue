<script setup lang="ts">
/**
 * ChatHistory — 展开状态的对话历史 + 折叠状态的单条气泡
 */
import { computed } from 'vue';
import ChatBubble from './ChatBubble.vue';
import type { AgentMessage } from '../composables/useAgent';
import type { Emotion } from '../lib/emotion-engine';

const props = defineProps<{
  messages: AgentMessage[];
  currentEmotion: Emotion;
  isStreaming: boolean;
  currentReply: string;
  isExpanded: boolean;
}>();

const emit = defineEmits<{
  (e: 'expand'): void;
  (e: 'collapse'): void;
  (e: 'clear'): void;
}>();

const latestAiMessage = computed<AgentMessage | null>(() => {
  for (let i = props.messages.length - 1; i >= 0; i--) {
    if (props.messages[i].role === 'assistant') return props.messages[i];
  }
  return null;
});

const collapsedContent = computed(() => {
  if (props.isStreaming && props.currentReply) return { type: 'streaming' as const, text: props.currentReply };
  if (props.isStreaming) return { type: 'loading' as const, text: '' };
  if (latestAiMessage.value) return { type: 'message' as const, text: latestAiMessage.value.content, emotion: latestAiMessage.value.emotion ?? 'neutral' as Emotion, timestamp: latestAiMessage.value.timestamp };
  return { type: 'welcome' as const, text: '你好呀，我是 Fairy (◕‿◕)' };
});

function getMessageEmotion(msg: AgentMessage): Emotion {
  return (msg.role === 'assistant' && msg.emotion) ? msg.emotion : 'neutral';
}
</script>

<template>
  <!-- 展开状态 -->
  <div v-if="isExpanded" class="history-section">
    <div class="history-list">
      <ChatBubble v-if="messages.length === 0" message="你好呀，我是 Fairy (◕‿◕) 有什么想聊的吗？" :is-user="false" emotion="happy" />
      <ChatBubble v-for="msg in messages" :key="msg.id" :message="msg.content" :is-user="msg.role === 'user'" :emotion="getMessageEmotion(msg)" :timestamp="msg.timestamp" />
      <ChatBubble v-if="isStreaming && !currentReply" message="" :is-user="false" :is-loading="true" />
      <ChatBubble v-if="isStreaming && currentReply" :message="currentReply" :is-user="false" :emotion="currentEmotion" />
    </div>
    <div class="history-toolbar">
      <button class="toolbar-btn" aria-label="折叠对话" @click="emit('collapse')">收起</button>
      <button v-if="messages.length > 0" class="toolbar-btn" aria-label="清空对话" @click="emit('clear')">清空</button>
    </div>
  </div>

  <!-- 折叠状态 -->
  <div v-else class="collapsed-section" @click="emit('expand')">
    <div class="collapsed-bubble">
      <ChatBubble v-if="collapsedContent.type === 'streaming'" :message="collapsedContent.text" :is-user="false" :emotion="currentEmotion" />
      <ChatBubble v-else-if="collapsedContent.type === 'loading'" message="" :is-user="false" :is-loading="true" />
      <ChatBubble v-else-if="collapsedContent.type === 'message'" :message="collapsedContent.text" :is-user="false" :emotion="collapsedContent.emotion" :timestamp="collapsedContent.timestamp" />
      <ChatBubble v-else :message="collapsedContent.text" :is-user="false" emotion="happy" />
    </div>
    <span v-if="messages.length > 1" class="expand-hint">点击查看 {{ messages.length }} 条记录</span>
  </div>
</template>

<style scoped>
.history-section { flex: 1; display: flex; flex-direction: column; min-height: 0; }
.history-list {
  flex: 1; overflow-y: auto; padding: 10px 12px;
  display: flex; flex-direction: column; gap: 6px;
  max-height: calc(50vh - 120px);
  scrollbar-width: thin; scrollbar-color: rgba(255, 255, 255, 0.08) transparent;
}
.history-list::-webkit-scrollbar { width: 3px; }
.history-list::-webkit-scrollbar-thumb { background: rgba(255, 255, 255, 0.08); border-radius: 2px; }

.history-toolbar { display: flex; justify-content: space-between; padding: 4px 12px; border-top: 1px solid rgba(255, 255, 255, 0.04); }
.toolbar-btn { padding: 3px 8px; border: 1px solid rgba(255, 255, 255, 0.08); border-radius: 6px; background: transparent; color: rgba(255, 255, 255, 0.35); font-size: 0.6875rem; cursor: pointer; transition: color 0.2s ease, border-color 0.2s ease; }
.toolbar-btn:hover { color: rgba(255, 255, 255, 0.6); border-color: rgba(255, 255, 255, 0.15); }

.collapsed-section { padding: 10px 12px 6px; cursor: pointer; transition: background-color 0.2s ease; }
.collapsed-section:hover { background: rgba(255, 255, 255, 0.03); }
.collapsed-bubble { display: flex; justify-content: flex-start; }
.collapsed-bubble :deep(.chat-bubble) { max-width: 100%; pointer-events: none; }
.expand-hint { display: block; text-align: center; font-size: 0.6875rem; color: rgba(255, 255, 255, 0.25); margin-top: 4px; transition: color 0.2s ease; }
.collapsed-section:hover .expand-hint { color: rgba(255, 255, 255, 0.45); }
</style>
