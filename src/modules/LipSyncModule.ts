import type { VRMRenderer } from '../renderers/VRMRenderer';
import type { MouthInfluence } from '../lib/emotion-engine';

/**
 * LipSyncModule - 口型同步模块
 *
 * 分析音频信号，将振幅和频率映射到 VRM 口型 BlendShape。
 *
 * 两种输入模式：
 * 1. Web Audio API（AnalyserNode）— 适用于流式音频
 * 2. PCM 数据（Float32Array）— 适用于 TTS 引擎直接喂入
 *
 * 特性：
 * - Attack/Release 包络：平滑过渡，避免嘴型跳动
 * - 频段分析：5 个频段对应 A/I/U/E/O 五个元音
 * - 噪声门限：低于阈值不触发嘴型
 * - 情绪影响：通过 MouthInfluence 调节嘴型幅度和基线
 */

/** 口型 BlendShape 名称 */
type LipShape = 'aa' | 'ih' | 'ou' | 'ee' | 'oh';
const LIP_SHAPES: LipShape[] = ['aa', 'ih', 'ou', 'ee', 'oh'];

/** LipSync 配置 */
export interface LipSyncConfig {
  /** 噪声门限（RMS 低于此值不触发嘴型），默认 0.01 */
  noiseGate: number;
  /** Attack 系数（值越小，嘴张开越平滑），默认 0.4 */
  attack: number;
  /** Release 系数（值越小，嘴闭合越平滑），默认 0.15 */
  release: number;
  /** 振幅增益（放大 RMS 到可视范围），默认 5.0 */
  gain: number;
  /** 最大振幅限制，默认 1.0 */
  maxAmplitude: number;
}

const DEFAULT_CONFIG: LipSyncConfig = {
  noiseGate: 0.01,
  attack: 0.4,
  release: 0.15,
  gain: 5.0,
  maxAmplitude: 1.0,
};

export class LipSyncModule {
  private renderer: VRMRenderer;
  private isRunning = false;
  private analyser: AnalyserNode | null = null;
  private dataArray: Uint8Array | null = null;
  private audioContext: AudioContext | null = null;
  private config: LipSyncConfig;

  /** 当前各口型的平滑值 */
  private currentValues: Record<LipShape, number> = {
    aa: 0, ih: 0, ou: 0, ee: 0, oh: 0,
  };

  /** 当前 RMS 振幅（平滑后） */
  private currentRms = 0;

  /** 情绪对嘴型的影响（由外部设置） */
  private mouthInfluence: MouthInfluence = {
    amplitudeScale: 1.0,
    baselineOpen: 0,
  };

  /** 是否正在说话（RMS 超过噪声门限） */
  private speaking = false;

  constructor(renderer: VRMRenderer, config?: Partial<LipSyncConfig>) {
    this.renderer = renderer;
    this.config = { ...DEFAULT_CONFIG, ...config };
  }

  /**
   * 连接 Web Audio API 音频源
   * @param source - AudioNode（如 MediaElementSource, MediaStreamSource）
   */
  connectAudioSource(source: AudioNode): void {
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
   *
   * 接收 Float32Array PCM 帧，计算 RMS 振幅并进行频段分析，
   * 使用 attack/release 包络平滑过渡。
   *
   * @param pcm - Float32 PCM 数据
   * @param sampleRate - 采样率
   */
  feedPCM(pcm: Float32Array, sampleRate: number): void {
    if (!this.isRunning || pcm.length === 0) return;

    // 计算 RMS 振幅
    const rms = this.computeRMS(pcm);
    const rawAmplitude = Math.min(this.config.maxAmplitude, rms * this.config.gain);

    // 噪声门限
    if (rawAmplitude < this.config.noiseGate) {
      this.speaking = false;
      // 低于门限，release 平滑回到 0
      this.releaseAll();
      return;
    }

    this.speaking = true;

    // 简易频段分析（基于 PCM 自相关特征的启发式分配）
    const vowelWeights = this.analyzeFrequencyBands(pcm, sampleRate);

    // 应用振幅和情绪影响
    const amplitude = rawAmplitude * this.mouthInfluence.amplitudeScale;
    const baseline = this.mouthInfluence.baselineOpen;

    for (const shape of LIP_SHAPES) {
      const target = amplitude * vowelWeights[shape] + baseline * (shape === 'aa' ? 1 : 0);
      this.applyEnvelope(shape, target);
    }
  }

  start(): void {
    this.isRunning = true;
  }

  stop(): void {
    this.isRunning = false;
    this.speaking = false;
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
    const normalized = bands.map((b) => Math.min(1, (b / maxVal) * 3));

    // 噪声门限检测
    const totalEnergy = normalized.reduce((s, v) => s + v, 0);
    if (totalEnergy < this.config.noiseGate * 5) {
      this.speaking = false;
      this.releaseAll();
      return;
    }

    this.speaking = true;

    // 应用情绪影响
    const scale = this.mouthInfluence.amplitudeScale;
    const baseline = this.mouthInfluence.baselineOpen;

    for (let i = 0; i < LIP_SHAPES.length; i++) {
      const shape = LIP_SHAPES[i];
      const target = normalized[i] * scale + baseline * (shape === 'aa' ? 1 : 0);
      this.applyEnvelope(shape, target);
    }
  }

  /** 设置情绪对嘴型的影响 */
  setMouthInfluence(influence: MouthInfluence): void {
    this.mouthInfluence = { ...influence };
  }

  /**
   * 模拟口型（TTS 无 PCM 数据时使用）
   *
   * 由外部（如 TTS 事件）驱动，直接设置口型值。
   * 用于 MacSayTts 等不提供 PCM 数据的 TTS 引擎。
   *
   * @param open - 嘴巴张开程度 (0-1)
   * @param width - 嘴巴宽度 (0-1)
   */
  setSimulatedMouth(open: number, width: number): void {
    if (!this.isRunning) return;

    this.speaking = open > 0.05;

    // 将 open/width 映射到 5 个元音口型
    const aa = open * 0.6;
    const oh = open * 0.3;
    const ee = width * 0.4;
    const ih = width * 0.2;
    const ou = Math.max(0, open * 0.15 - width * 0.1);

    const scale = this.mouthInfluence.amplitudeScale;
    const baseline = this.mouthInfluence.baselineOpen;

    for (const shape of LIP_SHAPES) {
      const raw = { aa, ih, ou, ee, oh }[shape];
      const target = raw * scale + baseline * (shape === 'aa' ? 1 : 0);
      this.applyEnvelope(shape, target);
    }
  }

  /** 更新配置 */
  updateConfig(patch: Partial<LipSyncConfig>): void {
    this.config = { ...this.config, ...patch };
  }

  /** 获取当前配置 */
  getConfig(): Readonly<LipSyncConfig> {
    return { ...this.config };
  }

  isActive(): boolean {
    return this.isRunning;
  }

  /** 是否正在说话 */
  isSpeaking(): boolean {
    return this.speaking;
  }

  /** 获取当前 RMS 振幅 */
  getCurrentRms(): number {
    return this.currentRms;
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

  /** 计算 RMS 振幅 */
  private computeRMS(pcm: Float32Array): number {
    let sum = 0;
    for (let i = 0; i < pcm.length; i++) {
      sum += pcm[i] * pcm[i];
    }
    return Math.sqrt(sum / pcm.length);
  }

  /**
   * 基于频率特征的启发式元音分析
   *
   * 使用零交叉率（zero-crossing rate）区分元音倾向：
   * - 低频主导 → aa/oh（开口音）
   * - 中频主导 → ee/ih（前元音）
   * - 高频主导 → ou（圆唇音）
   *
   * 这是简化版本，不需要完整的 FFT。
   */
  private analyzeFrequencyBands(pcm: Float32Array, _sampleRate: number): Record<LipShape, number> {
    // 计算零交叉率（粗略频率估计）
    let zeroCrossings = 0;
    for (let i = 1; i < pcm.length; i++) {
      if ((pcm[i] >= 0) !== (pcm[i - 1] >= 0)) {
        zeroCrossings++;
      }
    }
    const zcr = zeroCrossings / pcm.length;

    // 基于零交叉率的启发式元音权重分配
    // zcr 低 → 低频 → aa/oh 主导
    // zcr 中 → 中频 → ee/ih 主导
    // zcr 高 → 高频 → ou 主导
    let aa: number, ih: number, ou: number, ee: number, oh: number;

    if (zcr < 0.05) {
      // 非常低频 — 大开口
      aa = 1.0; ih = 0.1; ou = 0.05; ee = 0.05; oh = 0.15;
    } else if (zcr < 0.1) {
      // 低频 — 开口音
      const t = (zcr - 0.05) / 0.05;
      aa = 1.0 - t * 0.3;
      oh = 0.15 + t * 0.4;
      ih = 0.1 + t * 0.1;
      ou = 0.05 + t * 0.1;
      ee = 0.05 + t * 0.1;
    } else if (zcr < 0.2) {
      // 中频 — 前元音
      const t = (zcr - 0.1) / 0.1;
      aa = 0.7 - t * 0.3;
      ee = 0.15 + t * 0.5;
      ih = 0.2 + t * 0.3;
      ou = 0.15 + t * 0.1;
      oh = 0.55 - t * 0.2;
    } else {
      // 高频 — 圆唇/闭合音
      const t = Math.min(1, (zcr - 0.2) / 0.15);
      ou = 0.25 + t * 0.4;
      ee = 0.65 - t * 0.3;
      ih = 0.5 - t * 0.2;
      aa = 0.4 - t * 0.25;
      oh = 0.35 - t * 0.15;
    }

    // 归一化使总和为 1
    const total = aa + ih + ou + ee + oh;
    return {
      aa: aa / total,
      ih: ih / total,
      ou: ou / total,
      ee: ee / total,
      oh: oh / total,
    };
  }

  /**
   * 应用 attack/release 包络
   * - 上升时（target > current）使用 attack 系数（快速响应）
   * - 下降时（target < current）使用 release 系数（缓慢释放）
   */
  private applyEnvelope(shape: LipShape, target: number): void {
    const current = this.currentValues[shape];
    const diff = target - current;

    let smoothed: number;
    if (diff > 0) {
      // Attack：嘴张开时快速响应
      smoothed = current + diff * this.config.attack;
    } else {
      // Release：嘴闭合时缓慢释放
      smoothed = current + diff * this.config.release;
    }

    const clamped = Math.max(0, Math.min(1, smoothed));
    this.currentValues[shape] = clamped;
    this.renderer.setLipSyncValue(shape, clamped);
  }

  /** 所有口型 release 回零 */
  private releaseAll(): void {
    for (const shape of LIP_SHAPES) {
      this.applyEnvelope(shape, 0);
    }
  }

  /** 重置嘴型 */
  private resetMouth(): void {
    for (const name of LIP_SHAPES) {
      this.currentValues[name] = 0;
      this.renderer.setLipSyncValue(name, 0);
    }
    this.currentRms = 0;
  }
}
