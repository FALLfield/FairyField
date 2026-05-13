<script setup lang="ts">
import { ref, computed } from 'vue';
import type { ToolLogEntry } from '../composables/useAgentStatus';

const props = defineProps<{ logs: readonly ToolLogEntry[] }>();
const emit = defineEmits<{
  (e: 'clear'): void;
}>();

const showPanel = ref(false);
const maxVisibleLogs = 50;

const displayLogs = computed(() => {
  const logs = [...props.logs];
  return logs.slice(-maxVisibleLogs).reverse();
});

function formatTime(ts: number): string {
  return new Date(ts).toLocaleTimeString('zh-CN', {
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  });
}

function clear(): void {
  emit('clear');
}
</script>

<template>
  <div class="agent-log-panel" :class="{ collapsed: !showPanel }">
    <button class="toggle-btn" @click="showPanel = !showPanel">
      {{ showPanel ? '▼ Agent 日志' : '▶ Agent 日志' }}
    </button>

    <div v-if="showPanel" class="panel-body">
      <div class="header-row">
        <span class="log-count">{{ logs.length }} 条记录</span>
        <button class="clear-btn" aria-label="清空日志" @click="clear" :disabled="logs.length === 0">
          清空
        </button>
      </div>

      <div v-if="displayLogs.length === 0" class="empty">
        暂无工具调用记录
      </div>

      <div v-for="(entry, i) in displayLogs" :key="i" class="log-entry">
        <div class="log-header">
          <span class="tool-name">{{ entry.tool }}</span>
          <span class="timestamp">{{ formatTime(entry.timestamp) }}</span>
        </div>
        <pre class="log-result">{{ entry.result }}</pre>
      </div>
    </div>
  </div>
</template>

<style scoped>
.agent-log-panel {
  position: fixed;
  right: 12px;
  bottom: 230px;
  width: 340px;
  z-index: 300;
  font-size: 12px;
  font-family: ui-monospace, monospace;
}

.collapsed {
  width: auto;
}

.toggle-btn {
  background: rgba(30, 30, 40, 0.85);
  color: #ccc;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 6px;
  padding: 4px 10px;
  cursor: pointer;
  font-size: 12px;
}

.toggle-btn:hover {
  background: rgba(50, 50, 60, 0.9);
}

.panel-body {
  margin-top: 6px;
  background: rgba(20, 20, 28, 0.92);
  backdrop-filter: blur(12px);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
  padding: 10px;
  max-height: 40vh;
  overflow-y: auto;
  color: #ddd;
}

.header-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
  padding-bottom: 6px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
}

.log-count {
  color: #888;
  font-size: 11px;
}

.clear-btn {
  background: rgba(248, 113, 113, 0.15);
  color: #f87171;
  border: 1px solid rgba(248, 113, 113, 0.25);
  border-radius: 4px;
  padding: 2px 8px;
  cursor: pointer;
  font-size: 11px;
}

.clear-btn:hover {
  background: rgba(248, 113, 113, 0.25);
}

.clear-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.empty {
  color: #666;
  font-size: 11px;
  text-align: center;
  padding: 12px 0;
}

.log-entry {
  padding: 6px 0;
  border-bottom: 1px solid rgba(255, 255, 255, 0.04);
}

.log-entry:last-child {
  border-bottom: none;
}

.log-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 3px;
}

.tool-name {
  color: #c084fc;
  font-weight: 600;
}

.timestamp {
  color: #666;
  font-size: 10px;
}

.log-result {
  color: #aaa;
  font-size: 11px;
  white-space: pre-wrap;
  word-break: break-all;
  margin: 0;
  background: rgba(255, 255, 255, 0.03);
  border-radius: 4px;
  padding: 4px 6px;
  max-height: 80px;
  overflow-y: auto;
}
</style>
