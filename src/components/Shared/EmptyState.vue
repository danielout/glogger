<template>
  <div
    :class="variant === 'panel'
      ? 'flex flex-col items-center justify-center h-full gap-1'
      : 'flex items-center gap-2'">
    <div :class="variant === 'panel' ? 'text-sm text-text-secondary' : 'text-xs text-text-secondary italic'">
      {{ primary }}
    </div>
    <div v-if="secondary" :class="variant === 'panel' ? 'text-xs text-text-muted' : 'text-xs text-text-muted'">
      {{ secondary }}
    </div>
  </div>
</template>

<script setup lang="ts">
withDefaults(defineProps<{
  primary: string;
  secondary?: string;
  variant?: "panel" | "compact";
}>(), {
  variant: "panel",
});
</script>
