<template>
  <div class="flex flex-col gap-3 h-full min-h-0">
    <!-- Stats header -->
    <div class="grid grid-cols-4 gap-3 flex-shrink-0">
      <StatCard
        label="ITEMS IN STOCK"
        :value="formatNumber(inStockCount)" />
      <StatCard
        label="EST. SHOP VALUE"
        :value="`${formatNumber(result?.estimated_value ?? 0)}g`" />
      <StatCard
        label="SOLD"
        :value="formatNumber(result?.total_sold ?? 0)" />
      <StatCard
        label="AVG DAILY REVENUE"
        :value="`${formatNumber(Math.round(result?.avg_daily_revenue ?? 0))}g`" />
    </div>

    <!-- Sales period dropdown + estimation note -->
    <div class="flex items-center justify-between flex-wrap gap-2 flex-shrink-0">
      <p class="text-xs text-text-secondary italic">
        Inventory is estimated from shop log events. It may be incomplete if older log data is missing.
      </p>
      <label class="flex items-center gap-2 text-xs text-text-secondary">
        Sales period
        <select
          v-model.number="periodDays"
          class="bg-surface-elevated border border-border-default rounded px-2 py-1 text-xs text-text-primary focus:outline-none focus:border-accent-gold/50">
          <option
            v-for="p in periodOptions"
            :key="p.value"
            :value="p.value">
            {{ p.label }}
          </option>
        </select>
      </label>
    </div>

    <!-- In-stock table -->
    <div class="flex-1 min-h-0 overflow-auto border border-border-default rounded">
      <table class="w-full text-xs">
        <thead class="sticky top-0 bg-surface-elevated z-10">
          <tr class="text-text-secondary">
            <th class="px-2 py-1.5 text-left">ITEM</th>
            <th class="px-2 py-1.5 text-right">QTY</th>
            <th class="px-2 py-1.5 text-right">PRICE</th>
            <th class="px-2 py-1.5 text-right">EST. VALUE</th>
            <th class="px-2 py-1.5 text-right">SOLD</th>
            <th class="px-2 py-1.5 text-right">AVG/DAY</th>
            <th class="px-2 py-1.5 text-right">LAST</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="item in inStockItems"
            :key="item.item"
            class="border-t border-border-default/40 hover:bg-surface-hover">
            <td class="px-2 py-1">
              <ItemInline :reference="item.item" />
            </td>
            <td class="px-2 py-1 text-right tabular-nums">{{ item.quantity }}</td>
            <td class="px-2 py-1 text-right">
              <span v-if="item.price_tiers.length === 1">
                {{ formatPriceTier(item.price_tiers[0]) }}
              </span>
              <span
                v-else-if="item.price_tiers.length > 1"
                class="flex flex-col items-end gap-0.5">
                <span
                  v-for="(tier, idx) in item.price_tiers"
                  :key="idx"
                  class="text-[11px]">
                  {{ tier.qty }}&times;{{ formatPriceTier(tier) }}
                </span>
              </span>
              <span
                v-else
                class="text-text-secondary">—</span>
            </td>
            <td class="px-2 py-1 text-right text-accent-gold tabular-nums">
              {{ formatGold(item.estimated_value) }}
            </td>
            <td class="px-2 py-1 text-right tabular-nums">{{ item.period_sold }}</td>
            <td class="px-2 py-1 text-right tabular-nums">{{ item.avg_per_day.toFixed(1) }}</td>
            <td class="px-2 py-1 text-right text-text-secondary whitespace-nowrap">
              {{ formatShortDate(item.last_activity_at) }}
            </td>
          </tr>
          <tr v-if="inStockItems.length === 0 && !loading">
            <td
              colspan="7"
              class="px-2 py-8 text-center text-text-secondary">
              No items currently in stock.
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Recently Sold Out collapsible — custom (not AccordionSection) so the
         window dropdown can be a sibling of the toggle button rather than
         nested inside it (a <select> inside a <button> would be invalid HTML
         and the click handlers fight each other). -->
    <div
      v-if="result"
      class="flex-shrink-0 border border-surface-elevated rounded">
      <div class="flex items-center justify-between px-3 py-2 hover:bg-surface-elevated/50 transition-colors">
        <button
          type="button"
          class="flex items-center gap-2 bg-transparent border-none cursor-pointer text-left flex-1"
          @click="soldOutOpen = !soldOutOpen">
          <span
            class="text-text-secondary text-xs transition-transform"
            :class="{ 'rotate-90': soldOutOpen }">
            ▶
          </span>
          <h4 class="text-text-secondary text-xs font-semibold uppercase tracking-wide m-0">
            Recently Sold Out ({{ recentlySoldOut.length }})
          </h4>
        </button>
        <label class="flex items-center gap-1 text-[10px] text-text-secondary">
          Window
          <select
            v-model.number="soldOutDays"
            class="bg-surface-elevated border border-border-default rounded px-1.5 py-0.5 text-[10px] text-text-primary focus:outline-none focus:border-accent-gold/50">
            <option :value="1">1 day</option>
            <option :value="3">3 days</option>
            <option :value="7">7 days</option>
            <option :value="14">14 days</option>
          </select>
        </label>
      </div>
      <div
        v-show="soldOutOpen"
        class="px-3 pb-3">
        <table class="w-full text-xs mt-2">
          <thead>
            <tr class="text-text-secondary">
              <th class="px-2 py-1 text-left">ITEM</th>
              <th class="px-2 py-1 text-right">LAST PRICE</th>
              <th class="px-2 py-1 text-right">SOLD</th>
              <th class="px-2 py-1 text-right">AVG/DAY</th>
              <th class="px-2 py-1 text-right">LAST ACTIVITY</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="item in recentlySoldOut"
              :key="item.item"
              class="border-t border-border-default/40 opacity-60 hover:opacity-100 hover:bg-surface-hover">
              <td class="px-2 py-1">
                <ItemInline :reference="item.item" />
              </td>
              <td class="px-2 py-1 text-right tabular-nums">
                {{ formatLastPrice(item.last_known_price) }}
              </td>
              <td class="px-2 py-1 text-right tabular-nums">{{ item.period_sold }}</td>
              <td class="px-2 py-1 text-right tabular-nums">{{ item.avg_per_day.toFixed(1) }}</td>
              <td class="px-2 py-1 text-right text-text-secondary whitespace-nowrap">
                {{ formatShortDate(item.last_activity_at) }}
              </td>
            </tr>
            <tr v-if="recentlySoldOut.length === 0">
              <td
                colspan="5"
                class="px-2 py-3 text-center text-text-secondary italic">
                Nothing has sold out in the selected window.
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <p
      v-if="error"
      class="text-xs text-red-400 flex-shrink-0">
      {{ error }}
    </p>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useStallTrackerStore } from '../../stores/stallTrackerStore'
import StatCard from './StatCard.vue'
import ItemInline from '../Shared/Item/ItemInline.vue'
import type {
  InventoryItem,
  InventoryResult,
  PriceTier,
  StallInventoryParams,
} from '../../types/stallTracker'

const store = useStallTrackerStore()

/** All-time sentinel: any value ≥ 99999 collapses to "all time" in the
 * Rust aggregator. 100000 is the canonical UI value. */
const ALL_TIME = 100000

const periodOptions: { value: number; label: string }[] = [
  { value: 1, label: 'Last day' },
  { value: 2, label: 'Last 2 days' },
  { value: 7, label: 'Last 7 days' },
  { value: 14, label: 'Last 14 days' },
  { value: 30, label: 'Last 30 days' },
  { value: ALL_TIME, label: 'All time' },
]

const periodDays = ref<number>(7)
const soldOutDays = ref<number>(3)
const soldOutOpen = ref<boolean>(false)

const result = ref<InventoryResult | null>(null)
const loading = ref(false)
const error = ref<string | null>(null)

let reloadToken = 0

const inStockItems = computed<InventoryItem[]>(() =>
  result.value ? result.value.items.filter((i) => i.quantity > 0) : [],
)

const inStockCount = computed(() => inStockItems.value.length)

/** Recently-sold-out: items with quantity 0 whose last activity falls within
 * the most recent N **distinct active dates** (not calendar days). The
 * backend exposes `active_dates` newest-first specifically so we can slice
 * here without recomputing the window — see plan §10.2. */
const recentlySoldOut = computed<InventoryItem[]>(() => {
  if (!result.value) return []
  const windowDates = new Set(result.value.active_dates.slice(0, soldOutDays.value))
  return result.value.items.filter((item) => {
    if (item.quantity > 0) return false
    if (!item.last_activity_at) return false
    return windowDates.has(item.last_activity_at.slice(0, 10))
  })
})

function buildParams(): StallInventoryParams {
  return {
    owner: store.currentOwner,
    periodDays: periodDays.value,
  }
}

async function reload() {
  if (!store.currentOwner) {
    result.value = null
    return
  }
  const token = ++reloadToken
  loading.value = true
  error.value = null
  try {
    const r = await invoke<InventoryResult>('get_stall_inventory', { params: buildParams() })
    if (token !== reloadToken) return
    result.value = r
  } catch (e) {
    if (token === reloadToken) error.value = String(e)
    console.error('[StallInventoryTab] reload failed:', e)
  } finally {
    if (token === reloadToken) loading.value = false
  }
}

function formatNumber(n: number): string {
  return n.toLocaleString()
}

function formatGold(n: number): string {
  return `${n.toLocaleString()}g`
}

function formatPriceTier(tier: PriceTier): string {
  if (tier.price === null) return '—'
  return formatGold(Math.round(tier.price))
}

/** Recently Sold Out reads the dedicated `last_known_price` field — survives
 * the sellout collapse where `price_tiers` becomes empty. */
function formatLastPrice(price: number | null): string {
  if (price === null) return '—'
  return formatGold(Math.round(price))
}

function formatShortDate(iso: string | null): string {
  if (!iso) return '—'
  // event_at is "YYYY-MM-DD HH:MM:SS"; render the date part as "Apr 13".
  const datePart = iso.slice(0, 10)
  const d = new Date(`${datePart}T00:00:00`)
  if (Number.isNaN(d.getTime())) return datePart
  return d.toLocaleDateString(undefined, { month: 'short', day: 'numeric' })
}

watch(periodDays, () => void reload())
watch(() => store.dataVersion, () => void reload())
watch(
  () => store.currentOwner,
  () => {
    periodDays.value = 7
    soldOutDays.value = 3
    soldOutOpen.value = false
    void reload()
  },
)

onMounted(reload)
</script>
