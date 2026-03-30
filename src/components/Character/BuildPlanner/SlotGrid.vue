<template>
  <div class="flex flex-col gap-1">
    <h3 class="text-xs font-semibold text-text-muted uppercase tracking-wider">Equipment Slots</h3>

    <div
      v-for="slot in EQUIPMENT_SLOTS"
      :key="slot.id"
      class="flex flex-col rounded border cursor-pointer transition-all"
      :class="slotClasses(slot.id)"
      @click="store.selectSlot(slot.id)">

      <!-- Top row: slot name, rarity, level, mod count, crafting points -->
      <div class="flex items-center gap-2 px-2 py-1.5">
        <span class="text-xs font-semibold w-20 shrink-0">{{ slot.label }}</span>

        <!-- Per-slot rarity -->
        <select
          class="bg-transparent border-none text-xs py-0 px-0.5 cursor-pointer min-w-0"
          :class="rarityColor(store.getSlotRarity(slot.id))"
          :value="store.getSlotRarity(slot.id)"
          @click.stop
          @change="onSlotRarityChange(slot.id, $event)">
          <option v-for="r in RARITY_DEFS" :key="r.id" :value="r.id">{{ r.label }}</option>
        </select>

        <!-- Per-slot level -->
        <input
          type="number"
          :value="store.getSlotLevel(slot.id)"
          min="1"
          max="125"
          class="bg-transparent border border-border-default/50 rounded px-1 py-0 text-xs text-text-primary w-10 text-center"
          @click.stop
          @change="onSlotLevelChange(slot.id, $event)" />

        <span class="flex-1" />

        <!-- Mod count -->
        <span class="text-xs" :class="fillColor(slot.id)">
          {{ store.slotModCounts[slot.id] ?? 0 }}/{{ store.getMaxModsForSlot(slot.id) }}
          <span v-if="store.slotHasAugment[slot.id]" class="text-purple-400">+A</span>
        </span>

        <!-- Crafting points (with augment cost shown) -->
        <span v-if="slotCraftPoints(slot.id) > 0" class="text-xs text-text-secondary">
          <template v-if="store.slotHasAugment[slot.id]">
            <span :class="slotCraftPoints(slot.id) - AUGMENT_CP_COST < 0 ? 'text-red-400' : ''">
              {{ slotCraftPoints(slot.id) - AUGMENT_CP_COST }}
            </span>/{{ slotCraftPoints(slot.id) }}cp
          </template>
          <template v-else>
            {{ slotCraftPoints(slot.id) }}cp
          </template>
        </span>
      </div>

      <!-- Bottom row: base item name + armor type + crafted/masterwork flags -->
      <div class="flex items-center gap-1.5 px-2 pb-1.5 -mt-0.5">
        <!-- Armor type badge -->
        <span
          v-if="store.slotArmorTypes[slot.id]"
          class="text-[11px] px-1 rounded shrink-0"
          :class="armorTypeBadge(store.slotArmorTypes[slot.id]!)">
          {{ store.slotArmorTypes[slot.id] }}
        </span>

        <span
          v-if="store.getSlotItem(slot.id) && store.getSlotItem(slot.id)!.item_id !== 0"
          class="text-[11px] text-entity-item truncate">
          {{ store.getSlotItem(slot.id)?.item_name ?? 'Unknown Item' }}
        </span>
        <span v-else class="text-[11px] text-text-muted italic">No item selected</span>

        <span class="flex-1" />

        <!-- Crafted toggle -->
        <label
          class="flex items-center gap-0.5 text-[11px] text-text-secondary cursor-pointer"
          @click.stop>
          <input
            type="checkbox"
            :checked="store.getSlotItem(slot.id)?.is_crafted ?? false"
            class="w-2.5 h-2.5 cursor-pointer"
            @change="onCraftedChange(slot.id, $event)" />
          <span>Crafted</span>
        </label>

        <!-- Masterwork toggle (only for Legendary rarity) -->
        <label
          v-if="store.getSlotRarity(slot.id) === 'Legendary'"
          class="flex items-center gap-0.5 text-[11px] text-text-secondary cursor-pointer"
          @click.stop>
          <input
            type="checkbox"
            :checked="store.getSlotItem(slot.id)?.is_masterwork ?? false"
            class="w-2.5 h-2.5 cursor-pointer"
            @change="onMasterworkChange(slot.id, $event)" />
          <span>MW</span>
        </label>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useBuildPlannerStore } from '../../../stores/buildPlannerStore'
import { EQUIPMENT_SLOTS, RARITY_DEFS, AUGMENT_CP_COST, getSlotCraftingPoints } from '../../../types/buildPlanner'

const store = useBuildPlannerStore()

function slotClasses(slotId: string): string {
  const isSelected = store.selectedSlot === slotId
  if (isSelected) {
    return 'bg-accent-gold/20 border-accent-gold/60 text-accent-gold'
  }
  const count = store.slotModCounts[slotId] ?? 0
  const max = store.getMaxModsForSlot(slotId)
  if (count >= max) {
    return 'bg-green-900/15 border-green-700/30 text-text-primary hover:bg-green-900/25'
  }
  if (count > 0) {
    return 'bg-yellow-900/15 border-yellow-700/30 text-text-primary hover:bg-yellow-900/25'
  }
  return 'bg-surface-elevated border-border-default text-text-primary hover:bg-surface-hover'
}

function fillColor(slotId: string): string {
  const count = store.slotModCounts[slotId] ?? 0
  const max = store.getMaxModsForSlot(slotId)
  if (count >= max) return 'text-green-400'
  if (count > 0) return 'text-yellow-400'
  return 'text-text-muted'
}

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

function slotCraftPoints(slotId: string): number {
  return getSlotCraftingPoints(store.getSlotItem(slotId))
}

async function onSlotLevelChange(slotId: string, e: Event) {
  const val = Number((e.target as HTMLInputElement).value)
  if (val >= 1 && val <= 125) {
    await store.updateSlotProps(slotId, { slot_level: val })
  }
}

async function onSlotRarityChange(slotId: string, e: Event) {
  const val = (e.target as HTMLSelectElement).value
  // Clear masterwork if downgrading from Legendary
  if (val !== 'Legendary' && store.getSlotItem(slotId)?.is_masterwork) {
    await store.updateSlotProps(slotId, { slot_rarity: val, is_masterwork: false })
  } else {
    await store.updateSlotProps(slotId, { slot_rarity: val })
  }
}

async function onCraftedChange(slotId: string, e: Event) {
  const checked = (e.target as HTMLInputElement).checked
  await store.updateSlotProps(slotId, { is_crafted: checked })
}

async function onMasterworkChange(slotId: string, e: Event) {
  const checked = (e.target as HTMLInputElement).checked
  await store.updateSlotProps(slotId, { is_masterwork: checked })
}
</script>
