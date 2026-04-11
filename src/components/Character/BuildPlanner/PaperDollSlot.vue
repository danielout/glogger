<template>
  <button
    class="flex p-0 m-0 items-center justify-center rounded border cursor-pointer transition-all w-14 h-14"
    :class="slotClasses"
    :title="slot.label"
    @click="store.selectSlot(slot.id)">
    <!-- Show item icon when an item is assigned, otherwise show slot name -->
    <GameIcon
      v-if="itemIconId"
      :icon-id="itemIconId"
      size="lg"
      class="shrink-0" />
    <span v-else class="text-[10px] font-semibold text-text-dim leading-tight text-center px-0.5">
      {{ slot.label }}
    </span>
  </button>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useBuildPlannerStore } from '../../../stores/buildPlannerStore'
import type { EquipSlotDef } from '../../../types/buildPlanner'
import { getRarityBorderColor } from '../../../types/buildPlanner'
import GameIcon from '../../Shared/GameIcon.vue'

const props = defineProps<{
  slot: EquipSlotDef
}>()

const store = useBuildPlannerStore()

const itemIconId = computed(() => store.resolvedSlotItems[props.slot.id]?.icon_id ?? null)
const slotRarity = computed(() => store.getSlotRarity(props.slot.id))

const slotClasses = computed(() => {
  const isSelected = store.selectedSlot === props.slot.id
  const rarityBorder = getRarityBorderColor(slotRarity.value)

  if (isSelected) {
    return `bg-accent-gold/15 border-accent-gold/50 ring-1 ring-accent-gold/30`
  }
  return `bg-surface-elevated ${rarityBorder} hover:bg-surface-hover`
})
</script>
