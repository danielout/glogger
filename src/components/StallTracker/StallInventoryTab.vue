<template>
  <div class="flex flex-col gap-4">
    <div v-if="store.loading" class="text-text-dim italic text-sm">Loading...</div>
    <div v-else-if="store.error" class="text-[#c87e7e] text-sm">{{ store.error }}</div>
    <EmptyState
      v-else-if="store.shopLog.length === 0"
      variant="panel"
      primary="No shop log data"
      secondary="Open your shop log book in-game to import stall data." />

    <template v-else>
      <!-- Summary stats + sales period dropdown -->
      <div class="flex items-center gap-6 flex-wrap">
        <div class="flex gap-6 flex-wrap text-center">
          <div>
            <div class="text-[0.65rem] text-text-muted uppercase tracking-wide">Items in Stock</div>
            <div class="text-lg font-bold text-text-primary">{{ inStockItems.length }}</div>
          </div>
          <div>
            <div class="text-[0.65rem] text-text-muted uppercase tracking-wide">Est. Shop Value</div>
            <div class="text-lg font-bold text-[#d4af37]">{{ estimatedValue.toLocaleString() }}g</div>
          </div>
          <div>
            <div class="text-[0.65rem] text-text-muted uppercase tracking-wide">Sold</div>
            <div class="text-lg font-bold text-text-primary">{{ periodStats.totalSold.toLocaleString() }}</div>
          </div>
          <div>
            <div class="text-[0.65rem] text-text-muted uppercase tracking-wide">Avg Daily Revenue</div>
            <div class="text-lg font-bold text-[#d4af37]">{{ periodStats.avgDailyRevenue.toLocaleString() }}g</div>
          </div>
        </div>
        <div class="flex items-center gap-2 ml-auto">
          <span class="text-[0.65rem] text-text-muted uppercase tracking-wide">Sales period</span>
          <select
            v-model.number="salesPeriodDays"
            class="px-2 py-1 bg-surface-base border border-border-default rounded text-xs text-text-primary focus:outline-none focus:border-accent-gold/50">
            <option :value="1">Last day</option>
            <option :value="2">Last 2 days</option>
            <option :value="7">Last 7 days</option>
            <option :value="14">Last 14 days</option>
            <option :value="30">Last 30 days</option>
            <option :value="Infinity">All time</option>
          </select>
        </div>
      </div>

      <div class="text-[0.6rem] text-text-dim italic">Inventory is estimated from shop log events. It may be incomplete if older log data is missing.</div>

      <!-- In Stock -->
      <div v-if="inStockItems.length > 0">
        <h3 class="text-xs text-text-muted uppercase tracking-wide mb-2">In Stock</h3>
        <div class="overflow-auto">
          <table class="w-full text-sm">
            <thead>
              <tr class="text-left text-text-muted text-xs uppercase tracking-wide border-b border-border-default">
                <th class="pb-2 pr-4 cursor-pointer hover:text-text-primary" @click="toggleSort('name', 'stock')">Item {{ sortIcon('name', 'stock') }}</th>
                <th class="pb-2 pr-4 text-right cursor-pointer hover:text-text-primary" @click="toggleSort('quantity', 'stock')">Qty {{ sortIcon('quantity', 'stock') }}</th>
                <th class="pb-2 pr-4 text-right cursor-pointer hover:text-text-primary" @click="toggleSort('price', 'stock')">Price {{ sortIcon('price', 'stock') }}</th>
                <th class="pb-2 pr-4 text-right cursor-pointer hover:text-text-primary" @click="toggleSort('value', 'stock')">Est. Value {{ sortIcon('value', 'stock') }}</th>
                <th class="pb-2 pr-4 text-right cursor-pointer hover:text-text-primary" @click="toggleSort('periodSold', 'stock')">Sold {{ sortIcon('periodSold', 'stock') }}</th>
                <th class="pb-2 pr-4 text-right cursor-pointer hover:text-text-primary" @click="toggleSort('avgPerDay', 'stock')">Avg/Day {{ sortIcon('avgPerDay', 'stock') }}</th>
                <th class="pb-2 cursor-pointer hover:text-text-primary" @click="toggleSort('lastSold', 'stock')">Last Sold {{ sortIcon('lastSold', 'stock') }}</th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="item in sortedInStock"
                :key="item.name"
                class="border-b border-border-light hover:bg-[#1a1a2e] transition-colors">
                <td class="py-1.5 pr-4"><ItemInline :reference="item.name" :show-icon="false" /></td>
                <td class="py-1.5 pr-4 text-right text-text-secondary">{{ item.quantity }}</td>
                <td class="py-1.5 pr-4 text-right text-text-secondary">{{ item.price != null ? Math.round(item.price).toLocaleString() + 'g' : '' }}</td>
                <td class="py-1.5 pr-4 text-right text-[#d4af37]">{{ item.value > 0 ? item.value.toLocaleString() + 'g' : '' }}</td>
                <td class="py-1.5 pr-4 text-right text-text-secondary">{{ item.periodSold || '' }}</td>
                <td class="py-1.5 pr-4 text-right text-text-secondary">{{ item.avgPerDay > 0 ? item.avgPerDay.toFixed(1) : '' }}</td>
                <td class="py-1.5 text-text-dim text-xs">{{ item.lastSoldLabel }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <!-- Recently Sold Out -->
      <div v-if="recentlySoldOut.length > 0">
        <div class="flex items-center gap-3 mt-2">
          <button
            class="text-xs text-text-muted uppercase tracking-wide hover:text-text-primary cursor-pointer"
            @click="soldOutExpanded = !soldOutExpanded">
            {{ soldOutExpanded ? '\u25BC' : '\u25B6' }} Recently Sold Out ({{ recentlySoldOut.length }})
          </button>
          <select
            v-model.number="soldOutDays"
            class="px-2 py-0.5 bg-surface-base border border-border-default rounded text-xs text-text-primary focus:outline-none focus:border-accent-gold/50">
            <option :value="3">Last 3 days</option>
            <option :value="5">Last 5 days</option>
            <option :value="7">Last 7 days</option>
            <option :value="Infinity">All</option>
          </select>
        </div>
        <div v-if="soldOutExpanded" class="overflow-auto mt-2">
          <table class="w-full text-sm">
            <thead>
              <tr class="text-left text-text-muted text-xs uppercase tracking-wide border-b border-border-default">
                <th class="pb-2 pr-4 cursor-pointer hover:text-text-primary" @click="toggleSort('name', 'soldout')">Item {{ sortIcon('name', 'soldout') }}</th>
                <th class="pb-2 pr-4 text-right cursor-pointer hover:text-text-primary" @click="toggleSort('price', 'soldout')">Last Price {{ sortIcon('price', 'soldout') }}</th>
                <th class="pb-2 pr-4 text-right cursor-pointer hover:text-text-primary" @click="toggleSort('periodSold', 'soldout')">Sold {{ sortIcon('periodSold', 'soldout') }}</th>
                <th class="pb-2 pr-4 text-right cursor-pointer hover:text-text-primary" @click="toggleSort('avgPerDay', 'soldout')">Avg/Day {{ sortIcon('avgPerDay', 'soldout') }}</th>
                <th class="pb-2 cursor-pointer hover:text-text-primary" @click="toggleSort('lastActivity', 'soldout')">Last Activity {{ sortIcon('lastActivity', 'soldout') }}</th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="item in sortedSoldOut"
                :key="item.name"
                class="border-b border-border-light hover:bg-[#1a1a2e] transition-colors text-text-dim">
                <td class="py-1.5 pr-4"><ItemInline :reference="item.name" :show-icon="false" /></td>
                <td class="py-1.5 pr-4 text-right">{{ item.price != null ? Math.round(item.price).toLocaleString() + 'g' : '' }}</td>
                <td class="py-1.5 pr-4 text-right">{{ item.periodSold || '' }}</td>
                <td class="py-1.5 pr-4 text-right">{{ item.avgPerDay > 0 ? item.avgPerDay.toFixed(1) : '' }}</td>
                <td class="py-1.5 text-xs">{{ item.lastActivityLabel }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import EmptyState from '../Shared/EmptyState.vue'
import ItemInline from '../Shared/Item/ItemInline.vue'
import { useStallTrackerStore } from '../../stores/stallTrackerStore'
import { timestampToSortKey, timestampToDateKey, timestampToDateLabel } from './stallTimestamp'

const store = useStallTrackerStore()

const salesPeriodDays = ref<number>(7)
const soldOutExpanded = ref(false)
const soldOutDays = ref<number>(3)

// ── Sorting state ──────────────────────────────────────────

type StockSortKey = 'name' | 'quantity' | 'price' | 'value' | 'periodSold' | 'avgPerDay' | 'lastSold'
type SoldOutSortKey = 'name' | 'price' | 'periodSold' | 'avgPerDay' | 'lastActivity'

const stockSortKey = ref<StockSortKey>('name')
const stockSortAsc = ref(true)
const soldOutSortKey = ref<SoldOutSortKey>('lastActivity')
const soldOutSortAsc = ref(false)

function toggleSort(key: string, table: 'stock' | 'soldout') {
  if (table === 'stock') {
    if (stockSortKey.value === key) {
      stockSortAsc.value = !stockSortAsc.value
    } else {
      stockSortKey.value = key as StockSortKey
      stockSortAsc.value = key === 'name'
    }
  } else {
    if (soldOutSortKey.value === key) {
      soldOutSortAsc.value = !soldOutSortAsc.value
    } else {
      soldOutSortKey.value = key as SoldOutSortKey
      soldOutSortAsc.value = key === 'name'
    }
  }
}

function sortIcon(key: string, table: 'stock' | 'soldout'): string {
  const active = table === 'stock' ? stockSortKey.value : soldOutSortKey.value
  const asc = table === 'stock' ? stockSortAsc.value : soldOutSortAsc.value
  if (active !== key) return ''
  return asc ? '\u25B2' : '\u25BC'
}

// ── All distinct dates (newest first) ──────────────────────

const allDateKeysDesc = computed(() => {
  const keys = [...new Set(store.shopLog.map(e => timestampToDateKey(e.event_timestamp)))]
  keys.sort((a, b) => b - a)
  return keys
})

// ── Sales period date set ──────────────────────────────────

const salesPeriodDateSet = computed(() => {
  const n = salesPeriodDays.value === Infinity ? allDateKeysDesc.value.length : salesPeriodDays.value
  return new Set(allDateKeysDesc.value.slice(0, n))
})

// ── Base inventory (cumulative, period-independent) ────────

interface BaseItem {
  name: string
  quantity: number
  price: number | null
  value: number
  lastSoldKey: number
  lastSoldLabel: string
  lastActivityKey: number
  lastActivityLabel: string
}

const baseInventory = computed(() => {
  const events = [...store.shopLog]
  events.sort((a, b) => timestampToSortKey(a.event_timestamp) - timestampToSortKey(b.event_timestamp))

  const items = new Map<string, {
    balance: number
    price: number | null
    lastSoldKey: number
    lastSoldTs: string
    lastActivityKey: number
    lastActivityTs: string
  }>()

  function getItem(name: string) {
    if (!items.has(name)) {
      items.set(name, { balance: 0, price: null, lastSoldKey: 0, lastSoldTs: '', lastActivityKey: 0, lastActivityTs: '' })
    }
    return items.get(name)!
  }

  for (const e of events) {
    if (!e.item) continue
    const item = getItem(e.item)
    const sk = timestampToSortKey(e.event_timestamp)

    if (sk > item.lastActivityKey) {
      item.lastActivityKey = sk
      item.lastActivityTs = e.event_timestamp
    }

    switch (e.action) {
      case 'added':
        if (item.balance < 0) item.balance = 0
        item.balance += e.quantity
        break
      case 'bought':
        item.balance -= e.quantity
        if (sk > item.lastSoldKey) { item.lastSoldKey = sk; item.lastSoldTs = e.event_timestamp }
        if (e.price_unit != null) item.price = e.price_unit
        break
      case 'removed':
        item.balance -= e.quantity
        break
      case 'configured':
      case 'visible':
        if (e.price_unit != null) item.price = e.price_unit
        break
    }
  }

  const result: BaseItem[] = []
  for (const [name, data] of items) {
    const qty = Math.max(data.balance, 0)
    const price = data.price
    result.push({
      name,
      quantity: qty,
      price,
      value: qty * (price != null ? Math.round(price) : 0),
      lastSoldKey: data.lastSoldKey,
      lastSoldLabel: data.lastSoldTs ? timestampToDateLabel(data.lastSoldTs) : '',
      lastActivityKey: data.lastActivityKey,
      lastActivityLabel: data.lastActivityTs ? timestampToDateLabel(data.lastActivityTs) : '',
    })
  }
  return result
})

// ── Period-scoped sales stats per item ─────────────────────

interface InventoryItem extends BaseItem {
  periodSold: number
  periodRevenue: number
  avgPerDay: number
}

const inventory = computed((): InventoryItem[] => {
  const periodDates = salesPeriodDateSet.value
  const windowSize = salesPeriodDays.value === Infinity
    ? Math.max(allDateKeysDesc.value.length, 1)
    : salesPeriodDays.value

  // Count sales per item within the period
  const periodSales = new Map<string, { sold: number, revenue: number }>()
  for (const e of store.shopLog) {
    if (e.action !== 'bought' || !e.item) continue
    const dk = timestampToDateKey(e.event_timestamp)
    if (!periodDates.has(dk)) continue
    const s = periodSales.get(e.item) ?? { sold: 0, revenue: 0 }
    s.sold += e.quantity
    s.revenue += e.price_total ?? 0
    periodSales.set(e.item, s)
  }

  return baseInventory.value.map(base => {
    const ps = periodSales.get(base.name)
    const periodSold = ps?.sold ?? 0
    const periodRevenue = ps?.revenue ?? 0
    return {
      ...base,
      periodSold,
      periodRevenue,
      avgPerDay: periodSold / windowSize,
    }
  })
})

// ── Sections ───────────────────────────────────────────────

const inStockItems = computed(() => inventory.value.filter(i => i.quantity > 0))

const soldOutDateSet = computed(() => {
  const n = soldOutDays.value === Infinity ? allDateKeysDesc.value.length : soldOutDays.value
  return new Set(allDateKeysDesc.value.slice(0, n))
})

const recentlySoldOut = computed(() => {
  const dates = soldOutDateSet.value
  return inventory.value.filter(i => {
    if (i.quantity > 0) return false
    const dk = timestampToDateKey(i.lastActivityLabel)
    return dates.has(dk)
  })
})

// ── Summary stats ──────────────────────────────────────────

const estimatedValue = computed(() =>
  inStockItems.value.reduce((sum, i) => sum + i.value, 0)
)

const periodStats = computed(() => {
  const totalSold = inventory.value.reduce((sum, i) => sum + i.periodSold, 0)
  const totalRevenue = inventory.value.reduce((sum, i) => sum + i.periodRevenue, 0)
  const windowSize = salesPeriodDays.value === Infinity
    ? Math.max(allDateKeysDesc.value.length, 1)
    : salesPeriodDays.value
  return {
    totalSold,
    avgDailyRevenue: Math.round(totalRevenue / windowSize),
  }
})

// ── Sorted lists ───────────────────────────────────────────

function sortItems(list: InventoryItem[], key: string, asc: boolean): InventoryItem[] {
  const sorted = [...list]
  const dir = asc ? 1 : -1
  sorted.sort((a, b) => {
    switch (key) {
      case 'name': return a.name.localeCompare(b.name) * dir
      case 'quantity': return (a.quantity - b.quantity) * dir
      case 'price': return ((a.price ?? 0) - (b.price ?? 0)) * dir
      case 'value': return (a.value - b.value) * dir
      case 'periodSold': return (a.periodSold - b.periodSold) * dir
      case 'avgPerDay': return (a.avgPerDay - b.avgPerDay) * dir
      case 'lastSold': return (a.lastSoldKey - b.lastSoldKey) * dir
      case 'lastActivity': return (a.lastActivityKey - b.lastActivityKey) * dir
      default: return 0
    }
  })
  return sorted
}

const sortedInStock = computed(() =>
  sortItems(inStockItems.value, stockSortKey.value, stockSortAsc.value)
)

const sortedSoldOut = computed(() =>
  sortItems(recentlySoldOut.value, soldOutSortKey.value, soldOutSortAsc.value)
)
</script>
