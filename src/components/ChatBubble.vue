<script setup lang="ts">
import { computed } from 'vue';
import type { Emotion } from '../lib/emotion-engine';

interface ChatBubbleProps {
  message: string;
  isUser: boolean;
  emotion?: Emotion;
  isLoading?: boolean;
  timestamp?: number;
}

const props = withDefaults(defineProps<ChatBubbleProps>(), {
  emotion: 'neutral',
  isLoading: false,
  timestamp: undefined,
});

/** 情绪对应的 AI 气泡背景色 */
const EMOTION_COLORS: Record<Emotion, string> = {
  happy: 'rgba(250, 204, 21, 0.18)',
  sad: 'rgba(96, 165, 250, 0.18)',
  angry: 'rgba(248, 113, 113, 0.18)',
  surprised: 'rgba(192, 132, 252, 0.18)',
  neutral: 'rgba(30, 30, 30, 0.8)',
  thinking: 'rgba(45, 55, 72, 0.85)',
};

/** 情绪对应的 AI 气泡边框色 */
const EMOTION_BORDERS: Record<Emotion, string> = {
  happy: 'rgba(250, 204, 21, 0.35)',
  sad: 'rgba(96, 165, 250, 0.35)',
  angry: 'rgba(248, 113, 113, 0.35)',
  surprised: 'rgba(192, 132, 252, 0.35)',
  neutral: 'transparent',
  thinking: 'rgba(96, 165, 250, 0.25)',
};

/** 情绪对应的说话指示点颜色 */
const EMOTION_DOT_COLORS: Record<Emotion, string> = {
  happy: '#facc15',
  sad: '#60a5fa',
  angry: '#f87171',
  surprised: '#c084fc',
  neutral: '#a3a3a3',
  thinking: '#60a5fa',
};

const aiBubbleStyle = computed(() => {
  if (props.isUser) return {};
  return {
    backgroundColor: EMOTION_COLORS[props.emotion],
    borderColor: EMOTION_BORDERS[props.emotion],
  };
});

const dotColor = computed(() => {
  return EMOTION_DOT_COLORS[props.emotion];
});

/** 将时间戳转换为相对时间字符串 */
const relativeTime = computed<string>(() => {
  if (!props.timestamp) return '';
  const now = Date.now();
  const diff = now - props.timestamp;

  if (diff < 0) return '';
  if (diff < 10_000) return '刚刚';
  if (diff < 60_000) return `${Math.floor(diff / 1000)}秒前`;
  if (diff < 3_600_000) return `${Math.floor(diff / 60_000)}分钟前`;
  if (diff < 86_400_000) return `${Math.floor(diff / 3_600_000)}小时前`;

  const date = new Date(props.timestamp);
  const month = date.getMonth() + 1;
  const day = date.getDate();
  return `${month}月${day}日`;
});
</script>

<template>
  <div
    class="chat-bubble"
    :class="{
      'is-user': isUser,
      'is-ai': !isUser,
      'is-loading': isLoading,
    }"
    :style="aiBubbleStyle"
    role="log"
    :aria-live="isLoading ? 'assertive' : 'polite'"
  >
    <!-- 加载动画 -->
    <div v-if="isLoading" class="typing-indicator" aria-label="正在输入">
      <span class="dot" />
      <span class="dot" />
      <span class="dot" />
    </div>

    <!-- 正常消息 -->
    <template v-else>
      <div class="bubble-row">
        <!-- AI 说话指示点（仅 AI 气泡显示） -->
        <span
          v-if="!isUser && emotion !== 'neutral'"
          class="speaking-dot"
          :style="{ backgroundColor: dotColor }"
          aria-hidden="true"
        />
        <p class="bubble-text">{{ message }}</p>
      </div>
      <time
        v-if="relativeTime"
        class="bubble-time"
        :datetime="new Date(timestamp!).toISOString()"
      >
        {{ relativeTime }}
      </time>
    </template>
  </div>
</template>

<style scoped>
.chat-bubble {
  max-width: 380px;
  padding: 10px 14px;
  border-radius: 12px;
  font-size: 0.875rem;
  line-height: 1.5;
  word-break: break-word;
  pointer-events: none;
  border: 1px solid transparent;
  transition: background-color 0.3s ease, border-color 0.3s ease;
}

.is-user {
  align-self: flex-end;
  background: rgba(59, 130, 246, 0.85);
  color: #fff;
  border-bottom-right-radius: 4px;
}

.is-ai {
  align-self: flex-start;
  background: rgba(30, 30, 30, 0.8);
  color: #e5e5e5;
  border-bottom-left-radius: 4px;
}

.is-loading {
  padding: 12px 18px;
}

/* === 气泡内容行 === */
.bubble-row {
  display: flex;
  align-items: flex-start;
  gap: 6px;
}

.bubble-text {
  margin: 0;
  flex: 1;
}

/* === AI 说话指示点 === */
.speaking-dot {
  flex-shrink: 0;
  width: 6px;
  height: 6px;
  border-radius: 50%;
  margin-top: 4px;
  opacity: 0.8;
  animation: speaking-fade 2s ease-in-out infinite;
}

@keyframes speaking-fade {
  0%, 100% {
    opacity: 0.4;
  }
  50% {
    opacity: 0.9;
  }
}

/* === 相对时间 === */
.bubble-time {
  display: block;
  margin-top: 4px;
  font-size: 0.625rem;
  color: rgba(255, 255, 255, 0.2);
  text-align: right;
  letter-spacing: 0.02em;
}

.is-ai .bubble-time {
  text-align: left;
  color: rgba(255, 255, 255, 0.18);
}

/* 打字指示器动画 */
.typing-indicator {
  display: flex;
  gap: 4px;
  align-items: center;
  height: 16px;
}

.dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: #a3a3a3;
  animation: dot-bounce 1.2s ease-in-out infinite;
}

.dot:nth-child(2) {
  animation-delay: 0.15s;
}

.dot:nth-child(3) {
  animation-delay: 0.3s;
}

@keyframes dot-bounce {
  0%, 60%, 100% {
    transform: translateY(0);
    opacity: 0.4;
  }
  30% {
    transform: translateY(-6px);
    opacity: 1;
  }
}

@media (prefers-reduced-motion: reduce) {
  .dot {
    animation: none;
  }
  .speaking-dot {
    animation: none;
  }
}
</style>
