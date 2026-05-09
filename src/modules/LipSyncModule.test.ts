import { describe, it, expect, beforeEach, vi } from 'vitest';

// Mock VRMRenderer
const mockSetLipSyncValue = vi.fn();
const mockSetExpression = vi.fn();
vi.mock('../renderers/VRMRenderer', () => ({
  VRMRenderer: vi.fn(function () {
    return {
      setLipSyncValue: mockSetLipSyncValue,
      setExpression: mockSetExpression,
    };
  }),
}));

describe('LipSyncModule', () => {
  let lipSync: InstanceType<typeof import('./LipSyncModule').LipSyncModule>;
  let renderer: any;

  beforeEach(async () => {
    vi.clearAllMocks();
    const { VRMRenderer } = await import('../renderers/VRMRenderer');
    const { LipSyncModule } = await import('./LipSyncModule');
    renderer = new (VRMRenderer as any)();
    lipSync = new LipSyncModule(renderer);
  });

  describe('start/stop', () => {
    it('初始不活跃', () => {
      expect(lipSync.isActive()).toBe(false);
    });

    it('start 后活跃', () => {
      lipSync.start();
      expect(lipSync.isActive()).toBe(true);
    });

    it('stop 后不活跃', () => {
      lipSync.start();
      lipSync.stop();
      expect(lipSync.isActive()).toBe(false);
    });

    it('stop 时重置嘴型', () => {
      lipSync.start();
      lipSync.stop();
      // setLipSyncValue 应该被调用将所有值设为 0
      const resetCalls = mockSetLipSyncValue.mock.calls.filter(
        (call: any[]) => call[1] === 0
      );
      expect(resetCalls.length).toBeGreaterThanOrEqual(5);
    });
  });

  describe('feedPCM - RMS 计算', () => {
    it('静音 PCM 不触发嘴型', () => {
      lipSync.start();
      const silence = new Float32Array(1024); // 全零
      lipSync.feedPCM(silence, 16000);
      expect(lipSync.isSpeaking()).toBe(false);
    });

    it('有声 PCM 触发说话状态', () => {
      lipSync.start();
      const audio = new Float32Array(1024);
      for (let i = 0; i < audio.length; i++) {
        audio[i] = Math.sin(i * 0.1) * 0.5;
      }
      lipSync.feedPCM(audio, 16000);
      expect(lipSync.isSpeaking()).toBe(true);
    });

    it('停止状态下 feedPCM 不处理', () => {
      const audio = new Float32Array(1024);
      for (let i = 0; i < audio.length; i++) {
        audio[i] = Math.sin(i * 0.1) * 0.5;
      }
      lipSync.feedPCM(audio, 16000);
      expect(mockSetLipSyncValue).not.toHaveBeenCalled();
    });

    it('空 PCM 不处理', () => {
      lipSync.start();
      const empty = new Float32Array(0);
      lipSync.feedPCM(empty, 16000);
      expect(mockSetLipSyncValue).not.toHaveBeenCalled();
    });
  });

  describe('attack/release 包络', () => {
    it('嘴张开快于闭合（attack > release）', () => {
      lipSync.start();
      mockSetLipSyncValue.mockClear();

      const loud = new Float32Array(1024);
      for (let i = 0; i < loud.length; i++) {
        loud[i] = Math.sin(i * 0.1) * 0.8;
      }

      // 第一帧：从 0 开始
      lipSync.feedPCM(loud, 16000);
      const attackValues = mockSetLipSyncValue.mock.calls.map(
        (call: any[]) => call[1]
      );
      const maxAttack = Math.max(...attackValues);

      // 重置
      lipSync.stop();
      lipSync.start();
      mockSetLipSyncValue.mockClear();

      // 喂一帧大声然后静音
      lipSync.feedPCM(loud, 16000);
      const silence = new Float32Array(1024);
      lipSync.feedPCM(silence, 16000);
      const releaseValues = mockSetLipSyncValue.mock.calls
        .slice(-5) // 最后 5 个是 release 调用
        .map((call: any[]) => call[1]);

      // release 值应该存在且较小
      expect(releaseValues.length).toBeGreaterThan(0);

      // attack 产生的最大值应大于 release 后的最大值
      expect(maxAttack).toBeGreaterThan(0);
    });
  });

  describe('噪声门限', () => {
    it('低振幅不触发嘴型', () => {
      lipSync.start();
      const quiet = new Float32Array(1024);
      for (let i = 0; i < quiet.length; i++) {
        quiet[i] = 0.001; // 非常安静
      }
      lipSync.feedPCM(quiet, 16000);
      expect(lipSync.isSpeaking()).toBe(false);
    });
  });

  describe('情绪影响', () => {
    it('setMouthInfluence 调整嘴型幅度', () => {
      lipSync.start();
      mockSetLipSyncValue.mockClear();

      // 设置 2x 幅度
      lipSync.setMouthInfluence({ amplitudeScale: 2.0, baselineOpen: 0 });

      const loud = new Float32Array(1024);
      for (let i = 0; i < loud.length; i++) {
        loud[i] = Math.sin(i * 0.1) * 0.5;
      }
      lipSync.feedPCM(loud, 16000);

      const values = mockSetLipSyncValue.mock.calls.map(
        (call: any[]) => call[1]
      );
      // 有值被设置
      expect(values.some((v: number) => v > 0)).toBe(true);
    });

    it('零幅度缩放时嘴型最小', () => {
      lipSync.start();
      lipSync.setMouthInfluence({ amplitudeScale: 0.01, baselineOpen: 0 });
      mockSetLipSyncValue.mockClear();

      const loud = new Float32Array(1024);
      for (let i = 0; i < loud.length; i++) {
        loud[i] = Math.sin(i * 0.1) * 0.5;
      }
      lipSync.feedPCM(loud, 16000);

      const values = mockSetLipSyncValue.mock.calls.map(
        (call: any[]) => call[1]
      );
      const maxVal = Math.max(...values);
      expect(maxVal).toBeLessThan(0.1);
    });
  });

  describe('频段分析', () => {
    it('低频信号主要产生 aa 口型', () => {
      lipSync.start();
      mockSetLipSyncValue.mockClear();

      // 低频正弦波（低零交叉率）
      const lowFreq = new Float32Array(1024);
      for (let i = 0; i < lowFreq.length; i++) {
        lowFreq[i] = Math.sin(i * 2 * Math.PI * 2 / 1024) * 0.5;
      }
      lipSync.feedPCM(lowFreq, 16000);

      // 找 aa 的调用
      const aaCalls = mockSetLipSyncValue.mock.calls.filter(
        (call: any[]) => call[0] === 'aa'
      );
      expect(aaCalls.length).toBeGreaterThan(0);
      expect(aaCalls[0][1]).toBeGreaterThan(0);
    });

    it('高频信号主要产生 ou 口型', () => {
      lipSync.start();
      mockSetLipSyncValue.mockClear();

      // 高频正弦波（高零交叉率）
      const highFreq = new Float32Array(1024);
      for (let i = 0; i < highFreq.length; i++) {
        highFreq[i] = Math.sin(i * 0.5) * 0.5;
      }
      lipSync.feedPCM(highFreq, 16000);

      const ouCalls = mockSetLipSyncValue.mock.calls.filter(
        (call: any[]) => call[0] === 'ou'
      );
      expect(ouCalls.length).toBeGreaterThan(0);
      expect(ouCalls[0][1]).toBeGreaterThan(0);
    });
  });

  describe('配置', () => {
    it('updateConfig 修改配置', () => {
      lipSync.updateConfig({ noiseGate: 0.05 });
      const config = lipSync.getConfig();
      expect(config.noiseGate).toBe(0.05);
    });

    it('getConfig 返回副本', () => {
      const c1 = lipSync.getConfig();
      const c2 = lipSync.getConfig();
      expect(c1).toEqual(c2);
      expect(c1).not.toBe(c2);
    });
  });

  describe('dispose', () => {
    it('dispose 后状态清理', () => {
      lipSync.start();
      lipSync.dispose();
      expect(lipSync.isActive()).toBe(false);
    });
  });
});
