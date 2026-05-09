<script setup lang="ts">
import { computed } from 'vue';
import type { AgentStatus } from '../composables/useAgentStatus';

const props = defineProps<{ status: AgentStatus }>();

const STATUS_TEXT: Record<AgentStatus, string> = {
  idle: '',
  thinking: '思考中...',
  searching_memory: '搜索记忆中...',
  executing_tool: '执行工具中...',
};

const text = computed(() => STATUS_TEXT[props.status] ?? '');
const visible = computed(() => props.status !== 'idle');
</script>

<template>
  <Transition name="fade">
    <div v-if="visible" class="status-badge">
      <span class="dot" :class="`dot--${props.status}`"></span>
      <span class="text">{{ text }}</span>
    </div>
  </Transition>
</template>

<style scoped>
.status-badge {
  position: fixed;
  bottom: 100px;
  left: 50%;
  transform: translateX(-50%);
  z-index: 260;
  display: flex;
  align-items: center;
  gap: 6px;
  background: rgba(20, 20, 28, 0.88);
  backdrop-filter: blur(10px);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 20px;
  padding: 4px 14px;
  font-size: 12px;
  font-family: ui-monospace, monospace;
  color: #ccc;
  pointer-events: none;
}

.dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: #00d4aa;
  animation: pulse 1.2s ease-in-out infinite;
}

.dot--thinking {
  background: #fbbf24;
}

.dot--searching_memory {
  background: #60a5fa;
}

.dot--executing_tool {
  background: #c084fc;
}

.text {
  white-space: nowrap;
}

@keyframes pulse {
  0%, 100% {
    opacity: 1;
  }
  50% {
    opacity: 0.3;
  }
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
