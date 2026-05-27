import { ref, onMounted } from 'vue';
import * as tauriCommands from '../lib/tauri-commands';

export interface OnboardingData {
  userName: string;
  callPreference: string;
  fairyName: string;
  personality: 'warm' | 'playful' | 'quiet' | 'custom';
  customPersonality?: string;
  language: string;
  llmProvider: string;
  llmApiKey: string;
}

export function useOnboarding() {
  const showOnboarding = ref(false);
  const isLoading = ref(true);
  const error = ref<string | null>(null);

  function hasTauriRuntime(): boolean {
    return Boolean((window as any).__TAURI_INTERNALS__ || (window as any).__TAURI__);
  }

  function localOnboardingCompleted(): boolean {
    const stored = localStorage.getItem('fairyfield_user');
    if (!stored) return false;
    const config = JSON.parse(stored);
    return Boolean(config.onboardingCompleted || config.onboarding_completed);
  }

  async function checkOnboardingStatus(): Promise<void> {
    isLoading.value = true;
    error.value = null;
    try {
      if (hasTauriRuntime()) {
        const config = await tauriCommands.userLoadConfig();
        showOnboarding.value = !config.onboarding_completed;
        return;
      }

      showOnboarding.value = !localOnboardingCompleted();
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
      try {
        showOnboarding.value = !localOnboardingCompleted();
      } catch {
        showOnboarding.value = true;
      }
    } finally {
      isLoading.value = false;
    }
  }

  async function completeOnboarding(data: OnboardingData): Promise<void> {
    error.value = null;
    const createdAt = new Date().toISOString();

    try {
      if (hasTauriRuntime()) {
        await tauriCommands.userSaveConfig({
            user_name: data.userName,
            call_preference: data.callPreference,
            fairy_name: data.fairyName,
            personality: data.personality,
            custom_personality: data.customPersonality || null,
            language: data.language,
            onboarding_completed: true,
            created_at: createdAt,
        });

        if (data.llmApiKey && data.llmProvider) {
          await tauriCommands.llmSaveApiKey(data.llmProvider, data.llmApiKey);
        }
      }

      localStorage.setItem('fairyfield_user', JSON.stringify({
        ...data,
        onboardingCompleted: true,
        createdAt,
      }));
      showOnboarding.value = false;
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
      throw e;
    }
  }

  onMounted(() => {
    checkOnboardingStatus();
  });

  return {
    showOnboarding,
    isLoading,
    error,
    checkOnboardingStatus,
    completeOnboarding,
  };
}
