<template>
  <div class="bg-surface-dark border border-surface-card p-2 text-xs flex flex-col gap-1">
    <!-- Core stats -->
    <div v-if="stats.damage != null" class="flex gap-2">
      <span class="text-text-muted min-w-24">Damage:</span>
      <span class="text-text-secondary">{{ stats.damage }}</span>
    </div>
    <div v-if="stats.power_cost != null" class="flex gap-2">
      <span class="text-text-muted min-w-24">Power Cost:</span>
      <span class="text-text-secondary">{{ stats.power_cost }}</span>
    </div>
    <div v-if="stats.rage_cost != null" class="flex gap-2">
      <span class="text-text-muted min-w-24">Rage Cost:</span>
      <span class="text-text-secondary">{{ stats.rage_cost }}</span>
    </div>
    <div v-if="stats.range != null" class="flex gap-2">
      <span class="text-text-muted min-w-24">Range:</span>
      <span class="text-text-secondary">{{ stats.range }}m</span>
    </div>
    <div v-if="stats.accuracy != null" class="flex gap-2">
      <span class="text-text-muted min-w-24">Accuracy:</span>
      <span class="text-text-secondary">{{ stats.accuracy }}</span>
    </div>

    <!-- Attribute modifier arrays -->
    <template v-for="(label, key) in attributeLabels" :key="key">
      <div v-if="getAttrArray(key).length" class="flex flex-col gap-0.5 mt-0.5">
        <span class="text-text-dim text-[0.65rem]">{{ label }}:</span>
        <div class="flex flex-wrap gap-1 pl-2">
          <span
            v-for="attr in getAttrArray(key)"
            :key="attr"
            class="text-[0.65rem] px-1 py-0.5 bg-surface-card border border-border-subtle text-text-secondary font-mono">
            {{ attr }}
          </span>
        </div>
      </div>
    </template>

    <!-- Extra fields -->
    <div v-if="hasExtra" class="mt-1 border-t border-surface-card pt-1">
      <span class="text-text-dim text-[0.65rem]">Other:</span>
      <pre class="text-[0.65rem] text-text-muted mt-0.5 m-0 whitespace-pre-wrap">{{ JSON.stringify(stats.extra, null, 2) }}</pre>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { CombatStats } from '../../types/gameData/abilities'

const props = defineProps<{
  stats: CombatStats
}>()

const attributeLabels: Record<string, string> = {
  attributes_that_delta_damage: 'Delta Damage',
  attributes_that_mod_base_damage: 'Mod Base Damage',
  attributes_that_mod_damage: 'Mod Damage',
  attributes_that_mod_crit_damage: 'Mod Crit Damage',
  attributes_that_delta_power_cost: 'Delta Power Cost',
  attributes_that_mod_power_cost: 'Mod Power Cost',
  attributes_that_delta_rage: 'Delta Rage',
  attributes_that_mod_rage: 'Mod Rage',
  attributes_that_delta_taunt: 'Delta Taunt',
  attributes_that_mod_taunt: 'Mod Taunt',
}

function getAttrArray(key: string): string[] {
  return (props.stats as unknown as Record<string, unknown>)[key] as string[] ?? []
}

const hasExtra = computed(() => {
  return props.stats.extra && Object.keys(props.stats.extra).length > 0
})
</script>
