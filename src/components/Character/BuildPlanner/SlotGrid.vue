<template>
  <div class="flex flex-col gap-2">
    <h3 class="text-xs font-semibold text-text-muted uppercase tracking-wider">Equipment Slots</h3>

    <div class="grid grid-cols-3 gap-1.5">
      <button
        v-for="slot in EQUIPMENT_SLOTS"
        :key="slot.id"
        class="flex flex-col items-center gap-0.5 px-2 py-2 rounded border text-xs cursor-pointer transition-all"
        :class="slotClasses(slot.id)"
        @click="store.selectSlot(slot.id)">
        <span class="font-medium">{{ slot.label }}</span>
        <span v-if="store.getSlotItem(slot.id)" class="text-[10px] text-entity-item truncate max-w-full">
          {{ store.getSlotItem(slot.id)?.item_name ?? 'Item' }}
        </span>
        <span class="text-[10px]" :class="fillColor(slot.id)">
          {{ store.slotModCounts[slot.id] ?? 0 }}/{{ store.maxModsPerSlot }}
          <span v-if="store.slotHasAugment[slot.id]" class="text-purple-400">+A</span>
        </span>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useBuildPlannerStore } from '../../../stores/buildPlannerStore'
import { EQUIPMENT_SLOTS } from '../../../types/buildPlanner'

const store = useBuildPlannerStore()

function slotClasses(slotId: string): string {
  const isSelected = store.selectedSlot === slotId
  if (isSelected) {
    return 'bg-accent-gold/20 border-accent-gold/60 text-accent-gold'
  }
  const count = store.slotModCounts[slotId] ?? 0
  if (count >= store.maxModsPerSlot) {
    return 'bg-green-900/15 border-green-700/30 text-text-primary hover:bg-green-900/25'
  }
  if (count > 0) {
    return 'bg-yellow-900/15 border-yellow-700/30 text-text-primary hover:bg-yellow-900/25'
  }
  return 'bg-surface-elevated border-border-default text-text-secondary hover:bg-surface-hover'
}

function fillColor(slotId: string): string {
  const count = store.slotModCounts[slotId] ?? 0
  if (count >= store.maxModsPerSlot) return 'text-green-400'
  if (count > 0) return 'text-yellow-400'
  return 'text-text-dim'
}
</script>
