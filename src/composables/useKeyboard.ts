import { onMounted, onBeforeUnmount, type Ref, type ComputedRef } from "vue";
import { useToastStore } from "../stores/toastStore";

export interface ListNavigationOptions {
  items: Ref<unknown[]> | ComputedRef<unknown[]>;
  selectedIndex: Ref<number>;
  onConfirm?: (index: number) => void;
  scrollContainerRef?: Ref<HTMLElement | null>;
}

export interface PaneNavigationOptions {
  panes: string[];
  activePane: Ref<string>;
}

export interface TabCyclingOptions {
  tabs: Ref<string[]> | ComputedRef<string[]> | string[];
  activeTab: Ref<string>;
}

export interface KeyboardOptions {
  listNavigation?: ListNavigationOptions;
  paneNavigation?: PaneNavigationOptions;
  tabCycling?: TabCyclingOptions;
  onEscape?: () => void;
}

function isInputElement(target: EventTarget | null): boolean {
  if (!target || !(target instanceof HTMLElement)) return false;
  const tag = target.tagName;
  return tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT";
}

function resolveArray(source: Ref<string[]> | ComputedRef<string[]> | string[]): string[] {
  return Array.isArray(source) ? source : source.value;
}

export function useKeyboard(options: KeyboardOptions) {
  const toastStore = useToastStore();

  function handleKeydown(event: KeyboardEvent) {
    const inInput = isInputElement(event.target);

    // Escape always fires
    if (event.key === "Escape") {
      if (options.onEscape) {
        options.onEscape();
      } else {
        toastStore.dismissTop();
      }
      event.preventDefault();
      return;
    }

    // List navigation: Arrow Up/Down (always), W/S (only outside inputs), Enter to confirm
    // Runs before input suppression so arrow keys work while typing in search fields
    if (options.listNavigation) {
      const { items, selectedIndex, onConfirm, scrollContainerRef } = options.listNavigation;
      const count = items.value.length;

      if (count > 0) {
        let handled = false;

        if (event.key === "ArrowUp" || (!inInput && event.key.toLowerCase() === "w")) {
          selectedIndex.value = Math.max(0, selectedIndex.value - 1);
          handled = true;
        } else if (event.key === "ArrowDown" || (!inInput && event.key.toLowerCase() === "s")) {
          selectedIndex.value = Math.min(count - 1, selectedIndex.value + 1);
          handled = true;
        } else if (event.key === "Enter" && onConfirm) {
          onConfirm(selectedIndex.value);
          handled = true;
        }

        if (handled) {
          event.preventDefault();
          // Scroll selected item into view
          if (scrollContainerRef?.value) {
            const children = scrollContainerRef.value.children;
            const child = children[selectedIndex.value] as HTMLElement | undefined;
            child?.scrollIntoView({ block: "nearest" });
          }
          return;
        }
      }
    }

    // Suppress remaining nav keys when typing in an input
    if (inInput) return;

    // Tab cycling: Shift+Arrow or Q/E
    if (options.tabCycling) {
      const { tabs: tabsSource, activeTab } = options.tabCycling;
      const tabs = resolveArray(tabsSource);
      let handled = false;

      if ((event.shiftKey && event.key === "ArrowLeft") || event.key.toLowerCase() === "q") {
        const idx = tabs.indexOf(activeTab.value);
        activeTab.value = tabs[(idx - 1 + tabs.length) % tabs.length];
        handled = true;
      } else if ((event.shiftKey && event.key === "ArrowRight") || event.key.toLowerCase() === "e") {
        const idx = tabs.indexOf(activeTab.value);
        activeTab.value = tabs[(idx + 1) % tabs.length];
        handled = true;
      }

      if (handled) {
        event.preventDefault();
        return;
      }
    }

    // Pane navigation: Arrow Left/Right or A/D (without shift)
    if (options.paneNavigation && !event.shiftKey) {
      const { panes, activePane } = options.paneNavigation;
      let handled = false;

      if (event.key === "ArrowLeft" || event.key.toLowerCase() === "a") {
        const idx = panes.indexOf(activePane.value);
        if (idx > 0) activePane.value = panes[idx - 1];
        handled = true;
      } else if (event.key === "ArrowRight" || event.key.toLowerCase() === "d") {
        const idx = panes.indexOf(activePane.value);
        if (idx < panes.length - 1) activePane.value = panes[idx + 1];
        handled = true;
      }

      if (handled) {
        event.preventDefault();
        return;
      }
    }
  }

  onMounted(() => {
    window.addEventListener("keydown", handleKeydown);
  });

  onBeforeUnmount(() => {
    window.removeEventListener("keydown", handleKeydown);
  });
}
