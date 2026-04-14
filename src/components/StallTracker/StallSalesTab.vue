<template>
  <div class="flex flex-col gap-4">
    <div v-if="loading && rows.length === 0" class="text-text-dim italic text-sm">Loading sales...</div>
    <div v-else-if="error" class="text-[#c87e7e] text-sm">{{ error }}</div>
    <EmptyState
      v-else-if="!loading && totalCount === 0"
      variant="panel"
      primary="No sales recorded"
      secondary="Open your shop log book in-game to start tracking, or use Import to load an exported book file." />

    <template v-else>
      <div class="flex gap-6 flex-wrap text-center">
        <div>
          <div class="text-[0.65rem] text-text-muted uppercase tracking-wide">Total Sales</div>
          <div class="text-lg font-bold text-text-primary">{{ (stats?.total_sales ?? 0).toLocaleString() }}</div>
        </div>
        <div>
          <div class="text-[0.65rem] text-text-muted uppercase tracking-wide">Total Revenue</div>
          <div class="text-lg font-bold text-[#d4af37]">{{ (stats?.total_revenue ?? 0).toLocaleString() }}g</div>
        </div>
        <div>
          <div class="text-[0.65rem] text-text-muted uppercase tracking-wide">Unique Buyers</div>
          <div class="text-lg font-bold text-text-primary">{{ (stats?.unique_buyers ?? 0).toLocaleString() }}</div>
        </div>
        <div>
          <div class="text-[0.65rem] text-text-muted uppercase tracking-wide">Unique Items</div>
          <div class="text-lg font-bold text-text-primary">{{ (stats?.unique_items ?? 0).toLocaleString() }}</div>
        </div>
      </div>

      <div class="flex items-center gap-3 flex-wrap">
        <SearchableSelect v-model="filterDateFrom" :options="store.filterOptions.dates" placeholder="From date" />
        <span class="text-text-dim text-xs">&ndash;</span>
        <SearchableSelect v-model="filterDateTo" :options="store.filterOptions.dates" placeholder="To date" />
        <SearchableSelect v-model="filterBuyer" :options="store.filterOptions.buyers" placeholder="All buyers" />
        <SearchableSelect v-model="filterItem" :options="store.filterOptions.items" placeholder="All items" />
        <span class="text-xs text-text-muted">Showing {{ rows.length.toLocaleString() }} of {{ totalCount.toLocaleString() }} sales</span>
        <button
          v-if="hasActiveFilters()"
          class="text-xs text-text-dim hover:text-text-primary transition-colors underline"
          @click="resetFilters">
          Clear filters
        </button>
      </div>

      <div class="overflow-auto">
        <table class="w-full text-sm">
          <thead>
            <tr class="text-left text-text-muted text-xs uppercase tracking-wide border-b border-border-default">
              <th class="pb-2 w-8"></th>
              <th class="pb-2 pr-4 cursor-pointer hover:text-text-primary" @click="toggleSort('event_at')">Date {{ sortIcon('event_at') }}</th>
              <th class="pb-2 pr-4 cursor-pointer hover:text-text-primary" @click="toggleSort('player')">Buyer {{ sortIcon('player') }}</th>
              <th class="pb-2 pr-4 cursor-pointer hover:text-text-primary" @click="toggleSort('item')">Item {{ sortIcon('item') }}</th>
              <th class="pb-2 pr-4 text-right cursor-pointer hover:text-text-primary" @click="toggleSort('quantity')">Qty {{ sortIcon('quantity') }}</th>
              <th class="pb-2 pr-4 text-right cursor-pointer hover:text-text-primary" @click="toggleSort('price_unit')">Unit Price {{ sortIcon('price_unit') }}</th>
              <th class="pb-2 text-right cursor-pointer hover:text-text-primary" @click="toggleSort('price_total')">Total {{ sortIcon('price_total') }}</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="sale in rows"
              :key="sale.id"
              class="border-b border-border-light hover:bg-[#1a1a2e] transition-colors"
              :class="{ 'opacity-35': sale.ignored }">
              <td class="py-1.5 pr-2 text-center">
                <button
                  class="text-text-dim hover:text-text-primary text-xs cursor-pointer"
                  :title="sale.ignored ? 'Include in stats' : 'Exclude from stats'"
                  @click="handleToggleIgnored(sale)">
                  {{ sale.ignored ? '\u25CB' : '\u2298' }}
                </button>
              </td>
              <td class="py-1.5 pr-4 text-text-dim text-xs whitespace-nowrap">{{ sale.event_timestamp }}</td>
              <td class="py-1.5 pr-4 text-entity-player">{{ sale.player }}</td>
              <td class="py-1.5 pr-4"><ItemInline v-if="sale.item" :reference="sale.item" :show-icon="false" /></td>
              <td class="py-1.5 pr-4 text-right text-text-secondary">{{ sale.quantity }}</td>
              <td class="py-1.5 pr-4 text-right text-text-secondary">{{ sale.price_unit != null ? Math.round(sale.price_unit).toLocaleString() + 'g' : '' }}</td>
              <td class="py-1.5 text-right text-[#d4af37] font-medium">{{ sale.price_total != null ? sale.price_total.toLocaleString() + 'g' : '' }}</td>
            </tr>
          </tbody>
        </table>
      </div>

      <div v-if="rows.length < totalCount" class="flex justify-center">
        <button
          class="text-xs text-text-dim hover:text-text-primary border border-border-default rounded px-3 py-1.5"
          :disabled="loading"
          @click="loadMore">
          {{ loading ? 'Loading...' : `Load more (${(totalCount - rows.length).toLocaleString()} remaining)` }}
        </button>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, shallowRef, watch, onMounted } from 'vue'
import { confirm } from '@tauri-apps/plugin-dialog'
import { invoke } from '@tauri-apps/api/core'
import EmptyState from '../Shared/EmptyState.vue'
import ItemInline from '../Shared/Item/ItemInline.vue'
import SearchableSelect from '../Shared/SearchableSelect.vue'
import { useStallTrackerStore } from '../../stores/stallTrackerStore'
import type { StallEvent, StallStats } from '../../types/stallTracker'

interface StallEventsPage {
  rows: StallEvent[]
  total_count: number
}

type SortKey = 'event_at' | 'player' | 'item' | 'quantity' | 'price_unit' | 'price_total'

const PAGE_SIZE = 500

const store = useStallTrackerStore()

const filterDateFrom = ref('')
const filterDateTo = ref('')
const filterBuyer = ref('')
const filterItem = ref('')

const sortKey = ref<SortKey>('event_at')
const sortAsc = ref(false)

const rows = shallowRef<StallEvent[]>([])
const totalCount = ref(0)
const stats = ref<StallStats | null>(null)
const loading = ref(false)
const error = ref<string | null>(null)

function buildParams(offset: number) {
  return {
    owner: store.currentOwner,
    action: 'bought',
    player: filterBuyer.value || null,
    item: filterItem.value || null,
    date_from: filterDateFrom.value || null,
    date_to: filterDateTo.value || null,
    include_ignored: true,
    sort_by: sortKey.value,
    sort_dir: sortAsc.value ? 'asc' : 'desc',
    limit: PAGE_SIZE,
    offset,
  }
}

function statsFilters() {
  return {
    owner: store.currentOwner,
    player: filterBuyer.value || null,
    item: filterItem.value || null,
    date_from: filterDateFrom.value || null,
    date_to: filterDateTo.value || null,
  }
}

async function reload() {
  loading.value = true
  error.value = null
  try {
    const [page, s] = await Promise.all([
      invoke<StallEventsPage>('get_stall_events', { params: buildParams(0) }),
      invoke<StallStats>('get_stall_stats', { filters: statsFilters() }),
    ])
    rows.value = page.rows
    totalCount.value = page.total_count
    stats.value = s
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
}

async function loadMore() {
  if (loading.value) return
  loading.value = true
  try {
    const page = await invoke<StallEventsPage>('get_stall_events', {
      params: buildParams(rows.value.length),
    })
    rows.value = [...rows.value, ...page.rows]
    totalCount.value = page.total_count
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
}

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

async function handleToggleIgnored(event: StallEvent) {
  const action = event.ignored ? 'Include' : 'Exclude'
  const item = event.item ?? 'this event'
  const ok = await confirm(
    `${action} "${item}" (${event.event_timestamp}, ${event.price_total?.toLocaleString() ?? 0}g) from stats?`,
    { title: `${action} Event`, kind: 'info' },
  )
  if (ok) {
    try {
      await store.toggleIgnored(event.id, !event.ignored)
    } catch (e) {
      error.value = `Failed to update event: ${e}`
    }
  }
}

// Auto-swap filter dates if the user ends up with from > to. This matches
// the user's likely intent (they picked two dates) rather than silently
// returning an empty result set.
watch([filterDateFrom, filterDateTo], ([from, to]) => {
  if (from && to && from > to) {
    filterDateFrom.value = to
    filterDateTo.value = from
  }
})

// Debounce so rapid filter typing doesn't spam invokes.
let reloadTimer: ReturnType<typeof setTimeout> | null = null
function scheduleReload() {
  if (reloadTimer) clearTimeout(reloadTimer)
  reloadTimer = setTimeout(() => reload(), 200)
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

watch([filterDateFrom, filterDateTo, filterBuyer, filterItem, sortKey, sortAsc], scheduleReload)
watch(() => store.dataVersion, () => reload())
// Reset filters when the active character changes — otherwise a buyer name
// from character A sticks in the filter and silently returns zero rows for
// character B.
watch(() => store.currentOwner, resetFilters)
</script>
