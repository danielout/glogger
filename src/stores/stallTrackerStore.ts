import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type { StallEvent, StallStats } from '../types/stallTracker'

export const useStallTrackerStore = defineStore('stallTracker', () => {
  const sales = ref<StallEvent[]>([])
  const shopLog = ref<StallEvent[]>([])
  const stats = ref<StallStats | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function loadSales(limit?: number, offset?: number) {
    loading.value = true
    error.value = null
    try {
      sales.value = await invoke<StallEvent[]>('get_stall_sales', { limit, offset })
    } catch (e) {
      error.value = String(e)
      console.error('[stallTrackerStore] Failed to load sales:', e)
    } finally {
      loading.value = false
    }
  }

  async function loadShopLog(limit?: number, offset?: number) {
    loading.value = true
    error.value = null
    try {
      shopLog.value = await invoke<StallEvent[]>('get_stall_log', { limit, offset })
    } catch (e) {
      error.value = String(e)
      console.error('[stallTrackerStore] Failed to load shop log:', e)
    } finally {
      loading.value = false
    }
  }

  async function loadStats() {
    try {
      stats.value = await invoke<StallStats>('get_stall_stats')
    } catch (e) {
      console.error('[stallTrackerStore] Failed to load stats:', e)
    }
  }

  async function loadAll() {
    await Promise.all([loadSales(), loadShopLog(), loadStats()])
  }

  async function toggleIgnored(id: number, ignored: boolean) {
    await invoke('toggle_stall_event_ignored', { id, ignored })
    await loadAll()
  }

  async function clearAll(): Promise<number> {
    const deleted = await invoke<number>('clear_stall_events')
    await loadAll()
    return deleted
  }

  // Listen for real-time updates from the coordinator
  listen<number>('stall-events-updated', () => {
    loadAll()
  })

  return {
    sales,
    shopLog,
    stats,
    loading,
    error,
    loadSales,
    loadShopLog,
    loadStats,
    loadAll,
    toggleIgnored,
    clearAll,
  }
})
