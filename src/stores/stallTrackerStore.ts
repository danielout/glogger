import { defineStore } from 'pinia'
import { ref, computed, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type { StallStats } from '../types/stallTracker'
import { useSettingsStore } from './settingsStore'

export interface StallFilterOptions {
  buyers: string[]
  players: string[]
  items: string[]
  dates: string[]
  actions: string[]
}

export interface StallStatsFilters {
  owner?: string | null
  action?: string | null
  player?: string | null
  item?: string | null
  date_from?: string | null
  date_to?: string | null
  include_ignored?: boolean
}

export const useStallTrackerStore = defineStore('stallTracker', () => {
  const settingsStore = useSettingsStore()

  const stats = ref<StallStats | null>(null)
  const filterOptions = ref<StallFilterOptions>({ buyers: [], players: [], items: [], dates: [], actions: [] })

  // All stall tracker queries are scoped to the active character. The
  // backend stores data from every character that has ever been played,
  // but the UI only ever shows the currently-loaded one — otherwise
  // multi-character users would see aggregated revenue, mixed tier stacks,
  // etc. Returns null only if no character has been selected yet.
  //
  // Empty strings are normalized to null so a misconfigured settings file
  // can't silently turn Clear into a wipe-all-characters operation.
  const currentOwner = computed<string | null>(() => {
    const name = settingsStore.settings.activeCharacterName
    return name && name.length > 0 ? name : null
  })

  // Monotonic version bumped when stall events change (coordinator insert,
  // toggle, clear, seed). Tabs watch this to trigger local re-fetches.
  const dataVersion = ref(0)

  async function loadStats(filters?: StallStatsFilters) {
    const scoped: StallStatsFilters = { ...(filters ?? {}), owner: currentOwner.value }
    try {
      stats.value = await invoke<StallStats>('get_stall_stats', { filters: scoped })
    } catch (e) {
      console.error('[stallTrackerStore] Failed to load stats:', e)
    }
  }

  async function loadFilterOptions() {
    try {
      filterOptions.value = await invoke<StallFilterOptions>('get_stall_filter_options', {
        owner: currentOwner.value,
      })
    } catch (e) {
      console.error('[stallTrackerStore] Failed to load filter options:', e)
    }
  }

  async function toggleIgnored(id: number, ignored: boolean) {
    await invoke('toggle_stall_event_ignored', { id, ignored })
    dataVersion.value++
  }

  async function clearAll(): Promise<number> {
    const deleted = await invoke<number>('clear_stall_events', { owner: currentOwner.value })
    dataVersion.value++
    await Promise.all([loadStats(), loadFilterOptions()])
    return deleted
  }

  // When the active character changes, every tab's data becomes stale.
  // Bumping dataVersion + refreshing shared metadata forces a full refetch.
  watch(currentOwner, () => {
    dataVersion.value++
    loadStats()
    loadFilterOptions()
  })

  // Real-time updates from the coordinator. Debounced because Player.log
  // catch-up can insert many books in a burst; a single trailing-edge refresh
  // is enough and avoids refetching on every batch.
  let coordTimer: ReturnType<typeof setTimeout> | null = null
  listen<number>('stall-events-updated', () => {
    if (coordTimer) clearTimeout(coordTimer)
    coordTimer = setTimeout(() => {
      dataVersion.value++
      loadStats()
      loadFilterOptions()
    }, 500)
  })

  // Dev benchmark helpers — exposed on window so you can call from DevTools:
  //   await stallBench.seed(100000)
  //   await stallBench.clear()
  //   await stallBench.bump()   // force tabs to reload
  if (typeof window !== 'undefined') {
    ;(window as unknown as { stallBench: unknown }).stallBench = {
      seed: async (count: number) => {
        const result = await invoke('seed_stall_events_dev', { count, owner: currentOwner.value })
        dataVersion.value++
        await Promise.all([loadStats(), loadFilterOptions()])
        return result
      },
      clear: async () => {
        const result = await invoke('clear_stall_events', { owner: currentOwner.value })
        dataVersion.value++
        await Promise.all([loadStats(), loadFilterOptions()])
        return result
      },
      bump: () => { dataVersion.value++ },
    }
  }

  return {
    stats,
    filterOptions,
    currentOwner,
    dataVersion,
    loadStats,
    loadFilterOptions,
    toggleIgnored,
    clearAll,
  }
})
