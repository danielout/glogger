import { ref, computed, watch, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useBuildPlannerStore } from '../stores/buildPlannerStore'
import { EQUIPMENT_SLOTS, getSlotCraftingPoints } from '../types/buildPlanner'

export interface ItemAttribute {
  label: string
  value: number
  formattedValue: string
  iconId: number | null
}

export function formatStatValue(value: number, displayType: string): string {
  switch (displayType) {
    case 'AsPercent':
    case 'AsBuffMod':
      return `+${Math.round(value * 100)}%`
    case 'AsDebuffMod':
      return `-${Math.round(Math.abs(value) * 100)}%`
    case 'AsBuffDelta':
    case 'AsInt':
      return `+${Math.round(value)}`
    case 'AsDebuffDelta':
      return `${Math.round(value)}`
    case 'AsBool':
      return 'Yes'
    default:
      if (value === 0) return ''
      return value > 0 ? `+${Math.round(value)}` : `${Math.round(value)}`
  }
}

export function useBuildStats() {
  const store = useBuildPlannerStore()
  const itemAttributes = ref<ItemAttribute[]>([])

  const totalCPBudget = computed(() => {
    let total = 0
    for (const slot of EQUIPMENT_SLOTS) {
      total += getSlotCraftingPoints(store.getSlotItem(slot.id))
    }
    return total
  })

  const totalCPUsed = computed(() => {
    let used = 0
    for (const slot of EQUIPMENT_SLOTS) {
      used += store.getSlotCpUsed(slot.id)
    }
    return used
  })

  async function refreshItemAttributes() {
    const totals = new Map<string, { value: number; displayType: string; iconId: number | null }>()

    for (const slot of EQUIPMENT_SLOTS) {
      const slotItem = store.getSlotItem(slot.id)
      if (!slotItem || slotItem.item_id === 0) continue

      try {
        const item = await invoke<{
          effect_descs: string[]
        } | null>('resolve_item', { reference: String(slotItem.item_id) })
        if (!item?.effect_descs?.length) continue

        const resolved = await invoke<Array<{
          label: string
          value: string
          display_type: string
          formatted: string
          icon_id: number | null
        }>>('resolve_effect_descs', { descs: item.effect_descs })

        for (const eff of resolved) {
          const numVal = parseFloat(eff.value) || 0
          if (numVal === 0 && eff.display_type !== 'AsBool') continue

          const existing = totals.get(eff.label)
          if (existing) {
            existing.value += numVal
          } else {
            totals.set(eff.label, {
              value: numVal,
              displayType: eff.display_type,
              iconId: eff.icon_id,
            })
          }
        }
      } catch {
        // Item might not resolve
      }
    }

    const results: ItemAttribute[] = []
    for (const [label, data] of totals) {
      results.push({
        label,
        value: data.value,
        formattedValue: formatStatValue(data.value, data.displayType),
        iconId: data.iconId,
      })
    }
    results.sort((a, b) => Math.abs(b.value) - Math.abs(a.value))
    itemAttributes.value = results
  }

  // Auto-refresh when preset or slot items change
  onMounted(() => {
    refreshItemAttributes()
  })

  watch(() => store.activePreset?.id, () => {
    itemAttributes.value = []
    refreshItemAttributes()
  })

  // Watch slot items for changes (item assignments)
  watch(() => store.slotItems, () => {
    refreshItemAttributes()
  })

  return {
    itemAttributes,
    totalCPBudget,
    totalCPUsed,
    refreshItemAttributes,
  }
}
