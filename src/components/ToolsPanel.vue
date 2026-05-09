<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useTools, type ToolInfo } from '../composables/useTools';
import { useSecurity } from '../composables/useSecurity';

const tools = useTools();
const security = useSecurity();
const toolList = ref<ToolInfo[]>([]);
const expanded = ref<string | null>(null);
const toolInput = ref('{}');
const toolResult = ref<string | null>(null);
const securityStatus = ref<string | null>(null);
const showPanel = ref(false);

onMounted(async () => {
  toolList.value = await tools.listTools();
});

function toggle(name: string) {
  expanded.value = expanded.value === name ? null : name;
  toolResult.value = null;
  securityStatus.value = null;
  toolInput.value = '{}';
}

async function runTool(name: string) {
  securityStatus.value = null;
  // Security check
  const guard = await security.checkCommand(name);
  if (guard && guard.level === 'dangerous') {
    securityStatus.value = `blocked: ${guard.reason}`;
    return;
  }
  const result = await tools.executeTool(name, toolInput.value);
  toolResult.value = result ?? '(null)';
  if (guard && guard.level === 'caution') {
    securityStatus.value = `caution: ${guard.reason}`;
  }
}
</script>

<template>
  <div class="tools-panel" :class="{ collapsed: !showPanel }">
    <button class="toggle-btn" @click="showPanel = !showPanel">
      {{ showPanel ? '▼ 工具' : '▶ 工具' }}
    </button>

    <div v-if="showPanel" class="panel-body">
      <div v-for="t in toolList" :key="t.name" class="tool-item">
        <div class="tool-header" @click="toggle(t.name)">
          <span class="tool-name">{{ t.name }}</span>
          <span class="tool-desc">{{ t.description }}</span>
          <span class="arrow">{{ expanded === t.name ? '▲' : '▼' }}</span>
        </div>

        <div v-if="expanded === t.name" class="tool-detail">
          <textarea
            v-model="toolInput"
            class="json-input"
            rows="3"
            placeholder='{"key": "value"}'
          ></textarea>
          <div class="btn-row">
            <button class="run-btn" @click="runTool(t.name)" :disabled="tools.loading.value">
              执行
            </button>
          </div>
          <div v-if="securityStatus" class="security-warn">{{ securityStatus }}</div>
          <div v-if="toolResult" class="result-box">
            <pre>{{ toolResult }}</pre>
          </div>
        </div>
      </div>

      <p v-if="tools.error.value" class="error">{{ tools.error.value }}</p>
    </div>
  </div>
</template>

<style scoped>
.tools-panel {
  position: fixed;
  right: 12px;
  bottom: 12px;
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
  max-height: 50vh;
  overflow-y: auto;
  color: #ddd;
}
.tool-item {
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  padding: 4px 0;
}
.tool-item:last-child {
  border-bottom: none;
}
.tool-header {
  display: flex;
  align-items: center;
  gap: 6px;
  cursor: pointer;
  padding: 2px 0;
}
.tool-name {
  color: #00d4aa;
  font-weight: 600;
  min-width: 100px;
}
.tool-desc {
  color: #888;
  flex: 1;
  font-size: 11px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.arrow {
  color: #666;
  font-size: 10px;
}
.tool-detail {
  padding: 6px 0;
}
.json-input {
  width: 100%;
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 4px;
  color: #eee;
  padding: 4px 6px;
  font-family: ui-monospace, monospace;
  font-size: 11px;
  resize: vertical;
  box-sizing: border-box;
}
.btn-row {
  margin-top: 4px;
}
.run-btn {
  background: rgba(0, 212, 170, 0.2);
  color: #00d4aa;
  border: 1px solid rgba(0, 212, 170, 0.3);
  border-radius: 4px;
  padding: 3px 12px;
  cursor: pointer;
  font-size: 12px;
}
.run-btn:hover {
  background: rgba(0, 212, 170, 0.3);
}
.run-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.security-warn {
  color: #fbbf24;
  font-size: 11px;
  margin-top: 4px;
}
.result-box {
  margin-top: 6px;
  background: rgba(255, 255, 255, 0.04);
  border-radius: 4px;
  padding: 6px;
}
.result-box pre {
  color: #aaa;
  font-size: 11px;
  white-space: pre-wrap;
  word-break: break-all;
  margin: 0;
}
.error {
  color: #f87171;
  font-size: 11px;
}
</style>
