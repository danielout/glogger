<template>
  <div class="flex h-full min-h-0 overflow-hidden">
    <SidePane
      v-if="leftPane"
      side="left"
      :title="leftPane.title"
      :screen-key="screenKey"
      :default-width="leftPane.defaultWidth ?? 320"
      :min-width="leftPane.minWidth ?? 200"
      :max-width="leftPane.maxWidth ?? 700">
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
      :max-width="rightPane.maxWidth ?? 700">
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
}

defineProps<{
  screenKey: string;
  leftPane?: PaneConfig;
  rightPane?: PaneConfig;
}>();
</script>
