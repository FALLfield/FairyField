import { ref } from 'vue';

/**
 * AI Agent 对话 composable（stub）
 *
 * 提供发送消息和接收流式回复的接口。
 * Phase 3 实现时接入 Rig Agent 后端。
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

  /** 发送消息并接收流式回复 */
  async function send(message: string): Promise<void> {
    // TODO: Phase 3 — 调用 tauriCommands.chatStream(message)
    void message;

    messages.value = [
      ...messages.value,
      {
        id: crypto.randomUUID(),
        role: 'user',
        content: message,
        timestamp: Date.now(),
      },
    ];
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
  }

  return {
    messages,
    isStreaming,
    currentReply,
    send,
    abort,
    clearHistory,
  };
}
