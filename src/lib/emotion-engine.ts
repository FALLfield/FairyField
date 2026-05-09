/**
 * EmotionEngine - 多维情绪引擎
 *
 * 管理角色的连续情绪状态，每个情绪维度独立取值 0.0-1.0。
 * 支持：
 * - 情绪到 BlendShape 的映射（一个情绪混合多个 BlendShape）
 * - 不同情绪有不同过渡速度
 * - 情绪叠加（happy + excited = 更夸张的开心）
 * - 自然衰减回 neutral idle
 */

/** 向后兼容的离散情绪标签 */
export type Emotion =
  | 'happy'
  | 'sad'
  | 'angry'
  | 'surprised'
  | 'neutral'
  | 'thinking';

/** 向后兼容的情绪状态快照 */
export interface EmotionState {
  current: Emotion;
  previous: Emotion;
  timestamp: number;
}

/** 连续情绪维度 */
export interface EmotionWeights {
  happy: number;
  sad: number;
  angry: number;
  neutral: number;
  excited: number;
  shy: number;
  upset: number;
}

/** 单个情绪的配置 */
interface EmotionConfig {
  /** BlendShape 映射：每个 BlendShape 名 → 权重系数 */
  blendMap: Record<string, number>;
  /** 过渡到目标值的速度（0-1，越大越快） */
  transitionSpeed: number;
  /** 自然衰减速率（每秒减少的值，0 = 不衰减） */
  decayRate: number;
}

/** 情绪配置表 */
const EMOTION_CONFIGS: Record<keyof EmotionWeights, EmotionConfig> = {
  happy: {
    blendMap: { happy: 0.8, relaxed: 0.3 },
    transitionSpeed: 0.12,
    decayRate: 0.08,
  },
  sad: {
    blendMap: { sad: 0.7 },
    transitionSpeed: 0.06,
    decayRate: 0.04,
  },
  angry: {
    blendMap: { angry: 0.8 },
    transitionSpeed: 0.15,
    decayRate: 0.1,
  },
  neutral: {
    blendMap: { neutral: 0.5, relaxed: 0.2 },
    transitionSpeed: 0.08,
    decayRate: 0,
  },
  excited: {
    blendMap: { happy: 0.6, surprised: 0.4 },
    transitionSpeed: 0.18,
    decayRate: 0.12,
  },
  shy: {
    blendMap: { happy: 0.3, relaxed: 0.5, blink_l: 0.4 },
    transitionSpeed: 0.08,
    decayRate: 0.06,
  },
  upset: {
    blendMap: { sad: 0.5, angry: 0.3 },
    transitionSpeed: 0.1,
    decayRate: 0.07,
  },
};

/** 所有情绪维度名 */
const EMOTION_KEYS = Object.keys(EMOTION_CONFIGS) as Array<keyof EmotionWeights>;

/** 默认情绪状态 */
function createDefaultWeights(): EmotionWeights {
  return {
    happy: 0,
    sad: 0,
    angry: 0,
    neutral: 1,
    excited: 0,
    shy: 0,
    upset: 0,
  };
}

/** 混合后应用到上半脸的 BlendShape 结果 */
export interface UpperFaceBlendShapes {
  [blendShapeName: string]: number;
}

/** 情绪对嘴型幅度的影响系数 */
export interface MouthInfluence {
  /** 嘴型开合幅度缩放（0-1+，1 = 正常） */
  amplitudeScale: number;
  /** 嘴型偏移（说话时的基线开合） */
  baselineOpen: number;
}

export interface EmotionEngine {
  /** 设置某个情绪维度的目标值 */
  setWeight(emotion: keyof EmotionWeights, value: number): void;
  /** 直接设置一组情绪（替换当前目标） */
  setWeights(weights: Partial<EmotionWeights>): void;
  /** 每帧更新（dt 秒），驱动过渡和衰减 */
  tick(dt: number): void;
  /** 获取当前情绪值（只读快照） */
  getWeights(): Readonly<EmotionWeights>;
  /** 计算当前上半脸 BlendShape 权重 */
  computeUpperFaceBlend(): UpperFaceBlendShapes;
  /** 计算情绪对嘴型的影响 */
  computeMouthInfluence(): MouthInfluence;
  /** 重置到 neutral */
  reset(): void;
  /** 向后兼容：设置离散情绪 */
  setEmotion(emotion: Emotion): void;
  /** 向后兼容：获取离散情绪状态 */
  getState(): EmotionState;
}

export function createEmotionEngine(): EmotionEngine {
  const current = createDefaultWeights();
  const target = createDefaultWeights();

  function setWeight(emotion: keyof EmotionWeights, value: number): void {
    target[emotion] = Math.max(0, Math.min(1, value));
  }

  function setWeights(weights: Partial<EmotionWeights>): void {
    for (const key of EMOTION_KEYS) {
      if (key in weights && weights[key] !== undefined) {
        target[key] = Math.max(0, Math.min(1, weights[key]!));
      }
    }
  }

  function tick(dt: number): void {
    for (const key of EMOTION_KEYS) {
      const config = EMOTION_CONFIGS[key];

      // 向目标值过渡（lerp）
      const diff = target[key] - current[key];
      const speed = config.transitionSpeed;
      const step = diff * Math.min(1, speed * 60 * dt);
      current[key] += step;

      // 自然衰减（仅对非 neutral 维度）
      if (config.decayRate > 0) {
        const decay = config.decayRate * dt;
        current[key] = Math.max(0, current[key] - decay);
        target[key] = Math.max(0, target[key] - decay);
      }

      // clamp
      current[key] = Math.max(0, Math.min(1, current[key]));
      target[key] = Math.max(0, Math.min(1, target[key]));
    }

    // 自动归一化：确保至少有一个主情绪
    const totalNonNeutral = EMOTION_KEYS
      .filter((k) => k !== 'neutral')
      .reduce((sum, k) => sum + current[k], 0);

    if (totalNonNeutral < 0.01) {
      // 没有活跃情绪时，neutral 趋向 1
      current.neutral += (1 - current.neutral) * 0.05;
    } else {
      // 有活跃情绪时，neutral 趋向 0
      current.neutral *= 0.95;
    }
  }

  function getWeights(): Readonly<EmotionWeights> {
    return { ...current };
  }

  function computeUpperFaceBlend(): UpperFaceBlendShapes {
    const result: UpperFaceBlendShapes = {};

    for (const key of EMOTION_KEYS) {
      const weight = current[key];
      if (weight < 0.001) continue;

      const config = EMOTION_CONFIGS[key];
      for (const [blendName, coefficient] of Object.entries(config.blendMap)) {
        const contribution = weight * coefficient;
        result[blendName] = (result[blendName] ?? 0) + contribution;
      }
    }

    // clamp 所有值到 0-1
    for (const key of Object.keys(result)) {
      result[key] = Math.max(0, Math.min(1, result[key]));
    }

    return result;
  }

  function computeMouthInfluence(): MouthInfluence {
    const w = current;

    // 开心时嘴张得更大
    const happyBoost = w.happy * 0.3 + w.excited * 0.2;
    // 悲伤时嘴张得更小
    const sadDampen = w.sad * 0.3 + w.upset * 0.15;
    // 害羞时嘴微微张开
    const shyBaseline = w.shy * 0.05;

    const amplitudeScale = Math.max(0.3, Math.min(1.5, 1.0 + happyBoost - sadDampen));
    const baselineOpen = Math.max(0, shyBaseline);

    return { amplitudeScale, baselineOpen };
  }

  function reset(): void {
    for (const key of EMOTION_KEYS) {
      current[key] = key === 'neutral' ? 1 : 0;
      target[key] = key === 'neutral' ? 1 : 0;
    }
  }

  /** 向后兼容：将离散 Emotion 映射到连续权重 */
  const EMOTION_MAP: Record<Emotion, Partial<EmotionWeights>> = {
    happy: { happy: 1.0, neutral: 0 },
    sad: { sad: 1.0, neutral: 0 },
    angry: { angry: 1.0, neutral: 0 },
    surprised: { excited: 0.8, neutral: 0 },
    thinking: { neutral: 0.3 },
    neutral: { neutral: 1.0 },
  };

  /** 向后兼容的离散情绪状态追踪 */
  let discreteCurrent: Emotion = 'neutral';
  let discretePrevious: Emotion = 'neutral';
  let discreteTimestamp = Date.now();

  function setEmotion(emotion: Emotion): void {
    const weights = EMOTION_MAP[emotion];
    if (weights) {
      setWeights(weights);
    }
    discretePrevious = discreteCurrent;
    discreteCurrent = emotion;
    discreteTimestamp = Date.now();
  }

  function getState(): EmotionState {
    return {
      current: discreteCurrent,
      previous: discretePrevious,
      timestamp: discreteTimestamp,
    };
  }

  return {
    setWeight,
    setWeights,
    tick,
    getWeights,
    computeUpperFaceBlend,
    computeMouthInfluence,
    reset,
    setEmotion,
    getState,
  } as const;
}
