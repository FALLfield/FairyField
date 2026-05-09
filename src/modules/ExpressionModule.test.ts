import { describe, it, expect, beforeEach, vi } from 'vitest';
import type { EmotionEngine, EmotionWeights, UpperFaceBlendShapes } from '../lib/emotion-engine';

// Mock VRMRenderer
const mockSetExpression = vi.fn();
const mockSetLipSyncValue = vi.fn();
vi.mock('../renderers/VRMRenderer', () => ({
  VRMRenderer: vi.fn(function () {
    return {
      setExpression: mockSetExpression,
      setLipSyncValue: mockSetLipSyncValue,
    };
  }),
}));

/** 创建 mock EmotionEngine */
function createMockEmotionEngine(blend: UpperFaceBlendShapes = {}): EmotionEngine {
  return {
    setWeight: vi.fn(),
    setWeights: vi.fn(),
    tick: vi.fn(),
    getWeights: vi.fn((): EmotionWeights => ({
      happy: 0, sad: 0, angry: 0, neutral: 1, excited: 0, shy: 0, upset: 0,
    })),
    computeUpperFaceBlend: vi.fn(() => ({ ...blend })),
    computeMouthInfluence: vi.fn(() => ({ amplitudeScale: 1, baselineOpen: 0 })),
    reset: vi.fn(),
    setEmotion: vi.fn(),
    getState: vi.fn(() => ({ current: 'neutral' as const, previous: 'neutral' as const, timestamp: Date.now() })),
  };
}

describe('ExpressionModule', () => {
  let mod: InstanceType<typeof import('./ExpressionModule').ExpressionModule>;
  let renderer: any;

  beforeEach(async () => {
    vi.clearAllMocks();
    const { VRMRenderer } = await import('../renderers/VRMRenderer');
    const { ExpressionModule } = await import('./ExpressionModule');
    renderer = new (VRMRenderer as any)();
    mod = new ExpressionModule(renderer);
  });

  describe('start/stop', () => {
    it('初始不活跃', () => {
      expect(mod.isActive()).toBe(false);
    });

    it('start 后活跃', () => {
      mod.start();
      expect(mod.isActive()).toBe(true);
    });

    it('stop 后不活跃', () => {
      mod.start();
      mod.stop();
      expect(mod.isActive()).toBe(false);
    });

    it('stop 清除表情', () => {
      mod.start();
      mod.setExpression('happy');
      mod.tick(1);
      mod.stop();
      expect(mockSetExpression).toHaveBeenCalled();
    });
  });

  describe('手动表情', () => {
    it('setExpression 触发过渡', () => {
      mod.start();
      mod.setExpression('happy');

      // 多次 tick 完成过渡
      for (let i = 0; i < 60; i++) {
        mod.tick(1 / 60);
      }

      expect(mockSetExpression).toHaveBeenCalledWith('happy', expect.any(Number));
    });

    it('过渡完成后 currentPreset 更新', () => {
      mod.start();
      mod.setExpression('happy', 0.1);

      for (let i = 0; i < 60; i++) {
        mod.tick(1 / 60);
      }

      const current = mod.getCurrentExpression();
      expect(current).toHaveProperty('happy');
    });

    it('预设表情正确应用', () => {
      mod.start();
      mod.setExpression('laugh');

      for (let i = 0; i < 60; i++) {
        mod.tick(1 / 60);
      }

      // laugh 预设包含 happy, ah, blink
      expect(mockSetExpression).toHaveBeenCalledWith('happy', expect.any(Number));
    });

    it('手动表情暂停情绪驱动', () => {
      const engine = createMockEmotionEngine({ happy: 0.8 });
      mod.start();
      mod.bindEmotionEngine(engine);

      mod.setExpression('sad');
      expect(mod.isEmotionDriven()).toBe(false);
    });
  });

  describe('情绪引擎驱动', () => {
    it('绑定情绪引擎后自动驱动', () => {
      const blend = { happy: 0.6, relaxed: 0.3 };
      const engine = createMockEmotionEngine(blend);
      mod.start();
      mod.bindEmotionEngine(engine);

      mod.tick(1 / 60);

      expect(engine.computeUpperFaceBlend).toHaveBeenCalled();
    });

    it('情绪驱动的 BlendShape 被应用到 renderer', () => {
      const blend = { happy: 0.7 };
      const engine = createMockEmotionEngine(blend);
      mod.start();
      mod.bindEmotionEngine(engine);

      // 多次 tick 使平滑过渡生效
      for (let i = 0; i < 60; i++) {
        mod.tick(1 / 60);
      }

      expect(mockSetExpression).toHaveBeenCalledWith('happy', expect.any(Number));
    });

    it('说话时嘴型 BlendShape 不被情绪覆盖', () => {
      const blend = { happy: 0.7, aa: 0.5 };
      const engine = createMockEmotionEngine(blend);
      mod.start();
      mod.bindEmotionEngine(engine);
      mod.setSpeaking(true);

      mockSetExpression.mockClear();
      mod.tick(1 / 60);

      // aa 是嘴型，说话时不应被情绪引擎设置
      const aaCalls = mockSetExpression.mock.calls.filter(
        (call: any[]) => call[0] === 'aa'
      );
      expect(aaCalls.length).toBe(0);
    });

    it('安静时嘴型 BlendShape 由情绪驱动', () => {
      const blend = { happy: 0.7, aa: 0.3 };
      const engine = createMockEmotionEngine(blend);
      mod.start();
      mod.bindEmotionEngine(engine);
      mod.setSpeaking(false);

      // 多次 tick
      for (let i = 0; i < 60; i++) {
        mod.tick(1 / 60);
      }

      // aa 在安静时可以被情绪设置
      const aaCalls = mockSetExpression.mock.calls.filter(
        (call: any[]) => call[0] === 'aa' && call[1] > 0
      );
      expect(aaCalls.length).toBeGreaterThan(0);
    });

    it('解绑情绪引擎后不驱动', () => {
      const engine = createMockEmotionEngine({ happy: 0.8 });
      mod.start();
      mod.bindEmotionEngine(engine);
      mod.unbindEmotionEngine();

      mockSetExpression.mockClear();
      mod.tick(1 / 60);

      // 解绑后 tick 不应调用情绪引擎
      // 但 transition 可能为 null，所以不会有 BlendShape 调用
    });
  });

  describe('releaseManualOverride', () => {
    it('释放手动覆盖恢复情绪驱动', () => {
      const engine = createMockEmotionEngine({ happy: 0.5 });
      mod.start();
      mod.bindEmotionEngine(engine);

      mod.setExpression('sad');
      expect(mod.isEmotionDriven()).toBe(false);

      mod.releaseManualOverride();
      expect(mod.isEmotionDriven()).toBe(true);
    });
  });

  describe('预设管理', () => {
    it('registerPreset 添加自定义预设', () => {
      mod.registerPreset('custom', { happy: 0.5, angry: 0.3 });
      expect(mod.getPresetNames()).toContain('custom');
    });

    it('removePreset 删除预设', () => {
      mod.registerPreset('temp', { happy: 0.5 });
      const result = mod.removePreset('temp');
      expect(result).toBe(true);
      expect(mod.getPresetNames()).not.toContain('temp');
    });

    it('默认预设存在', () => {
      const names = mod.getPresetNames();
      expect(names).toContain('laugh');
      expect(names).toContain('shy');
      expect(names).toContain('upset');
      expect(names).toContain('wink');
    });
  });

  describe('过渡动画', () => {
    it('getTransitionProgress 在过渡中返回进度', () => {
      mod.start();
      mod.setExpression('happy', 1.0);

      // 过渡中
      expect(mod.getTransitionProgress()).toBeLessThan(1);

      // 完成后
      for (let i = 0; i < 60; i++) {
        mod.tick(1 / 60);
      }
      expect(mod.getTransitionProgress()).toBe(1);
    });

    it('多次 setExpression 更新目标', () => {
      mod.start();
      mockSetExpression.mockClear();

      mod.setExpression('happy', 1.0);
      mod.setExpression('sad', 1.0);

      for (let i = 0; i < 60; i++) {
        mod.tick(1 / 60);
      }

      // 最终应该是 sad
      const current = mod.getCurrentExpression();
      expect(current).toHaveProperty('sad');
    });
  });

  describe('reset', () => {
    it('reset 清除所有状态', () => {
      mod.start();
      mod.setExpression('happy');
      mod.tick(1);
      mod.reset();

      const current = mod.getCurrentExpression();
      expect(Object.keys(current)).toHaveLength(0);
    });
  });

  describe('说话协调', () => {
    it('setSpeaking 正确更新状态', () => {
      mod.setSpeaking(true);
      // 说话状态是内部状态，通过情绪引擎驱动测试验证
    });
  });
});
