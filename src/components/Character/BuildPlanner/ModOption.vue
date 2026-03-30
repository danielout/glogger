<template>
  <div
    class="flex items-start gap-2 px-2 py-1.5 rounded text-sm cursor-pointer transition-all border"
    :class="isAssigned
      ? 'bg-surface-elevated border-border-default opacity-50'
      : 'bg-surface-elevated border-border-default hover:bg-surface-hover hover:border-accent-gold/30'"
    @click="handleClick">
    <div class="flex-1 min-w-0">
      <div class="flex items-center gap-1.5">
        <span
          v-if="power.skill"
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
    </div>

    <button
      v-if="!isAssigned"
      class="text-accent-gold/70 hover:text-accent-gold text-xs shrink-0 mt-0.5 cursor-pointer"
      title="Add mod"
      @click.stop="emit('add', false)">
      +
    </button>
    <button
      v-if="!isAssigned && !hasAugment"
      class="text-purple-400/70 hover:text-purple-400 text-[10px] shrink-0 mt-0.5 cursor-pointer"
      title="Add as augment"
      @click.stop="emit('add', true)">
      +A
    </button>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useBuildPlannerStore } from '../../../stores/buildPlannerStore'
import type { SlotTsysPower } from '../../../types/buildPlanner'
import { getPowerDisplayName } from '../../../types/buildPlanner'

const props = defineProps<{
  power: SlotTsysPower
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

const hasAugment = computed(() => {
  return store.slotHasAugment[store.selectedSlot ?? ''] ?? false
})

const skillBadgeClass = computed(() => {
  if (!props.power.skill || !store.activePreset) return 'text-text-dim bg-surface-hover'
  if (props.power.skill === store.activePreset.skill_primary) {
    return 'text-blue-300 bg-blue-900/30'
  }
  if (props.power.skill === store.activePreset.skill_secondary) {
    return 'text-emerald-300 bg-emerald-900/30'
  }
  return 'text-text-dim bg-surface-hover'
})

function handleClick() {
  if (!isAssigned.value) {
    emit('add', false)
  }
}
</script>
