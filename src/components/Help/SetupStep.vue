<template>
  <div class="flex gap-3 px-3 py-3 bg-surface-base rounded border border-border-default">
    <span class="shrink-0 w-6 h-6 rounded-full bg-surface-dark border border-border-default flex items-center justify-center text-accent-gold text-xs font-bold">
      {{ number }}
    </span>
    <div class="flex flex-col gap-0.5">
      <span class="text-sm text-text-primary font-medium">{{ title }}</span>
      <span class="text-xs text-text-muted leading-relaxed"><slot /></span>
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  number: string
  title: string
}>()
</script>
