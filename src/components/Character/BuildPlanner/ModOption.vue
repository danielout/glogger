<template>
  <div
    class="flex items-start gap-2 px-2 py-1.5 rounded text-sm transition-all border"
    :class="rowClass"
    :title="disabledReason"
    @click="handleClick">
    <GameIcon v-if="power.icon_id" :icon-id="power.icon_id" size="xs" class="shrink-0 mt-0.5" />
    <div class="flex-1 min-w-0">
      <div class="flex items-center gap-1.5">
        <span
          v-if="power.skill && power.skill !== 'AnySkill'"
          class="text-[10px] font-semibold px-1 py-0.5 rounded"
          :class="skillBadgeClass">
          {{ power.skill }}
        </span>
        <span v-else class="text-[10px] font-semibold text-text-dim px-1 py-0.5 rounded bg-surface-hover">
          Generic
        </span>
        <span class="font-medium text-text-primary truncate">{{ displayName }}</span>
        <!-- Tier selector -->
        <TierSelector
          v-if="hasTiers && !isAssigned && !isAssignedAsAugment"
          :tiers="power.available_tiers"
          :model-value="selectedTierId"
          class="ml-auto"
          @update:model-value="selectedTierId = $event" />
      </div>
      <template v-if="!compact">
        <div v-if="activeEffects.length > 0" class="mt-0.5">
          <EffectLine v-for="(effect, i) in activeEffects" :key="i" :text="effect" />
        </div>
        <div v-if="activePrereq" class="text-[10px] text-text-dim mt-0.5">
          Requires {{ power.skill ?? 'skill' }} level {{ activePrereq }}
        </div>
        <div v-if="boostedAbilities.length > 0" class="text-[10px] text-accent-gold/70 mt-0.5">
          boosts: {{ boostedAbilities.join(', ') }}
        </div>
        <div v-if="disabledReason" class="text-[10px] text-amber-400/80 mt-0.5">
          {{ disabledReason }}
        </div>
      </template>
    </div>

    <template v-if="props.augmentOnly">
      <button
        v-if="!hasAugment && !isAssignedAsAugment && !isAssigned"
        class="text-purple-400/70 hover:text-purple-400 text-xs shrink-0 mt-0.5 cursor-pointer"
        title="Add as augment"
        @click.stop="emitAdd(true)">
        +A
      </button>
      <span v-else-if="isAssignedAsAugment" class="text-[10px] text-purple-400/50 shrink-0 mt-0.5">
        augment
      </span>
      <span v-else-if="isAssigned" class="text-[10px] text-text-dim shrink-0 mt-0.5">
        mod
      </span>
    </template>
    <template v-else>
      <button
        v-if="!isAssigned && !props.disabled"
        class="text-accent-gold/70 hover:text-accent-gold text-xs shrink-0 mt-0.5 cursor-pointer"
        title="Add mod"
        @click.stop="emitAdd(false)">
        +
      </button>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useBuildPlannerStore } from '../../../stores/buildPlannerStore'
import type { SlotTsysPower } from '../../../types/buildPlanner'
import { getPowerDisplayName } from '../../../types/buildPlanner'
import GameIcon from '../../Shared/GameIcon.vue'
import TierSelector from './TierSelector.vue'
import EffectLine from './EffectLine.vue'
import { useBuildCrossRef } from '../../../composables/useBuildCrossRef'

const props = defineProps<{
  power: SlotTsysPower
  augmentOnly?: boolean
  disabled?: boolean
  disabledReason?: string
  compact?: boolean
}>()

const emit = defineEmits<{
  add: [isAugment: boolean, tierId?: string]
}>()

const store = useBuildPlannerStore()
const { getAbilitiesForPower } = useBuildCrossRef()

const boostedAbilities = computed(() => {
  const powerKey = props.power.internal_name ?? props.power.key
  return getAbilitiesForPower(powerKey)
})

const displayName = computed(() => getPowerDisplayName(props.power))

const hasTiers = computed(() => props.power.available_tiers.length > 1)

// Default to the auto-selected best tier for this slot's level
const selectedTierId = ref(props.power.tier_id ?? '')

// Reset selected tier when power changes
watch(() => props.power.tier_id, (newTier) => {
  selectedTierId.value = newTier ?? ''
})

/** Effects for the currently selected tier */
const activeEffects = computed(() => {
  if (!hasTiers.value) return props.power.effects
  const tier = props.power.available_tiers.find(t => t.tier_id === selectedTierId.value)
  return tier?.effects ?? props.power.effects
})

/** Skill level prereq for the currently selected tier */
const activePrereq = computed(() => {
  if (!hasTiers.value) return props.power.skill_level_prereq
  const tier = props.power.available_tiers.find(t => t.tier_id === selectedTierId.value)
  return tier?.skill_level_prereq ?? props.power.skill_level_prereq
})

function emitAdd(isAugment: boolean) {
  const tierId = hasTiers.value ? selectedTierId.value : undefined
  emit('add', isAugment, tierId)
}

const isAssigned = computed(() => {
  const powerName = props.power.internal_name ?? props.power.key
  return store.selectedSlotMods.some(m => m.power_name === powerName && !m.is_augment)
})

const isAssignedAsAugment = computed(() => {
  const powerName = props.power.internal_name ?? props.power.key
  return store.selectedSlotMods.some(m => m.power_name === powerName && m.is_augment)
})

const hasAugment = computed(() => {
  return store.slotHasAugment[store.selectedSlot ?? ''] ?? false
})

const rowClass = computed(() => {
  if (isAssigned.value || isAssignedAsAugment.value) {
    return 'bg-surface-elevated border-border-default opacity-40 cursor-default'
  }
  if (props.disabled) {
    return 'bg-surface-elevated border-border-default opacity-50 cursor-not-allowed'
  }
  return 'bg-surface-elevated border-border-default hover:bg-surface-hover hover:border-accent-gold/30 cursor-pointer'
})

const skillBadgeClass = computed(() => {
  if (!props.power.skill || !store.selectedSlot) return 'text-text-dim bg-surface-hover'
  const primary = store.getSlotSkillPrimary(store.selectedSlot)
  const secondary = store.getSlotSkillSecondary(store.selectedSlot)
  if (props.power.skill === primary) {
    return 'text-blue-300 bg-blue-900/30'
  }
  if (props.power.skill === secondary) {
    return 'text-emerald-300 bg-emerald-900/30'
  }
  return 'text-text-dim bg-surface-hover'
})

function handleClick() {
  if (props.disabled) return
  if (props.augmentOnly) {
    if (!hasAugment.value && !isAssignedAsAugment.value && !isAssigned.value) emitAdd(true)
  } else if (!isAssigned.value) {
    emitAdd(false)
  }
}
</script>
