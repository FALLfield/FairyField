/**
 * HologramRenderer - 全息模式适配器（Phase 4 Stub）
 *
 * 预留全息投影模式的渲染适配器。
 * Phase 4 将实现：
 * - 摄像头透视追踪（HeadTracker）
 * - 基于视角的全息投影矩阵变换
 * - 全息光效、色散、扫描线等视觉特效
 *
 * 当前仅作为占位模块导出。
 */

export class HologramRenderer {
  private enabled = false;

  /**
   * 启用全息模式
   */
  enable(): void {
    this.enabled = true;
  }

  /**
   * 禁用全息模式
   */
  disable(): void {
    this.enabled = false;
  }

  /**
   * 获取当前全息模式状态
   */
  isEnabled(): boolean {
    return this.enabled;
  }
}
