<template>
  <div class="flex flex-col gap-2">
    <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">
      Favor
    </div>

    <!-- Current favor summary -->
    <div class="flex items-center gap-3">
      <span
        class="text-xs px-2 py-0.5 rounded border font-semibold"
        :class="favorBadgeClasses(effectiveTier)">
        {{ tierDisplayName(effectiveTier) }}
      </span>
      <span v-if="snapshotTier && gamestateTier && snapshotTier !== gamestateTier" class="text-[0.6rem] text-text-dim">
        Snapshot: <span :class="favorColor(snapshotTier)">{{ tierDisplayName(snapshotTier) }}</span>
      </span>
      <span v-if="gamestateFavor?.cumulative_delta" class="text-[0.6rem] text-text-dim">
        Session: <span class="text-accent-gold">{{ gamestateFavor.cumulative_delta > 0 ? '+' : '' }}{{ gamestateFavor.cumulative_delta.toFixed(1) }}</span> favor
      </span>
    </div>

    <!-- Tier ladder -->
    <div class="flex flex-col gap-0.5">
      <div
        v-for="tier in FAVOR_TIERS"
        :key="tier"
        class="flex items-center gap-2 px-2 py-0.5 rounded text-xs"
        :class="tierRowClasses(tier)">
        <!-- Status indicator -->
        <span class="w-4 text-center shrink-0">
          <template v-if="tier === effectiveTier">&#x25C6;</template>
          <template v-else-if="isUnlocked(tier)">&#x2713;</template>
          <template v-else>&#x2022;</template>
        </span>

        <!-- Tier name -->
        <span class="min-w-24">{{ tierDisplayName(tier) }}</span>

        <!-- Points to reach (for tiers above current) -->
        <span v-if="!isUnlocked(tier) && pointsNeeded(tier) !== null" class="text-text-dim text-[0.6rem] ml-auto">
          {{ pointsNeeded(tier) }} favor
        </span>

        <!-- Progress bar for current tier -->
        <div
          v-if="tier === effectiveTier && progressPercent !== null"
          class="flex-1 flex items-center gap-1.5 ml-2">
          <div class="flex-1 h-1 bg-border-default rounded-sm overflow-hidden">
            <div class="h-full bg-accent-gold/60 rounded-sm" :style="{ width: progressPercent + '%' }"></div>
          </div>
          <span class="text-[0.6rem] text-text-dim shrink-0">{{ progressPercent }}%</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { GameStateFavor } from '../../../types/gameState'
import {
  FAVOR_TIERS,
  tierIndex,
  isTierAtOrAbove,
  favorColor,
  favorBadgeClasses,
  tierDisplayName,
  pointsToNextTier,
} from '../../../composables/useFavorTiers'

const props = defineProps<{
  snapshotTier: string | null
  gamestateFavor: GameStateFavor | null
}>()

const gamestateTier = computed(() => props.gamestateFavor?.favor_tier ?? null)
const effectiveTier = computed(() => gamestateTier.value ?? props.snapshotTier ?? 'Neutral')

function isUnlocked(tier: string): boolean {
  return isTierAtOrAbove(effectiveTier.value, tier)
}

function tierRowClasses(tier: string): string {
  if (tier === effectiveTier.value) {
    return 'bg-accent-gold/10 border border-accent-gold/30'
  }
  if (isUnlocked(tier)) {
    return 'text-text-secondary'
  }
  return 'text-text-dim opacity-50'
}

/** Points needed to reach a tier from the tier below it */
function pointsNeeded(tier: string): number | null {
  const idx = tierIndex(tier)
  if (idx >= FAVOR_TIERS.length - 1) return null
  const tierBelow = FAVOR_TIERS[idx + 1]
  return pointsToNextTier(tierBelow)
}

/** Estimated progress within current tier based on cumulative delta */
const progressPercent = computed(() => {
  if (!props.gamestateFavor?.cumulative_delta) return null
  const needed = pointsToNextTier(effectiveTier.value)
  if (!needed) return null
  const delta = props.gamestateFavor.cumulative_delta
  if (delta <= 0) return 0
  return Math.min(100, Math.round((delta / needed) * 100))
})
</script>
