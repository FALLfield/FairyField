import { describe, expect, it } from 'vitest';
import {
  agentChatStream,
  isTauriEnvironment,
  llmListProviders,
  voiceGetVadState,
  voiceStartAsr,
  voiceStartTts,
} from './tauri-commands';

describe('tauri-commands browser preview fallbacks', () => {
  it('detects jsdom as non-Tauri', () => {
    expect(isTauriEnvironment()).toBe(false);
  });

  it('streams a browser preview reply without Tauri IPC', async () => {
    const chunks: string[] = [];
    const response = await agentChatStream('hello', (event) => {
      if (event.type === 'chunk') chunks.push(event.content);
    });

    expect(response.reply).toContain('hello');
    expect(response.emotion.current).toBe('happy');
    expect(chunks.join('')).toBe(response.reply);
  });

  it('lists a preview provider without Tauri IPC', async () => {
    const providers = await llmListProviders();
    expect(providers[0]).toMatchObject({
      name: 'Browser Preview',
      provider_type: 'mock',
    });
  });

  it('provides voice preview fallbacks without Tauri IPC', async () => {
    await expect(voiceStartAsr()).resolves.toBe('');
    await expect(voiceStartTts('hello')).resolves.toBeUndefined();
    await expect(voiceGetVadState()).resolves.toBe(false);
  });
});
