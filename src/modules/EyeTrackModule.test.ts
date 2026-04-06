import { describe, it, expect, beforeEach, vi } from 'vitest';

// 创建一个支持链式调用的 Vector3 mock
function createMockVector3(x = 0, y = 0, z = 0) {
  const vec: any = { x, y, z };
  vec.set = vi.fn(function (nx: number, ny: number, nz: number) {
    vec.x = nx; vec.y = ny; vec.z = nz;
    return vec;
  });
  vec.clone = vi.fn(function () {
    return createMockVector3(vec.x, vec.y, vec.z);
  });
  vec.lerp = vi.fn(function (target: any, alpha: number) {
    vec.x += (target.x - vec.x) * alpha;
    vec.y += (target.y - vec.y) * alpha;
    vec.z += (target.z - vec.z) * alpha;
    return vec;
  });
  vec.copy = vi.fn(function (source: any) {
    vec.x = source.x; vec.y = source.y; vec.z = source.z;
    return vec;
  });
  return vec;
}

vi.mock('three', () => ({
  Vector3: vi.fn(function (x = 0, y = 0, z = 0) {
    return createMockVector3(x, y, z);
  }),
}));

// Mock VRMRenderer
const mockSetLookAtTarget = vi.fn();
vi.mock('../renderers/VRMRenderer', () => ({
  VRMRenderer: vi.fn(function (_canvas: any) {
    return { setLookAtTarget: mockSetLookAtTarget };
  }),
}));

describe('EyeTrackModule', () => {
  let canvas: HTMLCanvasElement;

  beforeEach(async () => {
    canvas = document.createElement('canvas');
    canvas.width = 800;
    canvas.height = 600;
    document.body.appendChild(canvas);
    vi.clearAllMocks();
  });

  it('构造函数应设置默认注视位置', async () => {
    const { EyeTrackModule } = await import('./EyeTrackModule');
    const { VRMRenderer } = await import('../renderers/VRMRenderer');
    const renderer = new VRMRenderer(canvas);
    const module = new EyeTrackModule(renderer as any);

    const pos = module.getCurrentPosition();
    expect(pos.x).toBe(0);
    expect(pos.y).toBe(0.9);
    expect(pos.z).toBe(0.5);
  });

  it('start/stop 应正确管理运行状态', async () => {
    const { EyeTrackModule } = await import('./EyeTrackModule');
    const { VRMRenderer } = await import('../renderers/VRMRenderer');
    const renderer = new VRMRenderer(canvas);
    const module = new EyeTrackModule(renderer as any);

    expect(module.isActive()).toBe(false);

    module.start();
    expect(module.isActive()).toBe(true);

    module.stop();
    expect(module.isActive()).toBe(false);
  });

  it('重复调用 start 不应产生副作用', async () => {
    const { EyeTrackModule } = await import('./EyeTrackModule');
    const { VRMRenderer } = await import('../renderers/VRMRenderer');
    const renderer = new VRMRenderer(canvas);
    const module = new EyeTrackModule(renderer as any);

    module.start();
    module.start();

    expect(module.isActive()).toBe(true);
    module.stop();
  });

  it('鼠标移动应更新目标位置', async () => {
    const { EyeTrackModule } = await import('./EyeTrackModule');
    const { VRMRenderer } = await import('../renderers/VRMRenderer');
    const renderer = new VRMRenderer(canvas);

    Object.defineProperty(window, 'innerWidth', { value: 1920, configurable: true });
    Object.defineProperty(window, 'innerHeight', { value: 1080, configurable: true });

    const module = new EyeTrackModule(renderer as any);
    module.start();

    const event = new MouseEvent('mousemove', { clientX: 1920, clientY: 0 });
    window.dispatchEvent(event);

    const target = module.getTargetPosition();
    expect(target.x).toBe(0.5);
    expect(target.y).toBe(1.2);
    expect(target.z).toBe(0.5);

    module.stop();

    Object.defineProperty(window, 'innerWidth', { value: 1024, configurable: true });
    Object.defineProperty(window, 'innerHeight', { value: 768, configurable: true });
  });

  it('鼠标移动到左下角应产生负值偏移', async () => {
    const { EyeTrackModule } = await import('./EyeTrackModule');
    const { VRMRenderer } = await import('../renderers/VRMRenderer');
    const renderer = new VRMRenderer(canvas);

    Object.defineProperty(window, 'innerWidth', { value: 1920, configurable: true });
    Object.defineProperty(window, 'innerHeight', { value: 1080, configurable: true });

    const module = new EyeTrackModule(renderer as any);
    module.start();

    const event = new MouseEvent('mousemove', { clientX: 0, clientY: 1080 });
    window.dispatchEvent(event);

    const target = module.getTargetPosition();
    expect(target.x).toBe(-0.5);
    expect(target.y).toBeCloseTo(0.6);
    expect(target.z).toBe(0.5);

    module.stop();

    Object.defineProperty(window, 'innerWidth', { value: 1024, configurable: true });
    Object.defineProperty(window, 'innerHeight', { value: 768, configurable: true });
  });

  it('setSmoothFactor 应验证参数范围', async () => {
    const { EyeTrackModule } = await import('./EyeTrackModule');
    const { VRMRenderer } = await import('../renderers/VRMRenderer');
    const renderer = new VRMRenderer(canvas);
    const module = new EyeTrackModule(renderer as any);

    module.setSmoothFactor(0.5);
    expect(module.getSmoothFactor()).toBe(0.5);

    expect(() => module.setSmoothFactor(-0.1)).toThrow('平滑系数必须在 0 到 1 之间');
    expect(() => module.setSmoothFactor(1.1)).toThrow('平滑系数必须在 0 到 1 之间');
  });

  it('stop 后应重置注视目标', async () => {
    const { EyeTrackModule } = await import('./EyeTrackModule');
    const { VRMRenderer } = await import('../renderers/VRMRenderer');
    const renderer = new VRMRenderer(canvas);
    const module = new EyeTrackModule(renderer as any);

    module.start();
    module.stop();

    expect(mockSetLookAtTarget).toHaveBeenCalled();
  });

  it('getCurrentPosition 应返回独立副本', async () => {
    const { EyeTrackModule } = await import('./EyeTrackModule');
    const { VRMRenderer } = await import('../renderers/VRMRenderer');
    const renderer = new VRMRenderer(canvas);
    const module = new EyeTrackModule(renderer as any);

    const pos1 = module.getCurrentPosition();
    const pos2 = module.getCurrentPosition();

    expect({ x: pos1.x, y: pos1.y, z: pos1.z }).toEqual({ x: pos2.x, y: pos2.y, z: pos2.z });
    expect(pos1).not.toBe(pos2);
  });
});
