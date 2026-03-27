<template>
  <div class="bg-[#1a1a2e] border border-border-default rounded-lg px-4 py-2 flex flex-col gap-2">
    <!-- Row 1: Status -->
    <div class="flex items-center gap-4 text-sm">
      <!-- Weather -->
      <div class="flex items-center gap-1.5">
        <span class="text-text-muted text-xs uppercase tracking-wide">Weather</span>
        <span v-if="weather" class="text-text-primary">{{ weather }}</span>
        <span v-else class="text-text-dim italic">Unknown</span>
      </div>

      <div class="w-px h-4 bg-border-default" />

      <!-- Combat / Mount status -->
      <div class="flex items-center gap-2">
        <span
          v-if="inCombat"
          class="px-2 py-0.5 rounded text-xs font-bold bg-red-900/40 text-red-400 border border-red-800/50">
          In Combat
        </span>
        <span
          v-if="isMounted"
          class="px-2 py-0.5 rounded text-xs font-bold bg-blue-900/40 text-blue-300 border border-blue-800/50">
          Mounted
        </span>
        <span
          v-if="!inCombat && !isMounted"
          class="text-text-dim text-xs italic">
          Idle
        </span>
      </div>

      <!-- Active Effects count (if any) -->
      <template v-if="store.namedEffects.length > 0">
        <div class="w-px h-4 bg-border-default" />
        <div class="flex items-center gap-1.5">
          <span class="text-text-muted text-xs uppercase tracking-wide">Effects</span>
          <span class="text-accent-gold font-bold">{{ store.namedEffects.length }}</span>
        </div>
      </template>
    </div>

    <!-- Row 2: Currencies (only non-zero, wrapped) -->
    <div v-if="visibleCurrencies.length > 0" class="flex flex-wrap items-center gap-x-4 gap-y-1 border-t border-border-default pt-2">
      <span
        v-for="c in visibleCurrencies"
        :key="c.currency_name"
        class="text-xs whitespace-nowrap">
        <span class="text-accent-gold font-bold">{{ c.amount.toLocaleString() }}</span>
        <span class="text-text-muted ml-1">{{ formatCurrencyName(c.currency_name) }}</span>
      </span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useGameStateStore } from '../../stores/gameStateStore'

const store = useGameStateStore()

const weather = computed(() => {
  const w = store.world.weather
  if (!w || !w.is_active) return null
  return w.weather_name
})

const inCombat = computed(() => store.world.combat?.in_combat ?? false)
const isMounted = computed(() => store.world.mount?.is_mounted ?? false)

/** Only show currencies with a non-zero balance */
const visibleCurrencies = computed(() =>
  store.currencies.filter(c => c.amount > 0)
)

/** Convert SCREAMING_SNAKE to Title Case */
function formatCurrencyName(name: string): string {
  return name
    .split('_')
    .map(w => w.charAt(0).toUpperCase() + w.slice(1).toLowerCase())
    .join(' ')
}
</script>
