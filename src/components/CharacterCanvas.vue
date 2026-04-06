<script setup lang="ts">
/**
 * CharacterCanvas - Three.js + VRM 角色渲染容器
 *
 * 作为顶层 Vue 组件，管理 VRMRenderer 的生命周期：
 * - onMounted: 创建 VRMRenderer，加载默认 VRM 模型
 * - onUnmounted: dispose 释放所有资源
 *
 * 通过 defineExpose 暴露 getRenderer()，供父组件和模块使用。
 */
import { ref, onMounted, onUnmounted } from 'vue';
import { VRMRenderer } from '../renderers/VRMRenderer';

const canvasRef = ref<HTMLCanvasElement | null>(null);
const isLoading = ref(true);
const errorMessage = ref('');
let rendererInstance: VRMRenderer | null = null;

/** 默认 VRM 模型路径（可在 public/models/default/ 下放置测试模型） */
const DEFAULT_MODEL_URL = '/models/default/2031903848872972972007.glb';

onMounted(async () => {
  if (!canvasRef.value) {
    errorMessage.value = 'Canvas 元素未找到';
    isLoading.value = false;
    return;
  }

  try {
    rendererInstance = new VRMRenderer(canvasRef.value);
    await rendererInstance.loadVRM(DEFAULT_MODEL_URL);
    isLoading.value = false;
  } catch (error: unknown) {
    const message = error instanceof Error ? error.message : String(error);
    errorMessage.value = `VRM 加载失败: ${message}`;
    isLoading.value = false;
  }
});

onUnmounted(() => {
  rendererInstance?.dispose();
  rendererInstance = null;
});

/**
 * 获取 VRMRenderer 实例
 * 供父组件、眼神跟随、表情管理等模块使用
 */
function getRenderer(): VRMRenderer | null {
  return rendererInstance;
}

defineExpose({
  getRenderer,
  isLoading,
  errorMessage,
});
</script>

<template>
  <div class="character-canvas">
    <!-- 加载指示 -->
    <div v-if="isLoading" class="character-canvas__overlay">
      <p>正在加载角色模型...</p>
    </div>

    <!-- 错误提示 -->
    <div v-if="errorMessage" class="character-canvas__overlay character-canvas__overlay--error">
      <p>{{ errorMessage }}</p>
    </div>

    <!-- Three.js 渲染画布 -->
    <canvas
      ref="canvasRef"
      class="character-canvas__canvas"
    />
  </div>
</template>

<style scoped>
.character-canvas {
  position: relative;
  width: 100vw;
  height: 100vh;
  overflow: hidden;
}

.character-canvas__canvas {
  display: block;
  width: 100%;
  height: 100%;
}

.character-canvas__overlay {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  text-align: center;
  padding: 20px;
  background: rgba(0, 0, 0, 0.8);
  color: white;
  border-radius: 8px;
  z-index: 1000;
  pointer-events: none;
}

.character-canvas__overlay--error {
  background: rgba(220, 38, 38, 0.9);
}
</style>
