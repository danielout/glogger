import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type { StallStats } from '../types/stallTracker'

export interface StallFilterOptions {
  buyers: string[]
  players: string[]
  items: string[]
  dates: string[]
  actions: string[]
}

export interface StallStatsFilters {
  action?: string | null
  player?: string | null
  item?: string | null
  date_from?: string | null
  date_to?: string | null
  include_ignored?: boolean
}

export const useStallTrackerStore = defineStore('stallTracker', () => {
  const stats = ref<StallStats | null>(null)
  const filterOptions = ref<StallFilterOptions>({ buyers: [], players: [], items: [], dates: [], actions: [] })

  // Monotonic version bumped when stall events change (coordinator insert,
  // toggle, clear, seed). Tabs watch this to trigger local re-fetches.
  const dataVersion = ref(0)

  async function loadStats(filters?: StallStatsFilters) {
    try {
      stats.value = await invoke<StallStats>('get_stall_stats', { filters: filters ?? null })
    } catch (e) {
      console.error('[stallTrackerStore] Failed to load stats:', e)
    }
  }

  async function loadFilterOptions() {
    try {
      filterOptions.value = await invoke<StallFilterOptions>('get_stall_filter_options')
    } catch (e) {
      console.error('[stallTrackerStore] Failed to load filter options:', e)
    }
  }

  async function toggleIgnored(id: number, ignored: boolean) {
    await invoke('toggle_stall_event_ignored', { id, ignored })
    dataVersion.value++
  }

  async function clearAll(): Promise<number> {
    const deleted = await invoke<number>('clear_stall_events')
    dataVersion.value++
    await Promise.all([loadStats(), loadFilterOptions()])
    return deleted
  }

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
        const result = await invoke('seed_stall_events_dev', { count })
        dataVersion.value++
        await Promise.all([loadStats(), loadFilterOptions()])
        return result
      },
      clear: async () => {
        const result = await invoke('clear_stall_events')
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
    dataVersion,
    loadStats,
    loadFilterOptions,
    toggleIgnored,
    clearAll,
  }
})
