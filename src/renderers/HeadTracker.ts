/**
 * HeadTracker - 摄像头透视追踪（Phase 1 Stub）
 *
 * 预留基于摄像头的头部/面部追踪模块。
 * Phase 1 将实现：
 * - 使用 getUserMedia 获取摄像头画面
 * - 面部关键点检测（MediaPipe 或等效方案）
 * - 输出头部 3D 位置/旋转，驱动 VRM 注视目标
 *
 * 当前仅作为占位模块导出。
 */

export class HeadTracker {
  private isTracking = false;

  /**
   * 启动摄像头追踪
   */
  async start(): Promise<void> {
    this.isTracking = true;
  }

  /**
   * 停止摄像头追踪
   */
  stop(): void {
    this.isTracking = false;
  }

  /**
   * 获取追踪状态
   */
  isActive(): boolean {
    return this.isTracking;
  }
}
