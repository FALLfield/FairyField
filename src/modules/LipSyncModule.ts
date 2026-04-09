import type { VRMRenderer } from '../renderers/VRMRenderer';

/**
 * LipSyncModule - 口型同步模块
 *
 * 分析音频频谱，将振幅映射到 VRM 口型 BlendShape。
 * 支持 5 种口型：aa, ih, ou, ee, oh
 */
export class LipSyncModule {
  private renderer: VRMRenderer;
  private isRunning = false;
  private analyser: AnalyserNode | null = null;
  private dataArray: Uint8Array | null = null;
  private smoothFactor: number;
  private currentValues: Record<string, number> = {
    aa: 0, ih: 0, ou: 0, ee: 0, oh: 0,
  };
  private audioContext: AudioContext | null = null;

  constructor(renderer: VRMRenderer, smoothFactor: number = 0.3) {
    this.renderer = renderer;
    this.smoothFactor = smoothFactor;
  }

  /**
   * 连接 Web Audio API 音频源
   * @param source - AudioNode（如 MediaElementSource, MediaStreamSource）
   */
  connectAudioSource(source: AudioNode): void {
    // 创建或复用 AudioContext
    if (!this.audioContext) {
      this.audioContext = new AudioContext();
    }

    this.analyser = this.audioContext.createAnalyser();
    this.analyser.fftSize = 256;
    this.analyser.smoothingTimeConstant = 0.8;

    const bufferLength = this.analyser.frequencyBinCount;
    this.dataArray = new Uint8Array(bufferLength);

    source.connect(this.analyser);
    // 不连接到 destination，避免回声
  }

  /**
   * 直接喂入 PCM 数据（供 TTS 引擎使用）
   * @param pcm - Float32 PCM 数据
   * @param sampleRate - 采样率
   */
  feedPCM(pcm: Float32Array, _sampleRate: number): void {
    if (!this.isRunning || pcm.length === 0) return;

    // 计算 RMS 振幅
    let sum = 0;
    for (let i = 0; i < pcm.length; i++) {
      sum += pcm[i] * pcm[i];
    }
    const rms = Math.sqrt(sum / pcm.length);
    const amplitude = Math.min(1, rms * 5); // 放大并限制到 0-1

    // 简单的频谱模拟：基于振幅分配口型
    this.updateLipSyncFromAmplitude(amplitude);
  }

  start(): void {
    this.isRunning = true;
  }

  stop(): void {
    this.isRunning = false;
    this.resetMouth();
  }

  /**
   * 每帧更新（从 AnalyserNode 读取频谱）
   */
  tick(): void {
    if (!this.isRunning || !this.analyser || !this.dataArray) return;

    this.analyser.getByteFrequencyData(this.dataArray);

    const bufferLength = this.dataArray.length;

    // 将频谱分为 5 个区间对应 5 种口型
    const segmentSize = Math.floor(bufferLength / 5);
    const bands = [0, 0, 0, 0, 0]; // aa, ih, ou, ee, oh

    for (let i = 0; i < bufferLength; i++) {
      const bandIndex = Math.min(4, Math.floor(i / segmentSize));
      bands[bandIndex] += this.dataArray[i];
    }

    // 归一化
    const maxVal = 255 * segmentSize;
    for (let i = 0; i < 5; i++) {
      bands[i] = Math.min(1, (bands[i] / maxVal) * 3);
    }

    // 映射到口型名
    const shapeNames = ['aa', 'ih', 'ou', 'ee', 'oh'];
    for (let i = 0; i < 5; i++) {
      const target = bands[i];
      const current = this.currentValues[shapeNames[i]];
      const smoothed = current + (target - current) * this.smoothFactor;
      this.currentValues[shapeNames[i]] = smoothed;
      this.renderer.setLipSyncValue(shapeNames[i], smoothed);
    }
  }

  isActive(): boolean {
    return this.isRunning;
  }

  setSmoothFactor(factor: number): void {
    if (factor < 0 || factor > 1) {
      throw new Error('平滑系数必须在 0 到 1 之间');
    }
    this.smoothFactor = factor;
  }

  dispose(): void {
    this.stop();
    if (this.audioContext) {
      this.audioContext.close();
      this.audioContext = null;
    }
    this.analyser = null;
    this.dataArray = null;
  }

  // -- 私有方法 --

  private updateLipSyncFromAmplitude(amplitude: number): void {
    // 基于振幅的简单口型映射
    const aa = amplitude;
    const ih = amplitude * 0.3;
    const ou = amplitude * 0.2;
    const ee = amplitude * 0.15;
    const oh = amplitude * 0.25;

    const shapes = { aa, ih, ou, ee, oh };
    for (const [name, target] of Object.entries(shapes)) {
      const current = this.currentValues[name];
      const smoothed = current + (target - current) * this.smoothFactor;
      this.currentValues[name] = smoothed;
      this.renderer.setLipSyncValue(name, smoothed);
    }
  }

  private resetMouth(): void {
    for (const name of Object.keys(this.currentValues)) {
      this.currentValues[name] = 0;
      this.renderer.setLipSyncValue(name, 0);
    }
  }
}
