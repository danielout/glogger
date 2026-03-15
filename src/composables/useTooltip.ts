import { ref, onBeforeUnmount } from "vue";

export function useTooltip(options?: {
  delay?: number;
  onHover?: () => void;
}) {
  const showTooltip = ref(false);
  const delay = options?.delay ?? 500;
  let timer: ReturnType<typeof setTimeout> | null = null;
  let hovered = false;

  function onMouseEnter() {
    if (!hovered && options?.onHover) {
      hovered = true;
      options.onHover();
    }
    timer = setTimeout(() => {
      showTooltip.value = true;
    }, delay);
  }

  function onMouseLeave() {
    if (timer) {
      clearTimeout(timer);
      timer = null;
    }
    showTooltip.value = false;
  }

  function cleanup() {
    if (timer) {
      clearTimeout(timer);
      timer = null;
    }
  }

  onBeforeUnmount(cleanup);

  return { showTooltip, onMouseEnter, onMouseLeave, cleanup };
}
