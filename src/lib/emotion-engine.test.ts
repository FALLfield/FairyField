import { describe, it, expect, beforeEach } from 'vitest';
import { createEmotionEngine } from './emotion-engine';

describe('EmotionEngine', () => {
  let engine: ReturnType<typeof createEmotionEngine>;

  beforeEach(() => {
    engine = createEmotionEngine();
  });

  describe('初始状态', () => {
    it('默认应为 neutral', () => {
      const weights = engine.getWeights();
      expect(weights.neutral).toBe(1);
      expect(weights.happy).toBe(0);
      expect(weights.sad).toBe(0);
      expect(weights.angry).toBe(0);
      expect(weights.excited).toBe(0);
      expect(weights.shy).toBe(0);
      expect(weights.upset).toBe(0);
    });

    it('getWeights 返回独立副本', () => {
      const w1 = engine.getWeights();
      const w2 = engine.getWeights();
      expect(w1).toEqual(w2);
      expect(w1).not.toBe(w2);
    });
  });

  describe('setWeight', () => {
    it('设置单个情绪维度', () => {
      engine.setWeight('happy', 0.8);
      // 目标值已设置，但需要 tick 来过渡
      engine.tick(1);
      const weights = engine.getWeights();
      expect(weights.happy).toBeGreaterThan(0);
    });

    it('clamp 到 0-1 范围', () => {
      engine.setWeight('happy', 2.0);
      engine.tick(1);
      const weights = engine.getWeights();
      expect(weights.happy).toBeLessThanOrEqual(1);
    });

    it('负值 clamp 到 0', () => {
      engine.setWeight('happy', -0.5);
      engine.tick(1);
      const weights = engine.getWeights();
      expect(weights.happy).toBeGreaterThanOrEqual(0);
    });
  });

  describe('setWeights', () => {
    it('批量设置多个情绪', () => {
      engine.setWeights({ happy: 0.6, sad: 0.3 });
      engine.tick(1);
      const weights = engine.getWeights();
      expect(weights.happy).toBeGreaterThan(0);
      expect(weights.sad).toBeGreaterThan(0);
    });

    it('未指定的维度不受影响', () => {
      engine.setWeights({ happy: 0.5 });
      engine.tick(1);
      const weights = engine.getWeights();
      expect(weights.happy).toBeGreaterThan(0);
    });
  });

  describe('tick 过渡', () => {
    it('情绪值平滑过渡到目标', () => {
      engine.setWeight('happy', 1.0);

      // 多次 tick 使过渡接近完成
      for (let i = 0; i < 60; i++) {
        engine.tick(1 / 60);
      }

      const weights = engine.getWeights();
      expect(weights.happy).toBeGreaterThan(0.9);
    });

    it('不同情绪有不同过渡速度', () => {
      engine.setWeight('happy', 1.0);
      engine.tick(1 / 60);
      const happyAfter1Tick = engine.getWeights().happy;

      engine.reset();
      engine.setWeight('sad', 1.0);
      engine.tick(1 / 60);
      const sadAfter1Tick = engine.getWeights().sad;

      // happy 配置的 transitionSpeed (0.12) > sad (0.06)
      expect(happyAfter1Tick).toBeGreaterThan(sadAfter1Tick);
    });

    it('情绪自然衰减', () => {
      engine.setWeight('happy', 0.5);

      // 多次 tick 使衰减生效
      for (let i = 0; i < 120; i++) {
        engine.tick(1 / 60);
      }

      const weights = engine.getWeights();
      // happy 有 decayRate=0.08，多次 tick 后应该衰减
      expect(weights.happy).toBeLessThan(0.5);
    });

    it('neutral 不衰减', () => {
      engine.setWeight('neutral', 1.0);
      for (let i = 0; i < 120; i++) {
        engine.tick(1 / 60);
      }
      const weights = engine.getWeights();
      // neutral 的 decayRate 为 0
      expect(weights.neutral).toBeGreaterThan(0.9);
    });
  });

  describe('computeUpperFaceBlend', () => {
    it('neutral 状态应包含 neutral BlendShape', () => {
      const blend = engine.computeUpperFaceBlend();
      expect(blend.neutral).toBeGreaterThan(0);
    });

    it('happy 情绪映射到 happy 和 relaxed BlendShape', () => {
      engine.setWeight('happy', 1.0);
      for (let i = 0; i < 60; i++) {
        engine.tick(1 / 60);
      }
      const blend = engine.computeUpperFaceBlend();
      expect(blend.happy).toBeGreaterThan(0);
      expect(blend.relaxed).toBeGreaterThan(0);
    });

    it('情绪叠加时 BlendShape 值累加', () => {
      engine.setWeights({ happy: 0.5, excited: 0.5 });
      for (let i = 0; i < 60; i++) {
        engine.tick(1 / 60);
      }
      const blend = engine.computeUpperFaceBlend();
      // happy → happy*0.8, excited → happy*0.6 → 累加
      expect(blend.happy).toBeGreaterThan(0);
    });

    it('BlendShape 值被 clamp 到 0-1', () => {
      engine.setWeights({ happy: 1.0, excited: 1.0 });
      for (let i = 0; i < 60; i++) {
        engine.tick(1 / 60);
      }
      const blend = engine.computeUpperFaceBlend();
      for (const value of Object.values(blend)) {
        expect(value).toBeGreaterThanOrEqual(0);
        expect(value).toBeLessThanOrEqual(1);
      }
    });
  });

  describe('computeMouthInfluence', () => {
    it('neutral 状态幅度正常', () => {
      const influence = engine.computeMouthInfluence();
      expect(influence.amplitudeScale).toBeCloseTo(1.0, 1);
      expect(influence.baselineOpen).toBe(0);
    });

    it('happy 增加嘴型幅度', () => {
      engine.setWeight('happy', 1.0);
      for (let i = 0; i < 60; i++) {
        engine.tick(1 / 60);
      }
      const influence = engine.computeMouthInfluence();
      expect(influence.amplitudeScale).toBeGreaterThan(1.0);
    });

    it('sad 减小嘴型幅度', () => {
      engine.setWeight('sad', 1.0);
      for (let i = 0; i < 60; i++) {
        engine.tick(1 / 60);
      }
      const influence = engine.computeMouthInfluence();
      expect(influence.amplitudeScale).toBeLessThan(1.0);
    });

    it('shy 增加基线嘴型', () => {
      engine.setWeight('shy', 1.0);
      for (let i = 0; i < 60; i++) {
        engine.tick(1 / 60);
      }
      const influence = engine.computeMouthInfluence();
      expect(influence.baselineOpen).toBeGreaterThan(0);
    });

    it('幅度缩放有下限', () => {
      engine.setWeights({ sad: 1.0, upset: 1.0 });
      for (let i = 0; i < 60; i++) {
        engine.tick(1 / 60);
      }
      const influence = engine.computeMouthInfluence();
      expect(influence.amplitudeScale).toBeGreaterThanOrEqual(0.3);
    });
  });

  describe('reset', () => {
    it('重置回 neutral', () => {
      engine.setWeights({ happy: 1.0, excited: 0.5 });
      engine.reset();
      const weights = engine.getWeights();
      expect(weights.neutral).toBe(1);
      expect(weights.happy).toBe(0);
      expect(weights.excited).toBe(0);
    });
  });

  describe('自动归一化', () => {
    it('所有情绪为 0 时 neutral 自动趋向 1', () => {
      engine.setWeight('happy', 0.5);
      engine.tick(1 / 60);

      // 衰减到 0
      for (let i = 0; i < 300; i++) {
        engine.tick(1 / 60);
      }

      const weights = engine.getWeights();
      expect(weights.neutral).toBeGreaterThan(0.5);
    });
  });
});
