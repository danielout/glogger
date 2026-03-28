<template>
  <div v-if="preferences.length" class="flex flex-col gap-1.5">
    <div class="text-[0.65rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5">
      Gift Preferences
    </div>

    <!-- Gift tier unlocks -->
    <div v-if="npc.item_gifts?.length" class="text-[0.6rem] text-text-dim px-2 mb-1">
      Gifting unlocked at:
      <span v-for="(tier, i) in npc.item_gifts" :key="tier">
        <span :class="favorColor(tier)">{{ tierDisplayName(tier) }}</span><span v-if="i < npc.item_gifts.length - 1">, </span>
      </span>
    </div>

    <!-- Preferences list -->
    <div class="flex flex-col gap-0.5">
      <div
        v-for="(pref, i) in preferences"
        :key="i"
        class="flex items-center gap-2 px-2 py-0.5 text-xs bg-[#151515] rounded">
        <!-- Desire badge -->
        <span
          class="text-[0.65rem] px-1.5 py-0.5 rounded border min-w-10 text-center shrink-0"
          :class="desireBadgeClasses(pref.desire)">
          {{ pref.desire }}
        </span>

        <!-- Item name or keywords -->
        <span class="text-text-secondary flex-1">
          {{ pref.name ?? pref.keywords.join(', ') }}
        </span>

        <!-- Pref value -->
        <span
          class="text-[0.65rem] shrink-0 font-mono"
          :class="pref.pref > 0 ? 'text-green-400' : 'text-red-400'">
          {{ pref.pref > 0 ? '+' : '' }}{{ pref.pref }}
        </span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { NpcInfo } from '../../../types/gameData'
import { favorColor, tierDisplayName } from '../../../composables/useFavorTiers'

const props = defineProps<{
  npc: NpcInfo
}>()

const preferences = computed(() =>
  [...(props.npc.preferences ?? [])].sort((a, b) => b.pref - a.pref)
)

function desireBadgeClasses(desire: string): string {
  switch (desire.toLowerCase()) {
    case 'love':
      return 'bg-red-900/30 border-red-700/40 text-red-300'
    case 'like':
      return 'bg-green-900/30 border-green-700/40 text-green-300'
    case 'hate':
      return 'bg-red-900/40 border-red-600/50 text-red-400'
    default:
      return 'bg-surface-elevated border-border-default text-text-muted'
  }
}
</script>
