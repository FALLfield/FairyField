/**
 * IdleAnimation - VRM 角色待机动画
 *
 * 纯程序化骨骼动画，不需要外部 VRMA 文件。
 * 首先将手臂从 T-Pose 旋转到自然下垂，再通过正弦曲线驱动
 * 呼吸、身体微晃、手臂轻摆等自然待机动作。
 *
 * 关键参数参考 docs/VRM_ANIMATION_GUIDE.md
 */
import * as THREE from 'three';
import type { VRM } from '@pixiv/three-vrm';
import type { VRMRenderer } from '../renderers/VRMRenderer';

interface BoneState {
  bone: THREE.Object3D;
  baseRotation: { x: number; y: number; z: number };
}

type GestureKind = 'nod' | 'lean' | 'shoulder' | 'hand';

interface GestureState {
  kind: GestureKind;
  elapsed: number;
  duration: number;
  strength: number;
  direction: 1 | -1;
}

/** 手臂从 T-Pose 放到自然下垂的角度（~70°） */
const ARM_DOWN_Z = 1.2;
/** 手肘微弯角度 */
const ELBOW_BEND_Z = 0.17;
/** 手臂微前伸角度 */
const ARM_FORWARD_X = -0.1;

export class IdleAnimation {
  private renderer: VRMRenderer;
  private vrm: VRM | null = null;
  private time = 0;
  private running = false;
  private nextGestureAt = 0;
  private gesture: GestureState | null = null;

  // 躯干
  private spine: BoneState | null = null;
  private chest: BoneState | null = null;
  private upperChest: BoneState | null = null;
  private hips: BoneState | null = null;
  private head: BoneState | null = null;

  // 手臂（含前臂）
  private leftUpperArm: BoneState | null = null;
  private rightUpperArm: BoneState | null = null;
  private leftLowerArm: BoneState | null = null;
  private rightLowerArm: BoneState | null = null;

  constructor(renderer: VRMRenderer) {
    this.renderer = renderer;
  }

  /** 初始化：获取骨骼引用并记录 T-Pose 基础旋转 */
  start(): void {
    this.vrm = this.renderer.getVRM();
    if (!this.vrm) return;

    const humanoid = this.vrm.humanoid;

    // 躯干
    this.spine = this.captureBone(humanoid, 'spine');
    this.chest = this.captureBone(humanoid, 'chest');
    this.upperChest = this.captureBone(humanoid, 'upperChest');
    this.hips = this.captureBone(humanoid, 'hips');
    this.head = this.captureBone(humanoid, 'head');

    // 手臂
    this.leftUpperArm = this.captureBone(humanoid, 'leftUpperArm');
    this.rightUpperArm = this.captureBone(humanoid, 'rightUpperArm');
    this.leftLowerArm = this.captureBone(humanoid, 'leftLowerArm');
    this.rightLowerArm = this.captureBone(humanoid, 'rightLowerArm');

    this.running = true;
    this.time = 0;
    this.gesture = null;
    this.nextGestureAt = this.randomRange(1.2, 3.0);
  }

  stop(): void {
    this.running = false;
    this.resetAllBones();
  }

  /** 每帧更新 idle 动画 */
  tick(delta: number): void {
    if (!this.running) return;
    this.time += delta;

    // === 手臂：从 T-Pose 放到自然下垂 ===
    const gesture = this.updateGesture(delta);
    const armIdleSway = Math.sin(this.time * 1.8 + 0.3) * 0.02;
    const armBreathSway = Math.sin(this.time * 1.8) * 0.5 + 0.5;
    const handLift = gesture.kind === 'hand' ? gesture.amount * 0.16 * gesture.direction : 0;
    const shoulderShift = gesture.kind === 'shoulder' ? gesture.amount * 0.04 * gesture.direction : 0;

    // 左臂：正 Z 旋转从 T-Pose 向下（VRM normalized bone 方向）
    this.applyBone(this.leftUpperArm, {
      z: ARM_DOWN_Z + armIdleSway - handLift,
      x: ARM_FORWARD_X + armBreathSway * 0.01,
    });
    // 右臂：负 Z 旋转从 T-Pose 向下
    this.applyBone(this.rightUpperArm, {
      z: -ARM_DOWN_Z - armIdleSway - handLift,
      x: ARM_FORWARD_X + armBreathSway * 0.01,
    });

    // 前臂：微弯，随呼吸轻微变化
    const elbowSway = Math.sin(this.time * 1.8 + 0.5) * 0.01;
    this.applyBone(this.leftLowerArm, {
      z: ELBOW_BEND_Z + elbowSway,
      x: -0.05,
    });
    this.applyBone(this.rightLowerArm, {
      z: -ELBOW_BEND_Z - elbowSway,
      x: -0.05,
    });

    // === 呼吸：胸腹起伏，周期 ~3.5 秒 ===
    const breathe = Math.sin(this.time * 1.8) * 0.5 + 0.5;
    const nod = gesture.kind === 'nod' ? gesture.amount * 0.045 : 0;
    const lean = gesture.kind === 'lean' ? gesture.amount * 0.035 * gesture.direction : 0;
    this.applyBone(this.chest, { x: breathe * 0.02 + nod * 0.25, z: shoulderShift });
    this.applyBone(this.upperChest, { x: breathe * 0.015, y: lean });

    // === 身体微晃：缓慢左右摇摆，周期 ~6 秒 ===
    const sway = Math.sin(this.time * 1.05);
    this.applyBone(this.hips, {
      z: sway * 0.008 - shoulderShift * 0.45,
      y: sway * 0.004 - lean * 0.35,
    });
    this.applyBone(this.spine, { z: -sway * 0.006 + shoulderShift * 0.55 });

    // === 头部微动：缓慢自然偏转 ===
    const headYaw = Math.sin(this.time * 0.7) * 0.015 + lean * 1.3;
    const headPitch = Math.sin(this.time * 0.5 + 1.0) * 0.008 + nod;
    this.applyBone(this.head, { y: headYaw, x: headPitch });
  }

  // -- 私有方法 --

  private captureBone(humanoid: VRM['humanoid'], name: string): BoneState | null {
    const bone = humanoid.getNormalizedBoneNode(name as never);
    if (!bone) return null;
    return {
      bone,
      baseRotation: { x: bone.rotation.x, y: bone.rotation.y, z: bone.rotation.z },
    };
  }

  private applyBone(state: BoneState | null, offset: { x?: number; y?: number; z?: number }): void {
    if (!state) return;
    state.bone.rotation.x = state.baseRotation.x + (offset.x ?? 0);
    state.bone.rotation.y = state.baseRotation.y + (offset.y ?? 0);
    state.bone.rotation.z = state.baseRotation.z + (offset.z ?? 0);
  }

  private updateGesture(delta: number): { kind: GestureKind | null; amount: number; direction: 1 | -1 } {
    if (!this.gesture && this.time >= this.nextGestureAt) {
      const kinds: GestureKind[] = ['nod', 'lean', 'shoulder', 'hand'];
      this.gesture = {
        kind: kinds[Math.floor(Math.random() * kinds.length)],
        elapsed: 0,
        duration: this.randomRange(0.8, 1.7),
        strength: this.randomRange(0.65, 1.0),
        direction: Math.random() > 0.5 ? 1 : -1,
      };
    }

    if (!this.gesture) {
      return { kind: null, amount: 0, direction: 1 };
    }

    this.gesture.elapsed += delta;
    const progress = Math.min(1, this.gesture.elapsed / this.gesture.duration);
    const amount = Math.sin(progress * Math.PI) * this.gesture.strength;
    const kind = this.gesture.kind;
    const direction = this.gesture.direction;

    if (progress >= 1) {
      this.gesture = null;
      this.nextGestureAt = this.time + this.randomRange(2.5, 6.0);
    }

    return { kind, amount, direction };
  }

  private randomRange(min: number, max: number): number {
    return min + Math.random() * (max - min);
  }

  private resetAllBones(): void {
    const bones = [
      this.spine, this.chest, this.upperChest, this.hips,
      this.leftUpperArm, this.rightUpperArm,
      this.leftLowerArm, this.rightLowerArm,
      this.head,
    ];
    for (const b of bones) {
      if (!b) continue;
      b.bone.rotation.x = b.baseRotation.x;
      b.bone.rotation.y = b.baseRotation.y;
      b.bone.rotation.z = b.baseRotation.z;
    }
  }
}
