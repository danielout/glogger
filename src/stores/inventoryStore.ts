import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { listen } from '@tauri-apps/api/event'
import type { PlayerEvent } from '../types/playerEvents'

export interface LiveInventoryItem {
  instance_id: number
  item_name: string
  item_type_id: number | null
  stack_size: number
  slot_index: number
  added_at: string
  is_new: boolean
}

export type InventoryEventKind = 'added' | 'removed' | 'stack_changed'

export interface InventoryEventLog {
  timestamp: string
  kind: InventoryEventKind
  item_name: string
  detail: string
}

const EVENT_LOG_MAX = 50

export const useInventoryStore = defineStore('inventory', () => {
  // ── State ──────────────────────────────────────────────────────────────────
  const itemMap = ref<Map<number, LiveInventoryItem>>(new Map())
  const eventLog = ref<InventoryEventLog[]>([])

  // ── Computed ───────────────────────────────────────────────────────────────
  const items = computed<LiveInventoryItem[]>(() => {
    return [...itemMap.value.values()].sort((a, b) => a.slot_index - b.slot_index)
  })

  const itemCount = computed(() => itemMap.value.size)

  const totalStacks = computed(() => {
    let total = 0
    for (const item of itemMap.value.values()) {
      total += item.stack_size
    }
    return total
  })

  const isPopulated = computed(() => itemMap.value.size > 0)

  // ── Internal Helpers ───────────────────────────────────────────────────────

  function pushEvent(kind: InventoryEventKind, item_name: string, detail: string, timestamp: string) {
    eventLog.value.unshift({ timestamp, kind, item_name, detail })
    if (eventLog.value.length > EVENT_LOG_MAX) {
      eventLog.value.length = EVENT_LOG_MAX
    }
  }

  function clearInventory() {
    itemMap.value = new Map()
    eventLog.value = []
  }

  // ── Event Handlers ─────────────────────────────────────────────────────────

  function handlePlayerEvent(event: PlayerEvent) {
    switch (event.kind) {
      case 'ItemAdded': {
        const entry: LiveInventoryItem = {
          instance_id: event.instance_id,
          item_name: event.item_name,
          item_type_id: null,
          stack_size: 0, // will be set by ItemStackChanged
          slot_index: event.slot_index,
          added_at: event.timestamp,
          is_new: event.is_new,
        }
        const newMap = new Map(itemMap.value)
        newMap.set(event.instance_id, entry)
        itemMap.value = newMap

        if (event.is_new) {
          pushEvent('added', event.item_name, `Slot ${event.slot_index}`, event.timestamp)
        }
        break
      }

      case 'ItemStackChanged': {
        const newMap = new Map(itemMap.value)
        const existing = newMap.get(event.instance_id)

        if (existing) {
          const updated = { ...existing, stack_size: event.new_stack_size }
          if (event.item_type_id && !existing.item_type_id) {
            updated.item_type_id = event.item_type_id
          }
          newMap.set(event.instance_id, updated)

          // Only log stack changes for items acquired during play (not login burst)
          if (existing.is_new && event.delta !== 0) {
            const sign = event.delta > 0 ? '+' : ''
            pushEvent('stack_changed', existing.item_name, `${sign}${event.delta} (now ${event.new_stack_size})`, event.timestamp)
          }
        } else {
          // ItemStackChanged arrived before ItemAdded (or standalone)
          const name = event.item_name ?? 'Unknown Item'
          newMap.set(event.instance_id, {
            instance_id: event.instance_id,
            item_name: name,
            item_type_id: event.item_type_id,
            stack_size: event.new_stack_size,
            slot_index: -1,
            added_at: event.timestamp,
            is_new: false,
          })
        }

        itemMap.value = newMap
        break
      }

      case 'ItemDeleted': {
        const newMap = new Map(itemMap.value)
        const removed = newMap.get(event.instance_id)
        newMap.delete(event.instance_id)
        itemMap.value = newMap

        if (removed) {
          const contextLabel = event.context === 'StorageTransfer' ? 'stored'
            : event.context === 'VendorSale' ? 'sold'
            : event.context === 'Consumed' ? 'consumed'
            : 'removed'
          pushEvent('removed', removed.item_name, contextLabel, event.timestamp)
        }
        break
      }
    }
  }

  // ── Event Listeners ────────────────────────────────────────────────────────

  listen<PlayerEvent>('player-event', (event) => {
    handlePlayerEvent(event.payload)
  })

  listen<string>('character-login', () => {
    clearInventory()
  })

  // ── Public API ─────────────────────────────────────────────────────────────

  return {
    items,
    itemCount,
    totalStacks,
    isPopulated,
    eventLog,
    clearInventory,
  }
})
