<template>
  <div
    class="shrink-0 flex transition-[width] duration-200 ease-out overflow-hidden"
    :style="{ width: isFixed ? `${fixedWidth}px` : collapsed ? '28px' : `${width}px` }">

    <!-- Collapsed strip (left pane: strip on left, right pane: strip on right) -->
    <div
      v-show="collapsed && !isFixed"
      class="w-7 shrink-0 flex flex-col items-center justify-center cursor-pointer border-border-default hover:bg-surface-card transition-colors"
      :class="side === 'left' ? 'border-r' : 'border-l'"
      @click="toggle">
      <span
        class="text-text-muted text-xs select-none tracking-wider py-4"
        :class="side === 'left' ? '[writing-mode:vertical-lr] rotate-180' : '[writing-mode:vertical-rl]'">
        {{ title }}
      </span>
    </div>

    <!-- Drag handle (right pane: handle on the left/interior edge) -->
    <div
      v-show="!collapsed && !isFixed && side === 'right'"
      class="w-1.5 shrink-0 cursor-col-resize flex items-center justify-center hover:bg-accent-gold/20 rounded transition-colors"
      :class="{ 'bg-accent-gold/30': isResizing }"
      @mousedown="startResize"
      @dblclick="resetWidth">
      <div class="w-px h-8 bg-border-default rounded-full" />
    </div>

    <!-- Pane content -->
    <div v-show="!collapsed || isFixed" class="flex-1 flex flex-col overflow-hidden min-w-0">
      <!-- Header -->
      <div class="flex items-center justify-between px-2 py-1.5 shrink-0">
        <span class="text-text-primary text-sm font-semibold">{{ title }}</span>
        <button
          v-if="!isFixed"
          class="text-text-muted text-xs cursor-pointer bg-transparent border-none hover:text-text-primary px-1"
          :title="'Collapse ' + title"
          @click="toggle">
          {{ side === 'left' ? '\u25C2' : '\u25B8' }}
        </button>
      </div>

      <!-- Scrollable content -->
      <div class="flex-1 overflow-y-auto min-h-0 pr-0.5">
        <slot />
      </div>
    </div>

    <!-- Drag handle (left pane: handle on the right/interior edge) -->
    <div
      v-show="!collapsed && !isFixed && side === 'left'"
      class="w-1.5 shrink-0 cursor-col-resize flex items-center justify-center hover:bg-accent-gold/20 rounded transition-colors"
      :class="{ 'bg-accent-gold/30': isResizing }"
      @mousedown="startResize"
      @dblclick="resetWidth">
      <div class="w-px h-8 bg-border-default rounded-full" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from "vue";
import { useViewPrefs } from "../../composables/useViewPrefs";
import { usePaneResize } from "../../composables/usePaneResize";

const props = withDefaults(defineProps<{
  side: "left" | "right";
  title: string;
  screenKey: string;
  defaultWidth?: number;
  minWidth?: number;
  maxWidth?: number;
  defaultCollapsed?: boolean;
  /** When set, locks the pane to this exact width with no resize or collapse. */
  fixedWidth?: number;
}>(), {
  defaultWidth: 320,
  minWidth: 200,
  maxWidth: 700,
  defaultCollapsed: false,
});

const isFixed = computed(() => props.fixedWidth != null);

const { prefs, update } = useViewPrefs(`${props.screenKey}.pane.${props.side}`, {
  collapsed: props.defaultCollapsed,
  width: props.defaultWidth,
});

const collapsed = computed(() => prefs.value.collapsed as boolean);
const width = ref(prefs.value.width as number);

function toggle() {
  update({ collapsed: !collapsed.value });
}

const { isResizing, startResize, resetWidth } = usePaneResize({
  side: props.side,
  minWidth: props.minWidth,
  maxWidth: props.maxWidth,
  get initialWidth() { return width.value; },
  defaultWidth: props.defaultWidth,
  onWidthChange: (w) => { width.value = w; },
  onResizeEnd: (w) => { width.value = w; update({ width: w }); },
});
</script>
