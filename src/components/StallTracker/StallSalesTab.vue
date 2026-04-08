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

const store = useStallTrackerStore()

const filterBuyer = ref('')
const filterItem = ref('')

const buyerOptions = computed(() =>
  [...new Set(store.sales.map(s => s.player))].sort((a, b) => a.localeCompare(b))
)
const itemOptions = computed(() =>
  [...new Set(store.sales.map(s => s.item).filter((v): v is string => v != null))].sort((a, b) => a.localeCompare(b))
)

const MONTHS: Record<string, number> = {
  Jan: 1, Feb: 2, Mar: 3, Apr: 4, May: 5, Jun: 6,
  Jul: 7, Aug: 8, Sep: 9, Oct: 10, Nov: 11, Dec: 12,
}

/** Convert "Sat Mar 28 15:39" to a sortable number (MMDDHHMM). */
function timestampToSortKey(ts: string): number {
  // Format: "Day Mon DD HH:MM"
  const parts = ts.split(/\s+/)
  if (parts.length < 4) return 0
  const mon = MONTHS[parts[1]] ?? 0
  const day = parseInt(parts[2]) || 0
  const [hh, mm] = (parts[3] ?? '0:0').split(':').map(Number)
  return mon * 1000000 + day * 10000 + (hh || 0) * 100 + (mm || 0)
}

type SortKey = 'event_timestamp' | 'player' | 'item' | 'quantity' | 'price_unit' | 'price_total'
const sortKey = ref<SortKey>('event_timestamp')
const sortAsc = ref(false)

const sortedSales = computed(() => {
  const fb = filterBuyer.value
  const fi = filterItem.value
  const list = store.sales.filter(s =>
    (!fb || s.player === fb) &&
    (!fi || s.item === fi)
  )
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
