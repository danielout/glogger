import { ref, onBeforeUnmount } from "vue";

export function useTooltip(options?: {
  delay?: number;
  interactive?: boolean;
  onHover?: () => void;
}) {
  const showTooltip = ref(false);
  const delay = options?.delay ?? 500;
  const interactive = options?.interactive ?? false;
  let showTimer: ReturnType<typeof setTimeout> | null = null;
  let hideTimer: ReturnType<typeof setTimeout> | null = null;
  let hovered = false;

  function clearTimers() {
    if (showTimer) { clearTimeout(showTimer); showTimer = null; }
    if (hideTimer) { clearTimeout(hideTimer); hideTimer = null; }
  }

  function onMouseEnter() {
    if (!hovered && options?.onHover) {
      hovered = true;
      options.onHover();
    }
    if (hideTimer) { clearTimeout(hideTimer); hideTimer = null; }
    showTimer = setTimeout(() => {
      showTooltip.value = true;
    }, delay);
  }

  function onMouseLeave() {
    if (showTimer) { clearTimeout(showTimer); showTimer = null; }
    if (interactive) {
      // Delay hide so user can move mouse into the tooltip
      hideTimer = setTimeout(() => {
        showTooltip.value = false;
      }, 150);
    } else {
      showTooltip.value = false;
    }
  }

  /** Call from tooltip element's mouseenter to keep it open */
  function onTooltipMouseEnter() {
    if (hideTimer) { clearTimeout(hideTimer); hideTimer = null; }
  }

  /** Call from tooltip element's mouseleave to dismiss */
  function onTooltipMouseLeave() {
    showTooltip.value = false;
  }

  function cleanup() {
    clearTimers();
  }

  onBeforeUnmount(cleanup);

  return { showTooltip, onMouseEnter, onMouseLeave, onTooltipMouseEnter, onTooltipMouseLeave, cleanup };
}
