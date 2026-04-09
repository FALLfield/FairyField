import { ref } from 'vue';
import * as tauriCommands from '../lib/tauri-commands';

/**
 * AI Agent 对话 composable
 *
 * 提供发送消息和接收流式回复的接口。
 * 当前通过 Tauri IPC 调用后端 chat 命令。
 */

export interface AgentMessage {
  id: string;
  role: 'user' | 'assistant';
  content: string;
  timestamp: number;
}

export function useAgent() {
  const messages = ref<AgentMessage[]>([]);
  const isStreaming = ref(false);
  const currentReply = ref('');
  const error = ref<string | null>(null);

  /** 发送消息并接收回复 */
  async function send(message: string): Promise<void> {
    error.value = null;

    messages.value = [
      ...messages.value,
      {
        id: crypto.randomUUID(),
        role: 'user',
        content: message,
        timestamp: Date.now(),
      },
    ];

    try {
      isStreaming.value = true;
      currentReply.value = '';

      // TODO: Phase 3 — 替换为 chatStream() 实现流式回复
      const reply = await tauriCommands.chat(message);

      currentReply.value = reply;

      messages.value = [
        ...messages.value,
        {
          id: crypto.randomUUID(),
          role: 'assistant',
          content: reply,
          timestamp: Date.now(),
        },
      ];
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : String(e);
    } finally {
      isStreaming.value = false;
      currentReply.value = '';
    }
  }

  /** 中断当前流式回复 */
  function abort(): void {
    isStreaming.value = false;
    currentReply.value = '';
  }

  /** 清空对话历史 */
  function clearHistory(): void {
    messages.value = [];
    currentReply.value = '';
    error.value = null;
  }

  return {
    messages,
    isStreaming,
    currentReply,
    error,
    send,
    abort,
    clearHistory,
  };
}
