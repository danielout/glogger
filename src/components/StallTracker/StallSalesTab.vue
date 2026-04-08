<template>
  <div class="flex flex-col gap-4">
    <div v-if="store.loading" class="text-text-dim italic text-sm">Loading sales...</div>
    <div v-else-if="store.error" class="text-[#c87e7e] text-sm">{{ store.error }}</div>
    <EmptyState
      v-else-if="store.sales.length === 0"
      variant="panel"
      primary="No sales recorded"
      secondary="Open your shop log book in-game to import stall data." />

    <template v-else>
      <!-- Stats summary -->
      <div class="flex gap-6 flex-wrap text-center">
        <div>
          <div class="text-[0.65rem] text-text-muted uppercase tracking-wide">Total Sales</div>
          <div class="text-lg font-bold text-text-primary">{{ filteredStats.totalSales.toLocaleString() }}</div>
        </div>
        <div>
          <div class="text-[0.65rem] text-text-muted uppercase tracking-wide">Total Revenue</div>
          <div class="text-lg font-bold text-[#d4af37]">{{ filteredStats.totalRevenue.toLocaleString() }}g</div>
        </div>
        <div>
          <div class="text-[0.65rem] text-text-muted uppercase tracking-wide">Unique Buyers</div>
          <div class="text-lg font-bold text-text-primary">{{ filteredStats.uniqueBuyers.toLocaleString() }}</div>
        </div>
        <div>
          <div class="text-[0.65rem] text-text-muted uppercase tracking-wide">Unique Items</div>
          <div class="text-lg font-bold text-text-primary">{{ filteredStats.uniqueItems.toLocaleString() }}</div>
        </div>
      </div>

      <!-- Filters -->
      <div class="flex items-center gap-3 flex-wrap">
        <SearchableSelect v-model="filterDateFrom" :options="dateOptions" placeholder="From date" />
        <span class="text-text-dim text-xs">&ndash;</span>
        <SearchableSelect v-model="filterDateTo" :options="dateOptions" placeholder="To date" />
        <SearchableSelect v-model="filterBuyer" :options="buyerOptions" placeholder="All buyers" />
        <SearchableSelect v-model="filterItem" :options="itemOptions" placeholder="All items" />
        <span class="text-xs text-text-muted">{{ sortedSales.length }} of {{ store.sales.length }} sales</span>
      </div>

      <!-- Sales table -->
      <div class="overflow-auto">
        <table class="w-full text-sm">
          <thead>
            <tr class="text-left text-text-muted text-xs uppercase tracking-wide border-b border-border-default">
              <th class="pb-2 pr-4 cursor-pointer hover:text-text-primary" @click="toggleSort('event_timestamp')">Date {{ sortIcon('event_timestamp') }}</th>
              <th class="pb-2 pr-4 cursor-pointer hover:text-text-primary" @click="toggleSort('player')">Buyer {{ sortIcon('player') }}</th>
              <th class="pb-2 pr-4 cursor-pointer hover:text-text-primary" @click="toggleSort('item')">Item {{ sortIcon('item') }}</th>
              <th class="pb-2 pr-4 text-right cursor-pointer hover:text-text-primary" @click="toggleSort('quantity')">Qty {{ sortIcon('quantity') }}</th>
              <th class="pb-2 pr-4 text-right cursor-pointer hover:text-text-primary" @click="toggleSort('price_unit')">Unit Price {{ sortIcon('price_unit') }}</th>
              <th class="pb-2 text-right cursor-pointer hover:text-text-primary" @click="toggleSort('price_total')">Total {{ sortIcon('price_total') }}</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="sale in sortedSales"
              :key="sale.id"
              class="border-b border-border-light hover:bg-[#1a1a2e] transition-colors">
              <td class="py-1.5 pr-4 text-text-dim text-xs whitespace-nowrap">{{ sale.event_timestamp }}</td>
              <td class="py-1.5 pr-4 text-entity-player">{{ sale.player }}</td>
              <td class="py-1.5 pr-4 text-entity-item">{{ sale.item }}</td>
              <td class="py-1.5 pr-4 text-right text-text-secondary">{{ sale.quantity }}</td>
              <td class="py-1.5 pr-4 text-right text-text-secondary">{{ sale.price_unit != null ? Math.round(sale.price_unit).toLocaleString() + 'g' : '' }}</td>
              <td class="py-1.5 text-right text-[#d4af37] font-medium">{{ sale.price_total != null ? sale.price_total.toLocaleString() + 'g' : '' }}</td>
            </tr>
          </tbody>
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
import { timestampToSortKey, timestampToDateKey, uniqueDates } from './stallTimestamp'

const store = useStallTrackerStore()

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

type SortKey = 'event_timestamp' | 'player' | 'item' | 'quantity' | 'price_unit' | 'price_total'
const sortKey = ref<SortKey>('event_timestamp')
const sortAsc = ref(false)

const sortedSales = computed(() => {
  const fb = filterBuyer.value
  const fi = filterItem.value
  const fromKey = filterDateFrom.value ? timestampToDateKey(filterDateFrom.value) : 0
  const toKey = filterDateTo.value ? timestampToDateKey(filterDateTo.value) : Infinity
  const list = store.sales.filter(s => {
    if (fb && s.player !== fb) return false
    if (fi && s.item !== fi) return false
    if (fromKey || toKey < Infinity) {
      const dk = timestampToDateKey(s.event_timestamp)
      if (dk < fromKey || dk > toKey) return false
    }
    return true
  })
  const dir = sortAsc.value ? 1 : -1
  const key = sortKey.value
  list.sort((a, b) => {
    if (key === 'event_timestamp') return (timestampToSortKey(a.event_timestamp) - timestampToSortKey(b.event_timestamp)) * dir
    const av = a[key]
    const bv = b[key]
    if (av == null && bv == null) return 0
    if (av == null) return 1
    if (bv == null) return -1
    if (typeof av === 'string') return av.localeCompare(bv as string) * dir
    return ((av as number) - (bv as number)) * dir
  })
  return list
})

const filteredStats = computed(() => {
  const list = sortedSales.value
  return {
    totalSales: list.length,
    totalRevenue: list.reduce((sum, s) => sum + (s.price_total ?? 0), 0),
    uniqueBuyers: new Set(list.map(s => s.player)).size,
    uniqueItems: new Set(list.map(s => s.item).filter(Boolean)).size,
  }
})

function toggleSort(key: SortKey) {
  if (sortKey.value === key) {
    sortAsc.value = !sortAsc.value
  } else {
    sortKey.value = key
    sortAsc.value = key === 'player' || key === 'item'
  }
}

function sortIcon(key: string): string {
  if (sortKey.value !== key) return ''
  return sortAsc.value ? '\u25B2' : '\u25BC'
}
</script>
