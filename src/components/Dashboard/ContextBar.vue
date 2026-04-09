<template>
  <div class="flex flex-col gap-2 text-sm">
    <!-- Identity section: Character / Server / Zone -->
    <div v-if="showAnyIdentity" class="flex flex-col gap-0.5">
      <div v-if="prefs.showCharacter && characterName" class="flex items-center gap-2">
        <span class="text-text-primary text-xs font-medium">{{ characterName }}</span>
        <span class="text-text-muted text-xs uppercase ">on</span>
        <span class="text-text-primary text-xs font-medium">{{ serverName }}</span>
      </div>
      <div v-if="prefs.showZone" class="flex items-center gap-2">
        <span class="text-text-muted text-xs uppercase tracking-wide ">In</span>
        <span v-if="zoneName" class="text-text-primary text-xs font-medium">{{ zoneName }}</span>
        <span v-else class="text-text-dim italic text-xs">Unknown</span>
      </div>
    </div>

    <!-- Divider -->
    <div
      v-if="showAnyIdentity && (showAnyTime || prefs.showMoon)"
      class="border-t border-border-default" />

    <!-- Times + Moon side by side -->
    <div v-if="showAnyTime || prefs.showMoon" class="flex gap-4">
      <!-- Left: Times stacked -->
      <div v-if="showAnyTime" class="flex flex-col gap-0.5">
        <div v-if="prefs.showGameTime" class="flex items-center gap-2">
          <span class="text-text-muted text-xs uppercase tracking-wide w-14">Game</span>
          <span class="text-text-primary font-mono text-xs">{{ formattedGameTime }}</span>
        </div>
        <div v-if="prefs.showServerTime" class="flex items-center gap-2">
          <span class="text-text-muted text-xs uppercase tracking-wide w-14">Server</span>
          <span class="text-text-primary font-mono text-xs">{{ formattedServerTime }}</span>
        </div>
        <div v-if="prefs.showLocalTime" class="flex items-center gap-2">
          <span class="text-text-muted text-xs uppercase tracking-wide w-14">Local</span>
          <span class="text-text-primary font-mono text-xs">{{ formattedLocalTime }}</span>
        </div>
      </div>

      <!-- Right: Moon phase -->
      <template v-if="prefs.showMoon && phase">
        <div v-if="showAnyTime" class="w-px bg-border-default" />
        <div class="flex flex-col items-center gap-0.5 min-w-20">
          <span class="text-3xl leading-none">{{ phase.emoji }}</span>
          <span class="text-text-primary text-xs font-medium">{{ phase.label }}</span>
          <span v-if="nextPhaseText" class="text-text-dim text-[10px]">{{ nextPhaseText }}</span>
          <span v-if="fullMoonText" class="text-text-dim text-[10px]">{{ fullMoonText }}</span>
        </div>
      </template>
    </div>

    <!-- Divider -->
    <div
      v-if="(showAnyTime || prefs.showMoon) && (prefs.showWeather || prefs.showCombat)"
      class="border-t border-border-default" />

    <!-- Weather & Status row -->
    <div v-if="prefs.showWeather || prefs.showCombat" class="flex items-center gap-4">
      <div v-if="prefs.showWeather" class="flex items-center gap-1.5">
        <span v-if="weather" class="text-text-primary text-xs">{{ weather }}</span>
        <span v-else class="text-text-dim italic text-xs">Unknown</span>
      </div>

      <template v-if="prefs.showCombat">
        <div v-if="prefs.showWeather" class="w-px h-4 bg-border-default" />

        <div class="flex items-center gap-1.5">
          <span class="text-text-muted text-xs uppercase tracking-wide">Status</span>
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

        <template v-if="store.namedEffects.length > 0">
          <div class="w-px h-4 bg-border-default" />
          <div class="flex items-center gap-1.5">
            <span class="text-text-muted text-xs uppercase tracking-wide">Effects</span>
            <span class="text-accent-gold font-bold text-xs">{{ store.namedEffects.length }}</span>
          </div>
        </template>
      </template>
    </div>

    <!-- Divider -->
    <div
      v-if="(prefs.showWeather || prefs.showCombat) && displayedCurrencies.length > 0"
      class="border-t border-border-default" />

    <!-- Currencies -->
    <div
      v-if="displayedCurrencies.length > 0"
      class="flex flex-col gap-0.5">
      <div
        v-for="c in displayedCurrencies"
        :key="c.currency_name"
        class="flex items-center justify-between text-xs">
        <span class="text-text-muted">{{ formatCurrencyName(c.currency_name) }}</span>
        <span class="text-accent-gold font-bold font-mono">{{ c.amount.toLocaleString() }}</span>
      </div>
    </div>

    <!-- Empty state -->
    <div
      v-if="!showAnyIdentity && !showAnyTime && !prefs.showMoon && !prefs.showWeather && !prefs.showCombat && displayedCurrencies.length === 0"
      class="text-text-dim text-xs italic">
      All sections hidden — use the gear icon to configure.
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch, onMounted, onUnmounted } from 'vue'
import { useGameStateStore } from '../../stores/gameStateStore'
import { useGameDataStore } from '../../stores/gameDataStore'
import { useSettingsStore } from '../../stores/settingsStore'
import { useMoonPhase } from '../../composables/useMoonPhase'
import { useViewPrefs } from '../../composables/useViewPrefs'
import { CURRENCY_DISPLAY_ORDER, CURRENCY_DISPLAY_NAMES, CONTEXT_BAR_DEFAULTS, type ContextBarPrefs } from './contextBarPrefs'

const store = useGameStateStore()
const gameData = useGameDataStore()
const settings = useSettingsStore()
const { phase, daysUntil } = useMoonPhase()

const { prefs, update } = useViewPrefs<ContextBarPrefs>('widget.context-bar', CONTEXT_BAR_DEFAULTS)

defineExpose({ prefs, update })

// --- Identity helpers ---

const characterName = computed(() => settings.settings.activeCharacterName)
const serverName = computed(() => settings.settings.activeServerName)

const areaKey = computed(() => store.world.area?.area_name ?? null)
const zoneName = ref<string | null>(null)

watch(areaKey, async (key) => {
  if (!key) { zoneName.value = null; return }
  const area = await gameData.resolveArea(key)
  zoneName.value = area?.short_friendly_name ?? area?.friendly_name ?? key
}, { immediate: true })

const showAnyIdentity = computed(() =>
  (prefs.value.showCharacter && !!characterName.value)
  || (prefs.value.showServer && !!serverName.value)
  || prefs.value.showZone
)

// --- Time helpers ---

const showAnyTime = computed(() =>
  prefs.value.showGameTime || prefs.value.showServerTime || prefs.value.showLocalTime
)

function formatTo12h(time24: string): string {
  if (time24 === '--:--') return time24
  const [hStr, mStr] = time24.split(':')
  const h = parseInt(hStr, 10)
  const period = h >= 12 ? 'PM' : 'AM'
  const h12 = h === 0 ? 12 : h > 12 ? h - 12 : h
  return `${h12}:${mStr} ${period}`
}

const formattedServerTime = computed(() =>
  prefs.value.use24h ? store.serverTime : formatTo12h(store.serverTime)
)

const formattedGameTime = computed(() =>
  prefs.value.use24h ? store.gameTime : formatTo12h(store.gameTime)
)

// --- Local time ---
const localTime24 = ref(formatLocalTime24())
let clockInterval: ReturnType<typeof setInterval> | null = null

function formatLocalTime24(): string {
  return new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', hour12: false })
}

const formattedLocalTime = computed(() => {
  if (prefs.value.use24h) return localTime24.value
  return formatTo12h(localTime24.value)
})

onMounted(() => {
  clockInterval = setInterval(() => {
    localTime24.value = formatLocalTime24()
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

const fullMoonText = computed(() => {
  if (!phase.value) return null
  if (phase.value.name === 'FullMoon') return null
  const fullMoonEntry = daysUntil.value.find(d => d.game_phase === 'FullMoon')
  if (!fullMoonEntry) return null
  return `Full Moon in ${fullMoonEntry.days}d`
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

/** Currencies filtered by per-currency toggle, sorted in display order */
const displayedCurrencies = computed(() => {
  const visible = store.currencies.filter(c => {
    if (c.amount <= 0) return false
    const prefKey = currencyPrefKey(c.currency_name)
    // Default to true for unknown currencies (unless explicitly hidden)
    return prefs.value[prefKey] !== false
  })

  // Sort by CURRENCY_DISPLAY_ORDER, unknowns go to end
  return visible.sort((a, b) => {
    const aIdx = CURRENCY_DISPLAY_ORDER.indexOf(a.currency_name.toLowerCase())
    const bIdx = CURRENCY_DISPLAY_ORDER.indexOf(b.currency_name.toLowerCase())
    const aOrder = aIdx === -1 ? 999 : aIdx
    const bOrder = bIdx === -1 ? 999 : bIdx
    return aOrder - bOrder
  })
})

function currencyPrefKey(name: string): string {
  return `currency_${name.toLowerCase()}`
}

function formatCurrencyName(name: string): string {
  const lower = name.toLowerCase()
  if (CURRENCY_DISPLAY_NAMES[lower]) return CURRENCY_DISPLAY_NAMES[lower]
  return name
    .split('_')
    .map(w => w.charAt(0).toUpperCase() + w.slice(1).toLowerCase())
    .join(' ')
}
</script>
