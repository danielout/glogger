<template>
  <div class="flex gap-2.5 px-3 py-2.5 bg-surface-base rounded border border-border-default">
    <span class="shrink-0 mt-0.5" :class="severityClass">{{ severityIcon }}</span>
    <div class="flex flex-col gap-0.5">
      <span class="text-sm text-text-primary font-medium">{{ issue.title }}</span>
      <span class="text-xs text-text-muted leading-relaxed">{{ issue.description }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  issue: {
    severity: 'bug' | 'limitation' | 'cosmetic'
    title: string
    description: string
  }
}>()

const severityIcon = computed(() => {
  switch (props.issue.severity) {
    case 'bug': return '!'
    case 'limitation': return '~'
    case 'cosmetic': return '*'
  }
})

const severityClass = computed(() => {
  switch (props.issue.severity) {
    case 'bug': return 'text-accent-red'
    case 'limitation': return 'text-accent-warning'
    case 'cosmetic': return 'text-accent-blue'
  }
})
</script>
