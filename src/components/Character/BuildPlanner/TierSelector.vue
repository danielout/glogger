<template>
  <div class="inline-flex items-center rounded border border-border-default overflow-hidden shrink-0" @click.stop>
    <button
      v-for="tier in tiers"
      :key="tier.tier_id"
      type="button"
      class="px-1.5 py-0.5 text-[10px] cursor-pointer transition-colors border-r border-border-default last:border-r-0"
      :class="tier.tier_id === modelValue
        ? 'bg-accent-gold/20 text-accent-gold font-semibold'
        : 'bg-surface-elevated text-text-muted hover:bg-surface-hover hover:text-text-secondary'"
      :title="`Level ${tier.min_level}–${tier.max_level}`"
      @click="emit('update:modelValue', tier.tier_id)">
      {{ tier.min_level }}–{{ tier.max_level }}
    </button>
  </div>
</template>

<script setup lang="ts">
import type { TsysTierSummary } from '../../../types/buildPlanner'

defineProps<{
  tiers: TsysTierSummary[]
  modelValue: string
}>()

const emit = defineEmits<{
  'update:modelValue': [tierId: string]
}>()
</script>
