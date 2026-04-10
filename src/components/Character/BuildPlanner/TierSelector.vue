<template>
  <!-- Compact mode for many tiers: show selected + dropdown -->
  <div v-if="tiers.length > 5" class="inline-flex items-center shrink-0" @click.stop>
    <StyledSelect
      :model-value="modelValue"
      :options="tierOptions"
      size="xs"
      color-class="text-accent-gold"
      @update:model-value="emit('update:modelValue', $event)" />
  </div>

  <!-- Segmented control for few tiers -->
  <div v-else class="inline-flex items-center rounded border border-border-default overflow-hidden shrink-0" @click.stop>
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
import { computed } from 'vue'
import type { TsysTierSummary } from '../../../types/buildPlanner'
import StyledSelect from '../../Shared/StyledSelect.vue'

const props = defineProps<{
  tiers: TsysTierSummary[]
  modelValue: string
}>()

const emit = defineEmits<{
  'update:modelValue': [tierId: string]
}>()

const tierOptions = computed(() =>
  props.tiers.map(t => ({
    value: t.tier_id,
    label: `Lv ${t.min_level}–${t.max_level}`,
  }))
)
</script>
