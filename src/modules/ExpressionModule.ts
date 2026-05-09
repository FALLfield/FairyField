import type { VRMRenderer } from '../renderers/VRMRenderer';
import type { EmotionEngine, UpperFaceBlendShapes } from '../lib/emotion-engine';

/**
 * ExpressionModule - 表情管理模块
 *
 * 管理两种表情驱动源：
 * 1. 手动表情预设（点击按钮、脚本触发）
 * 2. 情绪引擎驱动（EmotionEngine 连续情绪 → BlendShape）
 *
 * 协调规则：
 * - 上半脸（眉毛、眼睛）由情绪引擎驱动
 * - 下半脸（嘴巴）在说话时由 LipSync 驱动，安静时由情绪驱动
 * - 手动表情预设优先级高于情绪引擎
 */

export type ExpressionName =
  | 'neutral'
  | 'happy'
  | 'angry'
  | 'sad'
  | 'relaxed'
  | 'surprised'
  | 'ee'
  | 'ah'
  | 'oh'
  | 'ou'
  | 'blink'
  | 'blink_l'
  | 'blink_r'
  | 'lookup'
  | 'lookdown'
  | 'lookleft'
  | 'lookright';

/** 组合表情：多个 BlendShape 的权重映射 */
export type ExpressionPreset = Partial<Record<ExpressionName, number>>;

/** 下半脸 BlendShape（由 LipSync 控制） */
const MOUTH_SHAPES = new Set<string>(['aa', 'ih', 'ou', 'ee', 'oh', 'ah']);

interface TransitionState {
  fromPreset: ExpressionPreset | null;
  toPreset: ExpressionPreset;
  progress: number;
  duration: number;
}

export class ExpressionModule {
  private renderer: VRMRenderer;
  private isRunning = false;
  private currentPreset: ExpressionPreset = {};
  private transition: TransitionState | null = null;
  private activeWeights: Map<string, number> = new Map();
  private presets: Map<string, ExpressionPreset> = new Map();

  /** 情绪引擎引用（可选） */
  private emotionEngine: EmotionEngine | null = null;

  /** 当前是否由情绪引擎驱动 */
  private emotionDriven = false;

  /** 上次情绪引擎计算的 BlendShape */
  private lastEmotionBlend: UpperFaceBlendShapes = {};

  /** 说话状态（由 LipSync 设置） */
  private isSpeaking = false;

  constructor(renderer: VRMRenderer) {
    this.renderer = renderer;
    this.registerDefaultPresets();
  }

  /**
   * 绑定情绪引擎
   * 绑定后，表情将由情绪引擎自动驱动（除非手动设置预设覆盖）
   */
  bindEmotionEngine(engine: EmotionEngine): void {
    this.emotionEngine = engine;
    this.emotionDriven = true;
  }

  /** 解除情绪引擎绑定 */
  unbindEmotionEngine(): void {
    this.emotionEngine = null;
    this.emotionDriven = false;
  }

  /** 设置说话状态（由 LipSync 调用） */
  setSpeaking(speaking: boolean): void {
    this.isSpeaking = speaking;
  }

  start(): void {
    if (this.isRunning) return;
    this.isRunning = true;
    this.applyPresetImmediate(this.currentPreset);
  }

  stop(): void {
    if (!this.isRunning) return;
    this.isRunning = false;
    this.transition = null;
    this.applyPresetImmediate({});
    this.currentPreset = {};
  }

  /**
   * 设置表情（带平滑过渡）
   * 手动设置表情会暂时覆盖情绪引擎驱动，直到调用 releaseManualOverride()
   * @param name - 标准表情名或已注册的预设名
   * @param transitionDuration - 过渡时长（秒），默认 0.3
   */
  setExpression(name: string, transitionDuration: number = 0.3): void {
    if (!this.isRunning) return;

    // 手动设置表情时暂停情绪引擎驱动
    this.emotionDriven = false;

    const target = this.resolvePreset(name);

    if (this.transition) {
      this.transition.toPreset = target;
      this.transition.progress = 0;
      this.transition.duration = transitionDuration;
    } else {
      this.transition = {
        fromPreset: { ...this.currentPreset },
        toPreset: target,
        progress: 0,
        duration: transitionDuration,
      };
    }
  }

  /** 释放手动覆盖，恢复情绪引擎驱动 */
  releaseManualOverride(transitionDuration: number = 0.5): void {
    if (!this.emotionEngine) return;

    // 从当前手动表情过渡到情绪引擎驱动
    this.transition = {
      fromPreset: { ...this.currentPreset },
      toPreset: {}, // 清空手动预设，由 tick 中的情绪引擎接管
      progress: 0,
      duration: transitionDuration,
    };
    this.emotionDriven = true;
  }

  /** 每帧更新 */
  tick(delta: number): void {
    if (!this.isRunning) return;

    // 优先处理手动表情过渡
    if (this.transition) {
      this.tickTransition(delta);
      return;
    }

    // 情绪引擎驱动模式
    if (this.emotionDriven && this.emotionEngine) {
      this.tickEmotion(delta);
    }
  }

  reset(): void {
    this.transition = null;
    this.currentPreset = {};
    this.applyPresetImmediate({});
  }

  registerPreset(name: string, blendShapes: ExpressionPreset): void {
    this.presets.set(name, blendShapes);
  }

  removePreset(name: string): boolean {
    return this.presets.delete(name);
  }

  getCurrentExpression(): ExpressionPreset {
    return { ...this.currentPreset };
  }

  getTransitionProgress(): number {
    return this.transition?.progress ?? 1;
  }

  isActive(): boolean {
    return this.isRunning;
  }

  getPresetNames(): string[] {
    return Array.from(this.presets.keys());
  }

  isEmotionDriven(): boolean {
    return this.emotionDriven;
  }

  // -- 私有方法 --

  /** 处理手动表情过渡 */
  private tickTransition(delta: number): void {
    if (!this.transition) return;

    const step = this.transition.duration > 0
      ? delta / this.transition.duration
      : 1;
    this.transition.progress = Math.min(1, this.transition.progress + step);

    const t = this.easeInOutCubic(this.transition.progress);

    // 混合 from 和 to 的权重
    const blended: ExpressionPreset = {};
    const fromKeys = new Set(Object.keys(this.transition.fromPreset ?? {}));
    const toKeys = new Set(Object.keys(this.transition.toPreset));
    const allKeys = new Set([...fromKeys, ...toKeys]);

    for (const key of allKeys) {
      const from = this.transition.fromPreset?.[key as ExpressionName] ?? 0;
      const to = this.transition.toPreset[key as ExpressionName] ?? 0;
      const value = from * (1 - t) + to * t;
      if (Math.abs(value) > 0.001) {
        blended[key as ExpressionName] = value;
      }
    }

    this.applyPresetImmediate(blended);

    if (this.transition.progress >= 1) {
      this.currentPreset = { ...this.transition.toPreset };
      this.transition = null;
    }
  }

  /** 情绪引擎驱动每帧更新 */
  private tickEmotionBlend(delta: number): void {
    if (!this.emotionEngine) return;

    const newBlend = this.emotionEngine.computeUpperFaceBlend();

    // 平滑过渡到新的情绪 BlendShape
    const lerpSpeed = 0.15;
    const smoothed: UpperFaceBlendShapes = {};

    const allKeys = new Set([
      ...Object.keys(this.lastEmotionBlend),
      ...Object.keys(newBlend),
    ]);

    for (const key of allKeys) {
      const from = this.lastEmotionBlend[key] ?? 0;
      const to = newBlend[key] ?? 0;
      const t = Math.min(1, lerpSpeed * 60 * delta);
      smoothed[key] = from + (to - from) * t;
    }

    this.lastEmotionBlend = { ...smoothed };

    // 应用到模型（跳过嘴型 BlendShape，由 LipSync 控制）
    this.applyEmotionBlendShapes(smoothed);
  }

  /** 应用情绪 BlendShape 到模型 */
  private applyEmotionBlendShapes(blend: UpperFaceBlendShapes): void {
    // 清除上次的情绪权重
    for (const key of Object.keys(this.lastEmotionBlend)) {
      if (!MOUTH_SHAPES.has(key)) {
        this.renderer.setExpression(key, 0);
      }
    }

    // 设置新的情绪权重（仅上半脸）
    for (const [name, weight] of Object.entries(blend)) {
      if (MOUTH_SHAPES.has(name)) {
        // 说话时嘴型由 LipSync 控制，不覆盖
        if (this.isSpeaking) continue;

        // 安静时情绪可以影响嘴型（如微笑）
        this.renderer.setExpression(name, Math.max(0, Math.min(1, weight)));
      } else {
        // 上半脸 BlendShape 直接应用
        this.renderer.setExpression(name, Math.max(0, Math.min(1, weight)));
      }
    }
  }

  /** 情绪引擎驱动的 tick */
  private tickEmotion(delta: number): void {
    this.tickEmotionBlend(delta);
    this.currentPreset = {};
  }

  private resolvePreset(name: string): ExpressionPreset {
    if (this.presets.has(name)) {
      return this.presets.get(name)!;
    }
    // 单个标准表情 → 单键 preset
    const single: ExpressionPreset = {};
    single[name as ExpressionName] = 1.0;
    return single;
  }

  private applyPresetImmediate(preset: ExpressionPreset): void {
    // 先清除当前活跃的权重
    for (const [name] of this.activeWeights) {
      this.renderer.setExpression(name, 0);
    }
    this.activeWeights.clear();

    // 应用新权重
    for (const [name, weight] of Object.entries(preset)) {
      if (weight > 0.001) {
        this.renderer.setExpression(name, weight);
        this.activeWeights.set(name, weight);
      }
    }
  }

  private easeInOutCubic(t: number): number {
    return t < 0.5 ? 4 * t * t * t : 1 - Math.pow(-2 * t + 2, 3) / 2;
  }

  private registerDefaultPresets(): void {
    this.presets.set('laugh', { happy: 1.0, ah: 0.8, blink: 0.3 });
    this.presets.set('shy', { happy: 0.5, relaxed: 0.8, blink_l: 0.5, lookdown: 0.2 });
    this.presets.set('upset', { sad: 0.8, ah: 0.3, blink: 0.2, lookdown: 0.5 });
    this.presets.set('wink', { happy: 0.3, blink_l: 1.0 });
  }
}
