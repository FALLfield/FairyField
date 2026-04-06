/**
 * ExpressionModule - 表情管理模块（Phase 1 Stub）
 *
 * 管理 VRM 模型的表情切换和混合。
 * Phase 1 将实现：
 * - 表情预设管理（happy, sad, angry, surprised, neutral 等）
 * - 表情过渡动画（平滑 blend）
 * - 与 AI 情绪系统的对接接口
 *
 * 当前定义基础类型和接口，具体实现在 Phase 1 完成。
 */

/** VRM 表情预设名称 */
export type ExpressionName =
  | 'neutral'
  | 'happy'
  | 'angry'
  | 'sad'
  | 'relaxed'
  | 'surprised'
  | 'ee'
  | 'ah'
  | 'oh'
  | 'ou'
  | 'blink'
  | 'blink_l'
  | 'blink_r'
  | 'lookup'
  | 'lookdown'
  | 'lookleft'
  | 'lookright';

export class ExpressionModule {
  private currentExpression: ExpressionName = 'neutral';
  private previousExpression: ExpressionName | null = null;
  private transitionWeight = 1.0;

  /**
   * 设置当前表情
   * @param name - 表情名称
   * @param transitionDuration - 过渡时长（秒），默认 0.3
   */
  setExpression(name: ExpressionName, _transitionDuration: number = 0.3): void {
    this.previousExpression = this.currentExpression;
    this.currentExpression = name;
    this.transitionWeight = 0;
  }

  /**
   * 获取当前表情名称
   */
  getCurrentExpression(): ExpressionName {
    return this.currentExpression;
  }

  /**
   * 获取上一个表情名称
   */
  getPreviousExpression(): ExpressionName | null {
    return this.previousExpression;
  }

  /**
   * 获取过渡权重（0-1）
   */
  getTransitionWeight(): number {
    return this.transitionWeight;
  }
}
