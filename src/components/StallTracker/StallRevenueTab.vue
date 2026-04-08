<template>
  <div class="flex flex-col gap-4">
    <div v-if="store.loading" class="text-text-dim italic text-sm">Loading...</div>
    <div v-else-if="store.error" class="text-[#c87e7e] text-sm">{{ store.error }}</div>
    <EmptyState
      v-else-if="store.sales.length === 0"
      variant="panel"
      primary="No sales recorded"
      secondary="Open your shop log book in-game to import stall data." />

    <template v-else>
      <!-- Granularity toggle -->
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

        <!-- Filters -->
        <SearchableSelect v-model="filterDateFrom" :options="dateOptions" placeholder="From date" />
        <span class="text-text-dim text-xs">&ndash;</span>
        <SearchableSelect v-model="filterDateTo" :options="dateOptions" placeholder="To date" />
        <SearchableSelect v-model="filterBuyer" :options="buyerOptions" placeholder="All buyers" />
        <SearchableSelect v-model="filterItem" :options="itemOptions" placeholder="All items" />
      </div>

      <!-- Pivot table -->
      <div class="overflow-auto max-h-[70vh]">
        <table class="text-xs border-collapse">
          <thead class="sticky top-0 z-10">
            <tr class="bg-surface-base">
              <th class="sticky left-0 z-20 bg-surface-base text-left text-text-muted uppercase tracking-wide px-2 py-1.5 border-b border-r border-border-default min-w-[180px]">
                Item
              </th>
              <th
                v-for="period in pivot.periods"
                :key="period.key"
                class="text-right text-text-muted uppercase tracking-wide px-2 py-1.5 border-b border-border-default whitespace-nowrap">
                {{ period.label }}
              </th>
              <th class="text-right text-text-primary font-bold uppercase tracking-wide px-2 py-1.5 border-b border-l border-border-default">
                Total
              </th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="item in pivot.items"
              :key="item"
              class="hover:bg-[#1a1a2e] transition-colors">
              <td class="sticky left-0 bg-surface-base text-entity-item px-2 py-1 border-r border-b border-border-light whitespace-nowrap">
                {{ item }}
              </td>
              <td
                v-for="period in pivot.periods"
                :key="period.key"
                class="text-right text-[#d4af37] px-2 py-1 border-b border-border-light whitespace-nowrap">
                {{ formatCell(pivot.cells.get(item)?.get(period.key)) }}
              </td>
              <td class="text-right text-[#d4af37] font-bold px-2 py-1 border-l border-b border-border-light whitespace-nowrap">
                {{ formatCell(pivot.rowTotals.get(item)) }}
              </td>
            </tr>
          </tbody>
          <tfoot class="sticky bottom-0">
            <tr class="bg-surface-base font-bold">
              <td class="sticky left-0 z-20 bg-surface-base text-text-primary px-2 py-1.5 border-t border-r border-border-default">
                Total
              </td>
              <td
                v-for="period in pivot.periods"
                :key="period.key"
                class="text-right text-[#d4af37] px-2 py-1.5 border-t border-border-default whitespace-nowrap">
                {{ formatCell(pivot.colTotals.get(period.key)) }}
              </td>
              <td class="text-right text-[#d4af37] px-2 py-1.5 border-t border-l border-border-default whitespace-nowrap">
                {{ formatCell(pivot.grandTotal) }}
              </td>
            </tr>
          </tfoot>
        </table>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import EmptyState from '../Shared/EmptyState.vue'
import SearchableSelect from '../Shared/SearchableSelect.vue'
import { useStallTrackerStore } from '../../stores/stallTrackerStore'
import {
  timestampToDateKey,
  timestampToPeriod,
  collectPeriods,
  uniqueDates,
  type Granularity,
} from './stallTimestamp'

const store = useStallTrackerStore()

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

const dateOptions = computed(() => uniqueDates(store.sales.map(s => s.event_timestamp)))
const buyerOptions = computed(() =>
  [...new Set(store.sales.map(s => s.player))].sort((a, b) => a.localeCompare(b))
)
const itemOptions = computed(() =>
  [...new Set(store.sales.map(s => s.item).filter((v): v is string => v != null))].sort((a, b) => a.localeCompare(b))
)

const filteredSales = computed(() => {
  const fb = filterBuyer.value
  const fi = filterItem.value
  const fromKey = filterDateFrom.value ? timestampToDateKey(filterDateFrom.value) : 0
  const toKey = filterDateTo.value ? timestampToDateKey(filterDateTo.value) : Infinity
  return store.sales.filter(s => {
    if (fb && s.player !== fb) return false
    if (fi && s.item !== fi) return false
    if (fromKey || toKey < Infinity) {
      const dk = timestampToDateKey(s.event_timestamp)
      if (dk < fromKey || dk > toKey) return false
    }
    return true
  })
})

const pivot = computed(() => {
  const sales = filteredSales.value
  const g = granularity.value

  // Build cells: item → periodKey → sum
  const cells = new Map<string, Map<number, number>>()
  const rowTotals = new Map<string, number>()
  const colTotals = new Map<number, number>()
  let grandTotal = 0

  for (const s of sales) {
    const item = s.item ?? '(unknown)'
    const amount = s.price_total ?? 0
    const period = timestampToPeriod(s.event_timestamp, g)

    // Cell
    if (!cells.has(item)) cells.set(item, new Map())
    const row = cells.get(item)!
    row.set(period.key, (row.get(period.key) ?? 0) + amount)

    // Row total
    rowTotals.set(item, (rowTotals.get(item) ?? 0) + amount)

    // Column total
    colTotals.set(period.key, (colTotals.get(period.key) ?? 0) + amount)

    grandTotal += amount
  }

  // Sorted items
  const items = [...cells.keys()].sort((a, b) => a.localeCompare(b))

  // Sorted periods
  const periods = collectPeriods(sales.map(s => s.event_timestamp), g)

  return { items, periods, cells, rowTotals, colTotals, grandTotal }
})

function formatCell(value: number | undefined): string {
  if (!value) return ''
  return value.toLocaleString()
}
</script>
