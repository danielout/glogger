<template>
  <div class="flex gap-3 px-4 py-3 bg-surface-base/60 rounded-lg border border-border-default hover:border-border-light transition-colors">
    <span
      class="shrink-0 mt-0.5 w-5 h-5 rounded flex items-center justify-center text-xs font-bold"
      :class="badgeClass">
      {{ severityIcon }}
    </span>
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

const badgeClass = computed(() => {
  switch (props.issue.severity) {
    case 'bug': return 'bg-accent-red/15 text-accent-red border border-accent-red/30'
    case 'limitation': return 'bg-accent-warning/15 text-accent-warning border border-accent-warning/30'
    case 'cosmetic': return 'bg-accent-blue/15 text-accent-blue border border-accent-blue/30'
  }
})
</script>
