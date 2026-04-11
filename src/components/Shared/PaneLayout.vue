<template>
  <div class="flex h-full min-h-0 overflow-hidden">
    <SidePane
      v-if="leftPane"
      side="left"
      :title="leftPane.title"
      :screen-key="screenKey"
      :default-width="leftPane.defaultWidth ?? 320"
      :min-width="leftPane.minWidth ?? 200"
      :max-width="leftPane.maxWidth ?? 700"
      :default-collapsed="leftPane.defaultCollapsed ?? false"
      :fixed-width="leftPane.fixedWidth">
      <slot name="left" />
    </SidePane>

    <div class="flex-1 min-w-0 overflow-hidden min-h-0">
      <slot />
    </div>

    <SidePane
      v-if="rightPane"
      side="right"
      :title="rightPane.title"
      :screen-key="screenKey"
      :default-width="rightPane.defaultWidth ?? 320"
      :min-width="rightPane.minWidth ?? 200"
      :max-width="rightPane.maxWidth ?? 700"
      :default-collapsed="rightPane.defaultCollapsed ?? false"
      :fixed-width="rightPane.fixedWidth">
      <slot name="right" />
    </SidePane>
  </div>
</template>

<script setup lang="ts">
import SidePane from "./SidePane.vue";

export interface PaneConfig {
  title: string;
  defaultWidth?: number;
  minWidth?: number;
  maxWidth?: number;
  defaultCollapsed?: boolean;
  /** When set, the pane is locked to this exact width with no resize or collapse. */
  fixedWidth?: number;
}

defineProps<{
  screenKey: string;
  leftPane?: PaneConfig;
  rightPane?: PaneConfig;
}>();
</script>
