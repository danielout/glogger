import { defineStore } from 'pinia'
import { ref, computed, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { useSettingsStore } from './settingsStore'
import type {
  StallStats,
  StallFilterOptions,
  StallEventsFilters,
} from '../types/stallTracker'

/**
 * Shared Stall Tracker state.
 *
 * The store deliberately holds **only** the cross-tab data (stats header
 * numbers + filter dropdown options) and a `dataVersion` counter that tabs
 * watch to know when to refetch their own row data. Per-tab queries
 * (sales list, revenue pivot, inventory snapshot) are owned by the tab
 * components, not here, so the store stays small and the heavy queries
 * fire only when the user actually opens the relevant tab.
 *
 * `dataVersion` is a monotonic counter (not a boolean) so two back-to-back
 * mutations always produce a new value tabs can react to.
 */
export const useStallTrackerStore = defineStore('stallTracker', () => {
  const settingsStore = useSettingsStore()

  const stats = ref<StallStats | null>(null)
  const filterOptions = ref<StallFilterOptions>({
    buyers: [],
    players: [],
    items: [],
    dates: [],
    actions: [],
  })
  const dataVersion = ref(0)

  /** Active character, normalized — empty/whitespace becomes `null`. */
  const currentOwner = computed<string | null>(() => {
    const name = settingsStore.settings.activeCharacterName
    return name && name.trim().length > 0 ? name : null
  })

  /** True when the current character has any persisted stall data. */
  const hasData = computed<boolean>(() => (stats.value?.total_sales ?? 0) > 0)

  async function loadStats(filters?: StallEventsFilters): Promise<void> {
    if (!currentOwner.value) {
      stats.value = null
      return
    }
    try {
      const merged: StallEventsFilters = {
        ...(filters ?? {}),
        owner: currentOwner.value,
      }
      stats.value = await invoke<StallStats>('get_stall_stats', { filters: merged })
    } catch (e) {
      console.error('[stallTrackerStore] Failed to load stats:', e)
    }
  }

  async function loadFilterOptions(): Promise<void> {
    if (!currentOwner.value) {
      filterOptions.value = { buyers: [], players: [], items: [], dates: [], actions: [] }
      return
    }
    try {
      filterOptions.value = await invoke<StallFilterOptions>('get_stall_filter_options', {
        owner: currentOwner.value,
      })
    } catch (e) {
      console.error('[stallTrackerStore] Failed to load filter options:', e)
    }
  }

  /** Toggle a single event's `ignored` flag. Bumps `dataVersion` on success.
   *
   * Does NOT refresh `stats` here: stats are recomputed by the active tab's
   * reload (triggered via the dataVersion watcher), with that tab's local
   * filter set threaded through. Refreshing stats here would briefly show
   * unfiltered numbers between this call and the tab's reload. */
  async function toggleIgnored(id: number, ignored: boolean): Promise<void> {
    if (!currentOwner.value) {
      throw new Error('No active character — cannot toggle ignored flag')
    }
    await invoke('toggle_stall_event_ignored', {
      id,
      ignored,
      owner: currentOwner.value,
    })
    dataVersion.value++
  }

  /** Delete every stall event for the active character. Returns row count. */
  async function clearAll(): Promise<number> {
    if (!currentOwner.value) {
      throw new Error('No active character — cannot clear stall data')
    }
    const deleted = await invoke<number>('clear_stall_events', {
      owner: currentOwner.value,
    })
    dataVersion.value++
    await Promise.all([loadStats(), loadFilterOptions()])
    return deleted
  }

  /** Public refresh — call this from the import command after a successful import. */
  async function refresh(): Promise<void> {
    dataVersion.value++
    await Promise.all([loadStats(), loadFilterOptions()])
  }

  // Character switch: invalidate stats, reload, and bump dataVersion so all
  // tabs refetch with the new owner. Each tab also watches currentOwner
  // directly to reset its own filter state — see plan §11.
  watch(currentOwner, () => {
    dataVersion.value++
    void loadStats()
    void loadFilterOptions()
  })

  // Coordinator side-channel: live-ingest fires `stall-events-updated` after
  // every successful batch insert. Player.log catch-up can replay many books
  // in a burst, so we debounce 500ms — a single trailing refresh is enough
  // and avoids spamming the backend with identical stats queries.
  let coordTimer: ReturnType<typeof setTimeout> | null = null
  void listen<number>('stall-events-updated', () => {
    if (coordTimer) clearTimeout(coordTimer)
    coordTimer = setTimeout(() => {
      dataVersion.value++
      void loadStats()
      void loadFilterOptions()
    }, 500)
  })

  return {
    stats,
    filterOptions,
    dataVersion,
    currentOwner,
    hasData,
    loadStats,
    loadFilterOptions,
    toggleIgnored,
    clearAll,
    refresh,
  }
})
