<template>
  <div class="flex flex-col gap-2">
    <h3 class="text-xs font-semibold text-text-muted uppercase tracking-wider">Ability Bars</h3>

    <div class="flex flex-col gap-1">
      <button
        v-for="bar in bars"
        :key="bar.id"
        class="flex items-center justify-between px-2 py-1.5 rounded border text-xs cursor-pointer transition-all"
        :class="barClasses(bar.id)"
        @click="store.selectBar(bar.id)">
        <span class="font-medium">{{ barLabel(bar.id) }}</span>
        <span class="text-[10px]" :class="fillColor(bar.id)">
          {{ store.barAbilityCounts[bar.id] }}/{{ bar.slots }}
        </span>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useBuildPlannerStore } from '../../../stores/buildPlannerStore'
import { ABILITY_BARS } from '../../../types/buildPlanner'

const store = useBuildPlannerStore()
const bars = ABILITY_BARS

function barLabel(barId: string): string {
  if (barId === 'primary') return store.activePreset?.skill_primary ?? 'Primary'
  if (barId === 'secondary') return store.activePreset?.skill_secondary ?? 'Secondary'
  return 'Sidebar'
}

function barClasses(barId: string): string {
  const isSelected = store.activeBar === barId
  if (isSelected) {
    return 'bg-accent-gold/20 border-accent-gold/60 text-accent-gold'
  }
  const count = store.barAbilityCounts[barId as keyof typeof store.barAbilityCounts] ?? 0
  const max = ABILITY_BARS.find(b => b.id === barId)?.slots ?? 6
  if (count >= max) {
    return 'bg-green-900/15 border-green-700/30 text-text-primary hover:bg-green-900/25'
  }
  if (count > 0) {
    return 'bg-yellow-900/15 border-yellow-700/30 text-text-primary hover:bg-yellow-900/25'
  }
  return 'bg-surface-elevated border-border-default text-text-secondary hover:bg-surface-hover'
}

function fillColor(barId: string): string {
  const count = store.barAbilityCounts[barId as keyof typeof store.barAbilityCounts] ?? 0
  const max = ABILITY_BARS.find(b => b.id === barId)?.slots ?? 6
  if (count >= max) return 'text-green-400'
  if (count > 0) return 'text-yellow-400'
  return 'text-text-dim'
}
</script>
