import { computed, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useGameDataStore } from '../stores/gameDataStore'
import { useGameStateStore } from '../stores/gameStateStore'
import { useSettingsStore } from '../stores/settingsStore'
import type { NpcInfo } from '../types/gameData/npcs'

export interface GiftLogEntry {
  npc_key: string
  npc_name: string
  gifted_at: string
  favor_delta: number
}

export interface StatehelmNpcStatus {
  npc: NpcInfo
  giftsThisWeek: number
  maxGifts: number
  giftLog: GiftLogEntry[]
  favorTier: string | null
}

const MAX_GIFTS_PER_WEEK = 5

/** Get the Monday 00:00 UTC boundary for the current week */
function getCurrentWeekStart(): Date {
  const now = new Date()
  const utcDay = now.getUTCDay() // 0=Sun, 1=Mon, ...
  const daysSinceMonday = utcDay === 0 ? 6 : utcDay - 1
  const monday = new Date(Date.UTC(
    now.getUTCFullYear(),
    now.getUTCMonth(),
    now.getUTCDate() - daysSinceMonday,
    0, 0, 0, 0
  ))
  return monday
}

export function useStatehelmTracker() {
  const gameData = useGameDataStore()
  const gameState = useGameStateStore()
  const settings = useSettingsStore()

  const giftLog = ref<GiftLogEntry[]>([])
  const loading = ref(false)

  const statehelmNpcs = computed<NpcInfo[]>(() => {
    const allNpcs = Object.values(gameData.npcsByKey)
    return allNpcs.filter(npc => {
      const area = (npc.area_friendly_name ?? npc.area_name ?? '').toLowerCase()
      return area.includes('statehelm') && npc.preferences.length > 0
    }).sort((a, b) => a.name.localeCompare(b.name))
  })

  const weekStart = computed(() => getCurrentWeekStart())
  const weekStartIso = computed(() => weekStart.value.toISOString())

  const giftsThisWeek = computed(() => {
    const cutoff = weekStartIso.value
    const counts: Record<string, GiftLogEntry[]> = {}
    for (const entry of giftLog.value) {
      if (entry.gifted_at >= cutoff) {
        if (!counts[entry.npc_key]) counts[entry.npc_key] = []
        counts[entry.npc_key].push(entry)
      }
    }
    return counts
  })

  const npcStatuses = computed<StatehelmNpcStatus[]>(() => {
    return statehelmNpcs.value.map(npc => {
      const gifts = giftsThisWeek.value[npc.key] ?? []
      const favorData = gameState.favorByNpc[npc.key]
      return {
        npc,
        giftsThisWeek: gifts.length,
        maxGifts: MAX_GIFTS_PER_WEEK,
        giftLog: gifts,
        favorTier: favorData?.favor_tier ?? null,
      }
    })
  })

  const totalGiftsGiven = computed(() =>
    npcStatuses.value.reduce((sum, s) => sum + s.giftsThisWeek, 0)
  )

  const totalGiftsMax = computed(() =>
    npcStatuses.value.length * MAX_GIFTS_PER_WEEK
  )

  async function loadGiftLog() {
    const characterName = settings.settings.activeCharacterName
    const serverName = settings.settings.activeServerName
    if (!characterName || !serverName) return

    loading.value = true
    try {
      giftLog.value = await invoke<GiftLogEntry[]>('get_gift_log', {
        characterName,
        serverName,
      })
    } catch (e) {
      console.error('Failed to load gift log:', e)
    } finally {
      loading.value = false
    }
  }

  async function addGift(npcKey: string, npcName: string) {
    const characterName = settings.settings.activeCharacterName
    const serverName = settings.settings.activeServerName
    if (!characterName || !serverName) return

    await invoke('add_manual_gift', {
      characterName,
      serverName,
      npcKey,
      npcName,
    })
    await loadGiftLog()
  }

  async function removeGift(npcKey: string) {
    const characterName = settings.settings.activeCharacterName
    const serverName = settings.settings.activeServerName
    if (!characterName || !serverName) return

    await invoke('remove_last_gift', {
      characterName,
      serverName,
      npcKey,
      weekStart: weekStartIso.value,
    })
    await loadGiftLog()
  }

  // Watch for real-time gift events via the favor activity feed
  watch(
    () => gameState.favorChanges,
    (changes) => {
      if (changes.length === 0) return
      const latest = changes[0]
      if (latest.detail === 'gift') {
        loadGiftLog()
      }
    },
    { deep: true }
  )

  // Also watch the DB-backed favor data — this is always refreshed by
  // game-state-updated events regardless of which page is active, so
  // the gift log stays current even when the statehelm page isn't open.
  watch(
    () => gameState.favor,
    () => loadGiftLog(),
    { deep: true }
  )

  return {
    statehelmNpcs,
    npcStatuses,
    totalGiftsGiven,
    totalGiftsMax,
    loading,
    loadGiftLog,
    addGift,
    removeGift,
    weekStart,
  }
}
