<template>
  <div class="bg-surface-elevated border border-border-default rounded p-3">
    <div class="text-[10px] uppercase tracking-wider text-text-secondary">{{ label }}</div>
    <div class="text-xl text-text-primary font-medium tabular-nums">{{ value }}</div>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  label: string
  value: string
}>()
</script>
