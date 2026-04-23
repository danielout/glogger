<template>
  <div class="flex flex-col gap-2 text-sm h-full min-h-0">
    <!-- Self-milked counter (cow form self-milking) -->
    <div v-if="selfMilkedCount > 0" class="flex items-center justify-between px-2 shrink-0">
      <span class="text-xs text-text-muted">Self-milked</span>
      <span class="text-xs text-accent-gold">{{ selfMilkedCount }}</span>
    </div>

    <!-- Tab bar -->
    <div class="flex gap-1 shrink-0">
      <button
        v-for="tab in tabs"
        :key="tab.key"
        class="px-2 py-1 text-xs rounded transition-colors cursor-pointer"
        :class="activeTab === tab.key
          ? 'bg-accent-gold/20 text-accent-gold border border-accent-gold/40'
          : 'text-text-muted hover:text-text-primary hover:bg-surface-elevated'"
        @click="activeTab = tab.key">
        {{ tab.label }}
      </button>
    </div>

    <!-- Timers tab -->
    <div v-if="activeTab === 'timers'" class="flex-1 overflow-y-auto min-h-0">
      <div v-if="timers.length === 0" class="text-xs text-text-dim italic">
        No cows milked yet. Milk an NPC cow to start tracking cooldowns.
      </div>

      <template v-for="group in groupedTimers" :key="group.zone">
        <div>
          <div class="flex items-center gap-1.5 mb-1">
            <span
              v-if="group.isCurrentZone"
              class="text-[10px] font-semibold uppercase tracking-wider text-accent-gold">
              Current Area:
            </span>
            <AreaInline :reference="group.zone" class="text-xs font-semibold text-text-secondary" />
          </div>
          <div class="flex flex-col gap-0.5">
            <div
              v-for="timer in group.cows"
              :key="timer.cow_name"
              class="flex items-center justify-between gap-2 py-0.5 pl-2">
              <span class="text-text-primary text-xs truncate">{{ timer.cow_name }}</span>
              <span
                class="text-xs whitespace-nowrap shrink-0"
                :class="timer.remaining <= 0 ? 'text-green-400' : 'text-text-dim'">
                {{ timer.remaining <= 0 ? 'Ready!' : formatRemaining(timer.remaining) }}
              </span>
            </div>
          </div>
        </div>
      </template>
    </div>

    <!-- Players Milked tab -->
    <div v-else-if="activeTab === 'milked'" class="flex-1 overflow-y-auto min-h-0">
      <div v-if="milkedLeaderboard.length === 0" class="text-xs text-text-dim italic">
        No players milked yet. Milk a player-cow to start tracking.
      </div>
      <div v-else class="flex flex-col gap-0.5">
        <div
          v-for="(entry, i) in milkedLeaderboard"
          :key="entry.player_name"
          class="flex items-center justify-between gap-2 py-0.5 px-2">
          <div class="flex items-center gap-2 min-w-0">
            <span class="text-xs text-text-dim w-4 text-right shrink-0">{{ i + 1 }}</span>
            <span class="text-text-primary text-xs truncate">{{ entry.player_name }}</span>
          </div>
          <span class="text-xs text-accent-gold shrink-0">{{ entry.count }}</span>
        </div>
      </div>
    </div>

    <!-- Milked By tab -->
    <div v-else-if="activeTab === 'milked_by'" class="flex-1 overflow-y-auto min-h-0">
      <div v-if="milkedByLeaderboard.length === 0" class="text-xs text-text-dim italic">
        No one has milked you yet.
      </div>
      <div v-else class="flex flex-col gap-0.5">
        <div
          v-for="(entry, i) in milkedByLeaderboard"
          :key="entry.player_name"
          class="flex items-center justify-between gap-2 py-0.5 px-2">
          <div class="flex items-center gap-2 min-w-0">
            <span class="text-xs text-text-dim w-4 text-right shrink-0">{{ i + 1 }}</span>
            <span class="text-text-primary text-xs truncate">{{ entry.player_name }}</span>
          </div>
          <span class="text-xs text-accent-gold shrink-0">{{ entry.count }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useGameStateStore } from '../../../stores/gameStateStore'
import { useSettingsStore } from '../../../stores/settingsStore'
import AreaInline from '../../Shared/Area/AreaInline.vue'

interface MilkingTimer {
  cow_name: string
  zone: string
  last_milked_at: string
}

interface TimerWithRemaining extends MilkingTimer {
  remaining: number
}

interface ZoneGroup {
  zone: string
  isCurrentZone: boolean
  cows: TimerWithRemaining[]
}

interface PlayerMilkingEntry {
  player_name: string
  count: number
}

const tabs = [
  { key: 'timers', label: 'Timers' },
  { key: 'milked', label: 'Players Milked' },
  { key: 'milked_by', label: 'Milked By' },
] as const

type TabKey = typeof tabs[number]['key']

const COOLDOWN_MS = 60 * 60 * 1000

const gameState = useGameStateStore()
const settings = useSettingsStore()
const activeTab = ref<TabKey>('timers')
const timers = ref<MilkingTimer[]>([])
const milkedLeaderboard = ref<PlayerMilkingEntry[]>([])
const milkedByLeaderboard = ref<PlayerMilkingEntry[]>([])
const selfMilkedCount = ref(0)
const now = ref(Date.now())

let refreshInterval: ReturnType<typeof setInterval> | null = null
let unlisten: UnlistenFn | null = null

const currentZone = computed(() => gameState.world.area?.area_name ?? null)

const groupedTimers = computed<ZoneGroup[]>(() => {
  const withRemaining: TimerWithRemaining[] = timers.value.map(t => {
    const milkedAt = new Date(t.last_milked_at).getTime()
    const remaining = Math.ceil((milkedAt + COOLDOWN_MS - now.value) / 1000)
    return { ...t, remaining }
  })

  const zones = new Map<string, TimerWithRemaining[]>()
  for (const t of withRemaining) {
    const list = zones.get(t.zone) || []
    list.push(t)
    zones.set(t.zone, list)
  }

  for (const cows of zones.values()) {
    cows.sort((a, b) => a.cow_name.localeCompare(b.cow_name))
  }

  const groups: ZoneGroup[] = []
  const sortedZones = [...zones.keys()].sort((a, b) => a.localeCompare(b))

  for (const zone of sortedZones) {
    groups.push({
      zone,
      isCurrentZone: zone === currentZone.value,
      cows: zones.get(zone)!,
    })
  }

  groups.sort((a, b) => {
    if (a.isCurrentZone && !b.isCurrentZone) return -1
    if (!a.isCurrentZone && b.isCurrentZone) return 1
    return 0
  })

  return groups
})

function formatRemaining(seconds: number): string {
  if (seconds <= 0) return 'Ready!'
  const m = Math.ceil(seconds / 60)
  if (m >= 60) {
    const h = Math.floor(m / 60)
    const rm = m % 60
    return rm > 0 ? `${h}h ${rm}m` : `${h}h`
  }
  return `${m}m`
}

async function loadTimers() {
  const char = settings.settings.activeCharacterName
  const server = settings.settings.activeServerName
  if (!char || !server) return

  try {
    timers.value = await invoke<MilkingTimer[]>('get_milking_timers', {
      characterName: char,
      serverName: server,
    })
  } catch (e) {
    console.error('Failed to load milking timers:', e)
  }
}

async function loadLeaderboards() {
  const char = settings.settings.activeCharacterName
  const server = settings.settings.activeServerName
  if (!char || !server) return

  try {
    const [milked, milkedBy, selfMilked] = await Promise.all([
      invoke<PlayerMilkingEntry[]>('get_player_milking_leaderboard', {
        characterName: char,
        serverName: server,
        direction: 'milked',
      }),
      invoke<PlayerMilkingEntry[]>('get_player_milking_leaderboard', {
        characterName: char,
        serverName: server,
        direction: 'milked_by',
      }),
      invoke<PlayerMilkingEntry[]>('get_player_milking_leaderboard', {
        characterName: char,
        serverName: server,
        direction: 'self_milked',
      }),
    ])
    milkedLeaderboard.value = milked
    milkedByLeaderboard.value = milkedBy
    selfMilkedCount.value = selfMilked.reduce((sum, e) => sum + e.count, 0)
  } catch (e) {
    console.error('Failed to load milking leaderboards:', e)
  }
}

onMounted(async () => {
  await Promise.all([loadTimers(), loadLeaderboards()])

  refreshInterval = setInterval(() => {
    now.value = Date.now()
  }, 30_000)

  unlisten = await listen<string[]>('game-state-updated', (event) => {
    if (event.payload.includes('milking')) {
      loadTimers()
      loadLeaderboards()
    }
  })
})

onUnmounted(() => {
  if (refreshInterval) clearInterval(refreshInterval)
  if (unlisten) unlisten()
})
</script>
