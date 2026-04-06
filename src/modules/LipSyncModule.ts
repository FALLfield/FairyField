/**
 * LipSyncModule - 口型同步模块（Phase 2 Stub）
 *
 * 预留音频驱动的口型同步模块。
 * Phase 2 将实现：
 * - 接收 TTS 输出的音频 PCM 数据
 * - 实时分析音频振幅/频率
 * - 映射到 VRM 的 'aa', 'ih', 'ou', 'ee', 'oh' BlendShape
 *
 * 当前仅作为占位模块导出。
 */

export class LipSyncModule {
  private isRunning = false;

  /**
   * 启动口型同步
   */
  start(): void {
    this.isRunning = true;
  }

  /**
   * 停止口型同步
   */
  stop(): void {
    this.isRunning = false;
  }

  /**
   * 获取模块运行状态
   */
  isActive(): boolean {
    return this.isRunning;
  }
}
