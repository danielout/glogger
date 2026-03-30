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
      </div>
      <div v-if="power.effects.length > 0" class="text-xs text-text-secondary mt-0.5">
        <div v-for="(effect, i) in power.effects" :key="i">{{ effect }}</div>
      </div>
      <div v-if="power.skill_level_prereq" class="text-[10px] text-text-dim mt-0.5">
        Requires {{ power.skill ?? 'skill' }} level {{ power.skill_level_prereq }}
      </div>
      <!-- Blocked reason shown inline -->
      <div v-if="disabledReason" class="text-[10px] text-amber-400/80 mt-0.5">
        {{ disabledReason }}
      </div>
    </div>

    <template v-if="props.augmentOnly">
      <button
        v-if="!hasAugment && !isAssignedAsAugment && !isAssigned"
        class="text-purple-400/70 hover:text-purple-400 text-xs shrink-0 mt-0.5 cursor-pointer"
        title="Add as augment"
        @click.stop="emit('add', true)">
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
        @click.stop="emit('add', false)">
        +
      </button>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useBuildPlannerStore } from '../../../stores/buildPlannerStore'
import type { SlotTsysPower } from '../../../types/buildPlanner'
import { getPowerDisplayName } from '../../../types/buildPlanner'
import GameIcon from '../../Shared/GameIcon.vue'

const props = defineProps<{
  power: SlotTsysPower
  augmentOnly?: boolean
  disabled?: boolean
  disabledReason?: string
}>()

const emit = defineEmits<{
  add: [isAugment: boolean]
}>()

const store = useBuildPlannerStore()

const displayName = computed(() => getPowerDisplayName(props.power))

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
    if (!hasAugment.value && !isAssignedAsAugment.value && !isAssigned.value) emit('add', true)
  } else if (!isAssigned.value) {
    emit('add', false)
  }
}
</script>
