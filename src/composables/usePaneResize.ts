import { ref, onBeforeUnmount } from "vue";

export interface UsePaneResizeOptions {
  side: "left" | "right";
  minWidth: number;
  maxWidth: number;
  initialWidth: number;
  defaultWidth: number;
  onWidthChange: (width: number) => void;
  onResizeEnd?: (width: number) => void;
}

export function usePaneResize(options: UsePaneResizeOptions) {
  const isResizing = ref(false);
  let startX = 0;
  let startWidth = 0;
  let currentWidth = 0;

  function onMouseMove(e: MouseEvent) {
    const delta = e.clientX - startX;
    // Left pane: drag right = wider. Right pane: drag left = wider.
    const adjusted = options.side === "left" ? startWidth + delta : startWidth - delta;
    currentWidth = Math.min(options.maxWidth, Math.max(options.minWidth, adjusted));
    options.onWidthChange(currentWidth);
  }

  function onMouseUp() {
    isResizing.value = false;
    document.removeEventListener("mousemove", onMouseMove);
    document.removeEventListener("mouseup", onMouseUp);
    document.body.style.cursor = "";
    document.body.style.userSelect = "";
    options.onResizeEnd?.(currentWidth);
  }

  function startResize(e: MouseEvent) {
    isResizing.value = true;
    startX = e.clientX;
    startWidth = options.initialWidth;
    currentWidth = startWidth;
    document.addEventListener("mousemove", onMouseMove);
    document.addEventListener("mouseup", onMouseUp);
    document.body.style.cursor = "col-resize";
    document.body.style.userSelect = "none";
  }

  function resetWidth() {
    options.onWidthChange(options.defaultWidth);
    options.onResizeEnd?.(options.defaultWidth);
  }

  onBeforeUnmount(() => {
    document.removeEventListener("mousemove", onMouseMove);
    document.removeEventListener("mouseup", onMouseUp);
  });

  return { isResizing, startResize, resetWidth };
}
