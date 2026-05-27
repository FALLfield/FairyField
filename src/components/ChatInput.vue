<script setup lang="ts">
import { ref } from 'vue';

defineProps<{
  inputText: string;
  isStreaming: boolean;
  isRecording: boolean;
  recordingText: string;
}>();

const emit = defineEmits<{
  (e: 'update:inputText', value: string): void;
  (e: 'send'): void;
  (e: 'abort'): void;
  (e: 'toggle-mic'): void;
}>();

const textareaRef = ref<HTMLTextAreaElement | null>(null);

function autoResize(): void {
  const ta = textareaRef.value;
  if (!ta) return;
  ta.rows = 1;
  const lineHeight = 24;
  const padding = 14;
  const scrollContentHeight = ta.scrollHeight - padding;
  const newRows = Math.min(5, Math.max(1, Math.ceil(scrollContentHeight / lineHeight)));
  ta.rows = newRows;
}

function onInput(e: Event): void {
  emit('update:inputText', (e.target as HTMLTextAreaElement).value);
  autoResize();
}

function onKeydown(e: KeyboardEvent): void {
  if (e.key === 'Enter' && !e.shiftKey) {
    e.preventDefault();
    emit('send');
  }
}
</script>

<template>
  <div class="input-area">
    <!-- 录音指示器（录制中替代 textarea） -->
    <div v-if="isRecording" class="recording-indicator">
      <span class="pulse-dot" />
      <span class="recording-text">{{ recordingText || '正在聆听...' }}</span>
      <button class="stop-recording-btn" @click="emit('toggle-mic')" aria-label="停止录音">停止</button>
    </div>
    <textarea
      ref="textareaRef"
      v-else
      :value="inputText"
      class="message-input"
      placeholder="跟 Fairy 说话..."
      rows="1"
      :disabled="isStreaming"
      @input="onInput"
      @keydown="onKeydown"
    />

    <button
      class="mic-btn"
      :class="{ 'is-recording': isRecording }"
      :disabled="isStreaming"
      aria-label="语音输入"
      title="语音输入"
      @click.stop="emit('toggle-mic')"
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
      @click="emit('send')"
    >{{ isStreaming ? '...' : '\u2191' }}</button>

    <button
      v-if="isStreaming"
      class="abort-btn"
      aria-label="中断回复"
      @click="emit('abort')"
    >停止</button>
  </div>
</template>

<style scoped>
.input-area {
  display: flex; gap: 6px; padding: 8px 10px;
  border-top: 1px solid rgba(255, 255, 255, 0.05);
  background: rgba(15, 15, 15, 0.35);
}

.message-input {
  flex: 1; resize: none;
  border: 1px solid rgba(255, 255, 255, 0.08); border-radius: 10px;
  background: rgba(255, 255, 255, 0.04); color: #e5e5e5;
  font-size: 0.8125rem; line-height: 1.5; padding: 7px 10px;
  font-family: inherit; outline: none;
  transition: border-color 0.2s ease, background-color 0.2s ease;
}
.message-input:focus { border-color: rgba(255, 255, 255, 0.15); background: rgba(255, 255, 255, 0.06); }
.message-input::placeholder { color: rgba(255, 255, 255, 0.22); }
.message-input:disabled { opacity: 0.45; }

.mic-btn {
  flex-shrink: 0; width: 34px; height: 34px; align-self: flex-end;
  border: 1px solid rgba(255, 255, 255, 0.08); border-radius: 10px;
  background: transparent; color: rgba(255, 255, 255, 0.45); cursor: pointer;
  display: flex; align-items: center; justify-content: center;
  transition: background 0.2s ease, color 0.2s ease, box-shadow 0.2s ease, border-color 0.2s ease;
}
.mic-btn:hover:not(:disabled) { background: rgba(255, 255, 255, 0.08); color: rgba(255, 255, 255, 0.75); border-color: rgba(255, 255, 255, 0.15); }
.mic-btn:disabled { opacity: 0.25; cursor: not-allowed; }
.mic-btn.is-recording {
  background: rgba(239, 68, 68, 0.2); color: #f87171;
  border-color: rgba(239, 68, 68, 0.45);
  box-shadow: 0 0 12px rgba(239, 68, 68, 0.3);
  animation: mic-glow 1.5s ease-in-out infinite;
}

@keyframes mic-glow { 0%, 100% { box-shadow: 0 0 12px rgba(239, 68, 68, 0.3); } 50% { box-shadow: 0 0 20px rgba(239, 68, 68, 0.55); } }

@media (prefers-reduced-motion: reduce) {
  .mic-btn.is-recording {
    animation: none;
  }
  .pulse-dot {
    animation: none;
  }
}

.recording-indicator {
  flex: 1; display: flex; align-items: center; gap: 8px;
  padding: 7px 10px; border: 1px solid rgba(239, 68, 68, 0.3); border-radius: 10px;
  background: rgba(239, 68, 68, 0.08); min-height: 34px;
}

.pulse-dot { flex-shrink: 0; width: 8px; height: 8px; border-radius: 50%; background: #f87171; animation: pulse-scale 1.2s ease-in-out infinite; }
@keyframes pulse-scale { 0%, 100% { transform: scale(1); opacity: 0.8; } 50% { transform: scale(1.6); opacity: 1; } }

.recording-text { flex: 1; font-size: 0.8125rem; color: rgba(248, 113, 113, 0.85); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

.stop-recording-btn {
  flex-shrink: 0; padding: 3px 10px; border: 1px solid rgba(239, 68, 68, 0.3); border-radius: 6px;
  background: rgba(239, 68, 68, 0.15); color: #fca5a5; font-size: 0.6875rem; cursor: pointer;
  transition: background 0.2s ease, color 0.2s ease;
}
.stop-recording-btn:hover { background: rgba(239, 68, 68, 0.3); color: #fecaca; }

.send-btn {
  flex-shrink: 0; width: 34px; height: 34px; align-self: flex-end;
  border: none; border-radius: 10px; background: rgba(255, 255, 255, 0.1);
  color: #e5e5e5; font-size: 14px; cursor: pointer;
  transition: background 0.2s ease, transform 0.15s ease;
}
.send-btn:hover:not(:disabled) { background: rgba(255, 255, 255, 0.18); transform: scale(1.05); }
.send-btn:disabled { opacity: 0.25; cursor: not-allowed; }

.abort-btn {
  flex-shrink: 0; height: 34px; padding: 0 10px; align-self: flex-end;
  border: none; border-radius: 10px; background: rgba(239, 68, 68, 0.45);
  color: #fff; font-size: 0.75rem; cursor: pointer; transition: background 0.2s ease;
}
.abort-btn:hover { background: rgba(239, 68, 68, 0.65); }
</style>
