# VRM 程序化动画开发指南

> FairyField 动画开发参考。每次修改 IdleAnimation 或新增动画时必读。

## 核心概念

### VRM T-Pose 问题

VRM 模型默认加载为 **T-Pose**（双臂水平伸展）。所有程序化动画的第一步是**将手臂从 T-Pose 旋转到自然下垂姿势**。

### VRM 骨骼 Z 轴镜像规则

VRM normalized bones 的左右手臂 Z 轴旋转方向**相反**：

```typescript
// 左臂：正 Z 旋转 = 向下（从 T-Pose 放下）
leftUpperArm.rotation.z = +armDownAngle

// 右臂：负 Z 旋转 = 向下（从 T-Pose 放下）
rightUpperArm.rotation.z = -armDownAngle
```

> **注意**：部分参考资料（如 posers 项目）使用相反的符号。
> 经实测，FairyField 使用的 VRM 模型 normalized bones 方向为左正右负。
> 如果更换模型后手臂方向不对，先尝试翻转 Z 轴符号。

### 自然站姿参数（来自 Animaze + posers 项目实测）

| 骨骼 | 旋转轴 | 角度 | 弧度 | 说明 |
|------|--------|------|------|------|
| leftUpperArm | Z | ~70° | ~1.2 | 从 T-Pose 放下到体侧 |
| rightUpperArm | Z | ~70° | ~1.2 | 同上，方向相反 |
| leftLowerArm | Z | ~10° | ~0.17 | 手肘微弯 |
| rightLowerArm | Z | ~10° | ~0.17 | 同上 |
| leftUpperArm | X | ~-5° | ~-0.09 | 手臂微前伸 |
| rightUpperArm | X | ~-5° | ~-0.09 | 同上 |

### 呼吸动画参数（Animaze 实测）

- 影响骨骼：`chest`, `upperChest`
- 动画类型：uniform scale（不是 rotation）
- 范围：`1.0` 到 `1.025`
- 周期：约 3-5 秒

## VRM Humanoid 骨骼层级

```
hips
├── spine
│   └── chest
│       └── upperChest
│           ├── neck
│           │   └── head
│           ├── leftShoulder
│           │   └── leftUpperArm
│           │       └── leftLowerArm
│           │           └── leftHand
│           └── rightShoulder
│               └── rightUpperArm
│                   └── rightLowerArm
│                       └── rightHand
├── leftUpperLeg
│   └── leftLowerLeg
│       └── leftFoot
│           └── leftToes
└── rightUpperLeg
    └── rightLowerLeg
        └── rightFoot
            └── rightToes
```

## 动画开发清单

### Idle 动画（最低标准）

- [ ] 手臂从 T-Pose 放下（leftUpperArm Z=+1.2, rightUpperArm Z=-1.2）
- [ ] 手肘微弯（leftLowerArm Z=+0.17, rightLowerArm Z=-0.17）
- [ ] 呼吸起伏（chest/upperChest scale 1.0-1.025, 周期 3.5s）
- [ ] 身体微晃（hips/spine 微小 Z 轴摆动）
- [ ] 头部微动（head 缓慢 Y/X 偏转）

### 动画幅度参考

| 动作类型 | 幅度范围 | 说明 |
|---------|---------|------|
| 呼吸 | scale 0.01-0.025 | 几乎不可见的微动 |
| 身体晃动 | rotation 0.005-0.02 rad | 极轻微的摆动 |
| 头部微动 | rotation 0.01-0.03 rad | 缓慢自然偏转 |
| 手臂摆动 | rotation 0.02-0.05 rad | 随呼吸的小幅摆动 |
| 情绪动画 | rotation 0.1-0.5 rad | 明显的姿态变化 |

## three-vrm API 使用要点

### 获取骨骼

```typescript
// 推荐使用 normalized bones（自动同步到 raw bones）
const bone = vrm.humanoid.getNormalizedBoneNode('leftUpperArm');
```

### 修改骨骼旋转

```typescript
// 直接修改 rotation（会在 vrm.update() 时同步到 raw bones）
bone.rotation.x = baseX + offsetX;
bone.rotation.y = baseY + offsetY;
bone.rotation.z = baseZ + offsetZ;
```

### VRM.update() 时序

```typescript
// 正确顺序：
// 1. 修改 normalized bone rotations
// 2. 调用 vrm.update(delta)
// vrm.update() 会自动将 normalized bone poses 同步到 raw bones
```

### 重置姿势

```typescript
// 重置到 T-Pose
vrm.humanoid.resetNormalizedPose();

// 获取当前 pose（相对于 rest pose）
const pose = vrm.humanoid.getNormalizedPose();

// 设置 pose
vrm.humanoid.setNormalizedPose(pose);
```

## 已知问题

1. **T-Pose 闪烁**：调用 `resetNormalizedPose()` 后切换动画会有短暂 T-Pose 闪烁。解决：在 reset 后立即设置目标 pose。
2. **Hip 下沉**：Mixamo FBX 动画的 hip position track 会导致累积偏移。解决：每次切换动画时创建新 AnimationMixer。
3. **Normalized bone scale 不同步**：three-vrm #1585 — scale 变更不会从 normalized 同步到 raw。解决：直接操作 raw bones 的 scale。
4. **手臂旋转方向不一致**：不同 VRM 模型、不同骨骼类型（normalized vs raw）的手臂 Z 轴旋转方向可能相反。见下方「踩坑记录」。

## 踩坑记录

### 1. 手臂 Z 轴旋转方向搞反（2026-04-10）

**现象**：应用 idle 动画后，手臂穿入身体/扭到背后，看起来非常抽象。

**原因**：VRM 骨骼的 Z 轴旋转方向在不同参考资料中相互矛盾：
- **posers 项目** 说：左臂 Z=-1.2（负），右臂 Z=+1.2（正）
- **three-vrm discussion #1343** 说：左臂 Z=+1.3（正），右臂 Z=-1.3（负）
- **实测 FairyField 模型**：左正右负才是正确的

**根因分析**：
- posers 使用 `quatFromAxisAngle` 设置四元数，我们使用 euler angles 直接设 rotation.z
- 不同 VRM 模型（VRoid vs Booth）的骨骼局部坐标系可能不同
- VRMRenderer 将场景旋转了 180°（`scene.rotation.y = Math.PI`），但不影响骨骼局部旋转

**解决方案**：
1. 先用小角度（0.3 rad）测试方向，确认手臂向下而不是向上/穿模
2. 确认方向后再调到目标角度（1.2 rad）
3. 更换模型后如果方向不对，翻转 Z 轴符号

**调试方法**：
```
// 在浏览器 Console 中快速测试：
const vrm = document.querySelector('canvas').__vrm; // 取决于暴露方式
const leftArm = vrm.humanoid.getNormalizedBoneNode('leftUpperArm');
leftArm.rotation.z = 0.3; // 看手臂是往下还是往上
```

### 2. 旋转幅度过小看不出效果（2026-04-10）

**现象**：idle 动画接入了但角色看起来完全没有动。

**原因**：第一版手臂偏移只有 `z: -0.05`（~3°），肉眼几乎不可见。
需要从 T-Pose 放下手臂的总旋转量是 ~1.2 rad（70°），而不是在 T-Pose 基础上加 0.05。

**教训**：
- 程序化动画的 offset 应包含「从 T-Pose 到目标姿势」的完整旋转量
- 调试时先设大角度确认方向，再微调到自然幅度
- 看不到效果时，先排除方向和幅度问题

### 3. 缺少 THREE 命名空间 import

**现象**：`vue-tsc --noEmit` 报错 `Cannot find namespace 'THREE'`。

**原因**：`IdleAnimation.ts` 中用了 `THREE.Object3D` 类型但没有 `import * as THREE from 'three'`。
模块只 import 了 `type { VRM }` 和 `type { VRMRenderer }`，遗漏了 THREE。

**教训**：新建文件时确保所有用到的类型都有对应的 import。TypeScript 的 `type` import 不会引入运行时依赖，但 `THREE.Object3D` 作为 interface field 的类型需要完整的模块 import。

## 外部动画方案

### Mixamo FBX → VRM

使用 three-vrm 提供的 `loadMixamoAnimation.js` 工具将 Mixamo FBX 动画重定向到 VRM：

```typescript
import { FBXLoader } from 'three/addons/loaders/FBXLoader.js';

// 需要手动实现 bone mapping（mixamo → VRM humanoid names）
const mixamoVRMRigMap = {
  mixamorigHips: 'hips',
  mixamorigSpine: 'spine',
  mixamorigSpine1: 'chest',
  mixamorigSpine2: 'upperChest',
  // ... 完整映射见 three-vrm 示例
};
```

### VRMA 格式

VRM Animation (.vrma) 是 VRM 原生动画格式，可直接使用 `@pixiv/three-vrm-animation` 加载。

## 参考项目

| 项目 | 说明 | URL |
|------|------|-----|
| posers | VRM 程序化动画引擎，含 idle-breathe/confident-stance 等示例 | github.com/thomasdavis/posers |
| three-vrm | pixiv 官方 VRM 加载库 | github.com/pixiv/three-vrm |
| Animaze VRM | VRM 动画标准文档（Idle Pose/Idle Motion 定义） | animaze.us/manual/vrmavatar/vrmanimations |
| r3f-vrm | React Three Fiber VRM 集成（FBX/BVH/VRMA 转换） | github.com/DavidCks/r3f-vrm |
| Synthetic Heart | VRM + Mixamo 动画系统（AnimationMixer crossfade） | synthetic-heart.readthedocs.io |
