import * as THREE from 'three';
import type { VRMRenderer } from '../renderers/VRMRenderer';

/**
 * EyeTrackModule - 眼神跟随模块（VRM 适配版）
 *
 * 负责实时采集鼠标位置并驱动 VRM 的 LookAt 注视目标。
 * 使用平滑插值避免眼神突变。
 *
 * **验证需求 3**：
 * - 需求 3.1: 持续采集全局鼠标坐标
 * - 需求 3.2: 将鼠标坐标映射为 3D 注视目标并更新 VRM
 * - 需求 3.3: 对眼神移动应用平滑插值
 */
export class EyeTrackModule {
  private renderer: VRMRenderer;
  private targetPosition: THREE.Vector3;
  private currentPosition: THREE.Vector3;
  private smoothFactor: number;
  private isRunning: boolean = false;
  private animationFrameId: number | null = null;
  private boundMouseMoveHandler: ((event: MouseEvent) => void) | null = null;

  /**
   * 构造函数
   * @param renderer - VRMRenderer 实例
   * @param smoothFactor - 平滑系数（0-1），默认 0.1，数值越大响应越快
   */
  constructor(renderer: VRMRenderer, smoothFactor: number = 0.1) {
    this.renderer = renderer;
    this.smoothFactor = smoothFactor;

    // 注视目标：默认在角色正前方
    this.targetPosition = new THREE.Vector3(0, 0.9, 0.5);
    this.currentPosition = new THREE.Vector3(0, 0.9, 0.5);
  }

  /**
   * 启动眼神跟随
   */
  start(): void {
    if (this.isRunning) {
      return;
    }

    this.isRunning = true;

    // 绑定鼠标移动事件
    this.boundMouseMoveHandler = this.onMouseMove.bind(this);
    window.addEventListener('mousemove', this.boundMouseMoveHandler);

    // 启动插值更新循环
    this.update();
  }

  /**
   * 停止眼神跟随
   */
  stop(): void {
    if (!this.isRunning) {
      return;
    }

    this.isRunning = false;

    if (this.boundMouseMoveHandler) {
      window.removeEventListener('mousemove', this.boundMouseMoveHandler);
      this.boundMouseMoveHandler = null;
    }

    if (this.animationFrameId !== null) {
      cancelAnimationFrame(this.animationFrameId);
      this.animationFrameId = null;
    }

    // 重置注视目标到正前方
    this.resetLookAt();
  }

  /**
   * 鼠标移动事件处理
   * 将屏幕坐标映射为 3D 注视目标位置
   */
  private onMouseMove(event: MouseEvent): void {
    // 将屏幕坐标归一化到 [-1, 1]
    const normalizedX = (event.clientX / window.innerWidth) * 2 - 1;
    const normalizedY = -(event.clientY / window.innerHeight) * 2 + 1;

    // 映射到 3D 空间：水平范围 ±0.5，垂直范围 ±0.3
    // 以角色脸部中心 (0, 0.9, 0.5) 为基准偏移
    this.targetPosition.set(
      normalizedX * 0.5,
      0.9 + normalizedY * 0.3,
      0.5
    );
  }

  /**
   * 更新循环：平滑插值注视位置并更新 VRM
   */
  private update(): void {
    if (!this.isRunning) {
      return;
    }

    // 线性插值：current += (target - current) * factor
    this.currentPosition.lerp(this.targetPosition, this.smoothFactor);

    // 更新 VRM 注视目标
    this.renderer.setLookAtTarget(this.currentPosition);

    this.animationFrameId = requestAnimationFrame(() => this.update());
  }

  /**
   * 重置注视目标到角色正前方
   */
  private resetLookAt(): void {
    const defaultTarget = new THREE.Vector3(0, 0.9, 0.5);
    this.renderer.setLookAtTarget(defaultTarget);
    this.currentPosition.copy(defaultTarget);
    this.targetPosition.copy(defaultTarget);
  }

  /**
   * 设置平滑系数
   * @param factor - 平滑系数（0-1），数值越大响应越快
   */
  setSmoothFactor(factor: number): void {
    if (factor < 0 || factor > 1) {
      throw new Error('平滑系数必须在 0 到 1 之间');
    }
    this.smoothFactor = factor;
  }

  /**
   * 获取当前平滑系数
   */
  getSmoothFactor(): number {
    return this.smoothFactor;
  }

  /**
   * 检查模块是否正在运行
   */
  isActive(): boolean {
    return this.isRunning;
  }

  /**
   * 获取当前注视位置
   */
  getCurrentPosition(): THREE.Vector3 {
    return this.currentPosition.clone();
  }

  /**
   * 获取目标注视位置
   */
  getTargetPosition(): THREE.Vector3 {
    return this.targetPosition.clone();
  }
}
