<template>
  <div class="flex flex-col gap-4">
    <div v-if="loading" class="text-text-dim italic text-sm">Loading...</div>
    <div v-else-if="error" class="text-[#c87e7e] text-sm">{{ error }}</div>
    <EmptyState
      v-else-if="result && result.items.length === 0"
      variant="panel"
      primary="No shop log data"
      secondary="Open your shop log book in-game to start tracking, or use Import to load an exported book file." />

    <template v-else-if="result">
      <div class="flex items-center gap-6 flex-wrap">
        <div class="flex gap-6 flex-wrap text-center">
          <div>
            <div class="text-[0.65rem] text-text-muted uppercase tracking-wide">Items in Stock</div>
            <div class="text-lg font-bold text-text-primary">{{ inStockItems.length }}</div>
          </div>
          <div>
            <div class="text-[0.65rem] text-text-muted uppercase tracking-wide">Est. Shop Value</div>
            <div class="text-lg font-bold text-[#d4af37]">{{ result.estimated_value.toLocaleString() }}g</div>
          </div>
          <div>
            <div class="text-[0.65rem] text-text-muted uppercase tracking-wide">Sold</div>
            <div class="text-lg font-bold text-text-primary">{{ result.total_sold.toLocaleString() }}</div>
          </div>
          <div>
            <div class="text-[0.65rem] text-text-muted uppercase tracking-wide">Avg Daily Revenue</div>
            <div class="text-lg font-bold text-[#d4af37]">{{ result.avg_daily_revenue.toLocaleString() }}g</div>
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
            <option :value="100000">All time</option>
          </select>
        </div>
      </div>

      <div class="text-[0.6rem] text-text-dim italic">Inventory is estimated from shop log events. It may be incomplete if older log data is missing.</div>

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
                  <span v-if="item.price_tiers.length === 1">{{ formatPrice(item.price_tiers[0].price) }}</span>
                  <span v-else-if="item.price_tiers.length > 1" class="flex flex-col items-end gap-0.5">
                    <span v-for="(tier, idx) in item.price_tiers" :key="idx" class="text-xs">
                      {{ tier.qty }}&times;{{ formatPrice(tier.price) }}
                    </span>
                  </span>
                </td>
                <td class="py-1.5 pr-4 text-right text-[#d4af37]">{{ item.value > 0 ? item.value.toLocaleString() + 'g' : '' }}</td>
                <td class="py-1.5 pr-4 text-right text-text-secondary">{{ item.period_sold || '' }}</td>
                <td class="py-1.5 pr-4 text-right text-text-secondary">{{ item.avg_per_day > 0 ? item.avg_per_day.toFixed(1) : '' }}</td>
                <td class="py-1.5 text-text-dim text-xs">{{ formatDateLabel(item.last_sold_at) }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

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
            <option :value="100000">All</option>
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
                <td class="py-1.5 pr-4 text-right">{{ item.last_price != null ? formatPrice(item.last_price) : '' }}</td>
                <td class="py-1.5 pr-4 text-right">{{ item.period_sold || '' }}</td>
                <td class="py-1.5 pr-4 text-right">{{ item.avg_per_day > 0 ? item.avg_per_day.toFixed(1) : '' }}</td>
                <td class="py-1.5 text-xs">{{ formatDateLabel(item.last_activity_at) }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import EmptyState from '../Shared/EmptyState.vue'
import ItemInline from '../Shared/Item/ItemInline.vue'
import { useStallTrackerStore } from '../../stores/stallTrackerStore'

const store = useStallTrackerStore()

interface PriceTier { qty: number, price: number | null }

interface InventoryItem {
  name: string
  quantity: number
  price_tiers: PriceTier[]
  last_price: number | null
  value: number
  last_sold_at: string | null
  last_activity_at: string | null
  period_sold: number
  period_revenue: number
  avg_per_day: number
}

interface InventoryResult {
  items: InventoryItem[]
  estimated_value: number
  total_sold: number
  avg_daily_revenue: number
  active_dates: string[]  // YYYY-MM-DD, newest first
}

const salesPeriodDays = ref<number>(7)
const soldOutExpanded = ref(false)
const soldOutDays = ref<number>(3)

const loading = ref(false)
const error = ref<string | null>(null)
const result = ref<InventoryResult | null>(null)

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

const MONTH_ABBR = ['Jan','Feb','Mar','Apr','May','Jun','Jul','Aug','Sep','Oct','Nov','Dec']

function formatDateLabel(iso: string | null): string {
  if (!iso) return ''
  // "2026-04-13 14:29:00" → "Apr 13"
  const m = /^(\d{4})-(\d{2})-(\d{2})/.exec(iso)
  if (!m) return ''
  return `${MONTH_ABBR[parseInt(m[2]) - 1]} ${parseInt(m[3])}`
}

const inStockItems = computed(() => result.value?.items.filter(i => i.quantity > 0) ?? [])

// "Sold out in the last N days" counts *distinct activity dates* to match the
// original semantics — sparse logs with gaps don't have the window collapse.
const soldOutDateSet = computed<Set<string>>(() => {
  const all = result.value?.active_dates ?? []
  const n = soldOutDays.value === 100000 ? all.length : soldOutDays.value
  return new Set(all.slice(0, n))
})

const recentlySoldOut = computed(() => {
  if (!result.value) return []
  const dates = soldOutDateSet.value
  if (dates.size === 0) return []
  return result.value.items.filter(i => {
    if (i.quantity > 0) return false
    if (!i.last_activity_at) return false
    const datePart = i.last_activity_at.slice(0, 10)
    return dates.has(datePart)
  })
})

function sortItems<T extends InventoryItem>(list: T[], key: string, asc: boolean): T[] {
  const sorted = [...list]
  const dir = asc ? 1 : -1
  sorted.sort((a, b) => {
    switch (key) {
      case 'name': return a.name.localeCompare(b.name) * dir
      case 'quantity': return (a.quantity - b.quantity) * dir
      case 'price': {
        const ap = a.price_tiers[0]?.price ?? a.last_price ?? 0
        const bp = b.price_tiers[0]?.price ?? b.last_price ?? 0
        return (ap - bp) * dir
      }
      case 'value': return (a.value - b.value) * dir
      case 'periodSold': return (a.period_sold - b.period_sold) * dir
      case 'avgPerDay': return (a.avg_per_day - b.avg_per_day) * dir
      case 'lastSold': return ((a.last_sold_at ?? '').localeCompare(b.last_sold_at ?? '')) * dir
      case 'lastActivity': return ((a.last_activity_at ?? '').localeCompare(b.last_activity_at ?? '')) * dir
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

async function reload() {
  loading.value = true
  error.value = null
  try {
    result.value = await invoke<InventoryResult>('get_stall_inventory', {
      params: { owner: store.currentOwner, period_days: salesPeriodDays.value },
    })
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
}

onMounted(() => reload())
watch(salesPeriodDays, () => reload())
watch(() => store.dataVersion, () => reload())
</script>
