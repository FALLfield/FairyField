<script setup lang="ts">
import { ref } from 'vue';
import { useMemory, type Drawer, type WakeUpContext } from '../composables/useMemory';

const memory = useMemory();
const searchQuery = ref('');
const searchResults = ref<Drawer[]>([]);
const wakeUpCtx = ref<WakeUpContext | null>(null);
const showPanel = ref(false);

async function doSearch() {
  if (!searchQuery.value.trim()) return;
  searchResults.value = await memory.searchMemory(searchQuery.value);
}

async function doWakeUp() {
  wakeUpCtx.value = await memory.wakeUp();
}
</script>

<template>
  <div class="memory-panel" :class="{ collapsed: !showPanel }">
    <button class="toggle-btn" @click="showPanel = !showPanel">
      {{ showPanel ? '▼ 记忆' : '▶ 记忆' }}
    </button>

    <div v-if="showPanel" class="panel-body">
      <!-- Wake-up -->
      <section class="section">
        <button class="action-btn" @click="doWakeUp" :disabled="memory.loading.value">
          Wake Up
        </button>
        <div v-if="wakeUpCtx" class="wakeup-box">
          <p class="label">身份:</p>
          <pre>{{ wakeUpCtx.identity }}</pre>
          <p class="label">摘要:</p>
          <pre>{{ wakeUpCtx.summary }}</pre>
        </div>
      </section>

      <!-- Search -->
      <section class="section">
        <div class="search-row">
          <input
            v-model="searchQuery"
            placeholder="搜索记忆..."
            class="search-input"
            @keyup.enter="doSearch"
          />
          <button class="action-btn" @click="doSearch" :disabled="memory.loading.value">
            搜索
          </button>
        </div>
        <div v-if="searchResults.length" class="results">
          <div v-for="d in searchResults" :key="d.id" class="result-card">
            <div class="tags">
              <span class="tag">{{ d.wing }}</span>
              <span class="tag">{{ d.room }}</span>
              <span class="tag">{{ d.hall }}</span>
            </div>
            <p class="content">{{ d.content }}</p>
          </div>
        </div>
      </section>

      <p v-if="memory.error.value" class="error">{{ memory.error.value }}</p>
    </div>
  </div>
</template>

<style scoped>
.memory-panel {
  position: fixed;
  right: 12px;
  top: 60px;
  width: 340px;
  z-index: 300;
  font-size: 12px;
  font-family: ui-monospace, monospace;
}
.collapsed {
  width: auto;
}
.toggle-btn {
  background: rgba(30, 30, 40, 0.85);
  color: #ccc;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 6px;
  padding: 4px 10px;
  cursor: pointer;
  font-size: 12px;
}
.toggle-btn:hover {
  background: rgba(50, 50, 60, 0.9);
}
.panel-body {
  margin-top: 6px;
  background: rgba(20, 20, 28, 0.92);
  backdrop-filter: blur(12px);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
  padding: 10px;
  max-height: 70vh;
  overflow-y: auto;
  color: #ddd;
}
.section {
  margin-bottom: 10px;
}
.label {
  color: #888;
  margin: 4px 0 2px;
  font-size: 11px;
}
.search-row {
  display: flex;
  gap: 6px;
}
.search-input {
  flex: 1;
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 4px;
  color: #eee;
  padding: 4px 8px;
  font-size: 12px;
}
.action-btn {
  background: rgba(0, 212, 170, 0.2);
  color: #00d4aa;
  border: 1px solid rgba(0, 212, 170, 0.3);
  border-radius: 4px;
  padding: 4px 10px;
  cursor: pointer;
  font-size: 12px;
}
.action-btn:hover {
  background: rgba(0, 212, 170, 0.3);
}
.action-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.wakeup-box pre {
  background: rgba(255, 255, 255, 0.04);
  padding: 4px;
  border-radius: 4px;
  font-size: 11px;
  max-height: 100px;
  overflow-y: auto;
  white-space: pre-wrap;
  margin: 2px 0;
}
.result-card {
  background: rgba(255, 255, 255, 0.04);
  border-radius: 6px;
  padding: 6px 8px;
  margin-top: 6px;
}
.tags {
  display: flex;
  gap: 4px;
  margin-bottom: 4px;
}
.tag {
  background: rgba(0, 212, 170, 0.15);
  color: #00d4aa;
  padding: 1px 6px;
  border-radius: 3px;
  font-size: 10px;
}
.content {
  color: #bbb;
  line-height: 1.4;
}
.error {
  color: #f87171;
  font-size: 11px;
}
</style>
