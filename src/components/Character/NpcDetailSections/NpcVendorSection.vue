<template>
  <div class="flex flex-col gap-1.5">
    <div class="text-[10px] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">
      Vendor Status
    </div>

    <template v-if="vendorStatus">
      <!-- Gold display -->
      <div class="flex items-center gap-2 px-2 text-xs">
        <span class="text-text-muted">Gold:</span>
        <span class="font-bold" :class="goldColorClass">
          {{ formatGold(vendorStatus.vendor_gold_available) }}
        </span>
        <span v-if="vendorStatus.vendor_gold_max != null" class="text-text-dim">
          / {{ vendorStatus.vendor_gold_max.toLocaleString() }}
        </span>
        <span v-else-if="currentCapGold != null" class="text-text-dim">
          / ~{{ currentCapGold.toLocaleString() }}
        </span>
        <span class="text-text-dim text-[10px]">councils</span>
      </div>

      <!-- Timer -->
      <div v-if="resetTimeLabel" class="flex items-center gap-2 px-2 text-xs">
        <span class="text-text-muted">Resets in:</span>
        <span class="text-text-secondary">~{{ resetTimeLabel }}</span>
        <span class="text-text-dim text-[10px] italic">(estimated)</span>
      </div>

      <!-- Last sell -->
      <div v-if="lastSellLabel" class="flex items-center gap-2 px-2 text-xs">
        <span class="text-text-muted">Last sold:</span>
        <span class="text-text-secondary">{{ lastSellLabel }}</span>
      </div>
    </template>

    <div v-else class="text-xs text-text-dim italic px-2">
      No vendor data yet — sell an item to start tracking
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { NpcInfo } from '../../../types/gameData'
import type { GameStateVendor } from '../../../types/gameState'
import { goldCapAtTier } from '../../../composables/useNpcServices'

const props = defineProps<{
  vendorStatus: GameStateVendor | null
  npc: NpcInfo
  playerTier: string
}>()

const currentCapGold = computed(() => {
  const cap = goldCapAtTier(props.npc, props.playerTier)
  return cap?.maxGold ?? null
})

const goldColorClass = computed(() => {
  const v = props.vendorStatus
  if (!v || v.vendor_gold_available == null) return 'text-text-secondary'
  const max = v.vendor_gold_max ?? currentCapGold.value
  if (!max || max === 0) return 'text-text-secondary'
  const ratio = v.vendor_gold_available / max
  if (ratio >= 0.7) return 'text-green-400'
  if (ratio >= 0.3) return 'text-yellow-400'
  return 'text-red-400'
})

function formatGold(val: number | null | undefined): string {
  if (val == null) return '?'
  return val.toLocaleString()
}

const VENDOR_RESET_HOURS = 168

const resetTimeLabel = computed(() => {
  const start = props.vendorStatus?.vendor_gold_timer_start
  if (!start) return null
  const startDate = new Date(start + 'Z')
  if (isNaN(startDate.getTime())) return null
  const resetAt = new Date(startDate.getTime() + VENDOR_RESET_HOURS * 3600 * 1000)
  const now = new Date()
  const diffMs = resetAt.getTime() - now.getTime()
  if (diffMs <= 0) return 'soon'
  const diffH = Math.floor(diffMs / (3600 * 1000))
  const days = Math.floor(diffH / 24)
  const hours = diffH % 24
  if (days > 0) return `${days}d ${hours}h`
  return `${hours}h`
})

const lastSellLabel = computed(() => {
  const ts = props.vendorStatus?.last_sell_at
  if (!ts) return null
  const date = new Date(ts + 'Z')
  if (isNaN(date.getTime())) return null
  const now = new Date()
  const diffMs = now.getTime() - date.getTime()
  if (diffMs < 0) return 'just now'
  const diffMin = Math.floor(diffMs / 60000)
  if (diffMin < 1) return 'just now'
  if (diffMin < 60) return `${diffMin}m ago`
  const diffH = Math.floor(diffMin / 60)
  if (diffH < 24) return `${diffH}h ago`
  const diffD = Math.floor(diffH / 24)
  return `${diffD}d ago`
})
</script>
