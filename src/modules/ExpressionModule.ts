import type { VRMRenderer } from '../renderers/VRMRenderer';

/**
 * ExpressionModule - 表情管理模块
 *
 * 管理 VRM 模型的表情切换和平滑过渡。
 * 支持标准 VRM BlendShape 表情和自定义组合预设。
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

  constructor(renderer: VRMRenderer) {
    this.renderer = renderer;
    this.registerDefaultPresets();
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
   * @param name - 标准表情名或已注册的预设名
   * @param transitionDuration - 过渡时长（秒），默认 0.3
   */
  setExpression(name: string, transitionDuration: number = 0.3): void {
    if (!this.isRunning) return;

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

  /** 每帧更新过渡 */
  tick(delta: number): void {
    if (!this.isRunning || !this.transition) return;

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

  // -- 私有方法 --

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
