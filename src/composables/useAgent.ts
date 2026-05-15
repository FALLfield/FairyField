import { ref, readonly, onMounted, onUnmounted } from 'vue';
import * as tauriCommands from '../lib/tauri-commands';
import { createEmotionEngine, type Emotion } from '../lib/emotion-engine';
import type { EmotionKind } from '../lib/tauri-commands';

/** 将后端 EmotionKind 映射到前端 Emotion（后端多了 shy/upset/excited） */
function mapEmotion(kind: EmotionKind): Emotion {
  const mapping: Record<EmotionKind, Emotion> = {
    happy: 'happy',
    sad: 'sad',
    angry: 'angry',
    surprised: 'surprised',
    neutral: 'neutral',
    thinking: 'thinking',
    excited: 'happy',
    shy: 'surprised',
    upset: 'sad',
  };
  return mapping[kind] ?? 'neutral';
}

/**
 * AI Agent 对话 composable
 *
 * 提供发送消息和接收流式回复的接口。
 * 集成情绪引擎，根据 Fairy 情绪状态驱动 UI 变化。
 */

export interface AgentMessage {
  id: string;
  role: 'user' | 'assistant';
  content: string;
  timestamp: number;
  emotion?: Emotion;
}

export function useAgent() {
  const messages = ref<AgentMessage[]>([]);
  const isStreaming = ref(false);
  const currentReply = ref('');
  const error = ref<string | null>(null);
  const fairyEmotion = ref<Emotion>('neutral');

  const emotionEngine = createEmotionEngine();
  let emotionTickInterval: ReturnType<typeof setInterval> | null = null;

  /** 启动情绪衰减定时器 */
  function startEmotionTick(): void {
    if (emotionTickInterval) return;
    emotionTickInterval = setInterval(() => {
      emotionEngine.tick(1);
      fairyEmotion.value = emotionEngine.getState().current;
    }, 1000);
  }

  /** 停止情绪衰减定时器 */
  function stopEmotionTick(): void {
    if (emotionTickInterval) {
      clearInterval(emotionTickInterval);
      emotionTickInterval = null;
    }
  }

  // 流式超时保护：120s 后强制关闭 loading 状态
  let streamTimeout: ReturnType<typeof setTimeout> | null = null;

  /** 发送消息并接收流式回复 */
  async function send(message: string): Promise<void> {
    // 防止竞态：正在流式回复时拒绝新的发送
    if (isStreaming.value) return;
    error.value = null;

    const userMessage: AgentMessage = {
      id: crypto.randomUUID(),
      role: 'user',
      content: message,
      timestamp: Date.now(),
    };

    messages.value = [...messages.value, userMessage];

    try {
      isStreaming.value = true;
      currentReply.value = '';
      emotionEngine.setEmotion('thinking');
      fairyEmotion.value = 'thinking';

      // 安全超时：120s 后强制关闭 loading 状态，防止 UI 卡在"思考中..."
      streamTimeout = setTimeout(() => {
        if (isStreaming.value) {
          isStreaming.value = false;
          currentReply.value = '';
        }
      }, 120_000);

      let streamedContent = '';

      // 流式返回 + 获取后端情绪数据
      const response = await tauriCommands.agentChatStream(message, (event) => {
        if (event.type === 'chunk') {
          streamedContent += event.content;
          currentReply.value = streamedContent;
        }
      });

      // 使用后端返回的清理后回复（不含 [emotion:xxx] 标签）
      const replyText = response.reply || streamedContent || '(思考中...)';
      currentReply.value = replyText;

      // 从后端响应同步情绪状态（不需要额外 IPC 调用）
      try {
        const mappedEmotion = mapEmotion(response.emotion.current);
        emotionEngine.setEmotion(mappedEmotion);
        fairyEmotion.value = mappedEmotion;

        const assistantMessage: AgentMessage = {
          id: crypto.randomUUID(),
          role: 'assistant',
          content: replyText,
          timestamp: Date.now(),
          emotion: mappedEmotion,
        };
        messages.value = [...messages.value, assistantMessage];
      } catch {
        // 情绪映射失败，仍然添加消息
        const assistantMessage: AgentMessage = {
          id: crypto.randomUUID(),
          role: 'assistant',
          content: replyText,
          timestamp: Date.now(),
        };
        messages.value = [...messages.value, assistantMessage];
      }
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : String(e);
      emotionEngine.reset();
      fairyEmotion.value = 'neutral';
    } finally {
      if (streamTimeout) clearTimeout(streamTimeout);
      isStreaming.value = false;
      currentReply.value = '';
    }
  }

  /** 中断当前流式回复 */
  function abort(): void {
    if (streamTimeout) clearTimeout(streamTimeout);
    isStreaming.value = false;
    currentReply.value = '';
  }

  /** 清空对话历史 */
  function clearHistory(): void {
    messages.value = [];
    currentReply.value = '';
    error.value = null;
    emotionEngine.reset();
    fairyEmotion.value = 'neutral';
  }

  // 生命周期
  onMounted(() => {
    startEmotionTick();
  });

  onUnmounted(() => {
    stopEmotionTick();
  });

  return {
    messages,
    isStreaming: readonly(isStreaming),
    currentReply: readonly(currentReply),
    error: readonly(error),
    fairyEmotion: readonly(fairyEmotion),
    send,
    abort,
    clearHistory,
  };
}
