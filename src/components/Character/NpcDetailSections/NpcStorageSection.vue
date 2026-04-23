<template>
  <div class="flex flex-col gap-1.5">
    <div
      class="text-[10px] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5 cursor-pointer select-none flex items-center gap-1"
      @click="collapsed = !collapsed">
      <span>{{ collapsed ? '\u25B8' : '\u25BE' }}</span>
      <span>Storage</span>
      <span v-if="unlockedSlots != null" class="text-text-dim normal-case tracking-normal">
        ({{ usedSlots }}/{{ unlockedSlots }} slots)
      </span>
    </div>

    <template v-if="!collapsed">
      <!-- Slot usage -->
      <div v-if="vaultDetail" class="flex flex-col gap-1 px-2">
        <div class="flex items-center gap-2 text-xs">
          <span class="text-text-muted">Slots:</span>
          <span class="text-text-primary font-bold">{{ usedSlots }}</span>
          <span v-if="unlockedSlots != null" class="text-text-dim">
            / {{ unlockedSlots }} used
            <span v-if="usagePercent != null">({{ usagePercent }}%)</span>
          </span>
        </div>
        <div v-if="maxPossibleSlots != null && maxPossibleSlots !== unlockedSlots" class="text-[10px] text-text-dim">
          Max at Soul Mates: {{ maxPossibleSlots }} slots
        </div>
      </div>

      <!-- Stored items -->
      <div v-if="storedItems.length" class="flex flex-col gap-0.5 px-2">
        <div
          v-for="item in storedItems"
          :key="item.instance_id"
          class="flex items-center gap-2 text-xs bg-[#151515] rounded px-2 py-0.5">
          <ItemInline :reference="item.item_name" />
          <span v-if="item.stack_size > 1" class="text-text-dim text-[10px] ml-auto">
            x{{ item.stack_size }}
          </span>
        </div>
      </div>

      <div v-if="!vaultDetail && storedItems.length === 0" class="text-xs text-text-dim italic px-2">
        No storage data yet
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import type { NpcInfo } from '../../../types/gameData'
import { useGameStateStore } from '../../../stores/gameStateStore'
import ItemInline from '../../Shared/Item/ItemInline.vue'

const props = defineProps<{
  npcKey: string
  npc: NpcInfo
  favorTier: string | null
}>()

const collapsed = ref(true)
const gameState = useGameStateStore()

const vaultDetail = computed(() => gameState.storageVaultsByKey[props.npcKey] ?? null)

const storedItems = computed(() => gameState.storageByVault[props.npcKey] ?? [])

const usedSlots = computed(() => storedItems.value.length)

const unlockedSlots = computed(() => {
  const vault = vaultDetail.value
  if (!vault) return null
  return gameState.getVaultUnlockedSlots(vault)
})

const maxPossibleSlots = computed(() => {
  const vault = vaultDetail.value
  if (!vault) return null
  return gameState.getVaultMaxPossibleSlots(vault)
})

const usagePercent = computed(() => {
  if (unlockedSlots.value == null || unlockedSlots.value === 0) return null
  return Math.round((usedSlots.value / unlockedSlots.value) * 100)
})
</script>
