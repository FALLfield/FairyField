<script setup lang="ts">
/**
 * ChatPanel — 角色下方的对话面板
 *
 * 默认折叠：只显示最新一条回复气泡 + 输入框
 * 展开后：显示完整对话历史
 * 气泡边框/背景色跟随情绪变化
 */
import { ref, computed, nextTick, watch, useTemplateRef } from 'vue';
import ChatBubble from './ChatBubble.vue';
import type { AgentMessage } from '../composables/useAgent';
import type { Emotion } from '../lib/emotion-engine';

interface ChatPanelProps {
  messages: AgentMessage[];
  currentEmotion: Emotion;
  isStreaming: boolean;
  currentReply: string;
  error: string | null;
  isRecording: boolean;
  recordingText: string;
}

const props = defineProps<ChatPanelProps>();

const emit = defineEmits<{
  (e: 'send', text: string): void;
  (e: 'abort'): void;
  (e: 'clear'): void;
  (e: 'toggle-mic'): void;
}>();

const isExpanded = ref(false);
const inputText = ref('');
const historyListRef = useTemplateRef<HTMLDivElement>('historyList');

/** 最新一条 AI 回复（用于折叠态显示） */
const latestAiMessage = computed<AgentMessage | null>(() => {
  for (let i = props.messages.length - 1; i >= 0; i--) {
    if (props.messages[i].role === 'assistant') {
      return props.messages[i];
    }
  }
  return null;
});

/** 折叠态要显示的内容：流式回复 > 最新AI回复 > 欢迎 */
const collapsedContent = computed(() => {
  if (props.isStreaming && props.currentReply) {
    return { type: 'streaming' as const, text: props.currentReply };
  }
  if (props.isStreaming) {
    return { type: 'loading' as const, text: '' };
  }
  if (latestAiMessage.value) {
    return {
      type: 'message' as const,
      text: latestAiMessage.value.content,
      emotion: latestAiMessage.value.emotion ?? 'neutral',
      timestamp: latestAiMessage.value.timestamp,
    };
  }
  return { type: 'welcome' as const, text: '你好呀，我是 Natasha (◕‿◕)' };
});

/** 获取消息对应的情绪 */
function getMessageEmotion(msg: AgentMessage): Emotion {
  if (msg.role === 'assistant' && msg.emotion) {
    return msg.emotion;
  }
  return 'neutral';
}

/** 自动滚动历史列表到底部 */
function scrollToBottom(): void {
  nextTick(() => {
    if (historyListRef.value) {
      historyListRef.value.scrollTop = historyListRef.value.scrollHeight;
    }
  });
}

watch(() => props.messages, () => scrollToBottom(), { deep: true });
watch(() => props.currentReply, () => scrollToBottom());

/** 展开面板 */
function expand(): void {
  isExpanded.value = true;
  scrollToBottom();
}

/** 折叠面板 */
function collapse(): void {
  isExpanded.value = false;
}

/** 发送消息 */
function handleSend(): void {
  const text = inputText.value.trim();
  if (!text || props.isStreaming) return;
  inputText.value = '';
  emit('send', text);
}

/** 按键处理 */
function handleKeydown(event: KeyboardEvent): void {
  if (event.key === 'Enter' && !event.shiftKey) {
    event.preventDefault();
    handleSend();
  }
}

/** 折叠态气泡点击 → 展开 */
function onCollapsedBubbleClick(): void {
  if (!isExpanded.value) {
    expand();
  }
}
</script>

<template>
  <div class="chat-panel" :class="{ 'is-expanded': isExpanded }">
    <!-- 展开状态：完整历史列表 -->
    <div v-if="isExpanded" class="history-section">
      <div ref="historyList" class="history-list">
        <!-- 欢迎消息 -->
        <ChatBubble
          v-if="messages.length === 0"
          message="你好呀，我是 Natasha (◕‿◕) 有什么想聊的吗？"
          :is-user="false"
          emotion="happy"
        />

        <!-- 历史消息 -->
        <ChatBubble
          v-for="msg in messages"
          :key="msg.id"
          :message="msg.content"
          :is-user="msg.role === 'user'"
          :emotion="getMessageEmotion(msg)"
          :timestamp="msg.timestamp"
        />

        <!-- 流式加载指示 -->
        <ChatBubble
          v-if="isStreaming && !currentReply"
          message=""
          :is-user="false"
          :is-loading="true"
        />

        <!-- 流式回复中 -->
        <ChatBubble
          v-if="isStreaming && currentReply"
          :message="currentReply"
          :is-user="false"
          :emotion="currentEmotion"
        />
      </div>

      <!-- 历史面板底部工具栏 -->
      <div class="history-toolbar">
        <button
          class="toolbar-btn collapse-btn"
          aria-label="折叠对话"
          @click="collapse"
        >
          收起
        </button>
        <button
          v-if="messages.length > 0"
          class="toolbar-btn clear-btn"
          aria-label="清空对话"
          @click="emit('clear')"
        >
          清空
        </button>
      </div>
    </div>

    <!-- 折叠状态：单条最新回复 -->
    <div
      v-else
      class="collapsed-section"
      @click="onCollapsedBubbleClick"
    >
      <!-- 最新回复气泡 -->
      <div class="collapsed-bubble">
        <ChatBubble
          v-if="collapsedContent.type === 'streaming'"
          :message="collapsedContent.text"
          :is-user="false"
          :emotion="currentEmotion"
        />
        <ChatBubble
          v-else-if="collapsedContent.type === 'loading'"
          message=""
          :is-user="false"
          :is-loading="true"
        />
        <ChatBubble
          v-else-if="collapsedContent.type === 'message'"
          :message="collapsedContent.text"
          :is-user="false"
          :emotion="collapsedContent.emotion"
          :timestamp="collapsedContent.timestamp"
        />
        <ChatBubble
          v-else
          :message="collapsedContent.text"
          :is-user="false"
          emotion="happy"
        />
      </div>

      <!-- 展开提示（有历史时显示） -->
      <span v-if="messages.length > 1" class="expand-hint">
        点击查看 {{ messages.length }} 条记录
      </span>
    </div>

    <!-- 错误提示 -->
    <div v-if="error" class="chat-error">
      {{ error }}
    </div>

    <!-- 输入区域（始终可见） -->
    <div class="input-area">
      <!-- 录音指示器（录制中时显示，替代 textarea） -->
      <div v-if="isRecording" class="recording-indicator">
        <span class="pulse-dot" />
        <span class="recording-text">{{ recordingText || '正在聆听...' }}</span>
        <button
          class="stop-recording-btn"
          @click="emit('toggle-mic')"
          aria-label="停止录音"
        >
          停止
        </button>
      </div>
      <textarea
        v-else
        v-model="inputText"
        class="message-input"
        placeholder="跟 Natasha 说话..."
        rows="1"
        :disabled="isStreaming"
        @keydown="handleKeydown"
      />

      <!-- 语音输入按钮 -->
      <button
        class="mic-btn"
        :class="{ 'is-recording': isRecording }"
        :disabled="isStreaming"
        aria-label="语音输入"
        @click="emit('toggle-mic')"
        title="语音输入"
      >
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3z"/>
          <path d="M19 10v2a7 7 0 0 1-14 0v-2"/>
          <line x1="12" y1="19" x2="12" y2="23"/>
          <line x1="8" y1="23" x2="16" y2="23"/>
        </svg>
      </button>

      <button
        class="send-btn"
        :disabled="isStreaming || !inputText.trim()"
        aria-label="发送消息"
        @click="handleSend"
      >
        {{ isStreaming ? '...' : '\u2191' }}
      </button>
      <button
        v-if="isStreaming"
        class="abort-btn"
        aria-label="中断回复"
        @click="emit('abort')"
      >
        停止
      </button>
    </div>
  </div>
</template>

<style scoped>
.chat-panel {
  display: flex;
  flex-direction: column;
  width: 380px;
  max-width: 90vw;
  background: rgba(10, 10, 10, 0.65);
  backdrop-filter: blur(20px) saturate(1.4);
  -webkit-backdrop-filter: blur(20px) saturate(1.4);
  border-radius: 16px;
  border: 1px solid rgba(255, 255, 255, 0.07);
  pointer-events: auto;
  overflow: hidden;
  transition: max-height 0.35s cubic-bezier(0.16, 1, 0.3, 1),
              background-color 0.3s ease;
}

/* 折叠态：更透明、更紧凑 */
.chat-panel:not(.is-expanded) {
  max-height: 180px;
  background: rgba(10, 10, 10, 0.45);
  border-color: rgba(255, 255, 255, 0.04);
}

/* 展开态 */
.chat-panel.is-expanded {
  max-height: 50vh;
}

/* === 折叠区 === */
.collapsed-section {
  padding: 10px 12px 6px;
  cursor: pointer;
  transition: background-color 0.2s ease;
}

.collapsed-section:hover {
  background: rgba(255, 255, 255, 0.03);
}

.collapsed-bubble {
  display: flex;
  justify-content: flex-start;
}

.collapsed-bubble .chat-bubble {
  max-width: 100%;
  pointer-events: none;
}

.expand-hint {
  display: block;
  text-align: center;
  font-size: 0.6875rem;
  color: rgba(255, 255, 255, 0.25);
  margin-top: 4px;
  transition: color 0.2s ease;
}

.collapsed-section:hover .expand-hint {
  color: rgba(255, 255, 255, 0.45);
}

/* === 展开态：历史列表 === */
.history-section {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.history-list {
  flex: 1;
  overflow-y: auto;
  padding: 10px 12px;
  display: flex;
  flex-direction: column;
  gap: 6px;
  max-height: calc(50vh - 120px);
  scrollbar-width: thin;
  scrollbar-color: rgba(255, 255, 255, 0.08) transparent;
}

.history-list::-webkit-scrollbar {
  width: 3px;
}

.history-list::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.08);
  border-radius: 2px;
}

/* 历史面板工具栏 */
.history-toolbar {
  display: flex;
  justify-content: space-between;
  padding: 4px 12px;
  border-top: 1px solid rgba(255, 255, 255, 0.04);
}

.toolbar-btn {
  padding: 3px 8px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 6px;
  background: transparent;
  color: rgba(255, 255, 255, 0.35);
  font-size: 0.6875rem;
  cursor: pointer;
  transition: color 0.2s ease, border-color 0.2s ease;
}

.toolbar-btn:hover {
  color: rgba(255, 255, 255, 0.6);
  border-color: rgba(255, 255, 255, 0.15);
}

/* === 错误提示 === */
.chat-error {
  padding: 6px 12px;
  font-size: 0.75rem;
  color: #f87171;
  background: rgba(248, 113, 113, 0.06);
}

/* === 输入区域 === */
.input-area {
  display: flex;
  gap: 6px;
  padding: 8px 10px;
  border-top: 1px solid rgba(255, 255, 255, 0.05);
  background: rgba(15, 15, 15, 0.35);
}

.message-input {
  flex: 1;
  resize: none;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 10px;
  background: rgba(255, 255, 255, 0.04);
  color: #e5e5e5;
  font-size: 0.8125rem;
  line-height: 1.5;
  padding: 7px 10px;
  font-family: inherit;
  outline: none;
  transition: border-color 0.2s ease, background-color 0.2s ease;
}

.message-input:focus {
  border-color: rgba(255, 255, 255, 0.15);
  background: rgba(255, 255, 255, 0.06);
}

.message-input::placeholder {
  color: rgba(255, 255, 255, 0.22);
}

.message-input:disabled {
  opacity: 0.45;
}

/* === 语音输入按钮 === */
.mic-btn {
  flex-shrink: 0;
  width: 34px;
  height: 34px;
  align-self: flex-end;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 10px;
  background: transparent;
  color: rgba(255, 255, 255, 0.45);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background 0.2s ease, color 0.2s ease, box-shadow 0.2s ease,
              border-color 0.2s ease;
}

.mic-btn:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.08);
  color: rgba(255, 255, 255, 0.75);
  border-color: rgba(255, 255, 255, 0.15);
}

.mic-btn:disabled {
  opacity: 0.25;
  cursor: not-allowed;
}

.mic-btn.is-recording {
  background: rgba(239, 68, 68, 0.2);
  color: #f87171;
  border-color: rgba(239, 68, 68, 0.45);
  box-shadow: 0 0 12px rgba(239, 68, 68, 0.3);
  animation: mic-glow 1.5s ease-in-out infinite;
}

@keyframes mic-glow {
  0%, 100% {
    box-shadow: 0 0 12px rgba(239, 68, 68, 0.3);
  }
  50% {
    box-shadow: 0 0 20px rgba(239, 68, 68, 0.55);
  }
}

/* === 录音指示器 === */
.recording-indicator {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 7px 10px;
  border: 1px solid rgba(239, 68, 68, 0.3);
  border-radius: 10px;
  background: rgba(239, 68, 68, 0.08);
  min-height: 34px;
}

.pulse-dot {
  flex-shrink: 0;
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #f87171;
  animation: pulse-scale 1.2s ease-in-out infinite;
}

@keyframes pulse-scale {
  0%, 100% {
    transform: scale(1);
    opacity: 0.8;
  }
  50% {
    transform: scale(1.6);
    opacity: 1;
  }
}

.recording-text {
  flex: 1;
  font-size: 0.8125rem;
  color: rgba(248, 113, 113, 0.85);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.stop-recording-btn {
  flex-shrink: 0;
  padding: 3px 10px;
  border: 1px solid rgba(239, 68, 68, 0.3);
  border-radius: 6px;
  background: rgba(239, 68, 68, 0.15);
  color: #fca5a5;
  font-size: 0.6875rem;
  cursor: pointer;
  transition: background 0.2s ease, color 0.2s ease;
}

.stop-recording-btn:hover {
  background: rgba(239, 68, 68, 0.3);
  color: #fecaca;
}

.send-btn {
  flex-shrink: 0;
  width: 34px;
  height: 34px;
  align-self: flex-end;
  border: none;
  border-radius: 10px;
  background: rgba(255, 255, 255, 0.1);
  color: #e5e5e5;
  font-size: 14px;
  cursor: pointer;
  transition: background 0.2s ease, transform 0.15s ease;
}

.send-btn:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.18);
  transform: scale(1.05);
}

.send-btn:disabled {
  opacity: 0.25;
  cursor: not-allowed;
}

.abort-btn {
  flex-shrink: 0;
  height: 34px;
  padding: 0 10px;
  align-self: flex-end;
  border: none;
  border-radius: 10px;
  background: rgba(239, 68, 68, 0.45);
  color: #fff;
  font-size: 0.75rem;
  cursor: pointer;
  transition: background 0.2s ease;
}

.abort-btn:hover {
  background: rgba(239, 68, 68, 0.65);
}

/* === 进场动画 === */
.chat-panel {
  animation: panel-enter 0.4s cubic-bezier(0.16, 1, 0.3, 1) both;
}

@keyframes panel-enter {
  from {
    opacity: 0;
    transform: translateY(12px) scale(0.97);
  }
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}
</style>
