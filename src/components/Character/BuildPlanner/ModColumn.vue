<template>
  <div class="flex-1 flex flex-col gap-1.5 min-h-0">
    <!-- Column header with skill dropdown and fill indicator -->
    <div class="flex items-center gap-2">
      <h4 class="text-xs font-semibold uppercase tracking-wider shrink-0" :class="labelClass">
        {{ columnLabel }}
      </h4>
      <span
        class="text-xs font-semibold px-1.5 py-0.5 rounded"
        :class="fillBadgeClass">
        {{ assignedMods.length }}/{{ slotCount }}
      </span>
    </div>

    <StyledSelect
      :model-value="columnSkill"
      :options="availableSkills"
      size="xs"
      :color-class="labelClass"
      full-width
      @update:model-value="(val: string) => emit('update:columnSkill', val)" />

    <!-- Assigned mods (pinned to top with distinct styling) -->
    <div v-if="assignedMods.length > 0" class="flex flex-col gap-1">
      <ModAssignment
        v-for="mod in assignedMods"
        :key="mod.id"
        :mod="mod"
        @remove="emit('remove', mod)" />
    </div>

    <!-- Full indicator -->
    <div v-if="isFull && columnSkill" class="text-xs text-yellow-400/80 bg-yellow-900/15 border border-yellow-700/20 rounded px-2 py-1 text-center">
      Column full ({{ slotCount }}/{{ slotCount }} slots used)
    </div>

    <!-- Divider between assigned and available -->
    <div v-if="assignedMods.length > 0 && availablePowers.length > 0 && !isFull" class="border-t border-border-default/50" />

    <!-- Available (unassigned) mods -->
    <div class="flex-1 overflow-y-auto flex flex-col gap-1">
      <div v-if="!columnSkill" class="text-xs text-text-secondary py-2 text-center">
        Select a skill to browse mods
      </div>
      <div v-else-if="availablePowers.length === 0 && assignedMods.length === 0" class="text-xs text-text-secondary py-2 text-center">
        No mods available for this skill/slot
      </div>
      <template v-else-if="!isFull">
        <ModOption
          v-for="power in availablePowers"
          :key="power.key"
          :power="power"
          :compact="compact"
          :disabled="isModBlocked(power)"
          :disabled-reason="getModBlockedReason(power)"
          @add="(_isAugment: boolean, tierId?: string) => emit('add', power, tierId)" />
      </template>
      <!-- When full, show remaining mods dimmed -->
      <template v-else>
        <div
          v-for="power in availablePowers"
          :key="power.key"
          class="flex items-start gap-2 px-2 py-1.5 rounded text-sm border bg-surface-elevated border-border-default opacity-35 cursor-not-allowed">
          <GameIcon v-if="power.icon_id" :icon-id="power.icon_id" size="xs" class="shrink-0 mt-0.5" />
          <div class="flex-1 min-w-0">
            <span class="font-medium text-text-primary truncate">{{ getPowerDisplayName(power) }}</span>
          </div>
        </div>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { SlotTsysPower, BuildPresetMod } from '../../../types/buildPlanner'
import { getPowerDisplayName } from '../../../types/buildPlanner'
import { useBuildPlannerStore } from '../../../stores/buildPlannerStore'
import ModAssignment from './ModAssignment.vue'
import ModOption from './ModOption.vue'
import GameIcon from '../../Shared/GameIcon.vue'
import StyledSelect from '../../Shared/StyledSelect.vue'

const props = defineProps<{
  columnLabel: string
  columnSkill: string
  availableSkills: { value: string; label: string }[]
  powers: SlotTsysPower[]
  assignedMods: BuildPresetMod[]
  labelClass: string
  slotCount: number
  compact?: boolean
}>()

const emit = defineEmits<{
  'update:columnSkill': [skill: string]
  add: [power: SlotTsysPower, tierId?: string]
  remove: [mod: BuildPresetMod]
}>()

const store = useBuildPlannerStore()

const isFull = computed(() => props.assignedMods.length >= props.slotCount)

const fillBadgeClass = computed(() => {
  if (props.slotCount === 0) return 'bg-surface-hover text-text-dim'
  if (props.assignedMods.length >= props.slotCount) return 'bg-green-900/30 text-green-400'
  if (props.assignedMods.length > 0) return 'bg-yellow-900/30 text-yellow-400'
  return 'bg-surface-hover text-text-muted'
})

/** Powers not already assigned as regular mods */
const availablePowers = computed(() => {
  const assignedNames = new Set(props.assignedMods.map(m => m.power_name))
  return props.powers.filter(p => !assignedNames.has(p.internal_name ?? p.key))
})

/** Check if a mod is blocked from being added (e.g. already used as augment) */
function isModBlocked(power: SlotTsysPower): boolean {
  const powerName = power.internal_name ?? power.key
  // Already assigned as augment on this slot
  return store.selectedSlotMods.some(m => m.power_name === powerName && m.is_augment)
}

function getModBlockedReason(power: SlotTsysPower): string | undefined {
  const powerName = power.internal_name ?? power.key
  if (store.selectedSlotMods.some(m => m.power_name === powerName && m.is_augment)) {
    return 'Already assigned as augment on this slot'
  }
  return undefined
}

</script>
