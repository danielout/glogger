<template>
  <div class="flex flex-col gap-2 h-full">
    <!-- Filters -->
    <div class="flex items-center gap-3 flex-wrap">
      <input
        v-model="filterText"
        type="text"
        placeholder="Filter items..."
        class="px-3 py-1.5 bg-surface-base border border-border-default rounded text-sm text-text-primary placeholder-text-muted focus:outline-none focus:border-accent-gold/50 w-48" />

      <select
        v-model="filterVault"
        class="px-3 py-1.5 bg-surface-base border border-border-default rounded text-sm text-text-primary cursor-pointer">
        <option value="">All Locations</option>
        <option v-for="v in vaultOptions" :key="v" :value="v">{{ formatVault(v) }}</option>
      </select>

      <select
        v-model="filterRarity"
        class="px-3 py-1.5 bg-surface-base border border-border-default rounded text-sm text-text-primary cursor-pointer">
        <option value="">All Rarities</option>
        <option v-for="r in rarityOptions" :key="r" :value="r">{{ r }}</option>
      </select>

      <select
        v-model="filterSlot"
        class="px-3 py-1.5 bg-surface-base border border-border-default rounded text-sm text-text-primary cursor-pointer">
        <option value="">All Slots</option>
        <option v-for="s in slotOptions" :key="s" :value="s">{{ s }}</option>
      </select>

      <span class="text-xs text-text-muted">{{ filtered.length }} items</span>
    </div>

    <!-- Table -->
    <div class="overflow-auto flex-1 min-h-0">
      <table class="w-full text-sm border-collapse">
        <thead class="sticky top-0 bg-surface-base z-10">
          <tr class="text-left text-text-secondary border-b border-border-default">
            <th class="py-1.5 px-2 cursor-pointer hover:text-text-primary" @click="toggleSort('item_name')">
              Item {{ sortIcon('item_name') }}
            </th>
            <th class="py-1.5 px-2 cursor-pointer hover:text-text-primary text-right" @click="toggleSort('stack_size')">
              Qty {{ sortIcon('stack_size') }}
            </th>
            <th class="py-1.5 px-2 cursor-pointer hover:text-text-primary" @click="toggleSort('location')">
              Location {{ sortIcon('location') }}
            </th>
            <th class="py-1.5 px-2 cursor-pointer hover:text-text-primary" @click="toggleSort('rarity')">
              Rarity {{ sortIcon('rarity') }}
            </th>
            <th class="py-1.5 px-2 cursor-pointer hover:text-text-primary" @click="toggleSort('slot')">
              Slot {{ sortIcon('slot') }}
            </th>
            <th class="py-1.5 px-2 cursor-pointer hover:text-text-primary text-right" @click="toggleSort('level')">
              Lvl {{ sortIcon('level') }}
            </th>
            <th class="py-1.5 px-2 cursor-pointer hover:text-text-primary text-right" @click="toggleSort('value')">
              Value {{ sortIcon('value') }}
            </th>
            <th class="py-1.5 px-2">Mods</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="item in filtered"
            :key="item.id"
            class="border-b border-border-default/50 hover:bg-surface-elevated/50">
            <td class="py-1 px-2">
              <div class="flex flex-col">
                <ItemInline :name="item.item_name" :item-id="item.type_id" />
                <span v-if="item.crafter" class="text-xs text-text-muted">
                  Crafted by {{ item.crafter }}
                </span>
              </div>
            </td>
            <td class="py-1 px-2 text-right text-text-primary">{{ item.stack_size }}</td>
            <td class="py-1 px-2 text-text-secondary text-xs">{{ formatVault(getLocation(item)) }}</td>
            <td class="py-1 px-2">
              <span v-if="item.rarity" :class="rarityClass(item.rarity)" class="text-xs font-medium">
                {{ item.rarity }}
              </span>
            </td>
            <td class="py-1 px-2 text-text-secondary text-xs">{{ item.slot ?? '' }}</td>
            <td class="py-1 px-2 text-right text-text-secondary">{{ item.level ?? '' }}</td>
            <td class="py-1 px-2 text-right text-accent-gold">
              <template v-if="item.value">
                {{ formatNumber(item.value * item.stack_size) }}
              </template>
            </td>
            <td class="py-1 px-2">
              <ItemPowerList :json="item.tsys_powers" />
              <span v-if="item.tsys_imbue_power" class="text-xs text-blue-300">
                Imbue: {{ item.tsys_imbue_power }} (T{{ item.tsys_imbue_power_tier }})
              </span>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import type { SnapshotItem } from '../../types/database'
import ItemInline from '../Shared/Item/ItemInline.vue'
import ItemPowerList from './ItemPowerList.vue'

const props = defineProps<{
  items: SnapshotItem[]
}>()

const filterText = ref('')
const filterVault = ref('')
const filterRarity = ref('')
const filterSlot = ref('')

type SortKey = 'item_name' | 'stack_size' | 'location' | 'rarity' | 'slot' | 'level' | 'value'
const sortKey = ref<SortKey>('item_name')
const sortAsc = ref(true)

function getLocation(item: SnapshotItem): string {
  if (item.is_in_inventory) return 'Inventory'
  if (!item.storage_vault) return 'Unknown'
  return item.storage_vault
}

const vaultOptions = computed(() => {
  const vaults = new Set<string>()
  for (const item of props.items) {
    vaults.add(getLocation(item))
  }
  return [...vaults].sort()
})

const rarityOptions = computed(() => {
  const rarities = new Set<string>()
  for (const item of props.items) {
    if (item.rarity) rarities.add(item.rarity)
  }
  return ['Common', 'Uncommon', 'Rare', 'Exceptional', 'Epic', 'Legendary'].filter(r => rarities.has(r) || r === 'Common')
})

const slotOptions = computed(() => {
  const slots = new Set<string>()
  for (const item of props.items) {
    if (item.slot) slots.add(item.slot)
  }
  return [...slots].sort()
})

const rarityOrder: Record<string, number> = {
  Common: 0, Uncommon: 1, Rare: 2, Exceptional: 3, Epic: 4, Legendary: 5,
}

const filtered = computed(() => {
  const text = filterText.value.toLowerCase()
  let list = props.items.filter(item => {
    if (text && !item.item_name.toLowerCase().includes(text)) return false
    if (filterVault.value && getLocation(item) !== filterVault.value) return false
    if (filterRarity.value) {
      const itemRarity = item.rarity ?? 'Common'
      if (itemRarity !== filterRarity.value) return false
    }
    if (filterSlot.value && item.slot !== filterSlot.value) return false
    return true
  })

  list.sort((a, b) => {
    const dir = sortAsc.value ? 1 : -1
    switch (sortKey.value) {
      case 'item_name':
        return a.item_name.localeCompare(b.item_name) * dir
      case 'stack_size':
        return (a.stack_size - b.stack_size) * dir
      case 'location':
        return getLocation(a).localeCompare(getLocation(b)) * dir
      case 'rarity':
        return ((rarityOrder[a.rarity ?? 'Common'] ?? 0) - (rarityOrder[b.rarity ?? 'Common'] ?? 0)) * dir
      case 'slot':
        return (a.slot ?? '').localeCompare(b.slot ?? '') * dir
      case 'level':
        return ((a.level ?? 0) - (b.level ?? 0)) * dir
      case 'value':
        return (((a.value ?? 0) * a.stack_size) - ((b.value ?? 0) * b.stack_size)) * dir
      default:
        return 0
    }
  })

  return list
})

function toggleSort(key: SortKey) {
  if (sortKey.value === key) {
    sortAsc.value = !sortAsc.value
  } else {
    sortKey.value = key
    sortAsc.value = key === 'item_name' || key === 'location' || key === 'rarity' || key === 'slot'
  }
}

function sortIcon(key: string): string {
  if (sortKey.value !== key) return ''
  return sortAsc.value ? '\u25B2' : '\u25BC'
}

function formatNumber(n: number): string {
  return n.toLocaleString()
}

const VAULT_NAMES: Record<string, string> = {
  Inventory: 'Inventory',
  CouncilVault: 'Council Vault',
  TapestryInnChest: 'Tapestry Inn Chest',
  SerbuleCommunityChest: 'Serbule Community Chest',
  DreamRealmChest: 'Dream Realm Chest',
  PovusStorageChest: 'Povus Storage Chest',
  Saddlebag: 'Saddlebag',
  Unknown: 'Unknown',
}

function formatVault(vault: string): string {
  if (VAULT_NAMES[vault]) return VAULT_NAMES[vault]
  if (vault.startsWith('*AccountStorage_')) {
    const location = vault.replace('*AccountStorage_', '')
    return `Account Storage (${location})`
  }
  if (vault.startsWith('NPC_')) {
    const npcName = vault.replace('NPC_', '')
    return `${npcName}'s Storage`
  }
  return vault
}

function rarityClass(rarity: string): string {
  switch (rarity) {
    case 'Uncommon': return 'text-green-400'
    case 'Rare': return 'text-blue-400'
    case 'Exceptional': return 'text-purple-400'
    case 'Epic': return 'text-orange-400'
    case 'Legendary': return 'text-yellow-400'
    default: return 'text-text-secondary'
  }
}
</script>
