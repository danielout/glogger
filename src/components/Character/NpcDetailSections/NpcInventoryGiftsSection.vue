<template>
  <div class="flex flex-col gap-3">
    <!-- Inventory Gifts -->
    <div class="flex flex-col gap-1.5">
      <div
        class="text-[10px] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5 cursor-pointer select-none flex items-center gap-1"
        @click="inventoryCollapsed = !inventoryCollapsed">
        <span>{{ inventoryCollapsed ? '\u25B8' : '\u25BE' }}</span>
        <span>Giftable Items — Inventory</span>
        <span class="text-text-dim normal-case tracking-normal">({{ matchingGifts.length }})</span>
      </div>

      <template v-if="!inventoryCollapsed">
        <template v-if="matchingGifts.length">
          <div class="flex flex-col gap-0.5 px-2">
            <div
              v-for="gift in matchingGifts"
              :key="gift.itemName"
              class="flex items-center gap-2 text-xs bg-[#151515] rounded px-2 py-0.5">
              <ItemInline :reference="gift.itemName" />
              <span class="text-text-dim text-[10px]">x{{ gift.quantity }}</span>
              <span
                class="text-[10px] px-1 py-0.5 rounded border ml-auto shrink-0"
                :class="desireBadgeClasses(gift.desire)">
                {{ gift.desire }}
              </span>
              <span class="text-green-400 text-[10px] shrink-0">+{{ gift.prefValue }}</span>
            </div>
          </div>
          <div v-if="estimatedTotalFavor > 0" class="text-[10px] text-text-dim px-2">
            Estimated total favor from gifting all: <span class="text-accent-gold font-bold">~{{ estimatedTotalFavor }}</span>
          </div>
        </template>

        <div v-else class="text-xs text-text-dim italic px-2">
          No matching gifts in inventory
        </div>
      </template>
    </div>

    <!-- Storage Gifts -->
    <div class="flex flex-col gap-1.5">
      <div
        class="text-[10px] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5 cursor-pointer select-none flex items-center gap-1"
        @click="storageCollapsed = !storageCollapsed">
        <span>{{ storageCollapsed ? '\u25B8' : '\u25BE' }}</span>
        <span>Giftable Items — Storage</span>
        <span class="text-text-dim normal-case tracking-normal">({{ storageGifts.length }})</span>
      </div>

      <template v-if="!storageCollapsed">
        <template v-if="storageGifts.length">
          <div class="flex flex-col gap-0.5 px-2">
            <div
              v-for="gift in storageGifts"
              :key="`${gift.vaultKey}-${gift.itemName}`"
              class="flex items-center gap-2 text-xs bg-[#151515] rounded px-2 py-0.5">
              <ItemInline :reference="gift.itemName" />
              <span class="text-text-dim text-[10px]">x{{ gift.quantity }}</span>
              <span
                class="text-[10px] px-1 py-0.5 rounded border ml-auto shrink-0"
                :class="desireBadgeClasses(gift.desire)">
                {{ gift.desire }}
              </span>
              <span class="text-green-400 text-[10px] shrink-0">+{{ gift.prefValue }}</span>
              <span class="text-text-dim text-[10px] italic shrink-0">{{ gift.vaultLabel }}</span>
            </div>
          </div>
        </template>

        <div v-else class="text-xs text-text-dim italic px-2">
          No matching gifts in storage
        </div>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import type { NpcInfo } from '../../../types/gameData'
import type { ItemInfo } from '../../../types/gameData/items'
import { useGameStateStore } from '../../../stores/gameStateStore'
import { useGameDataStore } from '../../../stores/gameDataStore'
import { matchesPreference } from '../../../composables/useNpcGiftMatching'
import ItemInline from '../../Shared/Item/ItemInline.vue'

const props = defineProps<{
  npc: NpcInfo
}>()

const inventoryCollapsed = ref(true)
const storageCollapsed = ref(true)

const gameState = useGameStateStore()
const gameData = useGameDataStore()

interface MatchingGift {
  itemName: string
  quantity: number
  desire: string
  prefValue: number
}

interface StorageGift extends MatchingGift {
  vaultKey: string
  vaultLabel: string
}

// Cache of resolved item info keyed by item name
const resolvedItems = ref<Record<string, ItemInfo>>({})

// Collect all unique item names from inventory and storage
const allItemNames = computed<string[]>(() => {
  const owned = gameState.ownedItemCounts
  return Object.keys(owned).filter(name => owned[name] > 0)
})

// Resolve items whenever the item names list changes
watch(allItemNames, async (names) => {
  if (names.length === 0) {
    resolvedItems.value = {}
    return
  }
  try {
    resolvedItems.value = await gameData.resolveItemsBatch(names)
  } catch {
    resolvedItems.value = {}
  }
}, { immediate: true })

/**
 * Match an item (by name + resolved keywords) against NPC preferences.
 * Returns the best matching preference, or null if no match.
 */
function findBestPreference(
  itemName: string,
  prefs: { desire: string; prefValue: number; pref: typeof props.npc.preferences[number] }[],
  resolved: Record<string, ItemInfo>,
): { desire: string; prefValue: number } | null {
  const itemInfo = resolved[itemName]
  const itemKeywords = itemInfo?.keywords ?? []

  for (const p of prefs) {
    // Keyword-based matching (primary)
    if (itemKeywords.length > 0 && matchesPreference(itemKeywords, p.pref)) {
      return { desire: p.desire, prefValue: p.prefValue }
    }
    // Exact name match (fallback for prefs that have a name)
    if (p.pref.name && p.pref.name === itemName) {
      return { desire: p.desire, prefValue: p.prefValue }
    }
  }
  return null
}

const giftPrefs = computed(() => {
  const giftDesires = new Set(['Love', 'Like'])
  return props.npc.preferences
    .filter(p => giftDesires.has(p.desire))
    .map(p => ({ desire: p.desire, prefValue: p.pref, pref: p }))
    .sort((a, b) => b.prefValue - a.prefValue)
})

const matchingGifts = computed<MatchingGift[]>(() => {
  const owned = gameState.inventoryItemCounts
  if (!owned || !giftPrefs.value.length) return []

  const results: MatchingGift[] = []
  const seen = new Set<string>()

  for (const itemName of Object.keys(owned)) {
    const qty = owned[itemName]
    if (!qty || qty <= 0) continue
    if (seen.has(itemName)) continue

    const match = findBestPreference(itemName, giftPrefs.value, resolvedItems.value)
    if (match) {
      seen.add(itemName)
      results.push({
        itemName,
        quantity: qty,
        desire: match.desire,
        prefValue: match.prefValue,
      })
    }
  }

  return results.sort((a, b) => b.prefValue - a.prefValue)
})

const estimatedTotalFavor = computed(() => {
  let total = 0
  for (const gift of matchingGifts.value) {
    total += gift.prefValue * gift.quantity
  }
  return Math.round(total)
})

const storageGifts = computed<StorageGift[]>(() => {
  if (!giftPrefs.value.length) return []

  const results: StorageGift[] = []
  const allVaults = gameState.storageByVault

  for (const [vaultKey, items] of Object.entries(allVaults)) {
    if (!items) continue
    // Aggregate items by name within each vault
    const itemCounts = new Map<string, number>()
    for (const item of items) {
      const current = itemCounts.get(item.item_name) ?? 0
      itemCounts.set(item.item_name, current + item.stack_size)
    }

    for (const [itemName, qty] of itemCounts) {
      const match = findBestPreference(itemName, giftPrefs.value, resolvedItems.value)
      if (match) {
        const label = vaultKey.replace(/^NPC_/, '')
        results.push({
          itemName,
          quantity: qty,
          desire: match.desire,
          prefValue: match.prefValue,
          vaultKey,
          vaultLabel: label,
        })
      }
    }
  }

  return results.sort((a, b) => b.prefValue - a.prefValue)
})

function desireBadgeClasses(desire: string): string {
  switch (desire.toLowerCase()) {
    case 'love':
      return 'bg-red-900/30 border-red-700/40 text-red-300'
    case 'like':
      return 'bg-green-900/30 border-green-700/40 text-green-300'
    default:
      return 'bg-surface-elevated border-border-default text-text-muted'
  }
}
</script>
