<script setup lang="ts">
import { ref, useTemplateRef } from 'vue';
import CharacterCanvas from './components/CharacterCanvas.vue';
import ControlPanel from './components/ControlPanel.vue';
import { startDrag as tauriStartDrag } from './lib/tauri-commands';

const characterCanvasRef = useTemplateRef<InstanceType<typeof CharacterCanvas>>('characterCanvas');

const fps = ref(0);
const modelLoaded = ref(false);
const errorMessage = ref('');

function onModelLoaded(): void {
  modelLoaded.value = true;
  errorMessage.value = '';
}

function onModelError(message: string): void {
  modelLoaded.value = false;
  errorMessage.value = message;
}

function onFps(value: number): void {
  fps.value = value;
}

/** 窗口拖拽（通过 Tauri IPC） */
async function onDragStart(): Promise<void> {
  try {
    await tauriStartDrag();
  } catch {
    // 非 Tauri 环境下忽略
  }
}
</script>

<template>
  <main
    class="app-root"
    @mousedown="onDragStart"
  >
    <!-- 加载/错误提示（CharacterCanvas 内部已有，此处作为全局兜底） -->
    <div v-if="errorMessage" class="status-overlay status-error">
      <p>{{ errorMessage }}</p>
    </div>

    <!-- 3D 角色画布（Renderer Agent 创建） -->
    <CharacterCanvas
      ref="characterCanvas"
      @loaded="onModelLoaded"
      @error="onModelError"
      @fps="onFps"
    />

    <!-- 对话气泡插槽 -->
    <slot name="chat" />

    <!-- 调试控制面板 -->
    <ControlPanel
      :fps="fps"
      :model-loaded="modelLoaded"
      :expression-module="characterCanvasRef?.getExpressionModule() ?? null"
    />
  </main>
</template>

<style scoped>
.app-root {
  position: fixed;
  inset: 0;
  width: 100vw;
  height: 100vh;
  overflow: hidden;
  /* 透明背景 — Tauri 透明窗口需要 */
  background: transparent;
}

.status-overlay {
  position: fixed;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  z-index: 1000;
  padding: 1rem 1.5rem;
  border-radius: 8px;
  font-size: 0.875rem;
  text-align: center;
  pointer-events: none;
}

.status-error {
  background: rgba(220, 38, 38, 0.85);
  color: #fff;
}
</style>
