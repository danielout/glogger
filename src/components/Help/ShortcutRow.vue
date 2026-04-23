<template>
  <div class="flex items-center gap-3 px-3 py-2 bg-surface-base rounded border border-border-default">
    <kbd class="shrink-0 min-w-24 text-center text-xs text-accent-gold bg-surface-dark border border-border-default rounded px-2 py-1">
      {{ keys }}
    </kbd>
    <span class="text-sm text-text-secondary">{{ description }}</span>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  keys: string
  description: string
}>()
</script>
