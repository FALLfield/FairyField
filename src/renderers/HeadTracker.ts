import * as THREE from 'three';
import type { VRMRenderer } from './VRMRenderer';

/**
 * HeadTracker - 摄像头透视追踪模块
 *
 * 使用摄像头 + MediaPipe Face Landmarker 检测用户头部位置，
 * 输出归一化 3D 坐标驱动 VRM 注视目标。
 */
export interface HeadPosition {
  x: number;
  y: number;
  z: number;
}

type PositionCallback = (position: HeadPosition) => void;

export class HeadTracker {
  private renderer: VRMRenderer;
  private isTracking = false;
  private smoothFactor: number;
  private currentPosition: HeadPosition = { x: 0, y: 0, z: 0 };
  private targetPosition: HeadPosition = { x: 0, y: 0, z: 0 };
  private animationFrameId: number | null = null;
  private videoStream: MediaStream | null = null;
  private videoElement: HTMLVideoElement | null = null;
  private faceLandmarker: unknown = null;
  private lastTimestamp = -1;
  private callbacks: Set<PositionCallback> = new Set();

  // MediaPipe 动态加载状态
  private mediaPipeReady = false;
  private mediaPipeLoading = false;

  constructor(renderer: VRMRenderer, smoothFactor: number = 0.15) {
    this.renderer = renderer;
    this.smoothFactor = smoothFactor;
  }

  /**
   * 启动摄像头追踪
   */
  async start(): Promise<void> {
    if (this.isTracking) return;

    try {
      // 获取摄像头
      this.videoStream = await navigator.mediaDevices.getUserMedia({
        video: { width: 320, height: 240, facingMode: 'user' },
      });

      // 创建视频元素
      this.videoElement = document.createElement('video');
      this.videoElement.srcObject = this.videoStream;
      this.videoElement.setAttribute('playsinline', '');
      await this.videoElement.play();

      // 尝试加载 MediaPipe
      await this.initMediaPipe();

      this.isTracking = true;
      this.runDetectionLoop();
    } catch (error: unknown) {
      const message = error instanceof Error ? error.message : String(error);
      // 摄像头权限拒绝等错误：静默失败，不阻塞应用
      this.cleanup();
      throw new Error(`头部追踪启动失败: ${message}`);
    }
  }

  stop(): void {
    if (!this.isTracking) return;
    this.isTracking = false;

    if (this.animationFrameId !== null) {
      cancelAnimationFrame(this.animationFrameId);
      this.animationFrameId = null;
    }

    this.cleanup();
    this.resetLookAt();
  }

  isActive(): boolean {
    return this.isTracking;
  }

  setSmoothFactor(factor: number): void {
    if (factor < 0 || factor > 1) {
      throw new Error('平滑系数必须在 0 到 1 之间');
    }
    this.smoothFactor = factor;
  }

  getCurrentPosition(): HeadPosition {
    return { ...this.currentPosition };
  }

  onPositionUpdate(callback: PositionCallback): void {
    this.callbacks.add(callback);
  }

  removePositionUpdate(callback: PositionCallback): void {
    this.callbacks.delete(callback);
  }

  dispose(): void {
    this.stop();
    this.callbacks.clear();
    // 释放 MediaPipe 资源
    if (this.faceLandmarker && typeof (this.faceLandmarker as { close?: () => void }).close === 'function') {
      (this.faceLandmarker as { close: () => void }).close();
      this.faceLandmarker = null;
    }
  }

  // -- 私有方法 --

  private async initMediaPipe(): Promise<void> {
    if (this.mediaPipeReady || this.mediaPipeLoading) return;
    this.mediaPipeLoading = true;

    try {
      // 使用 new Function 避免构建工具静态解析 MediaPipe 依赖
      const dynamicImport = new Function('specifier', 'return import(specifier)') as (
        specifier: string,
      ) => Promise<unknown>;
      const vision = await dynamicImport('@mediapipe/tasks-vision') as {
        FaceLandmarker: { createFromOptions: (ctx: unknown, opts: unknown) => Promise<unknown> };
        FilesetResolver: { forVisionTasks: (url: string) => Promise<unknown> };
      };
      const { FaceLandmarker, FilesetResolver } = vision;

      const filesetResolver = await FilesetResolver.forVisionTasks(
        'https://cdn.jsdelivr.net/npm/@mediapipe/tasks-vision@latest/wasm',
      );

      this.faceLandmarker = await FaceLandmarker.createFromOptions(filesetResolver, {
        baseOptions: {
          modelAssetPath: 'https://storage.googleapis.com/mediapipe-models/face_landmarker/face_landmarker/float16/1/face_landmarker.task',
          delegate: 'GPU',
        },
        runningMode: 'VIDEO',
        numFaces: 1,
      });

      this.mediaPipeReady = true;
    } catch {
      // MediaPipe 加载失败，使用基于视频帧的简单检测作为 fallback
      this.mediaPipeReady = false;
    } finally {
      this.mediaPipeLoading = false;
    }
  }

  private runDetectionLoop(): void {
    if (!this.isTracking || !this.videoElement) return;

    this.animationFrameId = requestAnimationFrame(async () => {
      if (!this.isTracking || !this.videoElement) return;

      if (this.mediaPipeReady && this.faceLandmarker) {
        await this.detectWithMediaPipe();
      } else {
        this.detectFallback();
      }

      this.smoothAndUpdate();
      this.runDetectionLoop();
    });
  }

  private async detectWithMediaPipe(): Promise<void> {
    const landmark = this.faceLandmarker as {
      detectForVideo: (video: HTMLVideoElement, timestamp: number) => { faceLandmarks: Array<Array<{ x: number; y: number; z: number }>> };
    };

    const now = performance.now();
    if (now === this.lastTimestamp) return;
    this.lastTimestamp = now;

    try {
      const result = landmark.detectForVideo(this.videoElement!, now);
      if (result.faceLandmarks.length > 0) {
        const nose = result.faceLandmarks[0][1]; // 鼻尖
        // MediaPipe 坐标是 0-1 归一化的，镜像 x 轴
        this.targetPosition = {
          x: (0.5 - nose.x) * 2, // 镜像并映射到 [-1, 1]
          y: (0.5 - nose.y) * 2,
          z: -nose.z * 2,
        };
      }
    } catch {
      // 检测失败时保持当前位置
    }
  }

  /**
   * Fallback：使用视频帧的亮度中心作为粗略的头部位置估计
   */
  private detectFallback(): void {
    if (!this.videoElement) return;

    // 简单的基于视频帧中心的运动检测
    // 这是一个粗略的 fallback，实际效果有限
    const canvas = document.createElement('canvas');
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    canvas.width = 32;
    canvas.height = 24;
    ctx.drawImage(this.videoElement, 0, 0, 32, 24);

    const imageData = ctx.getImageData(0, 0, 32, 24);
    const data = imageData.data;

    let sumX = 0, sumY = 0, totalBrightness = 0;
    for (let y = 0; y < 24; y++) {
      for (let x = 0; x < 32; x++) {
        const idx = (y * 32 + x) * 4;
        const brightness = data[idx] * 0.299 + data[idx + 1] * 0.587 + data[idx + 2] * 0.114;
        sumX += x * brightness;
        sumY += y * brightness;
        totalBrightness += brightness;
      }
    }

    if (totalBrightness > 0) {
      this.targetPosition = {
        x: ((sumX / totalBrightness) / 32 - 0.5) * -2, // 镜像
        y: ((sumY / totalBrightness) / 24 - 0.5) * -2,
        z: 0,
      };
    }
  }

  private smoothAndUpdate(): void {
    const lerp = (a: number, b: number, t: number) => a + (b - a) * t;

    this.currentPosition.x = lerp(this.currentPosition.x, this.targetPosition.x, this.smoothFactor);
    this.currentPosition.y = lerp(this.currentPosition.y, this.targetPosition.y, this.smoothFactor);
    this.currentPosition.z = lerp(this.currentPosition.z, this.targetPosition.z, this.smoothFactor);

    // 映射到 VRM look-at 坐标空间（以角色脸部中心为基准）
    const lookAtX = this.currentPosition.x * 0.5;
    const lookAtY = 0.9 + this.currentPosition.y * 0.3;
    const lookAtZ = 0.5 + this.currentPosition.z * 0.2;

    this.renderer.setLookAtTarget(new THREE.Vector3(lookAtX, lookAtY, lookAtZ));

    // 通知回调
    for (const cb of this.callbacks) {
      cb({ ...this.currentPosition });
    }
  }

  private resetLookAt(): void {
    this.currentPosition = { x: 0, y: 0, z: 0 };
    this.targetPosition = { x: 0, y: 0, z: 0 };
    this.renderer.setLookAtTarget(new THREE.Vector3(0, 0.9, 0.5));
  }

  private cleanup(): void {
    if (this.videoStream) {
      this.videoStream.getTracks().forEach(track => track.stop());
      this.videoStream = null;
    }
    if (this.videoElement) {
      this.videoElement.srcObject = null;
      this.videoElement = null;
    }
  }
}
