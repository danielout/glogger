<template>
  <div class="flex flex-col gap-3">
    <!-- Armor group (body slots) -->
    <div class="flex flex-col gap-1">
      <h3 class="text-[10px] font-semibold text-text-dim uppercase tracking-wider px-1">Armor</h3>
      <SlotCard v-for="slot in armorSlots" :key="slot.id" :slot="slot" />
    </div>

    <!-- Weapons group -->
    <div class="flex flex-col gap-1">
      <h3 class="text-[10px] font-semibold text-text-dim uppercase tracking-wider px-1">Weapons</h3>
      <SlotCard v-for="slot in weaponSlots" :key="slot.id" :slot="slot" />
    </div>

    <!-- Jewelry group -->
    <div class="flex flex-col gap-1">
      <h3 class="text-[10px] font-semibold text-text-dim uppercase tracking-wider px-1">Jewelry</h3>
      <SlotCard v-for="slot in jewelrySlots" :key="slot.id" :slot="slot" />
    </div>

    <!-- Extra group (Belt, etc.) -->
    <div class="flex flex-col gap-1">
      <h3 class="text-[10px] font-semibold text-text-dim uppercase tracking-wider px-1">Other</h3>
      <SlotCard v-for="slot in extraSlots" :key="slot.id" :slot="slot" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { EQUIPMENT_SLOTS } from '../../../types/buildPlanner'
import SlotCard from './SlotCard.vue'

const armorSlots = EQUIPMENT_SLOTS.filter(s => s.group === 'armor')
const weaponSlots = EQUIPMENT_SLOTS.filter(s => s.group === 'weapon')
const jewelrySlots = EQUIPMENT_SLOTS.filter(s => s.group === 'jewelry')
const extraSlots = EQUIPMENT_SLOTS.filter(s => s.group === 'extra')
</script>
