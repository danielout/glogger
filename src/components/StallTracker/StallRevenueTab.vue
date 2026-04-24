<template>
  <div class="flex flex-col gap-3 h-full min-h-0">
    <!-- Granularity toggle + filter row -->
    <div class="flex items-center gap-2 flex-wrap flex-shrink-0">
      <div class="inline-flex rounded border border-border-default overflow-hidden">
        <button
          v-for="g in granularities"
          :key="g.value"
          type="button"
          class="px-3 py-1 text-xs transition-colors"
          :class="
            granularity === g.value
              ? 'bg-accent-gold/20 text-accent-gold font-medium'
              : 'bg-surface-elevated text-text-secondary hover:text-text-primary'
          "
          @click="setGranularity(g.value)">
          {{ g.label }}
        </button>
      </div>

      <DatePicker
        v-model="filterDateFrom"
        placeholder="From date" />
      <span class="text-text-secondary text-xs">–</span>
      <DatePicker
        v-model="filterDateTo"
        placeholder="To date" />
      <SearchableSelect
        v-model="filterBuyer"
        :options="store.filterOptions.buyers"
        all-label="All buyers" />
      <SearchableSelect
        v-model="filterItem"
        :options="store.filterOptions.items"
        all-label="All items" />

      <button
        v-if="hasActiveFilters"
        type="button"
        class="text-xs text-accent-gold hover:underline ml-1"
        @click="clearFilters">
        Clear filters
      </button>
    </div>

    <!-- Pivot table -->
    <div
      v-if="result && result.items.length > 0"
      class="flex-1 min-h-0 overflow-auto border border-border-default rounded">
      <table class="text-xs border-collapse">
        <thead>
          <tr>
            <th
              class="sticky top-0 left-0 z-30 bg-surface-elevated text-text-secondary px-2 py-1.5 text-left border-b border-r border-border-default min-w-[200px] w-[200px]">
              ITEM
            </th>
            <th
              class="sticky top-0 z-20 bg-surface-elevated text-text-secondary px-2 py-1.5 text-right border-b border-r border-border-default min-w-[100px]"
              style="left: 200px">
              TOTAL
            </th>
            <th
              v-for="period in displayPeriods"
              :key="period.key"
              class="sticky top-0 z-10 bg-surface-elevated text-text-secondary px-2 py-1.5 text-right border-b border-border-default min-w-[80px] whitespace-nowrap">
              {{ period.label }}
            </th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="item in result.items"
            :key="item"
            class="hover:bg-surface-hover">
            <td
              class="sticky left-0 z-20 bg-surface-base px-2 py-1 border-b border-r border-border-default/60 truncate max-w-[200px]"
              :title="item">
              <ItemInline :reference="item" />
            </td>
            <td
              class="sticky z-10 bg-surface-base px-2 py-1 text-right text-accent-gold border-b border-r border-border-default/60 tabular-nums"
              style="left: 200px">
              {{ formatTotal(rowTotalLookup.get(item)) }}
            </td>
            <td
              v-for="period in displayPeriods"
              :key="period.key"
              class="px-2 py-1 text-right border-b border-border-default/40 tabular-nums">
              {{ formatCell(cellLookup.get(`${item}|${period.key}`)) }}
            </td>
          </tr>
        </tbody>
        <tfoot>
          <tr>
            <th
              class="sticky bottom-0 left-0 z-30 bg-surface-elevated text-text-secondary px-2 py-1.5 text-left border-t border-r border-border-default">
              TOTAL
            </th>
            <th
              class="sticky bottom-0 z-20 bg-surface-elevated text-accent-gold px-2 py-1.5 text-right border-t border-r border-border-default tabular-nums"
              style="left: 200px">
              {{ formatTotal(result.grand_total) }}
            </th>
            <th
              v-for="period in displayPeriods"
              :key="period.key"
              class="sticky bottom-0 z-10 bg-surface-elevated text-accent-gold px-2 py-1.5 text-right border-t border-border-default tabular-nums">
              {{ formatTotal(colTotalLookup.get(period.key)) }}
            </th>
          </tr>
        </tfoot>
      </table>
    </div>

    <div
      v-else
      class="flex-1 min-h-0 flex items-center justify-center text-text-secondary text-sm">
      <SkeletonLoader v-if="loading" variant="text" :lines="3" width="w-48" />
      <span v-else-if="error">{{ error }}</span>
      <span v-else>No revenue data for the current filters.</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onBeforeUnmount } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useStallTrackerStore } from '../../stores/stallTrackerStore'
import SearchableSelect from '../Shared/SearchableSelect.vue'
import DatePicker from '../Shared/DatePicker.vue'
import ItemInline from '../Shared/Item/ItemInline.vue'
import type { Granularity, RevenueResult, StallRevenueParams } from '../../types/stallTracker'
import SkeletonLoader from '../Shared/SkeletonLoader.vue'

const store = useStallTrackerStore()

const granularities: { value: Granularity; label: string }[] = [
  { value: 'daily', label: 'Daily' },
  { value: 'weekly', label: 'Weekly' },
  { value: 'monthly', label: 'Monthly' },
]

const granularity = ref<Granularity>('daily')
const filterDateFrom = ref<string>('')
const filterDateTo = ref<string>('')
const filterBuyer = ref<string | null>(null)
const filterItem = ref<string | null>(null)

const result = ref<RevenueResult | null>(null)
const loading = ref(false)
const error = ref<string | null>(null)

let reloadToken = 0

const hasActiveFilters = computed(
  () => !!(filterDateFrom.value || filterDateTo.value || filterBuyer.value || filterItem.value),
)

/** Periods reversed so the most recent date is the LEFTMOST column after the
 * sticky Total. Recent data should be visible without horizontal scrolling.
 * Cells, row totals, and grand total are unaffected because they're keyed by
 * `period_key` and looked up via the Map below. */
const displayPeriods = computed(() =>
  result.value ? [...result.value.periods].reverse() : [],
)

/** O(1) cell lookup keyed by `${item}|${period_key}`. */
const cellLookup = computed(() => {
  const m = new Map<string, number>()
  if (!result.value) return m
  for (const cell of result.value.cells) {
    m.set(`${cell.item}|${cell.period_key}`, cell.revenue)
  }
  return m
})

const rowTotalLookup = computed(() => {
  const m = new Map<string, number>()
  if (!result.value) return m
  for (const [item, total] of result.value.row_totals) m.set(item, total)
  return m
})

const colTotalLookup = computed(() => {
  const m = new Map<string, number>()
  if (!result.value) return m
  for (const [key, total] of result.value.col_totals) m.set(key, total)
  return m
})

function setGranularity(g: Granularity) {
  if (g === granularity.value) return
  granularity.value = g
  void reload()
}

function clearFilters() {
  filterDateFrom.value = ''
  filterDateTo.value = ''
  filterBuyer.value = null
  filterItem.value = null
}

function buildParams(): StallRevenueParams {
  return {
    owner: store.currentOwner,
    granularity: granularity.value,
    dateFrom: filterDateFrom.value || null,
    dateTo: filterDateTo.value || null,
    player: filterBuyer.value,
    item: filterItem.value,
  }
}

async function reload() {
  if (!store.currentOwner) {
    result.value = null
    error.value = null
    return
  }
  const token = ++reloadToken
  loading.value = true
  error.value = null
  try {
    const r = await invoke<RevenueResult>('get_stall_revenue', { params: buildParams() })
    if (token !== reloadToken) return
    result.value = r
  } catch (e) {
    if (token === reloadToken) error.value = String(e)
    console.error('[StallRevenueTab] reload failed:', e)
  } finally {
    if (token === reloadToken) loading.value = false
  }
}

/** Pivot cells render as blank for both missing and zero values, matching
 * plan §9.2's "empty cells blank not 0" readability rule. */
function formatCell(n: number | undefined): string {
  if (n === undefined || n === 0) return ''
  return n.toLocaleString()
}

/** Row/col/grand totals only blank for missing values — a real zero total
 * (theoretically possible if every contributing sale has been muted) should
 * render as "0", not vanish. */
function formatTotal(n: number | undefined): string {
  if (n === undefined) return ''
  return n.toLocaleString()
}

// Auto-swap inverted date range.
watch([filterDateFrom, filterDateTo], ([from, to]) => {
  if (from && to && from > to) {
    filterDateFrom.value = to
    filterDateTo.value = from
  }
})

// 200ms debounced reload on filter change.
let filterTimer: ReturnType<typeof setTimeout> | null = null
watch([filterDateFrom, filterDateTo, filterBuyer, filterItem], () => {
  if (filterTimer) clearTimeout(filterTimer)
  filterTimer = setTimeout(() => void reload(), 200)
})

watch(() => store.dataVersion, () => void reload())

watch(
  () => store.currentOwner,
  () => {
    clearFilters()
    void reload()
  },
)

onMounted(reload)
onBeforeUnmount(() => {
  if (filterTimer) clearTimeout(filterTimer)
})
</script>
