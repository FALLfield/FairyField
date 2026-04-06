/**
 * 情绪状态机（stub）
 *
 * 管理角色的情绪状态，Phase 3 接入 Rig Agent 后完善。
 * 支持情绪优先级、衰减和过渡动画触发。
 */

export type Emotion =
  | 'happy'
  | 'sad'
  | 'angry'
  | 'surprised'
  | 'neutral'
  | 'thinking';

/** 情绪优先级（数值越大优先级越高） */
const EMOTION_PRIORITY: Record<Emotion, number> = {
  angry: 5,
  surprised: 4,
  happy: 3,
  sad: 2,
  thinking: 1,
  neutral: 0,
};

/** 情绪衰减时间（毫秒），高优先级情绪持续更久 */
const EMOTION_DECAY_MS: Record<Emotion, number> = {
  angry: 8000,
  surprised: 4000,
  happy: 6000,
  sad: 10000,
  thinking: 3000,
  neutral: 0,
};

export interface EmotionState {
  current: Emotion;
  previous: Emotion;
  timestamp: number;
}

export function createEmotionEngine() {
  const state: EmotionState = {
    current: 'neutral',
    previous: 'neutral',
    timestamp: Date.now(),
  };

  /** 设置新情绪（高优先级覆盖低优先级） */
  function setEmotion(emotion: Emotion): void {
    if (
      EMOTION_PRIORITY[emotion] > EMOTION_PRIORITY[state.current] ||
      state.current === 'neutral'
    ) {
      state.previous = state.current;
      state.current = emotion;
      state.timestamp = Date.now();
    }
  }

  /** 重置为 neutral */
  function reset(): void {
    state.previous = state.current;
    state.current = 'neutral';
    state.timestamp = Date.now();
  }

  /** 检查当前情绪是否已过期，自动衰减回 neutral */
  function tick(): void {
    const elapsed = Date.now() - state.timestamp;
    const decay = EMOTION_DECAY_MS[state.current];

    if (decay > 0 && elapsed >= decay) {
      reset();
    }
  }

  /** 获取当前情绪状态（只读副本） */
  function getState(): Readonly<EmotionState> {
    return { ...state };
  }

  return {
    setEmotion,
    reset,
    tick,
    getState,
  } as const;
}
