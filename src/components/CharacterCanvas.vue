<script setup lang="ts">
/**
 * CharacterCanvas - Three.js + VRM 角色渲染容器
 *
 * 管理渲染器和所有 Phase 1 模块的生命周期：
 * - VRMRenderer: 核心 3D 渲染
 * - ExpressionModule: 表情管理
 * - LipSyncModule: 口型同步
 * - EyeTrackModule: 眼神跟随
 * - EmotionEngine: 情绪引擎（驱动表情 + 影响嘴型）
 */
import { ref, watch, onMounted, onUnmounted } from 'vue';
import { VRMRenderer } from '../renderers/VRMRenderer';
import { ExpressionModule } from '../modules/ExpressionModule';
import { LipSyncModule } from '../modules/LipSyncModule';
import { EyeTrackModule } from '../modules/EyeTrackModule';
import { IdleAnimation } from '../modules/IdleAnimation';
import { HitTestModule } from '../modules/HitTestModule';
import { createEmotionEngine } from '../lib/emotion-engine';
import type { EmotionWeights, Emotion } from '../lib/emotion-engine';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

const props = withDefaults(defineProps<{
  emotion?: string;
}>(), {
  emotion: 'neutral',
});

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
let idleAnimation: IdleAnimation | null = null;
let hitTestModule: HitTestModule | null = null;
let tickFrameId: number | null = null;
let mouseDownHandler: ((e: MouseEvent) => void) | null = null;
let emotionUnlisten: (() => void) | null = null;
let pcmUnlisten: (() => void) | null = null;
let ttsStartUnlisten: (() => void) | null = null;
let ttsFinishUnlisten: (() => void) | null = null;
const emotionEngine = createEmotionEngine();

/** TTS 播放状态（由 tts-started/tts-finished 事件驱动） */
let isTtsSpeaking = false;
/** TTS 模拟口型的时间累加器 */
let ttsMouthTime = 0;

/** 思考状态动画时间累加器 */
let thinkingTime = 0;
/** 上一次思考眨眼触发时间 */
let lastThinkingBlink = 0;
/** 当前是否处于思考状态 */
let isThinking = false;

// Watch emotion prop from parent and forward to EmotionEngine
watch(() => props.emotion, (newEmotion) => {
  if (newEmotion) {
    isThinking = newEmotion === 'thinking';
    if (!isThinking) thinkingTime = 0;
    emotionEngine.setEmotion(newEmotion as Emotion);
    // 平滑过渡：触发 0.5s 的 eased 表情过渡
    if (expressionModule?.isActive()) {
      expressionModule.releaseManualOverride(0.5);
    }
  }
});

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
        idleAnimation = new IdleAnimation(rendererInstance);
        hitTestModule = new HitTestModule(rendererInstance);

        // 绑定情绪引擎到表情模块
        expressionModule.bindEmotionEngine(emotionEngine);

        expressionModule.start();
        lipSyncModule.start();
        eyeTrackModule.start();
        idleAnimation.start();

        // 注册调试热键：Ctrl+1~8 手动触发表情
        const EXPR_MAP: Record<string, string> = {
          '1': 'happy', '2': 'sad', '3': 'angry', '4': 'surprised',
          '5': 'laugh', '6': 'shy', '7': 'upset', '8': 'neutral',
        };
        const hotkeyHandler = (e: KeyboardEvent) => {
          if (!e.ctrlKey && !e.metaKey) return;
          const expr = EXPR_MAP[e.key];
          if (!expr) return;
          e.preventDefault();
          if (expressionModule?.isActive()) {
            expressionModule.setExpression(expr, 1.0);
          }
        };
        window.addEventListener('keydown', hotkeyHandler);
        (window as any).__fairyfield_hotkey_handler = hotkeyHandler;

        // 监听 Tauri 事件：情绪状态更新（由 soul Agent 发送）
        emotionUnlisten = await listen<EmotionWeights>('emotion-update', (event) => {
            emotionEngine.setWeights(event.payload);
        }).catch(() => null as (() => void) | null);

        // 监听 Tauri 事件：PCM 音频数据（由 voice Agent 发送）
        pcmUnlisten = await listen<ArrayBuffer>('audio-pcm', (event) => {
            if (lipSyncModule?.isActive()) {
                const pcm = new Float32Array(event.payload);
                lipSyncModule.feedPCM(pcm, 16000);
            }
        }).catch(() => null as (() => void) | null);

        // 监听 TTS 开始/结束，驱动口型
        ttsStartUnlisten = await listen('tts-started', () => {
            isTtsSpeaking = true;
            ttsMouthTime = 0;
        }).catch(() => null as (() => void) | null);
        ttsFinishUnlisten = await listen('tts-finished', () => {
            isTtsSpeaking = false;
            ttsMouthTime = 0;
            lipSyncModule?.setSimulatedMouth(0, 0);
        }).catch(() => null as (() => void) | null);

        // 点击策略：角色上拖动窗口，透明区域短暂穿透
        const handleMouseDown = (e: MouseEvent): void => {
            const target = e.target as Element | null;
            if (target?.closest('button,input,textarea,select,[role="dialog"],.chat-panel,.control-panel')) {
                return;
            }
            const hit = hitTestModule?.isHit(e.clientX, e.clientY) ?? false;
            if (hit) {
                // 命中角色 → 拖动窗口
                invoke('start_drag').catch(() => {});
            } else {
                // 透明区域 → 短暂穿透让点击传递到下层应用
                invoke('set_ignore_cursor_events', { ignore: true }).catch(() => {});
                setTimeout(() => {
                    invoke('set_ignore_cursor_events', { ignore: false }).catch(() => {});
                }, 100);
            }
        };

        mouseDownHandler = handleMouseDown;
        canvasRef.value.addEventListener('mousedown', handleMouseDown);

        // 模块 tick 循环
        let lastTime = performance.now();
        let frameCount = 0;
        let fpsTime = performance.now();
        const tick = (now: number): void => {
            const delta = (now - lastTime) / 1000;
            lastTime = now;

            // 情绪引擎 tick（驱动衰减和过渡）
            emotionEngine.tick(delta);

            // LipSync tick（从 AnalyserNode 读取频谱）
            lipSyncModule?.tick();

            // 同步 LipSync 说话状态到 ExpressionModule
            // 合并两个来源：PCM 驱动的 lipSync + TTS 事件驱动
            const lipSpeaking = lipSyncModule?.isSpeaking() ?? false;
            const speaking = lipSpeaking || isTtsSpeaking;
            expressionModule?.setSpeaking(speaking);

            // TTS 播放时模拟口型（MacSayTts 不提供 PCM 数据，需要模拟）
            if (isTtsSpeaking && !lipSpeaking) {
                ttsMouthTime += delta;
                // 使用多频率正弦波模拟自然说话的嘴型
                const mouthOpen = 0.3 + 0.4 * Math.abs(Math.sin(ttsMouthTime * 8));
                const mouthWidth = 0.2 + 0.2 * Math.abs(Math.sin(ttsMouthTime * 5.3));
                lipSyncModule?.setSimulatedMouth(mouthOpen, mouthWidth);
            }

            // 同步情绪对嘴型的影响到 LipSyncModule
            const mouthInfluence = emotionEngine.computeMouthInfluence();
            lipSyncModule?.setMouthInfluence(mouthInfluence);

            // ExpressionModule tick（情绪驱动表情 + 手动过渡）
            expressionModule?.tick(delta);

            // 思考姿势：眨眼动画 + 轻微低头
            if (isThinking) {
              thinkingTime += delta;
              const blinkInterval = 3.5 + 2.0 * Math.sin(thinkingTime * 0.3);
              if (thinkingTime - lastThinkingBlink > blinkInterval) {
                lastThinkingBlink = thinkingTime;
                expressionModule?.setExpression('blink', 0.25);
              }
              // 轻微低头（思考时 10% 的 lookdown blend）
              const lookdownWeight = 0.08 + 0.02 * Math.sin(thinkingTime * 0.5);
              rendererInstance?.setExpression('lookdown', lookdownWeight);
            }

            // IdleAnimation 和 EyeTrack
            idleAnimation?.tick(delta);

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

    // 取消 Tauri 事件监听
    emotionUnlisten?.();
    pcmUnlisten?.();
    ttsStartUnlisten?.();
    ttsFinishUnlisten?.();

    eyeTrackModule?.stop();
    idleAnimation?.stop();
    lipSyncModule?.dispose();
    expressionModule?.stop();

    // 移除鼠标事件
    if (mouseDownHandler) canvasRef.value?.removeEventListener('mousedown', mouseDownHandler);

    rendererInstance?.dispose();

    rendererInstance = null;
    expressionModule = null;
    lipSyncModule = null;
    eyeTrackModule = null;
    idleAnimation = null;
    hitTestModule = null;
    mouseDownHandler = null;
    emotionUnlisten = null;
    pcmUnlisten = null;
    ttsStartUnlisten = null;
    ttsFinishUnlisten = null;

    const hh = (window as any).__fairyfield_hotkey_handler;
    if (hh) window.removeEventListener('keydown', hh);
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

function getEmotionEngine() {
    return emotionEngine;
}

defineExpose({
    getRenderer,
    getExpressionModule,
    getLipSyncModule,
    getEmotionEngine,
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
