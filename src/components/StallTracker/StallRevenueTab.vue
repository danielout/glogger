<template>
  <div class="flex flex-col gap-4">
    <div v-if="loading" class="text-text-dim italic text-sm">Loading...</div>
    <div v-else-if="error" class="text-[#c87e7e] text-sm">{{ error }}</div>
    <EmptyState
      v-else-if="result && result.cells.length === 0"
      variant="panel"
      primary="No sales recorded"
      secondary="Open your shop log book in-game to start tracking, or use Import to load an exported book file." />

    <template v-else-if="result">
      <div class="flex items-center gap-3 flex-wrap">
        <div class="flex border border-border-default rounded overflow-hidden text-xs">
          <button
            v-for="g in granularities"
            :key="g.id"
            class="px-3 py-1.5 transition-colors"
            :class="granularity === g.id
              ? 'bg-accent-gold/20 text-accent-gold font-medium'
              : 'bg-surface-base text-text-muted hover:text-text-primary'"
            @click="granularity = g.id">
            {{ g.label }}
          </button>
        </div>

        <SearchableSelect v-model="filterDateFrom" :options="store.filterOptions.dates" placeholder="From date" />
        <span class="text-text-dim text-xs">&ndash;</span>
        <SearchableSelect v-model="filterDateTo" :options="store.filterOptions.dates" placeholder="To date" />
        <SearchableSelect v-model="filterBuyer" :options="store.filterOptions.buyers" placeholder="All buyers" />
        <SearchableSelect v-model="filterItem" :options="store.filterOptions.items" placeholder="All items" />
        <button
          v-if="hasActiveFilters()"
          class="text-xs text-text-dim hover:text-text-primary transition-colors underline"
          @click="resetFilters">
          Clear filters
        </button>
      </div>

      <div class="overflow-auto max-h-[70vh]">
        <table class="text-xs border-collapse">
          <thead class="sticky top-0 z-10">
            <tr class="bg-surface-base">
              <th class="sticky left-0 z-20 bg-surface-base text-left text-text-muted uppercase tracking-wide px-2 py-1.5 border-b border-r border-border-default w-[200px] min-w-[200px] max-w-[200px]">
                Item
              </th>
              <th class="sticky left-[200px] z-20 bg-surface-base text-right text-text-primary font-bold uppercase tracking-wide px-2 py-1.5 border-b border-r border-border-default">
                Total
              </th>
              <th
                v-for="period in displayPeriods"
                :key="period.key"
                class="text-right text-text-muted uppercase tracking-wide px-2 py-1.5 border-b border-border-default whitespace-nowrap">
                {{ period.label }}
              </th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="item in result.items"
              :key="item"
              class="hover:bg-[#1a1a2e] transition-colors">
              <td class="sticky left-0 bg-surface-base px-2 py-1 border-r border-b border-border-light w-[200px] min-w-[200px] max-w-[200px] overflow-hidden text-ellipsis whitespace-nowrap">
                <ItemInline :reference="item" :show-icon="false" />
              </td>
              <td class="sticky left-[200px] bg-surface-base text-right text-[#d4af37] font-bold px-2 py-1 border-r border-b border-border-light whitespace-nowrap">
                {{ formatCell(rowTotal(item)) }}
              </td>
              <td
                v-for="period in displayPeriods"
                :key="period.key"
                class="text-right text-[#d4af37] px-2 py-1 border-b border-border-light whitespace-nowrap">
                {{ formatCell(cellValue(item, period.key)) }}
              </td>
            </tr>
          </tbody>
          <tfoot class="sticky bottom-0">
            <tr class="bg-surface-base font-bold">
              <td class="sticky left-0 z-20 bg-surface-base text-text-primary px-2 py-1.5 border-t border-r border-border-default">
                Total
              </td>
              <td class="sticky left-[200px] z-20 bg-surface-base text-right text-[#d4af37] px-2 py-1.5 border-t border-r border-border-default whitespace-nowrap">
                {{ formatCell(result.grand_total) }}
              </td>
              <td
                v-for="period in displayPeriods"
                :key="period.key"
                class="text-right text-[#d4af37] px-2 py-1.5 border-t border-border-default whitespace-nowrap">
                {{ formatCell(colTotal(period.key)) }}
              </td>
            </tr>
          </tfoot>
        </table>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import EmptyState from '../Shared/EmptyState.vue'
import ItemInline from '../Shared/Item/ItemInline.vue'
import SearchableSelect from '../Shared/SearchableSelect.vue'
import { useStallTrackerStore } from '../../stores/stallTrackerStore'

const store = useStallTrackerStore()

type Granularity = 'daily' | 'weekly' | 'monthly'

interface RevenuePeriod { key: string, label: string }
interface RevenueCell { item: string, period_key: string, revenue: number }
interface RevenueResult {
  periods: RevenuePeriod[]
  items: string[]
  cells: RevenueCell[]
  row_totals: [string, number][]
  col_totals: [string, number][]
  grand_total: number
}

const granularities = [
  { id: 'daily' as Granularity, label: 'Daily' },
  { id: 'weekly' as Granularity, label: 'Weekly' },
  { id: 'monthly' as Granularity, label: 'Monthly' },
]
const granularity = ref<Granularity>('daily')

const filterDateFrom = ref('')
const filterDateTo = ref('')
const filterBuyer = ref('')
const filterItem = ref('')

const loading = ref(false)
const error = ref<string | null>(null)
const result = ref<RevenueResult | null>(null)

// Display periods newest-first so recent data is immediately visible
// without horizontal scrolling. Backend returns them chronologically ASC.
const displayPeriods = computed(() => {
  if (!result.value) return []
  return [...result.value.periods].reverse()
})

// Pre-indexed lookup tables, rebuilt when `result` changes.
const cellIndex = computed(() => {
  const m = new Map<string, number>()
  if (!result.value) return m
  for (const c of result.value.cells) {
    m.set(`${c.item}\t${c.period_key}`, c.revenue)
  }
  return m
})
const rowTotalIndex = computed(() => new Map(result.value?.row_totals ?? []))
const colTotalIndex = computed(() => new Map(result.value?.col_totals ?? []))

function cellValue(item: string, periodKey: string): number | undefined {
  return cellIndex.value.get(`${item}\t${periodKey}`)
}
function rowTotal(item: string): number | undefined {
  return rowTotalIndex.value.get(item)
}
function colTotal(periodKey: string): number | undefined {
  return colTotalIndex.value.get(periodKey)
}

function formatCell(value: number | undefined): string {
  if (!value) return ''
  return value.toLocaleString()
}

async function reload() {
  loading.value = true
  error.value = null
  try {
    result.value = await invoke<RevenueResult>('get_stall_revenue', {
      params: {
        owner: store.currentOwner,
        granularity: granularity.value,
        date_from: filterDateFrom.value || null,
        date_to: filterDateTo.value || null,
        buyer: filterBuyer.value || null,
        item: filterItem.value || null,
      },
    })
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
}

function resetFilters() {
  filterDateFrom.value = ''
  filterDateTo.value = ''
  filterBuyer.value = ''
  filterItem.value = ''
}

function hasActiveFilters() {
  return !!(filterDateFrom.value || filterDateTo.value || filterBuyer.value || filterItem.value)
}

onMounted(() => reload())

watch([granularity, filterDateFrom, filterDateTo, filterBuyer, filterItem], () => reload())
watch(() => store.dataVersion, () => reload())
watch(() => store.currentOwner, resetFilters)

watch([filterDateFrom, filterDateTo], ([from, to]) => {
  if (from && to && from > to) {
    filterDateFrom.value = to
    filterDateTo.value = from
  }
})
</script>
