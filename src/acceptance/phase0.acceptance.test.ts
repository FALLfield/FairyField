/**
 * Phase 0 Acceptance Tests — VRM 渲染管线
 *
 * 验证 VRMRenderer 核心功能和模块导出。
 * Phase 1 后部分模块 API 已更新，此处同步更新。
 */

import { describe, it, expect } from 'vitest';

/** 创建 mock VRMRenderer（用于不需要真实 Three.js 的测试） */
function createMockRenderer() {
  return {
    setExpression: () => {},
    setLipSyncValue: () => {},
    setLipSync: () => {},
    setLookAtTarget: () => {},
  } as unknown as import('../renderers/VRMRenderer').VRMRenderer;
}

describe('Phase 0 — VRM 渲染管线验收', () => {
  describe('模块导出', () => {
    it('ExpressionModule 应该可导入', async () => {
      const { ExpressionModule } = await import('../modules/ExpressionModule');
      expect(ExpressionModule).toBeDefined();
    });

    it('LipSyncModule 类应该可导入', async () => {
      const { LipSyncModule } = await import('../modules/LipSyncModule');
      expect(LipSyncModule).toBeDefined();
    });
  });

  describe('Stub 模块', () => {
    it('HologramRenderer 应该可导入', async () => {
      const { HologramRenderer } = await import('../renderers/HologramRenderer');
      expect(HologramRenderer).toBeDefined();
    });

    it('HeadTracker 应该可导入', async () => {
      const { HeadTracker } = await import('../renderers/HeadTracker');
      expect(HeadTracker).toBeDefined();
    });
  });

  describe('ExpressionModule 基础行为', () => {
    it('默认表情应为空', async () => {
      const { ExpressionModule } = await import('../modules/ExpressionModule');
      const module = new ExpressionModule(createMockRenderer());
      const current = module.getCurrentExpression();
      expect(Object.keys(current)).toHaveLength(0);
    });

    it('setExpression 应更新当前表情', async () => {
      const { ExpressionModule } = await import('../modules/ExpressionModule');
      const module = new ExpressionModule(createMockRenderer());
      module.start();
      module.setExpression('happy');
      // 立即完成过渡
      for (let i = 0; i < 100; i++) module.tick(1);
      const current = module.getCurrentExpression();
      expect(current.happy).toBe(1.0);
      module.stop();
    });
  });

  describe('HologramRenderer 基础行为', () => {
    it('默认应禁用', async () => {
      const { HologramRenderer } = await import('../renderers/HologramRenderer');
      const renderer = new HologramRenderer();
      expect(renderer.isEnabled()).toBe(false);
    });

    it('enable/disable 应切换状态', async () => {
      const { HologramRenderer } = await import('../renderers/HologramRenderer');
      const renderer = new HologramRenderer();
      renderer.enable();
      expect(renderer.isEnabled()).toBe(true);
      renderer.disable();
      expect(renderer.isEnabled()).toBe(false);
    });
  });

  describe('HeadTracker 基础行为', () => {
    it('默认应停止', async () => {
      const { HeadTracker } = await import('../renderers/HeadTracker');
      const tracker = new HeadTracker(createMockRenderer());
      expect(tracker.isActive()).toBe(false);
    });

    it('start 失败时应保持停止', async () => {
      const { HeadTracker } = await import('../renderers/HeadTracker');
      const tracker = new HeadTracker(createMockRenderer());
      // getUserMedia 在测试环境会失败
      await expect(tracker.start()).rejects.toThrow();
      expect(tracker.isActive()).toBe(false);
    });

    it('stop 不应抛出', async () => {
      const { HeadTracker } = await import('../renderers/HeadTracker');
      const tracker = new HeadTracker(createMockRenderer());
      expect(() => tracker.stop()).not.toThrow();
      expect(tracker.isActive()).toBe(false);
    });
  });
});
