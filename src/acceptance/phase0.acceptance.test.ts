/**
 * Phase 0 Acceptance Tests — VRM 渲染管线
 *
 * 验证 VRMRenderer 核心功能：
 * - 初始化 Three.js 场景
 * - VRM 模型加载
 * - 动画循环
 * - 表情/注视/口型接口
 *
 * 注意：
 * - VRMRenderer、EyeTrackModule、HitTestModule 依赖 WebGL/Three.js，
 *   在 jsdom 环境下需要完整 mock，单独的单元测试覆盖已在各自 .test.ts 中完成。
 * - 此处仅验证不依赖 Three.js 的模块导出。
 * - 部分 VRMRenderer 集成测试需要浏览器环境或 Playwright。
 */

import { describe, it, expect } from 'vitest';

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
    it('默认表情应为 neutral', async () => {
      const { ExpressionModule } = await import('../modules/ExpressionModule');
      const module = new ExpressionModule();
      expect(module.getCurrentExpression()).toBe('neutral');
    });

    it('setExpression 应更新当前表情', async () => {
      const { ExpressionModule } = await import('../modules/ExpressionModule');
      const module = new ExpressionModule();
      module.setExpression('happy');
      expect(module.getCurrentExpression()).toBe('happy');
    });
  });

  describe('HologramRenderer 基础行为', () => {
    it('默认应禁用', async () => {
      const { HologramRenderer } = await import('../renderers/HologramRenderer');
      const renderer = new HologramRenderer();
      expect(renderer.isEnabled()).toBe(false);
    });

    it('enable/enable 应切换状态', async () => {
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
      const tracker = new HeadTracker();
      expect(tracker.isActive()).toBe(false);
    });

    it('start/stop 应切换状态', async () => {
      const { HeadTracker } = await import('../renderers/HeadTracker');
      const tracker = new HeadTracker();
      await tracker.start();
      expect(tracker.isActive()).toBe(true);
      tracker.stop();
      expect(tracker.isActive()).toBe(false);
    });
  });
});
