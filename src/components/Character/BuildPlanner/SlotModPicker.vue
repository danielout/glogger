<template>
  <div class="flex flex-col gap-3 h-full min-h-0">
    <!-- Slot header -->
    <div class="flex items-center justify-between px-1">
      <div class="flex items-center gap-2">
        <h3 class="text-sm font-semibold text-text-primary">{{ slotLabel }}</h3>
        <span class="text-xs text-text-muted">
          {{ store.activePreset?.target_rarity }}
          ({{ store.slotModCounts[store.selectedSlot!] ?? 0 }}/{{ store.maxModsPerSlot }} mods
          <span v-if="store.slotHasAugment[store.selectedSlot!]">+ augment</span>)
        </span>
      </div>
    </div>

    <!-- Base item picker -->
    <SlotItemPicker />

    <div class="flex-1 flex gap-3 min-h-0">
      <!-- Assigned mods -->
      <div class="w-72 shrink-0 flex flex-col gap-1.5 min-h-0 overflow-y-auto">
        <h4 class="text-xs font-semibold text-text-muted uppercase tracking-wider">Selected Mods</h4>
        <div v-if="store.selectedSlotMods.length === 0" class="text-xs text-text-dim py-2">
          No mods assigned yet. Browse and add from the right.
        </div>
        <ModAssignment
          v-for="mod in store.selectedSlotMods"
          :key="mod.id"
          :mod="mod"
          @remove="store.removeMod(mod)" />
      </div>

      <!-- Available mods browser -->
      <div class="flex-1 flex flex-col gap-2 min-h-0">
        <div class="flex items-center gap-2">
          <h4 class="text-xs font-semibold text-text-muted uppercase tracking-wider">Available Mods</h4>
          <input
            v-model="store.modFilter"
            type="text"
            placeholder="Filter mods..."
            class="bg-surface-elevated border border-border-default rounded px-2 py-0.5 text-xs text-text-primary flex-1 max-w-60" />
          <span class="text-[10px] text-text-dim">{{ store.filteredPowers.length }} mods</span>
        </div>

        <div v-if="store.loadingPowers" class="text-xs text-text-muted py-4 text-center">
          Loading mods...
        </div>

        <div v-else class="flex-1 overflow-y-auto flex flex-col gap-1">
          <!-- Group by skill -->
          <template v-for="group in groupedPowers" :key="group.label">
            <div class="sticky top-0 bg-surface-base py-1 z-10">
              <h5 class="text-[10px] font-semibold uppercase tracking-wider"
                :class="group.labelClass">
                {{ group.label }} ({{ group.powers.length }})
              </h5>
            </div>
            <ModOption
              v-for="power in group.powers"
              :key="power.key"
              :power="power"
              @add="(isAug) => store.addMod(power, isAug)" />
          </template>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useBuildPlannerStore } from '../../../stores/buildPlannerStore'
import { EQUIPMENT_SLOTS } from '../../../types/buildPlanner'
import type { SlotTsysPower } from '../../../types/buildPlanner'
import ModAssignment from './ModAssignment.vue'
import ModOption from './ModOption.vue'
import SlotItemPicker from './SlotItemPicker.vue'

const store = useBuildPlannerStore()

const slotLabel = computed(() => {
  return EQUIPMENT_SLOTS.find(s => s.id === store.selectedSlot)?.label ?? store.selectedSlot ?? ''
})

interface PowerGroup {
  label: string
  labelClass: string
  powers: SlotTsysPower[]
}

const groupedPowers = computed((): PowerGroup[] => {
  const primary = store.activePreset?.skill_primary
  const secondary = store.activePreset?.skill_secondary
  const powers = store.filteredPowers

  const groups: PowerGroup[] = []

  if (primary) {
    const primaryPowers = powers.filter(p => p.skill === primary)
    if (primaryPowers.length > 0) {
      groups.push({
        label: `${primary} Mods`,
        labelClass: 'text-blue-400',
        powers: primaryPowers,
      })
    }
  }

  if (secondary) {
    const secondaryPowers = powers.filter(p => p.skill === secondary)
    if (secondaryPowers.length > 0) {
      groups.push({
        label: `${secondary} Mods`,
        labelClass: 'text-emerald-400',
        powers: secondaryPowers,
      })
    }
  }

  const genericPowers = powers.filter(p => !p.skill)
  if (genericPowers.length > 0) {
    groups.push({
      label: 'Generic Mods',
      labelClass: 'text-text-muted',
      powers: genericPowers,
    })
  }

  // Any remaining powers from other skills
  const coveredSkills = new Set([primary, secondary, null])
  const otherPowers = powers.filter(p => !coveredSkills.has(p.skill ?? null))
  if (otherPowers.length > 0) {
    groups.push({
      label: 'Other Mods',
      labelClass: 'text-text-dim',
      powers: otherPowers,
    })
  }

  return groups
})
</script>
