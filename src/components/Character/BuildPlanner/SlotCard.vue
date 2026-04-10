<template>
  <div
    class="flex flex-col rounded border cursor-pointer transition-all"
    :class="slotClasses"
    @click="store.selectSlot(slot.id)">

    <!-- Main row: icon, slot name, rarity, level, mod count, CP -->
    <div class="flex items-center gap-1.5 px-2 py-1">
      <GameIcon
        v-if="itemIconId"
        :icon-id="itemIconId"
        size="xs"
        class="shrink-0" />
      <div
        v-else
        class="w-5 h-5 rounded bg-surface-hover border border-border-default/50 shrink-0" />

      <div class="flex flex-col min-w-0 flex-1">
        <div class="flex items-center gap-1.5">
          <span class="text-xs font-semibold shrink-0">{{ slot.label }}</span>
          <div @click.stop>
            <StyledSelect
              :model-value="store.getSlotRarity(slot.id)"
              :options="rarityOptions"
              size="xs"
              :color-class="rarityColor(store.getSlotRarity(slot.id))"
              @update:model-value="onSlotRarityChange" />
          </div>
          <input
            type="number"
            :value="store.getSlotLevel(slot.id)"
            min="1"
            max="125"
            class="bg-transparent border border-border-default/50 rounded px-1 py-0 text-[10px] text-text-primary w-8 text-center"
            @click.stop
            @change="onSlotLevelChange" />
          <span class="flex-1" />
          <span class="text-[10px]" :class="fillColor">
            {{ store.slotModCounts[slot.id] ?? 0 }}/{{ store.getMaxModsForSlot(slot.id) }}
            <span v-if="store.slotHasAugment[slot.id]" class="text-purple-400">+A</span>
          </span>
          <CpProgressBar
            v-if="cpBudget > 0"
            :used="store.getSlotCpUsed(slot.id)"
            :budget="cpBudget"
            class="w-14" />
        </div>
        <!-- Item name + toggles (compact sub-row) -->
        <div class="flex items-center gap-1 -mt-0.5">
          <span
            v-if="store.slotArmorTypes[slot.id]"
            class="text-[9px] px-0.5 rounded shrink-0"
            :class="armorTypeBadge(store.slotArmorTypes[slot.id]!)">
            {{ store.slotArmorTypes[slot.id] }}
          </span>
          <span
            v-if="slotItem && slotItem.item_id !== 0"
            class="text-[10px] truncate text-entity-item"
            @click.stop>
            <ItemInline :reference="String(slotItem.item_id)" />
          </span>
          <span v-else class="text-[10px] text-text-dim italic">No item</span>
          <span class="flex-1" />
          <label class="flex items-center gap-0.5 text-[9px] text-text-dim cursor-pointer" @click.stop>
            <input type="checkbox" :checked="slotItem?.is_crafted ?? false" class="w-2 h-2 cursor-pointer" @change="onCraftedChange" />
            Crafted
          </label>
          <label v-if="store.getSlotRarity(slot.id) === 'Legendary'" class="flex items-center gap-0.5 text-[9px] text-text-dim cursor-pointer" @click.stop>
            <input type="checkbox" :checked="slotItem?.is_masterwork ?? false" class="w-2 h-2 cursor-pointer" @change="onMasterworkChange" />
            MW
          </label>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useBuildPlannerStore } from '../../../stores/buildPlannerStore'
import { getSlotCraftingPoints, getAllowedRarities, getDefaultRarityForSlot } from '../../../types/buildPlanner'
import type { EquipSlotDef } from '../../../types/buildPlanner'
import GameIcon from '../../Shared/GameIcon.vue'
import ItemInline from '../../Shared/Item/ItemInline.vue'
import StyledSelect from '../../Shared/StyledSelect.vue'
import CpProgressBar from './CpProgressBar.vue'

const props = defineProps<{
  slot: EquipSlotDef
}>()

const store = useBuildPlannerStore()

const rarityOptions = computed(() =>
  getAllowedRarities(props.slot).map(r => ({ value: r.id, label: r.label }))
)

// Auto-correct legacy data where belt was saved with an invalid rarity
onMounted(async () => {
  const currentRarity = store.getSlotRarity(props.slot.id)
  const allowed = getAllowedRarities(props.slot).map(r => r.id)
  if (!allowed.includes(currentRarity)) {
    await store.updateSlotProps(props.slot.id, { slot_rarity: getDefaultRarityForSlot(props.slot) })
  }
})

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
    case 'Common': return 'text-text-dim'
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
  const allowed = getAllowedRarities(props.slot).map(r => r.id)
  if (!allowed.includes(val)) return
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
