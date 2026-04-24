<template>
  <div class="flex gap-4 px-4 py-3.5 bg-surface-base/60 rounded-lg border border-border-default hover:border-border-light transition-colors">
    <span class="shrink-0 w-7 h-7 rounded-full bg-accent-gold/15 border border-accent-gold/30 flex items-center justify-center text-accent-gold text-xs font-bold">
      {{ number }}
    </span>
    <div class="flex flex-col gap-1">
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
