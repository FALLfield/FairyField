<script setup lang="ts">
interface ChatBubbleProps {
  message: string;
  isUser: boolean;
}

defineProps<ChatBubbleProps>();
</script>

<template>
  <div
    class="chat-bubble"
    :class="{ 'is-user': isUser, 'is-ai': !isUser }"
    role="log"
    aria-live="polite"
  >
    <p class="bubble-text">{{ message }}</p>
  </div>
</template>

<style scoped>
.chat-bubble {
  max-width: 320px;
  padding: 10px 14px;
  border-radius: 12px;
  font-size: 0.875rem;
  line-height: 1.5;
  word-break: break-word;
  pointer-events: none;
}

.is-user {
  align-self: flex-end;
  background: rgba(59, 130, 246, 0.85);
  color: #fff;
  border-bottom-right-radius: 4px;
}

.is-ai {
  align-self: flex-start;
  background: rgba(30, 30, 30, 0.8);
  color: #e5e5e5;
  border-bottom-left-radius: 4px;
}

.bubble-text {
  margin: 0;
}
</style>
