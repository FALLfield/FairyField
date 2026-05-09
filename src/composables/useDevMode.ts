import { ref, readonly, onMounted, onUnmounted } from 'vue';

/** Module-level singleton: devMode state is shared across all components */
const enabled = ref(false);

export function useDevMode() {
  function toggle(): void {
    enabled.value = !enabled.value;
  }

  function onKeyDown(e: KeyboardEvent): void {
    if (e.ctrlKey && e.shiftKey && e.key === 'D') {
      e.preventDefault();
      toggle();
    }
  }

  // Detect triple-click on avatar
  let clickCount = 0;
  let clickTimer: ReturnType<typeof setTimeout> | null = null;

  function onAvatarTripleClick(): void {
    clickCount++;
    if (clickTimer) clearTimeout(clickTimer);
    clickTimer = setTimeout(() => {
      clickCount = 0;
    }, 400);
    if (clickCount >= 3) {
      clickCount = 0;
      toggle();
    }
  }

  onMounted(() => {
    window.addEventListener('keydown', onKeyDown);
  });

  onUnmounted(() => {
    window.removeEventListener('keydown', onKeyDown);
  });

  return {
    devMode: readonly(enabled),
    toggleDevMode: toggle,
    onAvatarTripleClick,
  };
}
