<script setup lang="ts">
/**
 * CharacterCanvas - Three.js + VRM 角色渲染容器
 *
 * 管理渲染器和所有 Phase 1 模块的生命周期：
 * - VRMRenderer: 核心 3D 渲染
 * - ExpressionModule: 表情管理
 * - LipSyncModule: 口型同步
 * - EyeTrackModule: 眼神跟随
 */
import { ref, onMounted, onUnmounted } from 'vue';
import { VRMRenderer } from '../renderers/VRMRenderer';
import { ExpressionModule } from '../modules/ExpressionModule';
import { LipSyncModule } from '../modules/LipSyncModule';
import { EyeTrackModule } from '../modules/EyeTrackModule';
import { createEmotionEngine } from '../lib/emotion-engine';

const canvasRef = ref<HTMLCanvasElement | null>(null);
const isLoading = ref(true);
const errorMessage = ref('');

const emit = defineEmits<{
  (e: 'loaded'): void;
  (e: 'error', message: string): void;
  (e: 'fps', fps: number): void;
}>();

let rendererInstance: VRMRenderer | null = null;
let expressionModule: ExpressionModule | null = null;
let lipSyncModule: LipSyncModule | null = null;
let eyeTrackModule: EyeTrackModule | null = null;
let tickFrameId: number | null = null;
const emotionEngine = createEmotionEngine();

const DEFAULT_MODEL_URL = '/models/default/2031903848872972007.glb';

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
    emit('loaded');

    // 初始化 Phase 1 模块
    expressionModule = new ExpressionModule(rendererInstance);
    lipSyncModule = new LipSyncModule(rendererInstance);
    eyeTrackModule = new EyeTrackModule(rendererInstance);

    expressionModule.start();
    eyeTrackModule.start();

    // 模块 tick 循环
    let lastTime = performance.now();
    let frameCount = 0;
    let fpsTime = performance.now();
    const tick = (now: number): void => {
      const delta = (now - lastTime) / 1000;
      lastTime = now;

      expressionModule?.tick(delta);
      lipSyncModule?.tick();
      emotionEngine.tick();

      // 每秒统计 FPS
      frameCount++;
      if (now - fpsTime >= 1000) {
        emit('fps', frameCount);
        frameCount = 0;
        fpsTime = now;
      }

      tickFrameId = requestAnimationFrame(tick);
    };
    tickFrameId = requestAnimationFrame(tick);
  } catch (error: unknown) {
    const message = error instanceof Error ? error.message : String(error);
    errorMessage.value = `VRM 加载失败: ${message}`;
    isLoading.value = false;
    emit('error', errorMessage.value);
  }
});

onUnmounted(() => {
  if (tickFrameId !== null) {
    cancelAnimationFrame(tickFrameId);
  }

  eyeTrackModule?.stop();
  lipSyncModule?.dispose();
  expressionModule?.stop();
  rendererInstance?.dispose();

  rendererInstance = null;
  expressionModule = null;
  lipSyncModule = null;
  eyeTrackModule = null;
});

function getRenderer(): VRMRenderer | null {
  return rendererInstance;
}

function getExpressionModule(): ExpressionModule | null {
  return expressionModule;
}

function getLipSyncModule(): LipSyncModule | null {
  return lipSyncModule;
}

defineExpose({
  getRenderer,
  getExpressionModule,
  getLipSyncModule,
  isLoading,
  errorMessage,
});
</script>

<template>
  <div class="character-canvas">
    <div v-if="isLoading" class="character-canvas__overlay">
      <p>正在加载角色模型...</p>
    </div>

    <div v-if="errorMessage" class="character-canvas__overlay character-canvas__overlay--error">
      <p>{{ errorMessage }}</p>
    </div>

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
