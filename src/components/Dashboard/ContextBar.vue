<template>
  <div class="flex flex-col gap-2 text-sm">
    <!-- Row 1: Time displays & Moon -->
    <div v-if="prefs.showTime || prefs.showMoon" class="flex items-center gap-3 flex-wrap">
      <!-- Server & Game Time -->
      <template v-if="prefs.showTime">
        <div class="flex items-center gap-1.5">
          <span class="text-text-muted text-xs uppercase tracking-wide">Server</span>
          <span class="text-text-primary font-mono text-xs">{{ store.serverTime }}</span>
        </div>
        <div class="w-px h-4 bg-border-default" />
        <div class="flex items-center gap-1.5">
          <span class="text-text-muted text-xs uppercase tracking-wide">Game</span>
          <span class="text-text-primary font-mono text-xs">{{ store.gameTime }}</span>
        </div>
        <template v-if="prefs.showLocalTime">
          <div class="w-px h-4 bg-border-default" />
          <div class="flex items-center gap-1.5">
            <span class="text-text-muted text-xs uppercase tracking-wide">Local</span>
            <span class="text-text-primary font-mono text-xs">{{ localTime }}</span>
          </div>
        </template>
      </template>

      <!-- Moon Phase (compact) -->
      <template v-if="prefs.showMoon && phase">
        <div v-if="prefs.showTime" class="w-px h-4 bg-border-default" />
        <div class="flex items-center gap-1.5">
          <span class="text-base leading-none">{{ phase.emoji }}</span>
          <span class="text-text-primary text-xs">{{ phase.label }}</span>
          <span v-if="nextPhaseText" class="text-text-dim text-xs">({{ nextPhaseText }})</span>
        </div>
      </template>
    </div>

    <!-- Row 2: Weather / Combat / Effects -->
    <div v-if="prefs.showWeather || prefs.showCombat" class="flex items-center gap-4">
      <!-- Weather -->
      <div v-if="prefs.showWeather" class="flex items-center gap-1.5">
        <span class="text-text-muted text-xs uppercase tracking-wide">Weather</span>
        <span v-if="weather" class="text-text-primary">{{ weather }}</span>
        <span v-else class="text-text-dim italic">Unknown</span>
      </div>

      <template v-if="prefs.showCombat">
        <div v-if="prefs.showWeather" class="w-px h-4 bg-border-default" />

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
      </template>
    </div>

    <!-- Row 3: Currencies (only non-zero, wrapped) -->
    <div
      v-if="prefs.showCurrencies && visibleCurrencies.length > 0"
      class="flex flex-wrap items-center gap-x-4 gap-y-1 border-t border-border-default pt-2">
      <span
        v-for="c in visibleCurrencies"
        :key="c.currency_name"
        class="text-xs whitespace-nowrap">
        <span class="text-accent-gold font-bold">{{ c.amount.toLocaleString() }}</span>
        <span class="text-text-muted ml-1">{{ formatCurrencyName(c.currency_name) }}</span>
      </span>
    </div>

    <!-- Empty state when everything is hidden -->
    <div
      v-if="!prefs.showTime && !prefs.showMoon && !prefs.showWeather && !prefs.showCombat && !prefs.showCurrencies"
      class="text-text-dim text-xs italic">
      All sections hidden — use the gear icon to configure.
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted } from 'vue'
import { useGameStateStore } from '../../stores/gameStateStore'
import { useMoonPhase } from '../../composables/useMoonPhase'
import { useViewPrefs } from '../../composables/useViewPrefs'

const store = useGameStateStore()
const { phase, daysUntil } = useMoonPhase()

const { prefs, update } = useViewPrefs('widget.context-bar', {
  showTime: true,
  showLocalTime: false,
  showMoon: true,
  showWeather: true,
  showCombat: true,
  showCurrencies: true,
})

defineExpose({ prefs, update })

// --- Local time (only used when showLocalTime is on) ---
const localTime = ref(formatLocalTime())
let clockInterval: ReturnType<typeof setInterval> | null = null

function formatLocalTime(): string {
  return new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' })
}

onMounted(() => {
  clockInterval = setInterval(() => {
    localTime.value = formatLocalTime()
  }, 1000)
})

onUnmounted(() => {
  if (clockInterval) clearInterval(clockInterval)
})

// --- Moon ---
const nextPhaseText = computed(() => {
  if (daysUntil.value.length === 0) return null
  const next = daysUntil.value[0]
  return `${next.label} in ${next.days}d`
})

// --- Weather / Combat ---
const weather = computed(() => {
  const w = store.world.weather
  if (!w || !w.is_active) return null
  return w.weather_name
})

const inCombat = computed(() => store.world.combat?.in_combat ?? false)
const isMounted = computed(() => store.world.mount?.is_mounted ?? false)

// --- Currencies ---
const visibleCurrencies = computed(() =>
  store.currencies.filter(c => c.amount > 0)
)

function formatCurrencyName(name: string): string {
  return name
    .split('_')
    .map(w => w.charAt(0).toUpperCase() + w.slice(1).toLowerCase())
    .join(' ')
}
</script>
