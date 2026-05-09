import { invoke } from '@tauri-apps/api/core';
import { ref, type Ref } from 'vue';

export interface GuardResult {
  level: string;
  reason: string;
}

export interface InjectionResult {
  detected: boolean;
  categories: string[];
  score: number;
}

export interface RedactionResult {
  redacted: string;
  found_count: number;
}

export function useSecurity() {
  const error: Ref<string | null> = ref(null);

  async function checkCommand(cmd: string): Promise<GuardResult | null> {
    try {
      return await invoke<GuardResult>('security_check_command', { command: cmd });
    } catch (e) {
      error.value = String(e);
      return null;
    }
  }

  async function checkInjection(text: string): Promise<InjectionResult | null> {
    try {
      return await invoke<InjectionResult>('security_check_injection', { text });
    } catch (e) {
      error.value = String(e);
      return null;
    }
  }

  async function redact(text: string): Promise<RedactionResult | null> {
    try {
      return await invoke<RedactionResult>('security_redact', { text });
    } catch (e) {
      error.value = String(e);
      return null;
    }
  }

  return { error, checkCommand, checkInjection, redact };
}
