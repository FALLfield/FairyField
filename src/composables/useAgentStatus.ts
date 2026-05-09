import { ref, readonly, onMounted, onUnmounted } from 'vue';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

export type AgentStatus = 'idle' | 'thinking' | 'searching_memory' | 'executing_tool';

export interface ToolLogEntry {
  tool: string;
  result: string;
  timestamp: number;
}

/** Module-level singleton: status and toolLog are shared across all components */
const status = ref<AgentStatus>('idle');
const toolLog = ref<ToolLogEntry[]>([]);

let unlistenStatus: UnlistenFn | null = null;
let unlistenToolLog: UnlistenFn | null = null;

export function useAgentStatus() {
  onMounted(async () => {
    if (!unlistenStatus) {
      unlistenStatus = await listen<string>('agent:status', (event) => {
        status.value = event.payload as AgentStatus;
      });
    }
    if (!unlistenToolLog) {
      unlistenToolLog = await listen<{ tool: string; result: string }>('agent:tool_log', (event) => {
        toolLog.value = [
          ...toolLog.value,
          {
            tool: event.payload.tool,
            result: event.payload.result,
            timestamp: Date.now(),
          },
        ];
      });
    }
  });

  onUnmounted(() => {
    unlistenStatus?.();
    unlistenStatus = null;
    unlistenToolLog?.();
    unlistenToolLog = null;
  });

  function resetStatus(): void {
    status.value = 'idle';
    toolLog.value = [];
  }

  return {
    status: readonly(status),
    toolLog: readonly(toolLog),
    resetStatus,
  };
}
