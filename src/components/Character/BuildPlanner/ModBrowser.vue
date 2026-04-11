<template>
  <div class="flex flex-col gap-2 min-h-0 h-full">
    <!-- Filter controls -->
    <div class="flex flex-col gap-1.5 shrink-0">
      <div class="flex items-center gap-2">
        <StyledSelect
          v-model="skillFilter"
          :options="skillFilterOptions"
          size="xs"
          class="flex-1 min-w-0" />
        <label
          v-if="store.presetAbilities.length > 0"
          class="flex items-center gap-1 text-[10px] text-text-muted cursor-pointer shrink-0"
          title="Show only mods that affect your assigned abilities">
          <input
            v-model="myAbilitiesFilter"
            type="checkbox"
            class="w-2.5 h-2.5 cursor-pointer" />
          <span>My abilities</span>
        </label>
      </div>
      <div class="flex items-center gap-2">
        <input
          v-model="store.modFilter"
          type="text"
          placeholder="Filter mods..."
          class="bg-surface-elevated border border-border-default rounded px-2 py-0.5 text-xs text-text-primary flex-1 min-w-0" />
        <label
          class="flex items-center gap-1 text-[10px] text-text-muted cursor-pointer shrink-0"
          title="Hide effect details">
          <input
            v-model="compactMode"
            type="checkbox"
            class="w-2.5 h-2.5 cursor-pointer" />
          <span>Compact</span>
        </label>
      </div>
      <div class="text-[10px] text-text-dim">
        {{ filteredPowers.length }} mods available
      </div>
    </div>

    <!-- Loading -->
    <div v-if="store.loadingPowers" class="text-xs text-text-muted py-4 text-center">
      Loading mods...
    </div>

    <!-- Mod list -->
    <div v-else class="flex-1 overflow-y-auto flex flex-col gap-1">
      <ModBrowserItem
        v-for="power in filteredPowers"
        :key="power.key"
        :power="power"
        :compact="compactMode"
        @add="(tierId) => addMod(power, tierId)"
        @add-augment="(tierId) => addAugment(power, tierId)" />

      <div v-if="filteredPowers.length === 0" class="text-xs text-text-dim text-center py-4">
        No mods match your filters.
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useBuildPlannerStore } from '../../../stores/buildPlannerStore'
import type { SlotTsysPower } from '../../../types/buildPlanner'
import { computeSlotConstraints } from '../../../types/buildPlanner'
import { useBuildCrossRef } from '../../../composables/useBuildCrossRef'
import StyledSelect from '../../Shared/StyledSelect.vue'
import ModBrowserItem from './ModBrowserItem.vue'

const store = useBuildPlannerStore()
const { abilityBaseNames, isAbilityRelated } = useBuildCrossRef()

const skillFilter = ref('__build__')
const myAbilitiesFilter = ref(false)
const compactMode = ref(false)

const skillFilterOptions = computed(() => {
  const primary = store.selectedSlot ? store.getSlotSkillPrimary(store.selectedSlot) : store.activePreset?.skill_primary
  const secondary = store.selectedSlot ? store.getSlotSkillSecondary(store.selectedSlot) : store.activePreset?.skill_secondary
  const buildLabel = [primary, secondary].filter(Boolean).join(' + ') || 'Build Skills'

  const options: { value: string; label: string }[] = [
    { value: '__build__', label: `${buildLabel} + Generic` },
    { value: '', label: 'All Skills' },
    { value: '__generic__', label: 'Generic Only' },
    { value: 'Endurance', label: 'Endurance Only' },
  ]
  if (primary) options.push({ value: primary, label: `${primary} Only` })
  if (secondary && secondary !== primary) options.push({ value: secondary, label: `${secondary} Only` })
  return options
})

// Reset filter when slot changes
watch(() => store.selectedSlot, () => {
  skillFilter.value = '__build__'
})

function isGenericPower(power: { skill: string | null }): boolean {
  return !power.skill || power.skill === 'AnySkill'
}

const filteredPowers = computed(() => {
  let powers = store.filteredPowers

  // Apply skill filter
  if (skillFilter.value === '__build__') {
    const primary = store.selectedSlot ? store.getSlotSkillPrimary(store.selectedSlot) : store.activePreset?.skill_primary
    const secondary = store.selectedSlot ? store.getSlotSkillSecondary(store.selectedSlot) : store.activePreset?.skill_secondary
    powers = powers.filter(p =>
      isGenericPower(p) ||
      p.skill === 'Endurance' ||
      p.skill === primary ||
      p.skill === secondary
    )
  } else if (skillFilter.value === '__generic__') {
    powers = powers.filter(p => isGenericPower(p))
  } else if (skillFilter.value) {
    powers = powers.filter(p => p.skill === skillFilter.value)
  }

  // Apply ability filter
  if (myAbilitiesFilter.value && abilityBaseNames.value.size > 0) {
    powers = powers.filter(p => isAbilityRelated(p.internal_name ?? p.key))
  }

  return powers
})

function addMod(power: SlotTsysPower, tierId?: string) {
  if (!store.selectedSlot) return

  const regularMods = store.selectedSlotMods.filter(m => !m.is_augment)
  if (regularMods.length >= store.maxModsPerSlot) return

  const powerName = power.internal_name ?? power.key
  const alreadyAssigned = store.selectedSlotMods.some(m => m.power_name === powerName)
  if (alreadyAssigned) return

  // Check skill constraints using the constraint solver
  const isGeneric = !power.skill || power.skill === 'AnySkill' || power.skill === 'Endurance'
  const rarity = store.getSlotRarity(store.selectedSlot)

  const skillCounts = new Map<string, number>()
  let genCount = 0
  for (const m of regularMods) {
    const p = store.slotPowers.find(sp => (sp.internal_name ?? sp.key) === m.power_name)
    const s = p?.skill
    if (!s || s === 'AnySkill' || s === 'Endurance') {
      genCount++
    } else {
      skillCounts.set(s, (skillCounts.get(s) ?? 0) + 1)
    }
  }

  const constraints = computeSlotConstraints(rarity, skillCounts, genCount)
  if (isGeneric) {
    if (!constraints.canAddGeneric) return
  } else if (skillCounts.has(power.skill!)) {
    if (!constraints.canAddSkillMod) return
  } else {
    if (!constraints.canAddNewSkill) return
  }

  store.addMod(power, false, tierId)
}

function addAugment(power: SlotTsysPower, tierId?: string) {
  if (!store.selectedSlot) return

  const powerName = power.internal_name ?? power.key
  const alreadyAssigned = store.selectedSlotMods.some(m => m.power_name === powerName)
  if (alreadyAssigned) return

  // Replace existing augment if present
  const existingAug = store.selectedSlotMods.find(m => m.is_augment)
  if (existingAug) store.removeMod(existingAug)

  store.addMod(power, true, tierId)
}
</script>
