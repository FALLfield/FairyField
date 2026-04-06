import * as THREE from 'three';
import { GLTFLoader } from 'three/addons/loaders/GLTFLoader.js';
import { VRMLoaderPlugin, VRMUtils } from '@pixiv/three-vrm';
import type { VRM } from '@pixiv/three-vrm';

/**
 * VRMRenderer - Three.js + VRM 渲染器
 *
 * 负责初始化 Three.js 场景、加载 VRM 模型、驱动动画循环。
 * 支持表情控制、注视目标设置、口型同步等外部接口。
 *
 * **Phase 0 核心文件** — 后续 Phase 1+ 的模块（表情、眼神、口型）通过本类
 * 暴露的方法与 VRM 模型交互。
 */
export class VRMRenderer {
  private scene: THREE.Scene;
  private camera: THREE.PerspectiveCamera;
  private renderer: THREE.WebGLRenderer;
  private clock: THREE.Clock;
  private gltfLoader: GLTFLoader;
  private vrm: VRM | null = null;
  private animationFrameId: number | null = null;
  private isDisposed = false;

  /** VRM 加载完成回调 */
  private onVRMLoadedCallback: ((vrm: VRM) => void) | null = null;

  /** 窗口 resize 事件处理（箭头函数，自动绑定 this） */
  private handleResize = (): void => {
    const canvas = this.renderer.domElement;
    this.resize(canvas.clientWidth, canvas.clientHeight);
  };

  constructor(canvas: HTMLCanvasElement) {
    // 场景
    this.scene = new THREE.Scene();

    // 透视相机：45° FOV，适合半身/全身角色
    this.camera = new THREE.PerspectiveCamera(45, 1, 0.1, 20);
    this.camera.position.set(0, 1.3, 1.5);
    this.camera.lookAt(0, 0.9, 0);

    // WebGL 渲染器：透明背景，适配 Tauri 窗口
    this.renderer = new THREE.WebGLRenderer({
      canvas,
      alpha: true,
      premultipliedAlpha: false,
      antialias: true,
    });
    this.renderer.setClearColor(0x000000, 0);
    this.renderer.toneMapping = THREE.ACESFilmicToneMapping;
    this.renderer.toneMappingExposure = 1.0;
    this.renderer.outputColorSpace = THREE.SRGBColorSpace;

    // 时钟：用于计算 delta time
    this.clock = new THREE.Clock();

    // GLTF 加载器 + VRM 插件
    this.gltfLoader = new GLTFLoader();
    this.gltfLoader.register((parser) => new VRMLoaderPlugin(parser));

    // 光照
    this.setupLights();

    // 初始尺寸
    this.resize(canvas.clientWidth, canvas.clientHeight);

    // 启动动画循环
    this.startAnimationLoop();

    // 监听窗口 resize
    window.addEventListener('resize', this.handleResize);
  }

  /**
   * 设置场景光照
   * - 方向光：模拟自然光，从右上方照射
   * - 环境光：填充暗部，保证角色整体可见
   */
  private setupLights(): void {
    const directionalLight = new THREE.DirectionalLight(0xffffff, 1.0);
    directionalLight.position.set(1, 1, 1);
    this.scene.add(directionalLight);

    const ambientLight = new THREE.AmbientLight(0xffffff, 0.6);
    this.scene.add(ambientLight);
  }

  /**
   * 加载 VRM 模型
   * @param url - VRM 文件的 URL（通常在 public/models/ 下）
   */
  async loadVRM(url: string): Promise<void> {
    // 移除已有模型
    if (this.vrm) {
      this.scene.remove(this.vrm.scene);
      this.vrm = null;
    }

    const gltf = await this.gltfLoader.loadAsync(url);
    const loadedVrm = gltf.userData.vrm as VRM | undefined;

    if (!loadedVrm) {
      throw new Error(`加载的文件 ${url} 不包含 VRM 数据`);
    }

    // VRM 标准处理：旋转 Y 轴使角色面朝相机，清理不必要的 morph target
    VRMUtils.removeUnnecessaryVertices(gltf.scene);
    VRMUtils.removeUnnecessaryJoints(gltf.scene);
    loadedVrm.scene.rotation.y = Math.PI;

    // 禁用视锥体裁剪，避免透明窗口下角色被错误裁剪
    loadedVrm.scene.traverse((obj) => {
      obj.frustumCulled = false;
    });

    this.vrm = loadedVrm;
    this.scene.add(this.vrm.scene);

    // 触发回调
    if (this.onVRMLoadedCallback) {
      this.onVRMLoadedCallback(this.vrm);
    }
  }

  /**
   * 注册 VRM 加载完成回调
   */
  onVRMLoaded(callback: (vrm: VRM) => void): void {
    this.onVRMLoadedCallback = callback;
    // 如果 VRM 已加载，立即触发
    if (this.vrm) {
      callback(this.vrm);
    }
  }

  /**
   * 每帧更新 VRM 状态
   * @param delta - 与上一帧的时间差（秒）
   */
  update(delta: number): void {
    if (this.vrm) {
      this.vrm.update(delta);
    }
  }

  /**
   * 设置表情 BlendShape
   * @param name - 表情预设名称（如 'happy', 'sad', 'angry'）
   * @param weight - 权重 0-1，不传则由 VRM 默认处理
   */
  setExpression(name: string, weight?: number): void {
    if (!this.vrm) {
      return;
    }
    if (weight !== undefined) {
      this.vrm.expressionManager?.setValue(name, weight);
    } else {
      this.vrm.expressionManager?.setValue(name, 1.0);
    }
  }

  /**
   * 设置注视目标位置
   * @param position - 世界坐标系中的目标位置
   */
  setLookAtTarget(position: THREE.Vector3): void {
    if (!this.vrm || !this.vrm.lookAt) {
      return;
    }
    // three-vrm 的 lookAt.target 需要 Object3D，创建临时对象指向目标位置
    const targetObj = new THREE.Object3D();
    targetObj.position.copy(position);
    this.vrm.lookAt.target = targetObj;
  }

  /**
   * 设置口型同步值
   * @param value - 嘴巴张开程度 0-1
   */
  setLipSync(value: number): void {
    if (!this.vrm) {
      return;
    }
    // VRM 标准使用 'aa' BlendShape 控制嘴巴张开
    const clampedValue = Math.max(0, Math.min(1, value));
    this.vrm.expressionManager?.setValue('aa', clampedValue);
  }

  /**
   * 获取当前 VRM 实例
   */
  getVRM(): VRM | null {
    return this.vrm;
  }

  /**
   * 获取 Three.js 相机
   */
  getCamera(): THREE.PerspectiveCamera {
    return this.camera;
  }

  /**
   * 获取 Three.js 场景
   */
  getScene(): THREE.Scene {
    return this.scene;
  }

  /**
   * 获取 WebGLRenderer
   */
  getRenderer(): THREE.WebGLRenderer {
    return this.renderer;
  }

  /**
   * 更新渲染尺寸
   * @param width - 画布宽度（CSS 像素）
   * @param height - 画布高度（CSS 像素）
   */
  resize(width: number, height: number): void {
    const w = Math.max(1, width);
    const h = Math.max(1, height);

    this.camera.aspect = w / h;
    this.camera.updateProjectionMatrix();
    this.renderer.setSize(w, h);
    this.renderer.setPixelRatio(window.devicePixelRatio);
  }

  /**
   * 启动动画循环
   */
  private startAnimationLoop(): void {
    const loop = (): void => {
      if (this.isDisposed) {
        return;
      }

      this.animationFrameId = requestAnimationFrame(loop);

      const delta = this.clock.getDelta();
      this.update(delta);
      this.renderer.render(this.scene, this.camera);
    };

    loop();
  }

  /**
   * 释放所有资源，停止动画循环
   */
  dispose(): void {
    if (this.isDisposed) {
      return;
    }

    this.isDisposed = true;

    // 停止动画循环
    if (this.animationFrameId !== null) {
      cancelAnimationFrame(this.animationFrameId);
      this.animationFrameId = null;
    }

    // 移除事件监听
    window.removeEventListener('resize', this.handleResize);

    // 释放 VRM 资源
    if (this.vrm) {
      this.scene.remove(this.vrm.scene);
      this.vrm = null;
    }

    // 释放 Three.js 资源
    this.renderer.dispose();
    this.scene.clear();
  }
}
