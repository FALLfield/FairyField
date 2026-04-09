<script setup lang="ts">
import { ref } from 'vue';
import type { ExpressionModule } from '../modules/ExpressionModule';

interface ControlPanelProps {
  fps: number;
  modelLoaded: boolean;
  expressionModule: ExpressionModule | null;
}

const props = defineProps<ControlPanelProps>();

const collapsed = ref(false);

function toggleCollapse(): void {
  collapsed.value = !collapsed.value;
}

function triggerExpression(name: string): void {
  if (props.expressionModule) {
    props.expressionModule.setExpression(name, 1.0);
  }
}
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

      <!-- 表情控制 -->
      <div v-if="modelLoaded" class="section">
        <span class="label">表情</span>
        <div class="expr-buttons">
          <button @click="triggerExpression('happy')">开心</button>
          <button @click="triggerExpression('sad')">难过</button>
          <button @click="triggerExpression('angry')">生气</button>
          <button @click="triggerExpression('surprised')">惊讶</button>
          <button @click="triggerExpression('laugh')">笑</button>
          <button @click="triggerExpression('shy')">害羞</button>
          <button @click="triggerExpression('upset')">不高兴</button>
          <button @click="triggerExpression('neutral')">平静</button>
        </div>
      </div>
    </div>
  </aside>
</template>

<style scoped>
.control-panel {
  position: fixed;
  bottom: 16px;
  right: 16px;
  z-index: 200;
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

.expr-buttons {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.expr-buttons button {
  padding: 3px 8px;
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 4px;
  background: rgba(255, 255, 255, 0.06);
  color: #d4d4d4;
  font-size: 11px;
  cursor: pointer;
  transition: background 0.15s ease;
}

.expr-buttons button:hover {
  background: rgba(255, 255, 255, 0.15);
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.4; }
}
</style>
