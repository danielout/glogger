<template>
  <div class="flex items-center gap-1.5" :class="sizeClasses.container">
    <!-- Label -->
    <span v-if="label" class="shrink-0" :class="sizeClasses.label">{{ label }}</span>

    <!-- Bar -->
    <div class="flex-1 rounded-full overflow-hidden" :class="[sizeClasses.bar, 'bg-surface-hover']">
      <div
        class="h-full rounded-full transition-all"
        :class="barColor"
        :style="{ width: `${percentage}%` }" />
    </div>

    <!-- Value text -->
    <span class="shrink-0 font-medium tabular-nums" :class="[sizeClasses.value, textColor]">
      {{ used }}/{{ budget }}
    </span>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  used: number
  budget: number
  label?: string
  size?: 'xs' | 'sm'
}>(), {
  label: '',
  size: 'xs',
})

const percentage = computed(() => {
  if (props.budget <= 0) return 0
  return Math.min(100, (props.used / props.budget) * 100)
})

const isOverBudget = computed(() => props.used > props.budget)
const isFull = computed(() => props.used === props.budget)

const barColor = computed(() => {
  if (isOverBudget.value) return 'bg-red-500/70'
  if (isFull.value) return 'bg-green-500/50'
  if (props.used > 0) return 'bg-amber-500/50'
  return 'bg-surface-hover'
})

const textColor = computed(() => {
  if (isOverBudget.value) return 'text-red-400'
  if (isFull.value) return 'text-green-400'
  return 'text-text-muted'
})

const sizeClasses = computed(() => {
  if (props.size === 'sm') {
    return {
      container: 'text-xs',
      label: 'text-xs text-text-muted',
      bar: 'h-1.5',
      value: 'text-xs',
    }
  }
  return {
    container: 'text-[10px]',
    label: 'text-[10px] text-text-dim',
    bar: 'h-1',
    value: 'text-[10px]',
  }
})
</script>
