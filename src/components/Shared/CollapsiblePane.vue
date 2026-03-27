<template>
  <div class="flex shrink-0 transition-all" :class="collapsed ? 'w-6' : width">
    <!-- Toggle button -->
    <button
      v-if="side === 'right'"
      class="w-6 shrink-0 flex items-center justify-center bg-transparent border-none text-text-muted hover:text-text-primary cursor-pointer text-xs"
      :title="collapsed ? 'Expand pane' : 'Collapse pane'"
      @click="toggle">
      {{ collapsed ? '◂' : '▸' }}
    </button>

    <!-- Content -->
    <div v-show="!collapsed" class="flex-1 min-w-0 overflow-hidden">
      <slot />
    </div>

    <!-- Toggle button (left pane: button on the right edge) -->
    <button
      v-if="side === 'left'"
      class="w-6 shrink-0 flex items-center justify-center bg-transparent border-none text-text-muted hover:text-text-primary cursor-pointer text-xs"
      :title="collapsed ? 'Expand pane' : 'Collapse pane'"
      @click="toggle">
      {{ collapsed ? '▸' : '◂' }}
    </button>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { useViewPrefs } from "../../composables/useViewPrefs";

const props = withDefaults(defineProps<{
  side: "left" | "right";
  width?: string;
  screenKey: string;
}>(), {
  width: "w-80",
});

const { prefs, update } = useViewPrefs(`${props.screenKey}.pane.${props.side}`, {
  collapsed: false,
});

const collapsed = computed(() => prefs.value.collapsed as boolean);

function toggle() {
  update({ collapsed: !collapsed.value });
}
</script>
