<template>
  <div class="max-w-4xl h-full overflow-y-auto">
    <div class="flex items-center justify-between mb-4">
      <h2 class="text-accent-gold text-2xl m-0">Market Values</h2>
      <div class="flex items-center gap-2">
        <select
          :value="settingsStore.settings.marketPriceMode"
          class="input text-sm"
          @change="updateMode(($event.target as HTMLSelectElement).value)">
          <option value="universal">Universal prices</option>
          <option value="per_server">Per-server prices</option>
        </select>
        <button class="btn btn-secondary text-sm" @click="doExport">Export</button>
        <button class="btn btn-secondary text-sm" @click="showImport = true">Import</button>
      </div>
    </div>

    <p class="text-text-muted text-sm mb-4">
      Set player-to-player market values for items. These appear in item tooltips and are used for wealth calculations.
      <span v-if="settingsStore.settings.marketPriceMode === 'universal'" class="text-text-secondary">
        Prices are shared across all servers.
      </span>
      <span v-else class="text-text-secondary">
        Prices are specific to {{ settingsStore.settings.activeServerName ?? 'current server' }}.
      </span>
    </p>

    <!-- Item Valuation Mode -->
    <div class="card p-3 mb-4">
      <div class="flex items-center gap-3">
        <label class="text-text-muted text-sm whitespace-nowrap">Item valuation for calculations:</label>
        <select
          :value="settingsStore.settings.itemValuationMode"
          class="input text-sm flex-1"
          @change="updateValuationMode(($event.target as HTMLSelectElement).value)">
          <option value="highest_market_vendor">Highest of market or vendor value</option>
          <option value="highest_market_buy_used">Highest of market or buy-used value (2x vendor)</option>
          <option value="vendor_only">Vendor value only</option>
          <option value="buy_used_only">Buy-used value only (2x vendor)</option>
          <option value="market_only">Market value only</option>
        </select>
      </div>
      <p class="text-text-dim text-xs mt-1.5 mb-0">
        <template v-if="settingsStore.settings.itemValuationMode === 'vendor_only' || settingsStore.settings.itemValuationMode === 'buy_used_only' || settingsStore.settings.itemValuationMode === 'market_only'">
          Tooltips still show both vendor and market values, but only the selected source is used in wealth calculations.
        </template>
        <template v-else>
          Uses the higher of the two values for wealth calculations. Both are always shown in tooltips.
        </template>
      </p>
    </div>

    <!-- Search + Add -->
    <div class="flex gap-2 mb-4">
      <input
        v-model="search"
        placeholder="Search items..."
        class="input flex-1" />
      <button class="btn btn-primary text-sm" @click="openAddForm" v-if="!showAddForm">
        + Add Value
      </button>
    </div>

    <!-- Add form -->
    <div v-if="showAddForm" class="card p-4 mb-4">
      <div class="flex gap-2 items-end">
        <div class="flex-1 relative">
          <label class="text-text-muted text-xs block mb-1">Item</label>
          <input
            ref="itemSearchInput"
            v-model="addQuery"
            class="input w-full"
            placeholder="Search for an item..."
            autocomplete="off"
            @input="onItemSearch"
            @keydown.down.prevent="itemDropdownIndex = Math.min(itemDropdownIndex + 1, itemSuggestions.length - 1)"
            @keydown.up.prevent="itemDropdownIndex = Math.max(itemDropdownIndex - 1, 0)"
            @keydown.enter.prevent="selectSuggestion(itemSuggestions[itemDropdownIndex])"
            @keydown.escape="itemSuggestions = []" />
          <ul
            v-if="itemSuggestions.length > 0 && !addId"
            class="absolute z-10 left-0 right-0 top-full mt-0.5 bg-surface-card border border-border-default rounded shadow-lg max-h-48 overflow-y-auto list-none m-0 p-0">
            <li
              v-for="(item, idx) in itemSuggestions"
              :key="item.id"
              class="px-3 py-1.5 text-sm cursor-pointer hover:bg-surface-elevated"
              :class="{ 'bg-surface-elevated': idx === itemDropdownIndex }"
              @mousedown.prevent="selectSuggestion(item)">
              <span class="text-text-primary">{{ item.name }}</span>
              <span class="text-text-dim text-xs ml-2">#{{ item.id }}</span>
            </li>
          </ul>
        </div>
        <div class="w-32">
          <label class="text-text-muted text-xs block mb-1">Price (councils)</label>
          <input v-model.number="addPrice" type="number" min="0" class="input w-full" placeholder="0" />
        </div>
        <div class="w-40">
          <label class="text-text-muted text-xs block mb-1">Notes (optional)</label>
          <input v-model="addNotes" class="input w-full" placeholder="" />
        </div>
        <button
          class="btn btn-primary text-sm"
          :disabled="!addName.trim() || !addId || addPrice == null"
          @click="addValue">
          Save
        </button>
        <button class="btn btn-secondary text-sm" @click="cancelAdd">Cancel</button>
      </div>
    </div>

    <!-- Market values table -->
    <div v-if="marketStore.loading" class="text-text-muted py-8 text-center">Loading...</div>

    <EmptyState
      v-else-if="filteredValues.length === 0"
      variant="panel"
      :primary="search ? 'No matching market values' : 'No market values set'"
      :secondary="search ? undefined : 'Use item tooltips or the Add button to set prices.'" />

    <table v-else class="w-full text-sm">
      <thead>
        <tr class="text-text-muted text-xs text-left border-b border-border-default">
          <th class="py-2 pr-4 font-normal cursor-pointer hover:text-text-primary" @click="toggleSort('item_name')">
            Item {{ sortIcon('item_name') }}
          </th>
          <th class="py-2 pr-4 font-normal text-right cursor-pointer hover:text-text-primary" @click="toggleSort('market_value')">
            Price {{ sortIcon('market_value') }}
          </th>
          <th class="py-2 pr-4 font-normal">Notes</th>
          <th class="py-2 pr-4 font-normal cursor-pointer hover:text-text-primary" @click="toggleSort('updated_at')">
            Updated {{ sortIcon('updated_at') }}
          </th>
          <th class="py-2 w-16"></th>
        </tr>
      </thead>
      <tbody>
        <tr
          v-for="val in filteredValues"
          :key="val.item_type_id"
          class="border-b border-border-default/50 hover:bg-surface-elevated/30">
          <td class="py-2 pr-4 text-text-primary">
            {{ val.item_name }}
            <span class="text-text-muted text-xs ml-1">#{{ val.item_type_id }}</span>
          </td>
          <td class="py-2 pr-4 text-right text-accent-gold">
            <span v-if="editingId !== val.item_type_id">{{ val.market_value.toLocaleString() }}g</span>
            <input
              v-else
              v-model.number="editPrice"
              type="number"
              min="0"
              class="w-24 bg-surface-dark border border-border-default rounded px-1 py-0.5 text-sm text-text-primary text-right"
              @keydown.enter="saveEdit(val)"
              @keydown.escape="editingId = null" />
          </td>
          <td class="py-2 pr-4 text-text-muted text-xs">{{ val.notes ?? '' }}</td>
          <td class="py-2 pr-4 text-text-muted text-xs">{{ formatDate(val.updated_at) }}</td>
          <td class="py-2 text-right">
            <button
              v-if="editingId !== val.item_type_id"
              class="text-text-muted hover:text-text-primary bg-transparent border-none cursor-pointer text-xs mr-1"
              @click="startEdit(val)">
              Edit
            </button>
            <button
              v-if="editingId === val.item_type_id"
              class="text-accent-green hover:text-green-400 bg-transparent border-none cursor-pointer text-xs mr-1"
              @click="saveEdit(val)">
              Save
            </button>
            <button
              class="text-text-muted hover:text-red-400 bg-transparent border-none cursor-pointer text-xs"
              @click="deleteValue(val.item_type_id)">
              Del
            </button>
          </td>
        </tr>
      </tbody>
    </table>

    <div v-if="filteredValues.length > 0" class="text-text-muted text-xs mt-2">
      {{ filteredValues.length }} of {{ marketStore.values.length }} values shown
    </div>

    <!-- Import dialog -->
    <div v-if="showImport" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click.self="showImport = false">
      <div class="bg-surface-card border border-border-default rounded-lg p-6 w-full max-w-lg">
        <h3 class="text-text-primary text-lg mb-4 mt-0">Import Market Values</h3>
        <div class="mb-3">
          <label class="text-text-muted text-sm block mb-1">Paste JSON data</label>
          <textarea
            v-model="importJson"
            rows="6"
            class="input w-full font-mono text-xs"
            placeholder='[{"item_type_id": 123, "item_name": "Item", "market_value": 500, "notes": null, "updated_at": "2026-01-01 00:00:00"}]'
          ></textarea>
        </div>
        <div class="mb-4">
          <label class="text-text-muted text-sm block mb-1">Conflict resolution</label>
          <select v-model="importStrategy" class="input">
            <option value="newest">Accept newest (by updated_at)</option>
            <option value="overwrite">Overwrite all</option>
            <option value="keep_existing">Keep existing</option>
          </select>
        </div>
        <div v-if="importResult" class="text-sm mb-3 p-2 bg-surface-elevated rounded">
          Imported: {{ importResult.imported }} | Updated: {{ importResult.updated }} | Skipped: {{ importResult.skipped }}
        </div>
        <div class="flex gap-2 justify-end">
          <button class="btn btn-secondary" @click="showImport = false">Close</button>
          <button class="btn btn-primary" :disabled="!importJson.trim()" @click="doImport">Import</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, nextTick } from 'vue'
import { useToast } from '../../composables/useToast'
import { formatRelative } from '../../composables/useTimestamp'
import { useMarketStore, type MarketValue, type ImportMarketValuesResult } from '../../stores/marketStore'
import { useSettingsStore } from '../../stores/settingsStore'
import { useGameDataStore } from '../../stores/gameDataStore'
import type { ItemInfo } from '../../types/gameData'
import EmptyState from '../Shared/EmptyState.vue'

const toast = useToast()
const marketStore = useMarketStore()
const settingsStore = useSettingsStore()
const gameDataStore = useGameDataStore()

const search = ref('')
const sortField = ref<'item_name' | 'market_value' | 'updated_at'>('item_name')
const sortAsc = ref(true)

// Add form
const showAddForm = ref(false)
const addName = ref('')
const addId = ref<number | null>(null)
const addPrice = ref<number | null>(null)
const addNotes = ref('')
const addQuery = ref('')
const itemSuggestions = ref<ItemInfo[]>([])
const itemDropdownIndex = ref(0)
const itemSearchInput = ref<HTMLInputElement | null>(null)
let searchDebounce: ReturnType<typeof setTimeout> | null = null

// Edit
const editingId = ref<number | null>(null)
const editPrice = ref(0)

// Import
const showImport = ref(false)
const importJson = ref('')
const importStrategy = ref('newest')
const importResult = ref<ImportMarketValuesResult | null>(null)

const filteredValues = computed(() => {
  let vals = [...marketStore.values]

  if (search.value.trim()) {
    const q = search.value.toLowerCase()
    vals = vals.filter(v =>
      v.item_name.toLowerCase().includes(q) ||
      v.item_type_id.toString().includes(q) ||
      (v.notes?.toLowerCase().includes(q) ?? false)
    )
  }

  vals.sort((a, b) => {
    let cmp = 0
    if (sortField.value === 'item_name') {
      cmp = a.item_name.localeCompare(b.item_name)
    } else if (sortField.value === 'market_value') {
      cmp = a.market_value - b.market_value
    } else {
      cmp = a.updated_at.localeCompare(b.updated_at)
    }
    return sortAsc.value ? cmp : -cmp
  })

  return vals
})

function toggleSort(field: 'item_name' | 'market_value' | 'updated_at') {
  if (sortField.value === field) {
    sortAsc.value = !sortAsc.value
  } else {
    sortField.value = field
    sortAsc.value = true
  }
}

function sortIcon(field: string): string {
  if (sortField.value !== field) return ''
  return sortAsc.value ? '\u25B2' : '\u25BC'
}

function formatDate(dateStr: string): string {
  if (!dateStr) return ''
  return formatRelative(dateStr)
}

function openAddForm() {
  showAddForm.value = true
  nextTick(() => itemSearchInput.value?.focus())
}

function onItemSearch() {
  // Clear selection when user edits the query
  addId.value = null
  addName.value = ''
  itemDropdownIndex.value = 0

  if (searchDebounce) clearTimeout(searchDebounce)
  const q = addQuery.value.trim()
  if (!q) {
    itemSuggestions.value = []
    return
  }
  searchDebounce = setTimeout(async () => {
    try {
      let items = await gameDataStore.searchItems(q)
      if (!settingsStore.settings.showUnobtainableItems) {
        items = items.filter(i => !i.keywords.includes('Lint_NotObtainable'))
      }
      itemSuggestions.value = items
    } catch {
      itemSuggestions.value = []
    }
  }, 200)
}

function selectSuggestion(item: ItemInfo | undefined) {
  if (!item) return
  addId.value = item.id
  addName.value = item.name
  addQuery.value = item.name
  itemSuggestions.value = []
}

function cancelAdd() {
  showAddForm.value = false
  addQuery.value = ''
  addName.value = ''
  addId.value = null
  addPrice.value = null
  addNotes.value = ''
  itemSuggestions.value = []
}

async function addValue() {
  if (!addName.value.trim() || !addId.value || addPrice.value == null) return
  await marketStore.setValue(addId.value, addName.value.trim(), addPrice.value, addNotes.value || undefined)
  cancelAdd()
  showAddForm.value = false  // cancelAdd already does this, but explicit for clarity
}

function startEdit(val: MarketValue) {
  editingId.value = val.item_type_id
  editPrice.value = val.market_value
}

async function saveEdit(val: MarketValue) {
  await marketStore.setValue(val.item_type_id, val.item_name, editPrice.value, val.notes ?? undefined)
  editingId.value = null
}

async function deleteValue(itemTypeId: number) {
  await marketStore.deleteValue(itemTypeId)
}

async function doExport() {
  try {
    const json = await marketStore.exportValues()
    await navigator.clipboard.writeText(json)
    toast.success('Market values copied to clipboard')
  } catch (e) {
    console.error('Export failed:', e)
  }
}

async function doImport() {
  try {
    importResult.value = await marketStore.importValues(importJson.value, importStrategy.value)
  } catch (e) {
    console.error('Import failed:', e)
    toast.error('Import failed: ' + String(e))
  }
}

async function updateMode(mode: string) {
  await settingsStore.updateSettings({ marketPriceMode: mode })
  await marketStore.loadAll()
}

async function updateValuationMode(mode: string) {
  await settingsStore.updateSettings({ itemValuationMode: mode })
}

onMounted(() => {
  if (marketStore.values.length === 0) {
    marketStore.loadAll()
  }
})
</script>
