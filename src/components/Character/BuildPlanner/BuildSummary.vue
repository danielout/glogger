<template>
  <div v-if="store.presetMods.length > 0" class="flex flex-col gap-1.5">
    <h3 class="text-xs font-semibold text-text-muted uppercase tracking-wider">Build Summary</h3>

    <div class="text-xs space-y-1">
      <!-- Stats per slot -->
      <div v-for="slot in slotsWithMods" :key="slot.id" class="flex items-center gap-1.5">
        <span class="text-text-muted w-16 shrink-0">{{ slot.label }}:</span>
        <span class="text-text-secondary">
          {{ modsForSlot(slot.id).length }} mod{{ modsForSlot(slot.id).length !== 1 ? 's' : '' }}
          <span v-if="hasAugment(slot.id)" class="text-purple-400">+aug</span>
        </span>
      </div>

      <!-- Totals -->
      <div class="border-t border-border-default pt-1 mt-1 flex items-center gap-1.5">
        <span class="text-text-muted w-16 shrink-0">Total:</span>
        <span class="text-accent-gold font-medium">
          {{ store.presetMods.filter(m => !m.is_augment).length }} mods,
          {{ store.presetMods.filter(m => m.is_augment).length }} augments
        </span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useBuildPlannerStore } from '../../../stores/buildPlannerStore'
import { EQUIPMENT_SLOTS } from '../../../types/buildPlanner'

const store = useBuildPlannerStore()

const slotsWithMods = computed(() => {
  return EQUIPMENT_SLOTS.filter(s =>
    store.presetMods.some(m => m.equip_slot === s.id)
  )
})

function modsForSlot(slotId: string) {
  return store.presetMods.filter(m => m.equip_slot === slotId && !m.is_augment)
}

function hasAugment(slotId: string) {
  return store.presetMods.some(m => m.equip_slot === slotId && m.is_augment)
}
</script>
