import { describe, it, expect, vi, afterEach } from 'vitest';
import { IdleAnimation } from './IdleAnimation';

function createBone() {
  return {
    rotation: { x: 0, y: 0, z: 0 },
  };
}

function createMockRenderer() {
  const bones: Record<string, ReturnType<typeof createBone>> = {
    spine: createBone(),
    chest: createBone(),
    upperChest: createBone(),
    hips: createBone(),
    head: createBone(),
    leftUpperArm: createBone(),
    rightUpperArm: createBone(),
    leftLowerArm: createBone(),
    rightLowerArm: createBone(),
  };

  return {
    bones,
    renderer: {
      getVRM: () => ({
        humanoid: {
          getNormalizedBoneNode: (name: string) => bones[name] ?? null,
        },
      }),
    },
  };
}

describe('IdleAnimation', () => {
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('start 后 tick 会驱动基础待机骨骼', () => {
    const { renderer, bones } = createMockRenderer();
    const idle = new IdleAnimation(renderer as any);

    idle.start();
    idle.tick(0.5);

    expect(bones.leftUpperArm.rotation.z).toBeGreaterThan(1.0);
    expect(bones.rightUpperArm.rotation.z).toBeLessThan(-1.0);
    expect(bones.chest.rotation.x).toBeGreaterThan(0);
  });

  it('会周期性触发随机身体小动作', () => {
    const randomValues = [0, 0, 0, 1, 1];
    vi.spyOn(Math, 'random').mockImplementation(() => randomValues.shift() ?? 0.5);

    const { renderer, bones } = createMockRenderer();
    const idle = new IdleAnimation(renderer as any);

    idle.start();
    for (let i = 0; i < 14; i++) {
      idle.tick(0.1);
    }

    expect(bones.head.rotation.x).toBeGreaterThan(0.03);
    expect(bones.chest.rotation.x).toBeGreaterThan(0.02);
  });

  it('stop 会把骨骼恢复到初始旋转', () => {
    const { renderer, bones } = createMockRenderer();
    const idle = new IdleAnimation(renderer as any);

    idle.start();
    idle.tick(0.5);
    idle.stop();

    expect(bones.leftUpperArm.rotation.z).toBe(0);
    expect(bones.rightUpperArm.rotation.z).toBe(0);
    expect(bones.chest.rotation.x).toBe(0);
  });
});
