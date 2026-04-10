<template>
  <div class="rounded border border-border-default bg-surface-elevated px-3 py-2 space-y-1.5">
    <!-- Ability name + effect count -->
    <div class="flex items-center gap-2">
      <span class="text-xs font-semibold text-text-primary">{{ abilityName }}</span>
      <span class="text-[10px] text-text-dim">{{ effects.length }} effect{{ effects.length !== 1 ? 's' : '' }}</span>
    </div>

    <!-- Aggregated effects -->
    <div class="space-y-0.5">
      <div
        v-for="(effect, i) in effects"
        :key="i"
        class="flex items-center gap-2">
        <EffectLine
          :label="effect.label"
          :formatted-value="effect.formattedValue"
          :numeric-value="effect.numericValue"
          :icon-id="effect.iconId"
          class="flex-1" />
        <span class="text-[10px] text-text-dim shrink-0">{{ effect.source }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import EffectLine from './EffectLine.vue'

interface AbilityEffect {
  label: string
  numericValue: number
  formattedValue: string
  iconId: number | null
  source: string
}

defineProps<{
  abilityName: string
  effects: AbilityEffect[]
}>()
</script>
