import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useSettingsStore } from './settingsStore'

// ── Types ────────────────────────────────────────────────────────────────────

export interface CharacterItemBreakdown {
  character_name: string
  stack_size: number
}

export interface AggregateInventoryItem {
  item_name: string
  total_stack_size: number
  character_count: number
  characters: CharacterItemBreakdown[]
}

export interface CharacterCurrencyBreakdown {
  character_name: string
  amount: number
}

export interface AggregateCurrency {
  currency_name: string
  total_amount: number
  characters: CharacterCurrencyBreakdown[]
}

export interface CharacterWealth {
  character_name: string
  currency_total: number
  market_value_total: number
}

export interface AggregateWealth {
  total_currency: number
  total_market_value: number
  grand_total: number
  currencies: AggregateCurrency[]
  per_character: CharacterWealth[]
}

export interface CharacterSkillEntry {
  character_name: string
  level: number
  xp: number
}

export interface AggregateSkillSummary {
  skill_name: string
  characters: CharacterSkillEntry[]
}

// ── Store ────────────────────────────────────────────────────────────────────

export const useAggregateStore = defineStore('aggregate', () => {
  const settingsStore = useSettingsStore()

  const inventory = ref<AggregateInventoryItem[]>([])
  const wealth = ref<AggregateWealth | null>(null)
  const skills = ref<AggregateSkillSummary[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  const serverName = computed(() => settingsStore.settings.activeServerName)

  /** Aggregate item counts indexed by name */
  const inventoryByName = computed(() => {
    const map: Record<string, AggregateInventoryItem> = {}
    for (const item of inventory.value) map[item.item_name] = item
    return map
  })

  async function loadAll() {
    if (!serverName.value) return
    loading.value = true
    error.value = null
    try {
      const [inv, w, sk] = await Promise.all([
        invoke<AggregateInventoryItem[]>('get_aggregate_inventory', { serverName: serverName.value }),
        invoke<AggregateWealth>('get_aggregate_wealth', { serverName: serverName.value }),
        invoke<AggregateSkillSummary[]>('get_aggregate_skills', { serverName: serverName.value }),
      ])
      inventory.value = inv
      wealth.value = w
      skills.value = sk
    } catch (e) {
      error.value = String(e)
      console.error('[aggregateStore] Failed to load:', e)
    } finally {
      loading.value = false
    }
  }

  function clear() {
    inventory.value = []
    wealth.value = null
    skills.value = []
  }

  return {
    inventory,
    wealth,
    skills,
    loading,
    error,
    serverName,
    inventoryByName,
    loadAll,
    clear,
  }
})
