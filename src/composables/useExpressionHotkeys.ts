import { onMounted, onUnmounted } from 'vue';
import type { ExpressionModule } from '../modules/ExpressionModule';

const EXPRESSION_MAP: Record<string, string> = {
  '1': 'happy',
  '2': 'sad',
  '3': 'angry',
  '4': 'surprised',
  '5': 'laugh',
  '6': 'shy',
  '7': 'upset',
  '8': 'neutral',
};

export function useExpressionHotkeys(getModule: () => ExpressionModule | null) {
  function onKeyDown(e: KeyboardEvent): void {
    if (!e.ctrlKey && !e.metaKey) return;
    const expr = EXPRESSION_MAP[e.key];
    if (!expr) return;
    e.preventDefault();
    const mod = getModule();
    if (mod) mod.setExpression(expr, 1.0);
  }

  onMounted(() => window.addEventListener('keydown', onKeyDown));
  onUnmounted(() => window.removeEventListener('keydown', onKeyDown));
}
