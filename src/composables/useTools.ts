import { invoke } from '@tauri-apps/api/core';
import { ref, type Ref } from 'vue';

export interface ToolInfo {
  name: string;
  description: string;
  parameters: Record<string, unknown>;
}

export function useTools() {
  const loading = ref(false);
  const error: Ref<string | null> = ref(null);

  async function listTools(): Promise<ToolInfo[]> {
    loading.value = true;
    error.value = null;
    try {
      return await invoke<ToolInfo[]>('tools_list');
    } catch (e) {
      error.value = String(e);
      return [];
    } finally {
      loading.value = false;
    }
  }

  async function executeTool(name: string, input: string): Promise<string | null> {
    loading.value = true;
    error.value = null;
    try {
      return await invoke<string>('tools_execute', { toolName: name, input });
    } catch (e) {
      error.value = String(e);
      return null;
    } finally {
      loading.value = false;
    }
  }

  return { loading, error, listTools, executeTool };
}
