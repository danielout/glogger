<template>
  <div class="flex flex-col gap-3 h-full min-h-0">
    <!-- Slot header -->
    <div class="flex items-center justify-between px-1">
      <div class="flex items-center gap-3">
        <h3 class="text-sm font-semibold text-text-primary">{{ slotLabel }}</h3>
        <span class="text-xs text-text-muted">
          {{ store.getSlotRarity(store.selectedSlot!) }} Lv{{ store.getSlotLevel(store.selectedSlot!) }}
        </span>
        <span
          class="text-xs font-semibold px-1.5 py-0.5 rounded"
          :class="totalModCount >= store.maxModsPerSlot
            ? 'bg-green-900/30 text-green-400'
            : totalModCount > 0
              ? 'bg-yellow-900/30 text-yellow-400'
              : 'bg-surface-hover text-text-muted'">
          {{ totalModCount }}/{{ store.maxModsPerSlot }} mods
        </span>
        <span
          v-if="store.slotHasAugment[store.selectedSlot!]"
          class="text-xs font-semibold px-1.5 py-0.5 rounded bg-purple-900/30 text-purple-400">
          +1 augment
        </span>
      </div>
    </div>

    <!-- Base item picker (accordion) -->
    <div>
      <button
        class="w-full flex items-center gap-1.5 text-xs font-semibold text-text-muted uppercase tracking-wider cursor-pointer hover:text-text-secondary"
        @click="showItemPicker = !showItemPicker">
        <span class="transition-transform text-[10px]" :class="showItemPicker ? 'rotate-90' : ''">▶</span>
        Base Item
        <span v-if="currentItemName" class="normal-case font-normal text-entity-item ml-1 truncate">— {{ currentItemName }}</span>
      </button>
      <div v-if="showItemPicker" class="mt-1.5">
        <SlotItemPicker />
      </div>
    </div>

    <!-- Per-slot skill assignment (these define what skills the item actually has) -->
    <div class="flex items-center gap-2 px-1">
      <label class="text-[10px] text-text-muted uppercase tracking-wider shrink-0">Item Skills:</label>
      <StyledSelect
        :model-value="store.getSlotSkillPrimary(store.selectedSlot!) ?? ''"
        :options="slotSkillPrimaryOptions"
        size="xs"
        color-class="text-blue-400"
        @update:model-value="onSlotSkillPrimaryChange" />
      <StyledSelect
        :model-value="store.getSlotSkillSecondary(store.selectedSlot!) ?? ''"
        :options="slotSkillSecondaryOptions"
        size="xs"
        color-class="text-emerald-400"
        @update:model-value="onSlotSkillSecondaryChange" />
    </div>

    <!-- Filter bar -->
    <div class="flex items-center gap-2 px-1">
      <input
        v-model="store.modFilter"
        type="text"
        placeholder="Filter mods..."
        class="bg-surface-elevated border border-border-default rounded px-2 py-0.5 text-xs text-text-primary flex-1 max-w-60" />
      <label
        v-if="store.presetAbilities.length > 0"
        class="flex items-center gap-1 text-[10px] text-text-muted cursor-pointer shrink-0"
        title="Hide mods that boost abilities not in your build">
        <input
          v-model="hideUnusedAbilityMods"
          type="checkbox"
          class="w-2.5 h-2.5 cursor-pointer" />
        <span>My abilities</span>
      </label>
      <label
        class="flex items-center gap-1 text-[10px] text-text-muted cursor-pointer shrink-0"
        title="Hide effect details for a more compact view">
        <input
          v-model="compactMode"
          type="checkbox"
          class="w-2.5 h-2.5 cursor-pointer" />
        <span>Compact</span>
      </label>
      <span class="text-[10px] text-text-muted">{{ store.filteredPowers.length }} mods</span>
    </div>

    <!-- Loading state -->
    <div v-if="store.loadingPowers" class="text-xs text-text-muted py-4 text-center">
      Loading mods...
    </div>

    <!-- 3-column mod layout -->
    <div v-else class="flex-1 flex gap-3 min-h-0">
      <!-- Primary Skill Column -->
      <ModColumn
        v-model:column-skill="colPrimarySkill"
        :available-skills="columnSkillOptions"
        :powers="primaryColumnPowers"
        :assigned-mods="primaryColumnMods"
        label-class="text-blue-400"
        :slot-count="rarityDef.primarySlots"
        :compact="compactMode"
        column-label="Primary Skill"
        @add="(power, tierId) => store.addMod(power, false, tierId)"
        @remove="(mod) => store.removeMod(mod)" />

      <!-- Secondary Skill Column -->
      <ModColumn
        v-model:column-skill="colSecondarySkill"
        :available-skills="columnSkillOptions"
        :powers="secondaryColumnPowers"
        :assigned-mods="secondaryColumnMods"
        :compact="compactMode"
        label-class="text-emerald-400"
        :slot-count="rarityDef.secondarySlots"
        column-label="Secondary Skill"
        @add="(power, tierId) => store.addMod(power, false, tierId)"
        @remove="(mod) => store.removeMod(mod)" />

      <!-- Craft Points Column -->
      <div class="flex-1 flex flex-col gap-1.5 min-h-0 border-l border-border-default/50 pl-3">
        <div class="flex items-center justify-between">
          <h4 class="text-xs font-semibold text-amber-400 uppercase tracking-wider">Craft Points</h4>
          <span class="text-xs" :class="cpRemaining < 0 ? 'text-red-400 font-semibold' : 'text-text-muted'">
            {{ cpRemaining }}/{{ cpBudget }} CP
          </span>
        </div>

        <!-- Augment section -->
        <div class="flex flex-col gap-1">
          <div class="flex items-center gap-2">
            <span class="text-[10px] text-text-muted uppercase tracking-wider">Augment ({{ AUGMENT_CP_COST }} CP)</span>
          </div>

          <!-- Current augment -->
          <div v-if="currentAugment">
            <ModAssignment :mod="currentAugment" @remove="store.removeMod(currentAugment!)" />
          </div>
          <div v-else class="text-xs text-text-secondary py-1">
            No augment assigned. Pick from any skill below.
          </div>
        </div>

        <!-- Augment skill filter -->
        <div class="flex items-center gap-2">
          <StyledSelect
            v-model="augmentSkillFilter"
            :options="augmentSkillOptions"
            size="xs"
            full-width />
        </div>

        <!-- Available mods for augmenting -->
        <div class="flex-1 overflow-y-auto flex flex-col gap-1">
          <ModOption
            v-for="power in augmentPowers"
            :key="power.key"
            :power="power"
            :augment-only="true"
            @add="(_isAugment: boolean, tierId?: string) => store.addMod(power, true, tierId)" />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useBuildPlannerStore } from '../../../stores/buildPlannerStore'
import { EQUIPMENT_SLOTS, getRarityDef, AUGMENT_CP_COST, getSlotCraftingPoints } from '../../../types/buildPlanner'
import ModAssignment from './ModAssignment.vue'
import ModOption from './ModOption.vue'
import ModColumn from './ModColumn.vue'
import SlotItemPicker from './SlotItemPicker.vue'
import StyledSelect from '../../Shared/StyledSelect.vue'
import { useBuildCrossRef } from '../../../composables/useBuildCrossRef'

const store = useBuildPlannerStore()

const { abilityBaseNames, isAbilityRelated } = useBuildCrossRef()

const showItemPicker = ref(false)
const augmentSkillFilter = ref('')
const hideUnusedAbilityMods = ref(false)
const compactMode = ref(false)

// Local column skill state — defaults from slot skills, user can change freely
// These are browsing filters, NOT persisted. Use '__generic__' for generic mods.
const colPrimarySkill = ref('')
const colSecondarySkill = ref('')

const slotLabel = computed(() => {
  return EQUIPMENT_SLOTS.find(s => s.id === store.selectedSlot)?.label ?? store.selectedSlot ?? ''
})

const currentItemName = computed(() => {
  if (!store.selectedSlot) return null
  const item = store.getSlotItem(store.selectedSlot)
  if (!item || item.item_id === 0) return null
  return item.item_name ?? 'Unknown Item'
})

const totalModCount = computed(() => {
  return store.slotModCounts[store.selectedSlot!] ?? 0
})

const rarityDef = computed(() => {
  if (!store.selectedSlot) return getRarityDef('Epic')
  return getRarityDef(store.getSlotRarity(store.selectedSlot))
})

/** Skill options for column dropdowns: all combat skills + Generic + Endurance */
const columnSkillOptions = computed(() => {
  const options: { value: string; label: string }[] = [
    { value: '', label: 'None' },
    { value: '__generic__', label: 'Generic' },
    { value: 'Endurance', label: 'Endurance' },
  ]
  for (const skill of store.combatSkills) {
    if (skill.name === 'Endurance') continue // already added above
    options.push({ value: skill.name, label: skill.name })
  }
  return options
})

const slotSkillPrimaryOptions = computed(() => [
  { value: '', label: store.activePreset?.skill_primary ? `${store.activePreset.skill_primary} (default)` : 'None' },
  ...store.combatSkills.map(s => ({ value: s.name, label: s.name })),
])

const slotSkillSecondaryOptions = computed(() => [
  { value: '', label: store.activePreset?.skill_secondary ? `${store.activePreset.skill_secondary} (default)` : 'None' },
  ...store.combatSkills.map(s => ({ value: s.name, label: s.name })),
])

const augmentSkillOptions = computed(() => [
  { value: '', label: 'All Skills' },
  ...columnSkillOptions.value.filter(o => o.value !== ''),
])

/** Initialize column skills from slot skills when slot changes */
watch(() => store.selectedSlot, () => {
  showItemPicker.value = false
  augmentSkillFilter.value = ''
  syncColumnSkills()
}, { immediate: true })

// Also re-sync when slot powers reload (skills may have changed)
watch(() => store.slotPowers, () => {
  // Only re-sync if columns are still at defaults
}, { deep: false })

function syncColumnSkills() {
  if (!store.selectedSlot) {
    colPrimarySkill.value = ''
    colSecondarySkill.value = ''
    return
  }
  colPrimarySkill.value = store.getSlotSkillPrimary(store.selectedSlot) ?? ''
  colSecondarySkill.value = store.getSlotSkillSecondary(store.selectedSlot) ?? ''
}

/** Check if a power's skill is "generic" (AnySkill or null/undefined) */
function isGenericPower(power: { skill: string | null }): boolean {
  return !power.skill || power.skill === 'AnySkill'
}

// Filter powers for each column
function powersForSkill(skill: string) {
  if (!skill) return []
  const powers = store.filteredPowers
  if (skill === '__generic__') {
    return powers.filter(p => isGenericPower(p))
  }
  return powers.filter(p => p.skill === skill)
}

/**
 * When "My abilities" filter is active, sort powers that reference assigned abilities
 * to the top of the list. All powers remain visible.
 */
function applyAbilityFilter(powers: typeof store.slotPowers) {
  if (!hideUnusedAbilityMods.value || abilityBaseNames.value.size === 0) return powers
  return [...powers].sort((a, b) => {
    const aRelated = isAbilityRelated(a.internal_name ?? a.key) ? 0 : 1
    const bRelated = isAbilityRelated(b.internal_name ?? b.key) ? 0 : 1
    return aRelated - bRelated
  })
}

const primaryColumnPowers = computed(() => applyAbilityFilter(powersForSkill(colPrimarySkill.value)))
const secondaryColumnPowers = computed(() => applyAbilityFilter(powersForSkill(colSecondarySkill.value)))

/** Match assigned mods to columns by checking which skill the power belongs to */
function modsForSkill(skill: string) {
  if (!store.selectedSlot || !skill) return []
  return store.selectedSlotMods.filter(m => {
    if (m.is_augment) return false
    const power = store.slotPowers.find(p => (p.internal_name ?? p.key) === m.power_name)
    if (skill === '__generic__') {
      return power ? isGenericPower(power) : false
    }
    return power?.skill === skill
  })
}

const primaryColumnMods = computed(() => modsForSkill(colPrimarySkill.value))
const secondaryColumnMods = computed(() => modsForSkill(colSecondarySkill.value))

/** Current augment on this slot */
const currentAugment = computed(() => {
  return store.selectedSlotMods.find(m => m.is_augment) ?? null
})

/** Craft points budget and remaining */
const cpBudget = computed(() => {
  if (!store.selectedSlot) return 0
  return getSlotCraftingPoints(store.getSlotItem(store.selectedSlot))
})

const cpRemaining = computed(() => {
  let used = 0
  if (currentAugment.value) used += AUGMENT_CP_COST
  return cpBudget.value - used
})

/** Powers available for augmenting, filtered by the augment skill dropdown */
const augmentPowers = computed(() => {
  let powers = store.filteredPowers
  if (augmentSkillFilter.value) {
    if (augmentSkillFilter.value === '__generic__') {
      powers = powers.filter(p => isGenericPower(p))
    } else {
      powers = powers.filter(p => p.skill === augmentSkillFilter.value)
    }
  }
  return applyAbilityFilter(powers)
})

async function onSlotSkillPrimaryChange(val: string) {
  if (!store.selectedSlot) return
  await store.updateSlotProps(store.selectedSlot, {
    slot_skill_primary: val || null,
  })
  colPrimarySkill.value = val || store.activePreset?.skill_primary || ''
}

async function onSlotSkillSecondaryChange(val: string) {
  if (!store.selectedSlot) return
  await store.updateSlotProps(store.selectedSlot, {
    slot_skill_secondary: val || null,
  })
  colSecondarySkill.value = val || store.activePreset?.skill_secondary || ''
}
</script>
