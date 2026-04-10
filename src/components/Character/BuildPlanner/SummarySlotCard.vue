<template>
  <div class="rounded border border-border-default bg-surface-elevated px-3 py-2 space-y-1.5">
    <!-- Slot header: name + mod count + CP -->
    <div class="flex items-center gap-2">
      <GameIcon
        v-if="itemIconId"
        :icon-id="itemIconId"
        size="xs"
        class="shrink-0" />
      <span class="text-xs font-semibold text-text-primary">{{ slot.label }}</span>
      <span class="text-[10px]" :class="fillColor">
        {{ modCount }}/{{ maxMods }}
      </span>
      <span v-if="hasAug" class="text-[10px] text-purple-400">+aug</span>
      <span class="flex-1" />
      <CpProgressBar
        v-if="cpBudget > 0"
        :used="hasAug ? AUGMENT_CP_COST : 0"
        :budget="cpBudget"
        class="w-14" />
    </div>

    <!-- Mod list -->
    <div v-if="slotMods.length > 0" class="space-y-0.5">
      <div
        v-for="mod in slotMods"
        :key="mod.id"
        class="flex items-center gap-1.5 text-[10px]">
        <span v-if="mod.is_augment" class="text-purple-400 font-semibold shrink-0">AUG</span>
        <span class="text-text-secondary truncate">{{ mod.power_name }}</span>
      </div>
    </div>
    <div v-else class="text-[10px] text-text-dim italic">No mods</div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useBuildPlannerStore } from '../../../stores/buildPlannerStore'
import { AUGMENT_CP_COST, getSlotCraftingPoints } from '../../../types/buildPlanner'
import type { EquipSlotDef } from '../../../types/buildPlanner'
import GameIcon from '../../Shared/GameIcon.vue'
import CpProgressBar from './CpProgressBar.vue'

const props = defineProps<{
  slot: EquipSlotDef
}>()

const store = useBuildPlannerStore()

const slotMods = computed(() =>
  store.presetMods.filter(m => m.equip_slot === props.slot.id)
)

const modCount = computed(() =>
  slotMods.value.filter(m => !m.is_augment).length
)

const maxMods = computed(() => store.getMaxModsForSlot(props.slot.id))
const hasAug = computed(() => store.slotHasAugment[props.slot.id] ?? false)
const itemIconId = computed(() => store.resolvedSlotItems[props.slot.id]?.icon_id ?? null)
const cpBudget = computed(() => getSlotCraftingPoints(store.getSlotItem(props.slot.id)))

const fillColor = computed(() => {
  if (modCount.value >= maxMods.value) return 'text-green-400'
  if (modCount.value > 0) return 'text-yellow-400'
  return 'text-text-dim'
})
</script>
