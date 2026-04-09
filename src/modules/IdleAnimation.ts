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
    const armIdleSway = Math.sin(this.time * 1.8 + 0.3) * 0.02;
    const armBreathSway = Math.sin(this.time * 1.8) * 0.5 + 0.5;

    // 左臂：正 Z 旋转从 T-Pose 向下（VRM normalized bone 方向）
    this.applyBone(this.leftUpperArm, {
      z: ARM_DOWN_Z + armIdleSway,
      x: ARM_FORWARD_X + armBreathSway * 0.01,
    });
    // 右臂：负 Z 旋转从 T-Pose 向下
    this.applyBone(this.rightUpperArm, {
      z: -ARM_DOWN_Z - armIdleSway,
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
    this.applyBone(this.chest, { x: breathe * 0.02 });
    this.applyBone(this.upperChest, { x: breathe * 0.015 });

    // === 身体微晃：缓慢左右摇摆，周期 ~6 秒 ===
    const sway = Math.sin(this.time * 1.05);
    this.applyBone(this.hips, { z: sway * 0.008, y: sway * 0.004 });
    this.applyBone(this.spine, { z: -sway * 0.006 });

    // === 头部微动：缓慢自然偏转 ===
    const headYaw = Math.sin(this.time * 0.7) * 0.015;
    const headPitch = Math.sin(this.time * 0.5 + 1.0) * 0.008;
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
