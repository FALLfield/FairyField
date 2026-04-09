import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';

// Mock Three.js — 返回 class 而非箭头函数
vi.mock('three', () => ({
  Scene: vi.fn().mockImplementation(function () {
    return { children: [], add: vi.fn(), remove: vi.fn(), clear: vi.fn(), traverse: vi.fn() };
  }),
  PerspectiveCamera: vi.fn().mockImplementation(function () {
    return { position: { set: vi.fn() }, lookAt: vi.fn(), aspect: 1, updateProjectionMatrix: vi.fn() };
  }),
  WebGLRenderer: vi.fn().mockImplementation(function () {
    return {
      domElement: { clientWidth: 800, clientHeight: 600 },
      setClearColor: vi.fn(),
      setSize: vi.fn(),
      setPixelRatio: vi.fn(),
      dispose: vi.fn(),
      render: vi.fn(),
      toneMapping: 0,
      toneMappingExposure: 1,
      outputColorSpace: '',
    };
  }),
  Clock: vi.fn().mockImplementation(function () {
    return { getDelta: vi.fn(() => 0.016) };
  }),
  Vector3: vi.fn().mockImplementation(function (x = 0, y = 0, z = 0) {
    return { x, y, z };
  }),
  DirectionalLight: vi.fn().mockImplementation(function () {
    return { position: { set: vi.fn() } };
  }),
  AmbientLight: vi.fn().mockImplementation(function () {}),
  ACESFilmicToneMapping: 4,
  SRGBColorSpace: 'srgb',
  Box3: vi.fn().mockImplementation(function () {
    return {
      setFromObject: vi.fn().mockReturnThis(),
      getSize: vi.fn(() => ({ x: 1, y: 1.5, z: 1 })),
      getCenter: vi.fn(() => ({ x: 0, y: 0.8, z: 0 })),
    };
  }),
}));

// Mock GLTFLoader
vi.mock('three/addons/loaders/GLTFLoader.js', () => ({
  GLTFLoader: vi.fn().mockImplementation(function () {
    return { register: vi.fn(), loadAsync: vi.fn() };
  }),
}));

// Mock @pixiv/three-vrm
vi.mock('@pixiv/three-vrm', () => ({
  VRMLoaderPlugin: vi.fn().mockImplementation(function () {}),
  VRMUtils: {
    removeUnnecessaryVertices: vi.fn(),
    removeUnnecessaryJoints: vi.fn(),
  },
}));

describe('VRMRenderer', () => {
  let canvas: HTMLCanvasElement;

  beforeEach(() => {
    canvas = document.createElement('canvas');
    canvas.width = 800;
    canvas.height = 600;
    document.body.appendChild(canvas);
    vi.clearAllMocks();

    // jsdom 不提供 matchMedia，手动 mock
    Object.defineProperty(window, 'matchMedia', {
      writable: true,
      value: vi.fn().mockImplementation((query: string) => ({
        matches: false,
        media: query,
        onchange: null,
        addListener: vi.fn(),
        removeListener: vi.fn(),
        addEventListener: vi.fn(),
        removeEventListener: vi.fn(),
        dispatchEvent: vi.fn(),
      })),
    });
    Object.defineProperty(window, 'devicePixelRatio', {
      writable: true,
      value: 1,
    });
  });

  afterEach(() => {
    document.body.removeChild(canvas);
  });

  it('构造函数应创建 Three.js 场景和渲染器', async () => {
    const { VRMRenderer } = await import('./VRMRenderer');
    const THREE = await import('three');
    const renderer = new VRMRenderer(canvas);

    expect(THREE.Scene).toHaveBeenCalled();
    expect(THREE.PerspectiveCamera).toHaveBeenCalled();
    expect(THREE.WebGLRenderer).toHaveBeenCalled();
    expect(renderer.getVRM()).toBeNull();

    renderer.dispose();
  });

  it('构造函数应设置相机参数', async () => {
    const { VRMRenderer } = await import('./VRMRenderer');
    const THREE = await import('three');
    const renderer = new VRMRenderer(canvas);

    const cameraInstance = vi.mocked(THREE.PerspectiveCamera).mock.results[0].value;
    expect(cameraInstance.position.set).toHaveBeenCalledWith(0, 1.3, 1.5);
    expect(cameraInstance.lookAt).toHaveBeenCalledWith(0, 0.9, 0);

    renderer.dispose();
  });

  it('构造函数应初始化渲染器为透明模式', async () => {
    const { VRMRenderer } = await import('./VRMRenderer');
    const THREE = await import('three');
    const renderer = new VRMRenderer(canvas);

    const rendererInstance = vi.mocked(THREE.WebGLRenderer).mock.results[0].value;
    expect(rendererInstance.setClearColor).toHaveBeenCalledWith(0x000000, 0);

    renderer.dispose();
  });

  it('loadVRM 应加载 VRM 模型并添加到场景', async () => {
    const { VRMRenderer } = await import('./VRMRenderer');
    const { VRMUtils } = await import('@pixiv/three-vrm');

    const mockScene = { rotation: { y: 0 }, traverse: vi.fn() };
    const mockVRM = {
      scene: mockScene,
      expressionManager: { setValue: vi.fn() },
      lookAt: {},
      update: vi.fn(),
    };

    const { GLTFLoader } = await import('three/addons/loaders/GLTFLoader.js');
    (GLTFLoader as any).mockImplementation(function () {
      return {
        register: vi.fn(),
        loadAsync: vi.fn().mockResolvedValue({
          userData: { vrm: mockVRM },
          scene: mockScene,
        }),
      };
    });

    const renderer = new VRMRenderer(canvas);
    await renderer.loadVRM('/models/test.vrm');

    expect(VRMUtils.removeUnnecessaryVertices).toHaveBeenCalled();
    expect(VRMUtils.removeUnnecessaryJoints).toHaveBeenCalled();
    expect(mockScene.rotation.y).toBe(Math.PI);
    expect(renderer.getVRM()).toBe(mockVRM);

    renderer.dispose();
  });

  it('loadVRM 应在文件不包含 VRM 数据时抛出错误', async () => {
    const { VRMRenderer } = await import('./VRMRenderer');
    const { GLTFLoader } = await import('three/addons/loaders/GLTFLoader.js');

    (GLTFLoader as any).mockImplementation(function () {
      return {
        register: vi.fn(),
        loadAsync: vi.fn().mockResolvedValue({
          userData: {},
          scene: {},
        }),
      };
    });

    const renderer = new VRMRenderer(canvas);

    await expect(renderer.loadVRM('/models/invalid.vrm')).rejects.toThrow(
      '不包含 VRM 数据',
    );

    renderer.dispose();
  });

  it('setExpression 应调用 VRM 表情管理器', async () => {
    const { VRMRenderer } = await import('./VRMRenderer');

    const mockScene = { rotation: { y: 0 }, traverse: vi.fn() };
    const mockExpressionManager = { setValue: vi.fn() };
    const mockVRM = {
      scene: mockScene,
      expressionManager: mockExpressionManager,
      lookAt: {},
      update: vi.fn(),
    };

    const { GLTFLoader } = await import('three/addons/loaders/GLTFLoader.js');
    (GLTFLoader as any).mockImplementation(function () {
      return {
        register: vi.fn(),
        loadAsync: vi.fn().mockResolvedValue({
          userData: { vrm: mockVRM },
          scene: mockScene,
        }),
      };
    });

    const renderer = new VRMRenderer(canvas);
    await renderer.loadVRM('/models/test.vrm');

    renderer.setExpression('happy');
    expect(mockExpressionManager.setValue).toHaveBeenCalledWith('happy', 1.0);

    renderer.setExpression('sad', 0.5);
    expect(mockExpressionManager.setValue).toHaveBeenCalledWith('sad', 0.5);

    renderer.dispose();
  });

  it('setLipSync 应限制值在 0-1 范围', async () => {
    const { VRMRenderer } = await import('./VRMRenderer');

    const mockScene = { rotation: { y: 0 }, traverse: vi.fn() };
    const mockExpressionManager = { setValue: vi.fn() };
    const mockVRM = {
      scene: mockScene,
      expressionManager: mockExpressionManager,
      lookAt: {},
      update: vi.fn(),
    };

    const { GLTFLoader } = await import('three/addons/loaders/GLTFLoader.js');
    (GLTFLoader as any).mockImplementation(function () {
      return {
        register: vi.fn(),
        loadAsync: vi.fn().mockResolvedValue({
          userData: { vrm: mockVRM },
          scene: mockScene,
        }),
      };
    });

    const renderer = new VRMRenderer(canvas);
    await renderer.loadVRM('/models/test.vrm');

    renderer.setLipSync(1.5);
    expect(mockExpressionManager.setValue).toHaveBeenCalledWith('aa', 1.0);

    renderer.setLipSync(-0.5);
    expect(mockExpressionManager.setValue).toHaveBeenCalledWith('aa', 0.0);

    renderer.dispose();
  });

  it('resize 应更新相机和渲染器', async () => {
    const { VRMRenderer } = await import('./VRMRenderer');
    const THREE = await import('three');
    const renderer = new VRMRenderer(canvas);

    renderer.resize(1024, 768);

    const cameraInstance = vi.mocked(THREE.PerspectiveCamera).mock.results[0].value;
    expect(cameraInstance.aspect).toBe(1024 / 768);
    expect(cameraInstance.updateProjectionMatrix).toHaveBeenCalled();

    const rendererInstance = vi.mocked(THREE.WebGLRenderer).mock.results[0].value;
    expect(rendererInstance.setSize).toHaveBeenCalledWith(1024, 768);

    renderer.dispose();
  });

  it('dispose 应正确释放资源', async () => {
    const { VRMRenderer } = await import('./VRMRenderer');
    const THREE = await import('three');
    const renderer = new VRMRenderer(canvas);

    renderer.dispose();

    const rendererInstance = vi.mocked(THREE.WebGLRenderer).mock.results[0].value;
    expect(rendererInstance.dispose).toHaveBeenCalled();
    expect(renderer.getVRM()).toBeNull();
  });

  it('多次调用 dispose 不应抛出错误', async () => {
    const { VRMRenderer } = await import('./VRMRenderer');
    const renderer = new VRMRenderer(canvas);

    expect(() => {
      renderer.dispose();
      renderer.dispose();
    }).not.toThrow();
  });
});
