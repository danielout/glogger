<template>
  <div class="bg-surface-elevated border border-border-default rounded px-3 py-2">
    <div class="text-[10px] uppercase tracking-wider text-text-secondary">
      {{ label }}
    </div>
    <div class="text-base text-text-primary font-semibold tabular-nums" :class="valueClass">
      {{ value }}
    </div>
    <div v-if="sub" class="text-[0.6rem] text-text-dim tabular-nums">
      {{ sub }}
    </div>
  </div>
</template>

<script setup lang="ts">
// Standard stat tile for the Surveying screens. Mirrors StallTracker's
// StatCard in spacing/typography so the two feel consistent, and adds an
// optional "accent" knob for values that should render in the app's
// green/red tokens (e.g. profit numbers).
import { computed } from 'vue'

const props = defineProps<{
  label: string
  value: string | number
  /** Small hint below the value — e.g. "+500g/hr", "3 / 50 maps". */
  sub?: string
  /** Color the value — picks up the app's accent tokens. */
  accent?: 'positive' | 'negative'
}>()

const valueClass = computed(() => {
  if (props.accent === 'positive') return 'text-accent-green'
  if (props.accent === 'negative') return 'text-accent-red'
  return ''
})
</script>
