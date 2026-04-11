<template>
  <div class="flex items-center gap-3 text-xs text-text-muted">
    <!-- Slots configured -->
    <span :class="slotsConfigured === totalSlots ? 'text-green-400' : ''">
      {{ slotsConfigured }}/{{ totalSlots }} slots
    </span>

    <!-- Mods assigned -->
    <span :class="totalMods === totalModCapacity ? 'text-green-400' : ''">
      {{ totalMods }}/{{ totalModCapacity }} mods
    </span>

    <!-- Ability bars -->
    <span :class="barsConfigured === totalBars ? 'text-green-400' : ''">
      {{ barsConfigured }}/{{ totalBars }} bars
    </span>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useBuildPlannerStore } from '../../../stores/buildPlannerStore'
import { EQUIPMENT_SLOTS, ABILITY_BARS } from '../../../types/buildPlanner'

const store = useBuildPlannerStore()

const totalSlots = EQUIPMENT_SLOTS.length

const slotsConfigured = computed(() => {
  let count = 0
  for (const slot of EQUIPMENT_SLOTS) {
    if ((store.slotModCounts[slot.id] ?? 0) > 0 || store.slotHasAugment[slot.id]) {
      count++
    }
  }
  return count
})

const totalMods = computed(() => {
  let count = 0
  for (const slot of EQUIPMENT_SLOTS) {
    count += store.slotModCounts[slot.id] ?? 0
  }
  return count
})

const totalModCapacity = computed(() => {
  let count = 0
  for (const slot of EQUIPMENT_SLOTS) {
    count += store.getMaxModsForSlot(slot.id)
  }
  return count
})

const totalBars = ABILITY_BARS.length

const barsConfigured = computed(() => {
  let count = 0
  for (const bar of ABILITY_BARS) {
    if ((store.barAbilityCounts[bar.id] ?? 0) > 0) {
      count++
    }
  }
  return count
})
</script>
