<template>
  <div
    class="flex flex-col rounded border cursor-pointer transition-all"
    :class="slotClasses"
    @click="store.selectSlot(slot.id)">

    <!-- Top row: icon, slot name, rarity, level, mod count, CP -->
    <div class="flex items-center gap-1.5 px-2 py-1.5">
      <!-- Item icon -->
      <GameIcon
        v-if="itemIconId"
        :icon-id="itemIconId"
        size="xs"
        class="shrink-0" />
      <div
        v-else
        class="w-5 h-5 rounded bg-surface-hover border border-border-default/50 shrink-0" />

      <span class="text-xs font-semibold shrink-0">{{ slot.label }}</span>

      <!-- Per-slot rarity -->
      <div @click.stop>
        <StyledSelect
          :model-value="store.getSlotRarity(slot.id)"
          :options="rarityOptions"
          size="xs"
          :color-class="rarityColor(store.getSlotRarity(slot.id))"
          @update:model-value="onSlotRarityChange" />
      </div>

      <!-- Per-slot level -->
      <input
        type="number"
        :value="store.getSlotLevel(slot.id)"
        min="1"
        max="125"
        class="bg-transparent border border-border-default/50 rounded px-1 py-0 text-xs text-text-primary w-10 text-center"
        @click.stop
        @change="onSlotLevelChange" />

      <span class="flex-1" />

      <!-- Mod count -->
      <span class="text-xs" :class="fillColor">
        {{ store.slotModCounts[slot.id] ?? 0 }}/{{ store.getMaxModsForSlot(slot.id) }}
        <span v-if="store.slotHasAugment[slot.id]" class="text-purple-400">+A</span>
      </span>

      <!-- Crafting points -->
      <CpProgressBar
        v-if="cpBudget > 0"
        :used="store.slotHasAugment[slot.id] ? AUGMENT_CP_COST : 0"
        :budget="cpBudget"
        class="w-16" />
    </div>

    <!-- Bottom row: armor type, item name, crafted/MW toggles -->
    <div class="flex items-center gap-1.5 px-2 pb-1.5 -mt-0.5">
      <span
        v-if="store.slotArmorTypes[slot.id]"
        class="text-[11px] px-1 rounded shrink-0"
        :class="armorTypeBadge(store.slotArmorTypes[slot.id]!)">
        {{ store.slotArmorTypes[slot.id] }}
      </span>

      <span
        v-if="slotItem && slotItem.item_id !== 0"
        class="text-[11px] truncate"
        @click.stop>
        <ItemInline :reference="String(slotItem.item_id)" />
      </span>
      <span v-else class="text-[11px] text-text-muted italic">No item selected</span>

      <span class="flex-1" />

      <label
        class="flex items-center gap-0.5 text-[11px] text-text-secondary cursor-pointer"
        @click.stop>
        <input
          type="checkbox"
          :checked="slotItem?.is_crafted ?? false"
          class="w-2.5 h-2.5 cursor-pointer"
          @change="onCraftedChange" />
        <span>Crafted</span>
      </label>

      <label
        v-if="store.getSlotRarity(slot.id) === 'Legendary'"
        class="flex items-center gap-0.5 text-[11px] text-text-secondary cursor-pointer"
        @click.stop>
        <input
          type="checkbox"
          :checked="slotItem?.is_masterwork ?? false"
          class="w-2.5 h-2.5 cursor-pointer"
          @change="onMasterworkChange" />
        <span>MW</span>
      </label>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useBuildPlannerStore } from '../../../stores/buildPlannerStore'
import { RARITY_DEFS, AUGMENT_CP_COST, getSlotCraftingPoints } from '../../../types/buildPlanner'
import type { EquipSlotDef } from '../../../types/buildPlanner'
import GameIcon from '../../Shared/GameIcon.vue'
import ItemInline from '../../Shared/Item/ItemInline.vue'
import StyledSelect from '../../Shared/StyledSelect.vue'
import CpProgressBar from './CpProgressBar.vue'

const rarityOptions = RARITY_DEFS.map(r => ({ value: r.id, label: r.label }))

const props = defineProps<{
  slot: EquipSlotDef
}>()

const store = useBuildPlannerStore()

const slotItem = computed(() => store.getSlotItem(props.slot.id))
const itemIconId = computed(() => store.resolvedSlotItems[props.slot.id]?.icon_id ?? null)
const cpBudget = computed(() => getSlotCraftingPoints(slotItem.value))

const slotClasses = computed(() => {
  const slotId = props.slot.id
  const isSelected = store.selectedSlot === slotId
  if (isSelected) {
    return 'bg-accent-gold/15 border-accent-gold/50 text-text-primary'
  }
  const count = store.slotModCounts[slotId] ?? 0
  const max = store.getMaxModsForSlot(slotId)
  if (count >= max) {
    return 'bg-surface-elevated border-border-default text-text-primary hover:bg-surface-hover border-l-2 border-l-green-500/50'
  }
  if (count > 0) {
    return 'bg-surface-elevated border-border-default text-text-primary hover:bg-surface-hover border-l-2 border-l-yellow-500/40'
  }
  return 'bg-surface-elevated border-border-default text-text-primary hover:bg-surface-hover'
})

const fillColor = computed(() => {
  const count = store.slotModCounts[props.slot.id] ?? 0
  const max = store.getMaxModsForSlot(props.slot.id)
  if (count >= max) return 'text-green-400'
  if (count > 0) return 'text-yellow-400'
  return 'text-text-muted'
})

function rarityColor(rarity: string): string {
  switch (rarity) {
    case 'Legendary': return 'text-yellow-400'
    case 'Epic': return 'text-purple-400'
    case 'Exceptional': return 'text-blue-400'
    case 'Rare': return 'text-emerald-400'
    case 'Uncommon': return 'text-text-primary'
    default: return 'text-text-muted'
  }
}

function armorTypeBadge(type: string): string {
  switch (type) {
    case 'Cloth': return 'bg-blue-900/30 text-blue-300'
    case 'Leather': return 'bg-amber-900/30 text-amber-300'
    case 'Metal': return 'bg-slate-600/30 text-slate-300'
    case 'Organic': return 'bg-green-900/30 text-green-300'
    default: return 'bg-surface-hover text-text-muted'
  }
}

async function onSlotLevelChange(e: Event) {
  const val = Number((e.target as HTMLInputElement).value)
  if (val >= 1 && val <= 125) {
    await store.updateSlotProps(props.slot.id, { slot_level: val })
  }
}

async function onSlotRarityChange(val: string) {
  if (val !== 'Legendary' && slotItem.value?.is_masterwork) {
    await store.updateSlotProps(props.slot.id, { slot_rarity: val, is_masterwork: false })
  } else {
    await store.updateSlotProps(props.slot.id, { slot_rarity: val })
  }
}

async function onCraftedChange(e: Event) {
  const checked = (e.target as HTMLInputElement).checked
  await store.updateSlotProps(props.slot.id, { is_crafted: checked })
}

async function onMasterworkChange(e: Event) {
  const checked = (e.target as HTMLInputElement).checked
  await store.updateSlotProps(props.slot.id, { is_masterwork: checked })
}
</script>
