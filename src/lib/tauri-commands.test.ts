import { describe, expect, it } from 'vitest';
import {
  agentChatStream,
  developmentLoopPlan,
  developmentLoopRun,
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

  it('builds a multi-agent development loop preview without Tauri IPC', async () => {
    const plan = await developmentLoopPlan(
      'Fix web search UTF-8 truncation',
      ['No byte-index panic on Chinese search results'],
      ['src-tauri/src/tools/builtins/web.rs'],
      [],
      2,
    );

    expect(plan.max_iterations).toBe(2);
    expect(plan.agents.map((agent) => agent.role)).toEqual([
      'manager_agent',
      'coding_agent',
      'testing_agent',
      'goal_agent',
    ]);
    expect(plan.tasks).toHaveLength(4);
    expect(plan.tasks.find((task) => task.role === 'coding_agent')).toMatchObject({
      owner: 'tools',
      status: 'queued',
    });
  });

  it('returns a safe dry-run report for development loop preview', async () => {
    const report = await developmentLoopRun({
      objective: 'Finish the release loop',
      requirements: ['Coding, testing, and goal agents all produce evidence'],
      context_files: [],
      coding_agent: null,
      working_dir: null,
      test_commands: [],
      max_iterations: null,
      model: null,
      permission_mode: null,
      dry_run: true,
    });

    expect(report.success).toBe(false);
    expect(report.dry_run).toBe(true);
    expect(report.agent_results.map((result) => result.role)).toContain('goal_agent');
    expect(report.unmet_requirements[0]).toContain('Browser preview');
  });
});
