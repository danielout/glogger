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
                <td class="py-1.5 pr-4 text-right text-text-secondary">
                  <span v-if="item.priceTiers.length === 1">{{ formatPrice(item.priceTiers[0].price) }}</span>
                  <span v-else-if="item.priceTiers.length > 1" class="flex flex-col items-end gap-0.5">
                    <span v-for="(tier, idx) in item.priceTiers" :key="idx" class="text-xs">
                      {{ tier.qty }}&times;{{ formatPrice(tier.price) }}
                    </span>
                  </span>
                </td>
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
                <td class="py-1.5 pr-4 text-right">{{ item.lastPrice != null ? formatPrice(item.lastPrice) : '' }}</td>
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

function formatPrice(price: number | null): string {
  if (price == null) return ''
  return Math.round(price).toLocaleString() + 'g'
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

// ── Price tier type ────────────────────────────────────────

interface PriceTier {
  qty: number
  price: number | null
}

// ── Base inventory with price stack ────────────────────────

interface BaseItem {
  name: string
  quantity: number
  priceTiers: PriceTier[]   // in-stock tiers with qty > 0
  lastPrice: number | null  // most recent known price (for sold-out items)
  value: number             // sum of qty * price across tiers
  lastSoldKey: number
  lastSoldLabel: string
  lastActivityKey: number
  lastActivityLabel: string
}

const baseInventory = computed(() => {
  const events = [...store.shopLog]
  // Sort chronologically; use id as tiebreaker for events within the same
  // minute so that added → visible → bought order is preserved.
  events.sort((a, b) => timestampToSortKey(a.event_timestamp) - timestampToSortKey(b.event_timestamp) || a.id - b.id)

  const items = new Map<string, {
    tiers: PriceTier[]          // price stack: each entry is a (qty, price) slot
    lastPrice: number | null
    lastSoldKey: number
    lastSoldTs: string
    lastActivityKey: number
    lastActivityTs: string
  }>()

  function getItem(name: string) {
    if (!items.has(name)) {
      items.set(name, {
        tiers: [],
        lastPrice: null,
        lastSoldKey: 0,
        lastSoldTs: '',
        lastActivityKey: 0,
        lastActivityTs: '',
      })
    }
    return items.get(name)!
  }

  function totalQty(tiers: PriceTier[]): number {
    return tiers.reduce((sum, t) => sum + t.qty, 0)
  }

  for (const e of events) {
    if (!e.item || e.ignored) continue
    const item = getItem(e.item)
    const sk = timestampToSortKey(e.event_timestamp)

    if (sk > item.lastActivityKey) {
      item.lastActivityKey = sk
      item.lastActivityTs = e.event_timestamp
    }

    switch (e.action) {
      case 'added': {
        // Negative reset: if total balance < 0, clear tiers
        if (totalQty(item.tiers) < 0) {
          item.tiers = []
        }
        // Push a new unpriced tier
        item.tiers.push({ qty: e.quantity, price: null })
        break
      }
      case 'visible':
      case 'configured': {
        if (e.price_unit != null) {
          item.lastPrice = e.price_unit
          // Price up to e.quantity units across unpriced tiers.
          // The event says "N units at this price", so we apply to
          // that many units, not all unpriced stock.
          let remaining = e.quantity
          let foundUnpriced = false
          for (const tier of item.tiers) {
            if (remaining <= 0) break
            if (tier.price === null && tier.qty > 0) {
              const apply = Math.min(tier.qty, remaining)
              if (apply === tier.qty) {
                tier.price = e.price_unit
              } else {
                // Split tier: price part of it, leave the rest unpriced
                tier.qty -= apply
                item.tiers.push({ qty: apply, price: e.price_unit })
              }
              remaining -= apply
              foundUnpriced = true
            }
          }
          if (!foundUnpriced && item.tiers.length > 0) {
            // All tiers already priced — update the last one (price change)
            item.tiers[item.tiers.length - 1].price = e.price_unit
          }
        }
        break
      }
      case 'bought': {
        if (sk > item.lastSoldKey) {
          item.lastSoldKey = sk
          item.lastSoldTs = e.event_timestamp
        }
        if (e.price_unit != null) item.lastPrice = e.price_unit

        // Remove from the tier matching the bought price
        let remaining = e.quantity
        const price = e.price_unit
        // First pass: try to match by price
        if (price != null) {
          for (const tier of item.tiers) {
            if (remaining <= 0) break
            if (tier.price != null && Math.abs(tier.price - price) < 0.01 && tier.qty > 0) {
              const take = Math.min(tier.qty, remaining)
              tier.qty -= take
              remaining -= take
            }
          }
        }
        // Second pass: remove from any tier if price didn't match
        if (remaining > 0) {
          for (const tier of item.tiers) {
            if (remaining <= 0) break
            if (tier.qty > 0) {
              const take = Math.min(tier.qty, remaining)
              tier.qty -= take
              remaining -= take
            }
          }
        }
        // If still remaining (missing adds), push a negative tier to track it
        if (remaining > 0) {
          item.tiers.push({ qty: -remaining, price })
        }
        // Clean up empty tiers
        item.tiers = item.tiers.filter(t => t.qty !== 0)
        break
      }
      case 'removed': {
        // Remove from the most recently added tier (LIFO)
        let remaining = e.quantity
        for (let i = item.tiers.length - 1; i >= 0 && remaining > 0; i--) {
          if (item.tiers[i].qty > 0) {
            const take = Math.min(item.tiers[i].qty, remaining)
            item.tiers[i].qty -= take
            remaining -= take
          }
        }
        if (remaining > 0) {
          item.tiers.push({ qty: -remaining, price: null })
        }
        item.tiers = item.tiers.filter(t => t.qty !== 0)
        break
      }
    }
  }

  // Build result
  const result: BaseItem[] = []
  for (const [name, data] of items) {
    // Collapse tiers: merge same-price tiers, drop negative/zero
    const mergedMap = new Map<string, PriceTier>()
    for (const tier of data.tiers) {
      if (tier.qty <= 0) continue
      const key = tier.price != null ? String(Math.round(tier.price * 100)) : 'null'
      const existing = mergedMap.get(key)
      if (existing) {
        existing.qty += tier.qty
      } else {
        mergedMap.set(key, { qty: tier.qty, price: tier.price })
      }
    }
    const priceTiers = [...mergedMap.values()].sort((a, b) => (a.price ?? 0) - (b.price ?? 0))

    const quantity = priceTiers.reduce((sum, t) => sum + t.qty, 0)
    const value = priceTiers.reduce((sum, t) => sum + t.qty * Math.round(t.price ?? 0), 0)

    result.push({
      name,
      quantity,
      priceTiers,
      lastPrice: data.lastPrice,
      value,
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

  const periodSales = new Map<string, { sold: number, revenue: number }>()
  for (const e of store.shopLog) {
    if (e.action !== 'bought' || !e.item || e.ignored) continue
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
      case 'price': {
        // Sort by min price across tiers (or lastPrice for sold-out)
        const ap = a.priceTiers[0]?.price ?? a.lastPrice ?? 0
        const bp = b.priceTiers[0]?.price ?? b.lastPrice ?? 0
        return (ap - bp) * dir
      }
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
