import { describe, it, expect, beforeEach, vi } from 'vitest';

// Mock Three.js — 使用 class 以便 new 调用
vi.mock('three', () => ({
  Vector2: vi.fn().mockImplementation(function () {
    return { x: 0, y: 0 };
  }),
  Raycaster: vi.fn().mockImplementation(function () {
    return {
      setFromCamera: vi.fn(),
      intersectObjects: vi.fn().mockReturnValue([]),
    };
  }),
}));

// Mock @tauri-apps/api/core
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

describe('HitTestModule', () => {
  let renderer: {
    getVRM: ReturnType<typeof vi.fn>;
    getCamera: ReturnType<typeof vi.fn>;
    getRenderer: ReturnType<typeof vi.fn>;
  };

  beforeEach(async () => {
    renderer = {
      getVRM: vi.fn().mockReturnValue(null),
      getCamera: vi.fn().mockReturnValue({}),
      getRenderer: vi.fn().mockReturnValue({
        domElement: {
          getBoundingClientRect: vi.fn().mockReturnValue({
            left: 0, top: 0, width: 800, height: 600,
          }),
        },
      }),
    };

    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockReset();
  });

  it('VRM 未加载时应返回 false', async () => {
    const { HitTestModule } = await import('./HitTestModule');
    const module = new HitTestModule(renderer as any);

    expect(module.isHit(400, 300)).toBe(false);
  });

  it('VRM 已加载但射线未命中时应返回 false', async () => {
    const { HitTestModule } = await import('./HitTestModule');

    const mockVRM = { scene: { children: [] } };
    renderer.getVRM.mockReturnValue(mockVRM as any);

    const module = new HitTestModule(renderer as any);
    expect(module.isHit(0, 0)).toBe(false);
  });

  it('handleClick 未命中时应设置光标穿透', async () => {
    const { HitTestModule } = await import('./HitTestModule');
    const { invoke } = await import('@tauri-apps/api/core');

    const module = new HitTestModule(renderer as any);
    await module.handleClick(0, 0);

    expect(invoke).toHaveBeenCalledWith('set_ignore_cursor_events', { ignore: true });
  });

  it('getRaycaster 应返回 Raycaster 实例', async () => {
    const { HitTestModule } = await import('./HitTestModule');
    const module = new HitTestModule(renderer as any);

    expect(module.getRaycaster()).toBeDefined();
  });
});
