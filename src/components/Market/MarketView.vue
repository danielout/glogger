<template>
  <div class="flex flex-col gap-4 h-full min-h-0">
    <!-- Header -->
    <div class="flex items-center justify-between shrink-0">
      <div>
        <h2 class="text-accent-gold text-xl font-semibold m-0">Market Values</h2>
        <p class="text-text-dim text-xs mt-1 mb-0">
          Player-to-player market values for item tooltips and wealth calculations.
          <span v-if="settingsStore.settings.marketPriceMode === 'universal'" class="text-text-secondary">
            Prices shared across all servers.
          </span>
          <span v-else class="text-text-secondary">
            Prices for {{ settingsStore.settings.activeServerName ?? 'current server' }}.
          </span>
        </p>
      </div>
      <div class="flex items-center gap-2">
        <select
          :value="settingsStore.settings.marketPriceMode"
          class="input text-xs"
          @change="updateMode(($event.target as HTMLSelectElement).value)">
          <option value="universal">Universal prices</option>
          <option value="per_server">Per-server prices</option>
        </select>
        <button class="btn btn-secondary text-xs" @click="doExport">Export</button>
        <button class="btn btn-secondary text-xs" @click="showImport = true">Import</button>
      </div>
    </div>

    <!-- Summary stats -->
    <div v-if="marketStore.values.length > 0" class="grid grid-cols-4 gap-3 shrink-0">
      <div class="bg-surface-elevated border border-border-default rounded px-3 py-2">
        <div class="text-[10px] uppercase tracking-wider text-text-secondary">Items Tracked</div>
        <div class="text-base text-text-primary font-semibold tabular-nums">{{ marketStore.values.length }}</div>
      </div>
      <div class="bg-surface-elevated border border-border-default rounded px-3 py-2">
        <div class="text-[10px] uppercase tracking-wider text-text-secondary">Total Market Value</div>
        <div class="text-base text-accent-gold font-semibold tabular-nums">{{ totalMarketValue.toLocaleString() }}g</div>
      </div>
      <div class="bg-surface-elevated border border-border-default rounded px-3 py-2">
        <div class="text-[10px] uppercase tracking-wider text-text-secondary">Avg Price</div>
        <div class="text-base text-text-primary font-semibold tabular-nums">{{ avgPrice.toLocaleString() }}g</div>
      </div>
      <div class="bg-surface-elevated border border-border-default rounded px-3 py-2">
        <div class="text-[10px] uppercase tracking-wider text-text-secondary">Highest Priced</div>
        <div class="text-base text-text-primary font-semibold tabular-nums truncate" :title="highestPricedItem?.item_name">
          {{ highestPricedItem ? `${highestPricedItem.market_value.toLocaleString()}g` : '--' }}
        </div>
        <div v-if="highestPricedItem" class="text-[10px] text-text-dim truncate">{{ highestPricedItem.item_name }}</div>
      </div>
    </div>

    <!-- Settings accordion -->
    <AccordionSection :default-open="false" class="shrink-0">
      <template #title>Valuation Settings</template>
      <div class="flex items-center gap-3 mt-2">
        <label class="text-text-muted text-xs whitespace-nowrap">Item valuation for calculations:</label>
        <select
          :value="settingsStore.settings.itemValuationMode"
          class="input text-xs flex-1"
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
    </AccordionSection>

    <!-- Add item accordion -->
    <AccordionSection :default-open="false" class="shrink-0">
      <template #title>Add Market Value</template>
      <div class="flex gap-2 items-end mt-2">
        <div class="flex-1 relative">
          <label class="text-text-muted text-xs block mb-1">Item</label>
          <input
            ref="itemSearchInput"
            v-model="addQuery"
            class="input w-full text-sm"
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
          <input v-model.number="addPrice" type="number" min="0" class="input w-full text-sm" placeholder="0" />
        </div>
        <div class="w-40">
          <label class="text-text-muted text-xs block mb-1">Notes (optional)</label>
          <input v-model="addNotes" class="input w-full text-sm" placeholder="" />
        </div>
        <button
          class="btn btn-primary text-xs"
          :disabled="!addName.trim() || !addId || addPrice == null"
          @click="addValue">
          Save
        </button>
        <button class="btn btn-secondary text-xs" @click="cancelAdd">Clear</button>
      </div>
    </AccordionSection>

    <!-- Search / filter bar -->
    <div class="flex items-center gap-3 shrink-0">
      <input
        v-model="search"
        placeholder="Filter market values..."
        class="input flex-1 text-sm" />
      <span v-if="search && filteredValues.length !== marketStore.values.length" class="text-text-muted text-xs whitespace-nowrap">
        {{ filteredValues.length }} of {{ marketStore.values.length }}
      </span>
    </div>

    <!-- Market values table -->
    <div class="flex-1 min-h-0 overflow-auto border border-border-default rounded">
      <!-- Loading skeleton -->
      <div v-if="marketStore.loading" class="p-4">
        <div v-for="i in 8" :key="i" class="flex gap-4 py-2 border-b border-border-default/50">
          <div class="h-3 rounded animate-pulse bg-surface-elevated w-1/3" />
          <div class="h-3 rounded animate-pulse bg-surface-elevated w-16 ml-auto" />
          <div class="h-3 rounded animate-pulse bg-surface-elevated w-24" />
          <div class="h-3 rounded animate-pulse bg-surface-elevated w-20" />
        </div>
      </div>

      <EmptyState
        v-else-if="filteredValues.length === 0"
        variant="panel"
        :primary="search ? 'No matching market values' : 'No market values set'"
        :secondary="search ? undefined : 'Use item tooltips or the Add section above to set prices.'" />

      <table v-else class="w-full border-collapse text-xs">
        <thead class="sticky top-0 z-10 bg-surface-base border-b border-border-default">
          <tr>
            <th
              class="text-[10px] uppercase tracking-wider text-text-muted font-semibold text-left px-2 py-1 cursor-pointer hover:text-text-primary select-none"
              @click="toggleSort('item_name')">
              Item
              <span v-if="sortField === 'item_name'" class="ml-0.5">{{ sortAsc ? '\u25B2' : '\u25BC' }}</span>
            </th>
            <th
              class="text-[10px] uppercase tracking-wider text-text-muted font-semibold text-right px-2 py-1 cursor-pointer hover:text-text-primary select-none w-30"
              @click="toggleSort('market_value')">
              Price
              <span v-if="sortField === 'market_value'" class="ml-0.5">{{ sortAsc ? '\u25B2' : '\u25BC' }}</span>
            </th>
            <th class="text-[10px] uppercase tracking-wider text-text-muted font-semibold text-left px-2 py-1">
              Notes
            </th>
            <th
              class="text-[10px] uppercase tracking-wider text-text-muted font-semibold text-left px-2 py-1 cursor-pointer hover:text-text-primary select-none w-30"
              @click="toggleSort('updated_at')">
              Updated
              <span v-if="sortField === 'updated_at'" class="ml-0.5">{{ sortAsc ? '\u25B2' : '\u25BC' }}</span>
            </th>
            <th class="w-25 px-2 py-1"></th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="val in filteredValues"
            :key="val.item_type_id"
            class="border-b border-border-default/50 hover:bg-surface-row-hover">
            <td class="px-2 py-1 text-text-primary">
              <ItemInline :reference="val.item_name" />
            </td>
            <td class="px-2 py-1 text-right tabular-nums">
              <span v-if="editingId !== val.item_type_id" class="text-accent-gold">
                {{ val.market_value.toLocaleString() }}g
              </span>
              <input
                v-else
                v-model.number="editPrice"
                type="number"
                min="0"
                class="w-24 bg-surface-dark border border-border-default rounded px-1.5 py-0.5 text-xs text-text-primary text-right tabular-nums"
                @keydown.enter="saveEdit(val)"
                @keydown.escape="editingId = null" />
            </td>
            <td class="px-2 py-1 text-text-dim">{{ val.notes ?? '' }}</td>
            <td class="px-2 py-1 text-text-dim">{{ formatDate(val.updated_at) }}</td>
            <td class="px-2 py-1 text-right">
              <div class="flex items-center justify-end gap-1">
                <button
                  v-if="editingId !== val.item_type_id"
                  class="text-text-muted hover:text-text-primary bg-transparent border-none cursor-pointer text-xs px-1.5 py-0.5 rounded hover:bg-surface-elevated"
                  @click="startEdit(val)">
                  Edit
                </button>
                <button
                  v-if="editingId === val.item_type_id"
                  class="text-accent-green hover:text-green-400 bg-transparent border-none cursor-pointer text-xs px-1.5 py-0.5 rounded hover:bg-surface-elevated"
                  @click="saveEdit(val)">
                  Save
                </button>
                <button
                  v-if="editingId === val.item_type_id"
                  class="text-text-muted hover:text-text-primary bg-transparent border-none cursor-pointer text-xs px-1.5 py-0.5 rounded hover:bg-surface-elevated"
                  @click="editingId = null">
                  Cancel
                </button>
                <button
                  v-if="editingId !== val.item_type_id"
                  class="text-text-muted hover:text-red-400 bg-transparent border-none cursor-pointer text-xs px-1.5 py-0.5 rounded hover:bg-surface-elevated"
                  @click="deleteValue(val.item_type_id)">
                  Del
                </button>
              </div>
            </td>
          </tr>
        </tbody>
      </table>
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
import { ref, computed, onMounted } from 'vue'
import { useToast } from '../../composables/useToast'
import { formatRelative } from '../../composables/useTimestamp'
import { useMarketStore, type MarketValue, type ImportMarketValuesResult } from '../../stores/marketStore'
import { useSettingsStore } from '../../stores/settingsStore'
import { useGameDataStore } from '../../stores/gameDataStore'
import type { ItemInfo } from '../../types/gameData'
import EmptyState from '../Shared/EmptyState.vue'
import AccordionSection from '../Shared/AccordionSection.vue'
import ItemInline from '../Shared/Item/ItemInline.vue'

const toast = useToast()
const marketStore = useMarketStore()
const settingsStore = useSettingsStore()
const gameDataStore = useGameDataStore()

const search = ref('')
const sortField = ref<'item_name' | 'market_value' | 'updated_at'>('item_name')
const sortAsc = ref(true)

// Add form
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

// Summary stats
const totalMarketValue = computed(() =>
  marketStore.values.reduce((sum, v) => sum + v.market_value, 0)
)

const avgPrice = computed(() => {
  if (marketStore.values.length === 0) return 0
  return Math.round(totalMarketValue.value / marketStore.values.length)
})

const highestPricedItem = computed(() => {
  if (marketStore.values.length === 0) return null
  return marketStore.values.reduce((max, v) => v.market_value > max.market_value ? v : max, marketStore.values[0])
})

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

function formatDate(dateStr: string): string {
  if (!dateStr) return ''
  return formatRelative(dateStr)
}

function onItemSearch() {
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
