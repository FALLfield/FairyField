<script setup lang="ts">
import { ref, reactive, computed, watch } from 'vue';
import type { OnboardingData } from '../composables/useOnboarding';
import * as tauriCommands from '../lib/tauri-commands';
import type { ProviderPreset } from '../lib/tauri-commands';

const props = withDefaults(
  defineProps<{
    show?: boolean;
  }>(),
  { show: true },
);

const emit = defineEmits<{
  complete: [data: OnboardingData];
}>();

// ---------------------------------------------------------------------------
// Step state
// ---------------------------------------------------------------------------

const totalSteps = 4;
const currentStep = ref(1);
const direction = ref<'forward' | 'backward'>('forward');
const transitionName = computed(() =>
  direction.value === 'forward' ? 'step-forward' : 'step-backward',
);

// ---------------------------------------------------------------------------
// Form data
// ---------------------------------------------------------------------------

const form = reactive({
  userName: '',
  fairyName: 'Fairy',
  callPreference: 'Fairy',
  customCallPreference: '',
  personality: 'warm' as OnboardingData['personality'],
  customPersonality: '',
  language: 'zh-CN',
  llmProvider: '',
  llmApiKey: '',
});

// ---------------------------------------------------------------------------
// Derived
// ---------------------------------------------------------------------------

const effectiveFairyName = computed(() => {
  return form.fairyName;
});

const effectiveCallPreference = computed(() => {
  if (form.callPreference === 'custom') {
    return form.customCallPreference.trim() || 'Fairy';
  }
  return form.callPreference;
});

const canNextStep1 = computed(() => form.userName.trim().length > 0);
const canNextStep2 = computed(() => {
  if (form.callPreference === 'custom') {
    return form.customCallPreference.trim().length > 0;
  }
  return true;
});
const canNextStep3 = computed(() => {
  if (form.personality === 'custom' && form.customPersonality.trim().length === 0)
    return false;
  return true;
});

const showApiKey = ref(false);

// ---------------------------------------------------------------------------
// LLM provider state
// ---------------------------------------------------------------------------

const providers = ref<ProviderPreset[]>([]);
const providersLoading = ref(false);
const providersError = ref<string | null>(null);
const testingConnection = ref(false);
const connectionTestResult = ref<'idle' | 'success' | 'error'>('idle');
const connectionTestMessage = ref('');

async function loadProviders(): Promise<void> {
  providersLoading.value = true;
  providersError.value = null;
  try {
    const list = await tauriCommands.llmListProviders();
    providers.value = list;
    if (list.length > 0 && !form.llmProvider) {
      form.llmProvider = list[0].name;
    }
  } catch (e) {
    providersError.value =
      e instanceof Error ? e.message : '无法加载 LLM 提供商列表';
  } finally {
    providersLoading.value = false;
  }
}

async function testConnection(): Promise<void> {
  if (!form.llmProvider) return;
  testingConnection.value = true;
  connectionTestResult.value = 'idle';
  connectionTestMessage.value = '';
  try {
    if (form.llmApiKey.trim()) {
      await tauriCommands.llmTestProvider(
        form.llmProvider,
        form.llmApiKey.trim(),
      );
    } else {
      await tauriCommands.llmSwitchProvider(form.llmProvider);
    }
    connectionTestResult.value = 'success';
    connectionTestMessage.value = '连接成功！';
  } catch (e) {
    connectionTestResult.value = 'error';
    connectionTestMessage.value =
      e instanceof Error ? e.message : '连接失败，请检查配置';
  } finally {
    testingConnection.value = false;
  }
}

function toggleShowApiKey(): void {
  showApiKey.value = !showApiKey.value;
}

// ---------------------------------------------------------------------------
// Navigation
// ---------------------------------------------------------------------------

function goToStep(step: number): void {
  if (step < 1 || step > totalSteps) return;
  direction.value = step > currentStep.value ? 'forward' : 'backward';
  currentStep.value = step;

  // Load providers when entering step 4
  if (step === 4 && providers.value.length === 0 && !providersLoading.value) {
    loadProviders();
  }
}

function next(): void {
  if (currentStep.value === totalSteps) {
    emitComplete();
    return;
  }
  goToStep(currentStep.value + 1);
}

function prev(): void {
  goToStep(currentStep.value - 1);
}

function skipLlmConfig(): void {
  emitComplete();
}

function emitComplete(): void {
  const data: OnboardingData = {
    userName: form.userName.trim(),
    callPreference: effectiveCallPreference.value,
    fairyName: effectiveFairyName.value,
    personality: form.personality,
    customPersonality:
      form.personality === 'custom' ? form.customPersonality.trim() : undefined,
    language: form.language,
    llmProvider: form.llmProvider,
    llmApiKey: form.llmApiKey,
  };
  emit('complete', data);
}

/** Reset wizard state when re-shown */
watch(
  () => props.show,
  (visible) => {
    if (visible) {
      currentStep.value = 1;
      direction.value = 'forward';
      form.userName = '';
      form.fairyName = 'Fairy';
      form.callPreference = 'Fairy';
      form.customCallPreference = '';
      form.personality = 'warm';
      form.customPersonality = '';
      form.llmProvider = '';
      form.llmApiKey = '';
      showApiKey.value = false;
      connectionTestResult.value = 'idle';
      connectionTestMessage.value = '';
      providers.value = [];
      providersError.value = null;
    }
  },
);
</script>

<template>
  <Teleport to="body">
    <Transition name="overlay-fade">
      <div
        v-if="show"
        class="wizard-overlay"
        role="dialog"
        aria-label="初始设置向导"
        aria-modal="true"
      >
        <div class="wizard-card">
          <!-- Step indicator dots -->
          <nav class="step-dots" aria-label="步骤指示器">
            <button
              v-for="step in totalSteps"
              :key="step"
              class="step-dot"
              :class="{
                active: step === currentStep,
                completed: step < currentStep,
              }"
              :aria-label="`步骤 ${step}`"
              :aria-current="step === currentStep ? 'step' : undefined"
              :disabled="step > currentStep"
              @click="goToStep(step)"
            >
              <span class="dot-visual" />
            </button>
          </nav>

          <!-- Step content with slide transition -->
          <div class="step-viewport">
            <Transition :name="transitionName" mode="out-in">
              <!-- ========================================================= -->
              <!-- Step 1: User Name -->
              <!-- ========================================================= -->
              <div v-if="currentStep === 1" :key="1" class="step-panel">
                <h2 class="step-title">欢迎来到 FairyField</h2>
                <p class="step-subtitle">告诉我你的名字</p>

                <div class="step-body">
                  <label class="field-label" for="input-user-name">
                    你的名字
                  </label>
                  <input
                    id="input-user-name"
                    v-model="form.userName"
                    type="text"
                    class="text-input"
                    placeholder="你的名字"
                    maxlength="32"
                    autocomplete="given-name"
                    aria-required="true"
                    @keydown.enter="canNextStep1 && next()"
                  />
                </div>

                <div class="step-actions">
                  <button
                    class="btn btn-primary"
                    :disabled="!canNextStep1"
                    aria-label="下一步"
                    @click="next"
                  >
                    下一步
                  </button>
                </div>
              </div>

              <!-- ========================================================= -->
              <!-- Step 2: Call Preference -->
              <!-- ========================================================= -->
              <div v-else-if="currentStep === 2" :key="2" class="step-panel">
                <h2 class="step-title">称呼偏好</h2>
                <p class="step-subtitle">你希望 Fairy 怎样称呼你</p>

                <div class="step-body">
                  <fieldset class="radio-group">
                    <legend class="field-label">称呼方式</legend>

                    <label class="radio-item">
                      <input
                        v-model="form.callPreference"
                        type="radio"
                        name="call-preference"
                        value="Fairy"
                      />
                      <span class="radio-label">Fairy（温暖亲昵）</span>
                    </label>

                    <label class="radio-item">
                      <input
                        v-model="form.callPreference"
                        type="radio"
                        name="call-preference"
                        value="主人"
                      />
                      <span class="radio-label">主人（正式尊重）</span>
                    </label>

                    <label class="radio-item radio-item-custom">
                      <input
                        v-model="form.callPreference"
                        type="radio"
                        name="call-preference"
                        value="custom"
                      />
                      <span class="radio-label">自定义：</span>
                      <input
                        v-model="form.customCallPreference"
                        type="text"
                        class="text-input inline-input"
                        placeholder="她该怎么叫你"
                        maxlength="16"
                        :disabled="form.callPreference !== 'custom'"
                        aria-label="自定义称呼"
                      />
                    </label>
                  </fieldset>
                </div>

                <div class="step-actions">
                  <button
                    class="btn btn-secondary"
                    aria-label="上一步"
                    @click="prev"
                  >
                    上一步
                  </button>
                  <button
                    class="btn btn-primary"
                    :disabled="!canNextStep2"
                    aria-label="下一步"
                    @click="next"
                  >
                    下一步
                  </button>
                </div>
              </div>

              <!-- ========================================================= -->
              <!-- Step 3: Personality -->
              <!-- ========================================================= -->
              <div v-else-if="currentStep === 3" :key="3" class="step-panel">
                <h2 class="step-title">性格设定</h2>
                <p class="step-subtitle">选择她陪伴你的方式</p>

                <div class="step-body">

                  <fieldset class="radio-group">
                    <legend class="field-label">性格设定</legend>

                    <label class="radio-item">
                      <input
                        v-model="form.personality"
                        type="radio"
                        name="personality"
                        value="warm"
                      />
                      <span class="radio-label">
                        温暖贴心 — 像知心朋友
                      </span>
                    </label>

                    <label class="radio-item">
                      <input
                        v-model="form.personality"
                        type="radio"
                        name="personality"
                        value="playful"
                      />
                      <span class="radio-label">
                        活泼调皮 — 像元气少女
                      </span>
                    </label>

                    <label class="radio-item">
                      <input
                        v-model="form.personality"
                        type="radio"
                        name="personality"
                        value="quiet"
                      />
                      <span class="radio-label">
                        安静陪伴 — 像安静的守护者
                      </span>
                    </label>

                    <label class="radio-item radio-item-custom">
                      <input
                        v-model="form.personality"
                        type="radio"
                        name="personality"
                        value="custom"
                      />
                      <span class="radio-label">
                        自定义 — 用一段文字描述你的理想性格
                      </span>
                    </label>

                    <textarea
                      v-if="form.personality === 'custom'"
                      v-model="form.customPersonality"
                      class="text-area"
                      placeholder="描述你心中理想的她..."
                      maxlength="500"
                      rows="3"
                      aria-label="自定义性格描述"
                    />
                  </fieldset>
                </div>

                <div class="step-actions">
                  <button
                    class="btn btn-secondary"
                    aria-label="上一步"
                    @click="prev"
                  >
                    上一步
                  </button>
                  <button
                    class="btn btn-primary"
                    :disabled="!canNextStep3"
                    aria-label="下一步"
                    @click="next"
                  >
                    下一步
                  </button>
                </div>
              </div>

              <!-- ========================================================= -->
              <!-- Step 4: LLM Configuration -->
              <!-- ========================================================= -->
              <div v-else-if="currentStep === 4" :key="4" class="step-panel">
                <h2 class="step-title">连接 AI 大脑</h2>
                <p class="step-subtitle">配置大语言模型</p>

                <div class="step-body">
                  <!-- Provider selector -->
                  <div class="field-group">
                    <label class="field-label" for="select-provider">
                      AI 提供商
                    </label>

                    <div v-if="providersLoading" class="inline-hint">
                      加载中...
                    </div>
                    <div v-else-if="providersError" class="inline-hint error">
                      {{ providersError }}
                      <button
                        class="btn-link"
                        aria-label="重试加载提供商列表"
                        @click="loadProviders"
                      >
                        重试
                      </button>
                    </div>
                    <select
                      v-else
                      id="select-provider"
                      v-model="form.llmProvider"
                      class="select-input"
                      aria-label="选择 AI 提供商"
                      @change="connectionTestResult = 'idle'"
                    >
                      <option value="" disabled>选择提供商</option>
                      <option
                        v-for="p in providers"
                        :key="p.name"
                        :value="p.name"
                      >
                        {{ p.name }} ({{ p.model }})
                      </option>
                    </select>
                  </div>

                  <!-- API Key -->
                  <div class="field-group">
                    <label class="field-label" for="input-api-key">
                      API Key
                    </label>
                    <div class="input-with-toggle">
                      <input
                        id="input-api-key"
                        v-model="form.llmApiKey"
                        :type="showApiKey ? 'text' : 'password'"
                        class="text-input"
                        placeholder="sk-..."
                        autocomplete="off"
                        aria-label="API 密钥"
                      />
                      <button
                        class="btn-toggle"
                        :aria-label="showApiKey ? '隐藏密钥' : '显示密钥'"
                        @click="toggleShowApiKey"
                      >
                        {{ showApiKey ? '隐藏' : '显示' }}
                      </button>
                    </div>
                  </div>

                  <!-- Test connection -->
                  <div class="field-group">
                    <button
                      class="btn btn-primary btn-test"
                      :disabled="!form.llmProvider || testingConnection"
                      aria-label="测试连接"
                      @click="testConnection"
                    >
                      <span v-if="testingConnection" class="spinner" />
                      {{ testingConnection ? '测试中...' : '测试连接' }}
                    </button>

                    <p
                      v-if="connectionTestResult === 'success'"
                      class="inline-hint success"
                    >
                      {{ connectionTestMessage }}
                    </p>
                    <p
                      v-else-if="connectionTestResult === 'error'"
                      class="inline-hint error"
                    >
                      {{ connectionTestMessage }}
                    </p>
                  </div>
                </div>

                <div class="step-actions">
                  <button
                    class="btn btn-secondary"
                    aria-label="上一步"
                    @click="prev"
                  >
                    上一步
                  </button>
                  <button
                    class="btn btn-primary btn-cta"
                    aria-label="开始体验"
                    @click="emitComplete"
                  >
                    开始体验 →
                  </button>
                </div>

                <button
                  class="btn btn-skip"
                  aria-label="稍后配置"
                  @click="skipLlmConfig"
                >
                  稍后配置
                </button>
              </div>
            </Transition>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
/* =========================================================================
   Overlay
   ========================================================================= */

.wizard-overlay {
  position: fixed;
  inset: 0;
  z-index: 500;
  display: flex;
  align-items: safe center;
  justify-content: safe center;
  overflow-y: auto;
  overflow-x: hidden;
  background: rgba(0, 0, 0, 0.55);
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
  padding: 24px 0;
  scrollbar-width: thin;
  scrollbar-color: rgba(255, 255, 255, 0.08) transparent;
}
.wizard-overlay::-webkit-scrollbar { width: 3px; }
.wizard-overlay::-webkit-scrollbar-thumb { background: rgba(255, 255, 255, 0.08); border-radius: 2px; }

/* =========================================================================
   Card
   ========================================================================= */

.wizard-card {
  position: relative;
  width: 100%;
  max-width: 480px;
  min-height: 420px;
  margin: 0 16px;
  padding: 2rem 2rem 1.5rem;
  border-radius: 20px;
  background: rgba(22, 22, 30, 0.9);
  border: 1px solid rgba(255, 255, 255, 0.08);
  box-shadow:
    0 0 0 1px rgba(255, 255, 255, 0.04),
    0 24px 80px rgba(0, 0, 0, 0.5);
  display: flex;
  flex-direction: column;
}

/* =========================================================================
   Step dots
   ========================================================================= */

.step-dots {
  display: flex;
  justify-content: center;
  gap: 10px;
  margin-bottom: 1.5rem;
}

.step-dot {
  background: none;
  border: none;
  cursor: pointer;
  padding: 4px;
  border-radius: 50%;
  transition: opacity 0.2s;
}

.step-dot:disabled {
  cursor: default;
  opacity: 0.35;
}

.dot-visual {
  display: block;
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.2);
  transition:
    background 0.3s,
    transform 0.3s;
}

.step-dot.active .dot-visual {
  background: rgba(180, 160, 255, 0.9);
  transform: scale(1.35);
}

.step-dot.completed .dot-visual {
  background: rgba(140, 200, 140, 0.7);
}

/* =========================================================================
   Step viewport / transitions
   ========================================================================= */

.step-viewport {
  flex: 1;
  overflow: hidden;
  position: relative;
}

.step-panel {
  display: flex;
  flex-direction: column;
  min-height: 280px;
}

/* Forward: enter from right, leave to left */
.step-forward-enter-active,
.step-forward-leave-active {
  transition:
    transform 0.28s cubic-bezier(0.4, 0, 0.2, 1),
    opacity 0.2s ease;
}

.step-forward-enter-from {
  transform: translateX(48px);
  opacity: 0;
}

.step-forward-leave-to {
  transform: translateX(-48px);
  opacity: 0;
}

/* Backward: enter from left, leave to right */
.step-backward-enter-active,
.step-backward-leave-active {
  transition:
    transform 0.28s cubic-bezier(0.4, 0, 0.2, 1),
    opacity 0.2s ease;
}

.step-backward-enter-from {
  transform: translateX(-48px);
  opacity: 0;
}

.step-backward-leave-to {
  transform: translateX(48px);
  opacity: 0;
}

/* Overlay fade */
.overlay-fade-enter-active,
.overlay-fade-leave-active {
  transition: opacity 0.3s ease;
}

.overlay-fade-enter-from,
.overlay-fade-leave-to {
  opacity: 0;
}

/* =========================================================================
   Typography
   ========================================================================= */

.step-title {
  margin: 0 0 0.35rem;
  font-size: 1.4rem;
  font-weight: 700;
  color: #f0e8ff;
  letter-spacing: 0.01em;
}

.step-subtitle {
  margin: 0 0 1.25rem;
  font-size: 0.9rem;
  color: rgba(255, 255, 255, 0.5);
}

/* =========================================================================
   Form fields
   ========================================================================= */

.step-body {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 0.9rem;
}

.field-group {
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
}

.field-label {
  font-size: 0.8rem;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.55);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.text-input {
  width: 100%;
  padding: 0.7rem 0.85rem;
  border-radius: 10px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  background: rgba(255, 255, 255, 0.05);
  color: #f0e8ff;
  font-size: 0.95rem;
  outline: none;
  transition: border-color 0.2s;
  box-sizing: border-box;
}

.text-input::placeholder {
  color: rgba(255, 255, 255, 0.2);
}

.text-input:focus {
  border-color: rgba(180, 160, 255, 0.55);
}

.text-area {
  width: 100%;
  padding: 0.65rem 0.8rem;
  border-radius: 10px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  background: rgba(255, 255, 255, 0.05);
  color: #f0e8ff;
  font-size: 0.9rem;
  outline: none;
  resize: vertical;
  box-sizing: border-box;
  font-family: inherit;
}

.text-area::placeholder {
  color: rgba(255, 255, 255, 0.2);
}

.text-area:focus {
  border-color: rgba(180, 160, 255, 0.55);
}

.select-input {
  width: 100%;
  padding: 0.7rem 0.85rem;
  border-radius: 10px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  background: rgba(255, 255, 255, 0.05);
  color: #f0e8ff;
  font-size: 0.95rem;
  outline: none;
  cursor: pointer;
  box-sizing: border-box;
  appearance: none;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 12 12'%3E%3Cpath fill='rgba(255,255,255,0.4)' d='M6 8L1 3h10z'/%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 12px center;
  padding-right: 32px;
}

.select-input:focus {
  border-color: rgba(180, 160, 255, 0.55);
}

.select-input option {
  background: #1a1a24;
  color: #f0e8ff;
}

.input-with-toggle {
  display: flex;
  gap: 0;
}

.input-with-toggle .text-input {
  border-top-right-radius: 0;
  border-bottom-right-radius: 0;
  border-right: none;
}

.btn-toggle {
  padding: 0 1rem;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-left: none;
  border-radius: 0 10px 10px 0;
  background: rgba(255, 255, 255, 0.05);
  color: rgba(255, 255, 255, 0.5);
  font-size: 0.8rem;
  cursor: pointer;
  white-space: nowrap;
  transition:
    background 0.2s,
    color 0.2s;
}

.btn-toggle:hover {
  background: rgba(255, 255, 255, 0.1);
  color: rgba(255, 255, 255, 0.75);
}

.inline-input {
  flex: 1;
  min-width: 0;
}

/* =========================================================================
   Radio groups
   ========================================================================= */

.radio-group {
  border: none;
  padding: 0;
  margin: 0;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.radio-item {
  display: flex;
  align-items: flex-start;
  gap: 0.55rem;
  padding: 0.6rem 0.8rem;
  border-radius: 10px;
  border: 1px solid rgba(255, 255, 255, 0.06);
  background: rgba(255, 255, 255, 0.03);
  cursor: pointer;
  transition:
    background 0.2s,
    border-color 0.2s;
}

.radio-item:hover {
  background: rgba(255, 255, 255, 0.06);
}

.radio-item:has(input:checked) {
  border-color: rgba(180, 160, 255, 0.35);
  background: rgba(180, 160, 255, 0.08);
}

.radio-item input[type='radio'] {
  accent-color: #b4a0ff;
  margin-top: 2px;
  flex-shrink: 0;
}

.radio-label {
  font-size: 0.9rem;
  color: rgba(255, 255, 255, 0.75);
  line-height: 1.4;
}

.radio-item-custom {
  flex-wrap: wrap;
}

/* =========================================================================
   Buttons
   ========================================================================= */

.step-actions {
  display: flex;
  gap: 0.75rem;
  margin-top: 1.25rem;
  padding-top: 0.75rem;
  border-top: 1px solid rgba(255, 255, 255, 0.06);
}

.btn {
  padding: 0.65rem 1.5rem;
  border-radius: 10px;
  font-size: 0.9rem;
  font-weight: 600;
  cursor: pointer;
  border: none;
  transition:
    background 0.2s,
    opacity 0.2s,
    transform 0.15s;
}

.btn:active:not(:disabled) {
  transform: scale(0.97);
}

.btn:disabled {
  opacity: 0.35;
  cursor: not-allowed;
}

.btn-primary {
  background: rgba(160, 140, 240, 0.25);
  color: #e8dcff;
  border: 1px solid rgba(180, 160, 255, 0.2);
  flex: 1;
}

.btn-primary:hover:not(:disabled) {
  background: rgba(160, 140, 240, 0.4);
}

.btn-secondary {
  background: rgba(255, 255, 255, 0.06);
  color: rgba(255, 255, 255, 0.6);
  border: 1px solid rgba(255, 255, 255, 0.08);
}

.btn-secondary:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.12);
  color: rgba(255, 255, 255, 0.8);
}

.btn-cta {
  font-size: 0.95rem;
  padding: 0.75rem 2rem;
}

.btn-test {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
}

.btn-skip {
  display: block;
  width: 100%;
  margin-top: 0.75rem;
  background: none;
  border: none;
  color: rgba(255, 255, 255, 0.3);
  font-size: 0.8rem;
  cursor: pointer;
  text-align: center;
  padding: 0.4rem;
  transition: color 0.2s;
}

.btn-skip:hover {
  color: rgba(255, 255, 255, 0.55);
}

.btn-link {
  background: none;
  border: none;
  color: rgba(180, 160, 255, 0.7);
  cursor: pointer;
  font-size: inherit;
  text-decoration: underline;
  padding: 0;
}

.btn-link:hover {
  color: rgba(180, 160, 255, 0.95);
}

/* =========================================================================
   Hints / status
   ========================================================================= */

.inline-hint {
  font-size: 0.8rem;
  color: rgba(255, 255, 255, 0.4);
  margin: 0;
}

.inline-hint.success {
  color: rgba(140, 200, 140, 0.85);
}

.inline-hint.error {
  color: rgba(240, 120, 120, 0.85);
}

/* =========================================================================
   Spinner
   ========================================================================= */

.spinner {
  display: inline-block;
  width: 14px;
  height: 14px;
  border: 2px solid rgba(255, 255, 255, 0.2);
  border-top-color: rgba(255, 255, 255, 0.7);
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

/* =========================================================================
   Step 4: Ready / checkmark
   ========================================================================= */

.step-ready {
  align-items: center;
  text-align: center;
  gap: 1.25rem;
}

.checkmark-wrapper {
  margin-top: 0.5rem;
}

.checkmark {
  width: 64px;
  height: 64px;
}

.checkmark-circle {
  stroke: rgba(140, 200, 140, 0.7);
  stroke-width: 2;
  stroke-dasharray: 166;
  stroke-dashoffset: 166;
  animation: draw-circle 0.6s ease forwards 0.1s;
}

.checkmark-check {
  stroke: rgba(140, 200, 140, 0.95);
  stroke-width: 2;
  stroke-linecap: round;
  stroke-linejoin: round;
  stroke-dasharray: 48;
  stroke-dashoffset: 48;
  animation: draw-check 0.35s ease forwards 0.5s;
}

@keyframes draw-circle {
  to {
    stroke-dashoffset: 0;
  }
}

@keyframes draw-check {
  to {
    stroke-dashoffset: 0;
  }
}

.ready-summary {
  font-size: 0.95rem;
  color: rgba(255, 255, 255, 0.65);
  line-height: 1.6;
  max-width: 320px;
}

.ready-summary strong {
  color: rgba(255, 255, 255, 0.9);
}

/* =========================================================================
   Reduced motion
   ========================================================================= */

@media (prefers-reduced-motion: reduce) {
  .step-forward-enter-active,
  .step-forward-leave-active,
  .step-backward-enter-active,
  .step-backward-leave-active,
  .overlay-fade-enter-active,
  .overlay-fade-leave-active {
    transition: opacity 0.15s ease;
  }

  .step-forward-enter-from,
  .step-forward-leave-to,
  .step-backward-enter-from,
  .step-backward-leave-to {
    transform: none;
  }

  .checkmark-circle,
  .checkmark-check {
    animation: none;
    stroke-dashoffset: 0;
  }

  .spinner {
    animation: none;
  }
}
</style>
