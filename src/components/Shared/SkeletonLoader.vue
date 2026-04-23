<template>
  <!-- Text variant: multiple lines of varying width -->
  <div v-if="variant === 'text'" class="flex flex-col gap-2">
    <div
      v-for="i in lines"
      :key="i"
      class="animate-pulse rounded bg-surface-elevated"
      :class="[
        resolvedHeight,
        i === lines ? 'w-3/5' : resolvedWidth,
      ]" />
  </div>

  <!-- Circle variant: avatar/icon placeholder -->
  <div
    v-else-if="variant === 'circle'"
    class="animate-pulse rounded-full bg-surface-elevated"
    :class="[resolvedWidth, resolvedHeight]" />

  <!-- Rect variant: card/image placeholder -->
  <div
    v-else
    class="animate-pulse rounded bg-surface-elevated"
    :class="[resolvedWidth, resolvedHeight]" />
</template>

<script setup lang="ts">
import { computed } from "vue";

const props = withDefaults(defineProps<{
  variant?: "text" | "circle" | "rect";
  lines?: number;
  width?: string;
  height?: string;
}>(), {
  variant: "text",
  lines: 1,
});

const resolvedWidth = computed(() => {
  if (props.width) return props.width;
  switch (props.variant) {
    case "circle": return "w-10";
    case "rect": return "w-full";
    default: return "w-full";
  }
});

const resolvedHeight = computed(() => {
  if (props.height) return props.height;
  switch (props.variant) {
    case "circle": return "h-10";
    case "rect": return "h-20";
    default: return "h-4";
  }
});
</script>
