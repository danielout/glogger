import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface MarketValue {
  server_name: string
  item_type_id: number
  item_name: string
  market_value: number
  notes: string | null
  updated_at: string
}

export interface ImportMarketValuesResult {
  imported: number
  skipped: number
  updated: number
}

export const useMarketStore = defineStore('market', () => {
  const values = ref<MarketValue[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  /** All market values indexed by item_type_id for O(1) lookup */
  const valuesByItemId = computed(() => {
    const map: Record<number, MarketValue> = {}
    for (const v of values.value) map[v.item_type_id] = v
    return map
  })

  /** All market values indexed by item_name for display lookup */
  const valuesByName = computed(() => {
    const map: Record<string, MarketValue> = {}
    for (const v of values.value) map[v.item_name] = v
    return map
  })

  async function loadAll() {
    loading.value = true
    error.value = null
    try {
      values.value = await invoke<MarketValue[]>('get_market_values', {})
    } catch (e) {
      error.value = String(e)
      console.error('[marketStore] Failed to load market values:', e)
    } finally {
      loading.value = false
    }
  }

  async function getValue(itemTypeId: number): Promise<MarketValue | null> {
    try {
      return await invoke<MarketValue | null>('get_market_value', { itemTypeId })
    } catch (e) {
      console.error('[marketStore] Failed to get market value:', e)
      return null
    }
  }

  async function setValue(itemTypeId: number, itemName: string, marketValue: number, notes?: string) {
    error.value = null
    try {
      await invoke('set_market_value', {
        itemTypeId,
        itemName,
        marketValue,
        notes: notes ?? null,
      })
      // Update local cache
      const idx = values.value.findIndex(v => v.item_type_id === itemTypeId)
      const entry: MarketValue = {
        server_name: '*',
        item_type_id: itemTypeId,
        item_name: itemName,
        market_value: marketValue,
        notes: notes ?? null,
        updated_at: new Date().toISOString(),
      }
      if (idx >= 0) {
        values.value[idx] = entry
      } else {
        values.value.push(entry)
      }
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }

  async function deleteValue(itemTypeId: number) {
    error.value = null
    try {
      await invoke('delete_market_value', { itemTypeId })
      values.value = values.value.filter(v => v.item_type_id !== itemTypeId)
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }

  async function bulkUpdateValues(updates: { item_type_id: number; market_value: number }[]) {
    error.value = null
    try {
      await invoke('bulk_update_market_values', { updates })
      // Update local cache
      for (const upd of updates) {
        const idx = values.value.findIndex(v => v.item_type_id === upd.item_type_id)
        if (idx >= 0) {
          values.value[idx] = {
            ...values.value[idx],
            market_value: upd.market_value,
            updated_at: new Date().toISOString(),
          }
        }
      }
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }

  async function bulkDeleteValues(itemIds: number[]) {
    error.value = null
    try {
      await invoke('bulk_delete_market_values', { itemIds })
      const idSet = new Set(itemIds)
      values.value = values.value.filter(v => !idSet.has(v.item_type_id))
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }

  async function exportValues(): Promise<string> {
    return invoke<string>('export_market_values', {})
  }

  async function importValues(jsonData: string, strategy: string): Promise<ImportMarketValuesResult> {
    const result = await invoke<ImportMarketValuesResult>('import_market_values', {
      jsonData,
      strategy,
    })
    // Reload after import
    await loadAll()
    return result
  }

  return {
    values,
    loading,
    error,
    valuesByItemId,
    valuesByName,
    loadAll,
    getValue,
    setValue,
    deleteValue,
    bulkUpdateValues,
    bulkDeleteValues,
    exportValues,
    importValues,
  }
})
