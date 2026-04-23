<template>
  <div class="flex flex-col gap-1.5">
    <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">
      Gift Calculator
    </div>

    <div class="flex flex-col gap-2 px-2">
      <!-- Target tier selector -->
      <div class="flex items-center gap-2 text-xs">
        <label class="text-text-muted shrink-0">Target tier:</label>
        <select v-model="targetTier" class="input text-xs flex-1 cursor-pointer">
          <option v-for="tier in availableTargetTiers" :key="tier" :value="tier">
            {{ tierDisplayName(tier) }}
          </option>
        </select>
      </div>

      <!-- Item search -->
      <div class="flex flex-col gap-1">
        <label class="text-text-muted text-xs">Item to gift:</label>
        <input
          v-model="itemQuery"
          class="input text-xs"
          placeholder="Search item name..." />
        <div v-if="matchingPrefs.length" class="flex flex-col gap-0.5 max-h-24 overflow-y-auto">
          <div
            v-for="match in matchingPrefs"
            :key="match.name ?? match.keywords.join(',')"
            class="flex items-center gap-2 text-xs px-2 py-0.5 rounded cursor-pointer hover:bg-surface-elevated"
            :class="{ 'bg-accent-gold/10 border border-accent-gold/30': selectedPref === match }"
            @click="selectedPref = match">
            <span class="text-text-secondary flex-1">{{ match.name ?? match.keywords.join(', ') }}</span>
            <span class="text-value-positive font-mono text-[0.6rem]">+{{ match.pref }}</span>
          </div>
        </div>
        <div v-else-if="itemQuery.length >= 2" class="text-[0.6rem] text-text-dim italic">
          No matching preferences found
        </div>
      </div>

      <!-- Calculation result -->
      <div v-if="calculation" class="bg-surface-inset rounded p-2 flex flex-col gap-1">
        <div class="flex items-center gap-2 text-xs">
          <span class="text-text-muted">Favor needed:</span>
          <span class="text-accent-gold font-bold">{{ calculation.favorNeeded.toLocaleString() }}</span>
        </div>
        <div class="flex items-center gap-2 text-xs">
          <span class="text-text-muted">Per gift:</span>
          <span class="text-value-positive font-mono">+{{ calculation.prefValue }}</span>
        </div>
        <div class="flex items-center gap-2 text-xs">
          <span class="text-text-muted">Items needed:</span>
          <span class="text-accent-gold font-bold text-sm">~{{ calculation.itemsNeeded.toLocaleString() }}</span>
        </div>
        <div class="text-[0.55rem] text-text-dim italic mt-1">
          Estimate only — actual favor may vary with gift bonuses, tier, and other factors.
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import type { NpcInfo, NpcPreference } from '../../../types/gameData'
import {
  FAVOR_TIERS,
  tierIndex,
  tierDisplayName,
  pointsToNextTier,
} from '../../../composables/useFavorTiers'

const props = defineProps<{
  npc: NpcInfo
  currentFavorTier: string | null
}>()

const effectiveCurrent = computed(() => props.currentFavorTier ?? 'Neutral')
const targetTier = ref<string>('Friends')
const itemQuery = ref('')
const selectedPref = ref<NpcPreference | null>(null)

const availableTargetTiers = computed(() => {
  const currentIdx = tierIndex(effectiveCurrent.value)
  return FAVOR_TIERS.filter((_t, i) => i < currentIdx)
})

const matchingPrefs = computed(() => {
  if (itemQuery.value.length < 2) return []
  const q = itemQuery.value.toLowerCase()
  return (props.npc.preferences ?? []).filter(p => {
    if (p.pref <= 0) return false
    if (p.name && p.name.toLowerCase().includes(q)) return true
    return p.keywords.some(k => k.toLowerCase().includes(q))
  }).sort((a, b) => b.pref - a.pref)
})

const calculation = computed(() => {
  if (!selectedPref.value || selectedPref.value.pref <= 0) return null

  const currentIdx = tierIndex(effectiveCurrent.value)
  const targetIdx = tierIndex(targetTier.value)
  if (targetIdx >= currentIdx) return null

  let favorNeeded = 0
  for (const tier of FAVOR_TIERS) {
    const idx = tierIndex(tier)
    if (idx >= currentIdx) continue
    if (idx < targetIdx) continue
    const pts = pointsToNextTier(tier)
    if (pts != null) favorNeeded += pts
  }

  if (favorNeeded <= 0) return null

  const prefValue = selectedPref.value.pref
  const itemsNeeded = Math.ceil(favorNeeded / prefValue)

  return { favorNeeded, prefValue, itemsNeeded }
})
</script>
