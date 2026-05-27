<script setup lang="ts">
import { ref, onMounted } from 'vue';
import * as tauriCommands from '../lib/tauri-commands';
import type { ProviderPreset } from '../lib/tauri-commands';

interface ControlPanelProps {
  fps: number;
  modelLoaded: boolean;
}

defineProps<ControlPanelProps>();

const collapsed = ref(true);
const providers = ref<ProviderPreset[]>([]);
const activeProvider = ref<string>('');
const switching = ref(false);
const loadError = ref<string | null>(null);

function toggleCollapse(): void {
  collapsed.value = !collapsed.value;
}

async function loadProviders(): Promise<void> {
  loadError.value = null;
  try {
    const [list, active] = await Promise.all([
      tauriCommands.llmListProviders(),
      tauriCommands.llmGetActiveProvider(),
    ]);
    providers.value = list;
    activeProvider.value = active.name;
  } catch (e) {
    loadError.value = e instanceof Error ? e.message : String(e);
  }
}

async function switchProvider(name: string): Promise<void> {
  if (name === activeProvider.value || switching.value) return;
  switching.value = true;
  try {
    await tauriCommands.llmSwitchProvider(name);
    activeProvider.value = name;
  } catch (e) {
    if (import.meta.env.DEV) console.error('切换提供商失败:', e);
  } finally {
    switching.value = false;
  }
}

onMounted(() => {
  loadProviders();
});
</script>

<template>
  <aside
    class="control-panel"
    :class="{ collapsed }"
  >
    <!-- 折叠按钮 -->
    <button
      class="toggle-btn"
      aria-label="折叠/展开控制面板"
      @click="toggleCollapse"
    >
      {{ collapsed ? '◀' : '▶' }}
    </button>

    <!-- 面板内容 -->
    <div v-if="!collapsed" class="panel-content">
      <!-- FPS 显示 -->
      <div
        class="fps-badge"
        :class="{ good: fps >= 30, bad: fps < 30 }"
      >
        {{ fps }} FPS
      </div>

      <!-- 模型状态 -->
      <div class="status-row">
        <span class="label">模型</span>
        <span
          class="status-dot"
          :class="modelLoaded ? 'loaded' : 'loading'"
        />
        <span class="value">
          {{ modelLoaded ? '已加载' : '加载中...' }}
        </span>
      </div>

      <!-- LLM 提供商切换 -->
      <div class="section">
        <span class="label">LLM</span>
        <!-- 提供商按钮 -->
        <div v-if="providers.length > 0" class="provider-buttons">
          <button
            v-for="p in providers"
            :key="p.name"
            :class="{ active: p.name === activeProvider }"
            :disabled="switching"
            @click="switchProvider(p.name)"
          >
            {{ p.name }}
          </button>
        </div>
        <!-- 加载失败/无提供商 -->
        <div v-else-if="loadError" class="provider-error">
          加载失败: {{ loadError.substring(0, 60) }}
          <button class="retry-btn" @click="loadProviders()">重试</button>
        </div>
        <div v-else class="provider-info">
          加载中...
        </div>
        <!-- 活跃提供商模型信息 -->
        <div v-if="activeProvider" class="provider-info">
          {{ providers.find(p => p.name === activeProvider)?.model }}
        </div>
      </div>
    </div>
  </aside>
</template>

<style scoped>
.control-panel {
  position: fixed;
  top: 16px;
  left: 16px;
  z-index: 300;
  display: flex;
  align-items: flex-start;
  gap: 8px;
  background: rgba(15, 15, 15, 0.75);
  backdrop-filter: blur(12px);
  border-radius: 10px;
  padding: 10px;
  color: #e5e5e5;
  font-family: 'SF Mono', 'Fira Code', monospace;
  font-size: 12px;
  user-select: none;
  pointer-events: auto;
  transition: opacity 0.2s ease;
}

.toggle-btn {
  flex-shrink: 0;
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  border-radius: 4px;
  background: rgba(255, 255, 255, 0.08);
  color: #a3a3a3;
  cursor: pointer;
  font-size: 10px;
  transition: background 0.15s ease;
}

.toggle-btn:hover {
  background: rgba(255, 255, 255, 0.15);
}

.panel-content {
  display: flex;
  flex-direction: column;
  gap: 8px;
  min-width: 120px;
}

.fps-badge {
  text-align: center;
  padding: 4px 10px;
  border-radius: 6px;
  font-weight: 600;
  font-size: 13px;
}

.fps-badge.good {
  background: rgba(34, 197, 94, 0.15);
  color: #4ade80;
}

.fps-badge.bad {
  background: rgba(239, 68, 68, 0.15);
  color: #f87171;
}

.status-row {
  display: flex;
  align-items: center;
  gap: 6px;
}

.label {
  color: #737373;
}

.status-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
}

.status-dot.loaded {
  background: #4ade80;
}

.status-dot.loading {
  background: #facc15;
  animation: pulse 1.2s ease-in-out infinite;
}

.value {
  color: #d4d4d4;
}

.section {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.provider-buttons {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.provider-buttons button {
  padding: 3px 8px;
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 4px;
  background: rgba(255, 255, 255, 0.06);
  color: #d4d4d4;
  font-size: 11px;
  cursor: pointer;
  transition: background 0.15s ease, border-color 0.15s ease;
}

.provider-buttons button:hover {
  background: rgba(255, 255, 255, 0.15);
}

.provider-buttons button.active {
  background: rgba(99, 102, 241, 0.25);
  border-color: rgba(99, 102, 241, 0.6);
  color: #a5b4fc;
}

.provider-buttons button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.provider-info {
  font-size: 10px;
  color: #737373;
  margin-top: 2px;
}

.provider-error {
  font-size: 10px;
  color: #f87171;
  margin-top: 2px;
  display: flex;
  align-items: center;
  gap: 6px;
}

.retry-btn {
  padding: 1px 6px;
  border: 1px solid rgba(248, 113, 113, 0.3);
  border-radius: 3px;
  background: rgba(248, 113, 113, 0.1);
  color: #fca5a5;
  font-size: 10px;
  cursor: pointer;
}

.retry-btn:hover {
  background: rgba(248, 113, 113, 0.2);
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.4; }
}
</style>
