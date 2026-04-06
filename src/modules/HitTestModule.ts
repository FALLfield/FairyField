import * as THREE from 'three';
import type { VRMRenderer } from '../renderers/VRMRenderer';
import { invoke } from '@tauri-apps/api/core';

/**
 * HitTestModule - 点击检测模块（Three.js 适配版）
 *
 * 使用 Three.js Raycaster 进行 3D 射线检测，
 * 判断鼠标点击是否落在 VRM 角色身上。
 *
 * **验证需求 1.5 和 1.6**：
 * - 需求 1.5: 点击透明区域时穿透到底层窗口
 * - 需求 1.6: 点击人物身体区域时拦截事件并触发拖动
 */
export class HitTestModule {
  private renderer: VRMRenderer;
  private raycaster: THREE.Raycaster;
  private mouse: THREE.Vector2;

  /**
   * 构造函数
   * @param renderer - VRMRenderer 实例
   */
  constructor(renderer: VRMRenderer) {
    this.renderer = renderer;
    this.raycaster = new THREE.Raycaster();
    this.mouse = new THREE.Vector2();
  }

  /**
   * 检测指定屏幕坐标是否命中 VRM 角色
   * @param screenX - 屏幕 X 坐标
   * @param screenY - 屏幕 Y 坐标
   * @returns 如果命中角色网格则返回 true，否则返回 false
   */
  isHit(screenX: number, screenY: number): boolean {
    const vrm = this.renderer.getVRM();
    if (!vrm) {
      return false;
    }

    // 获取画布尺寸
    const canvas = this.renderer.getRenderer().domElement;
    const rect = canvas.getBoundingClientRect();

    // 将屏幕坐标转换为 NDC（归一化设备坐标）
    this.mouse.x = ((screenX - rect.left) / rect.width) * 2 - 1;
    this.mouse.y = -((screenY - rect.top) / rect.height) * 2 + 1;

    // 从相机发射射线
    const camera = this.renderer.getCamera();
    this.raycaster.setFromCamera(this.mouse, camera);

    // 检测与 VRM 场景中所有 Mesh 的交叉
    const intersects = this.raycaster.intersectObjects(vrm.scene.children, true);

    // 只要有交叉结果，就算命中
    return intersects.length > 0;
  }

  /**
   * 处理点击事件
   * - 命中角色：拦截事件，触发拖动
   * - 未命中：穿透到底层窗口
   */
  async handleClick(screenX: number, screenY: number): Promise<void> {
    if (this.isHit(screenX, screenY)) {
      // 命中角色身体区域，触发拖动
      try {
        await invoke('start_drag');
      } catch (error: unknown) {
        const message = error instanceof Error ? error.message : String(error);
        console.error(`拖动启动失败: ${message}`);
      }
    } else {
      // 点击透明区域，穿透到底层窗口
      try {
        await invoke('set_ignore_cursor_events', { ignore: true });

        // 短暂延迟后恢复事件捕获
        setTimeout(async () => {
          try {
            await invoke('set_ignore_cursor_events', { ignore: false });
          } catch (error: unknown) {
            const message = error instanceof Error ? error.message : String(error);
            console.error(`恢复光标事件失败: ${message}`);
          }
        }, 100);
      } catch (error: unknown) {
        const message = error instanceof Error ? error.message : String(error);
        console.error(`设置光标穿透失败: ${message}`);
      }
    }
  }

  /**
   * 获取 Three.js Raycaster 实例（供高级用法使用）
   */
  getRaycaster(): THREE.Raycaster {
    return this.raycaster;
  }
}
