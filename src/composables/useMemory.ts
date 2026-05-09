import { invoke } from '@tauri-apps/api/core';
import { ref, type Ref } from 'vue';

export interface WakeUpContext {
  identity: string;
  summary: string;
  index: string[];
}

export interface Drawer {
  id: number;
  content: string;
  wing: string;
  room: string;
  hall: string;
  created_at: number;
  metadata: Record<string, string>;
}

export function useMemory() {
  const loading = ref(false);
  const error: Ref<string | null> = ref(null);

  async function wakeUp(): Promise<WakeUpContext | null> {
    loading.value = true;
    error.value = null;
    try {
      return await invoke<WakeUpContext>('memory_wake_up');
    } catch (e) {
      error.value = String(e);
      return null;
    } finally {
      loading.value = false;
    }
  }

  async function searchMemory(query: string, limit?: number): Promise<Drawer[]> {
    loading.value = true;
    error.value = null;
    try {
      return await invoke<Drawer[]>('memory_search', { query, limit });
    } catch (e) {
      error.value = String(e);
      return [];
    } finally {
      loading.value = false;
    }
  }

  async function recallMemory(wing: string, limit?: number): Promise<Drawer[]> {
    loading.value = true;
    error.value = null;
    try {
      return await invoke<Drawer[]>('memory_recall', { wing, limit });
    } catch (e) {
      error.value = String(e);
      return [];
    } finally {
      loading.value = false;
    }
  }

  async function addDrawer(
    content: string,
    wing: string,
    room: string,
    hall: string,
  ): Promise<number | null> {
    loading.value = true;
    error.value = null;
    try {
      return await invoke<number>('memory_add_drawer', { content, wing, room, hall });
    } catch (e) {
      error.value = String(e);
      return null;
    } finally {
      loading.value = false;
    }
  }

  async function listWings(): Promise<unknown[]> {
    try {
      return await invoke<unknown[]>('memory_list_wings');
    } catch (e) {
      error.value = String(e);
      return [];
    }
  }

  return { loading, error, wakeUp, searchMemory, recallMemory, addDrawer, listWings };
}
