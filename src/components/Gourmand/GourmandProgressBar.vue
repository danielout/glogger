<template>
  <div class="flex items-center gap-2">
    <span class="text-text-secondary text-xs whitespace-nowrap min-w-20">{{ label }}</span>
    <div class="flex-1 h-3 bg-surface-dark rounded overflow-hidden border border-border-default">
      <div
        class="h-full bg-accent-green transition-all duration-300"
        :style="{ width: `${percentage}%` }"
      />
    </div>
    <span class="text-text-primary text-xs whitespace-nowrap min-w-20 text-right">
      {{ eaten }}/{{ total }} ({{ percentage.toFixed(0) }}%)
    </span>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  label: string
  eaten: number
  total: number
}>()

const percentage = computed(() => props.total > 0 ? (props.eaten / props.total) * 100 : 0)
</script>
