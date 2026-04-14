<template>
  <div class="flex flex-col gap-3 h-full min-h-0">
    <!-- Stats header -->
    <div class="grid grid-cols-4 gap-3 flex-shrink-0">
      <StatCard
        label="TOTAL SALES"
        :value="formatNumber(store.stats?.total_sales ?? 0)" />
      <StatCard
        label="TOTAL REVENUE"
        :value="`${formatNumber(store.stats?.total_revenue ?? 0)}g`" />
      <StatCard
        label="UNIQUE BUYERS"
        :value="formatNumber(store.stats?.unique_buyers ?? 0)" />
      <StatCard
        label="UNIQUE ITEMS"
        :value="formatNumber(store.stats?.unique_items ?? 0)" />
    </div>

    <!-- Filter row -->
    <div class="flex items-center gap-2 flex-wrap flex-shrink-0">
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
      <span class="text-xs text-text-secondary ml-auto">
        Showing {{ rows.length.toLocaleString() }} of {{ totalCount.toLocaleString() }} sales
      </span>
    </div>

    <!-- Table -->
    <div class="flex-1 min-h-0 overflow-auto border border-border-default rounded">
      <table class="w-full text-xs">
        <thead class="sticky top-0 bg-surface-elevated z-10">
          <tr class="text-text-secondary">
            <th class="px-2 py-1.5 text-center w-8"></th>
            <th
              class="px-2 py-1.5 text-left cursor-pointer hover:text-text-primary"
              @click="toggleSort('event_at')">
              DATE {{ sortIndicator('event_at') }}
            </th>
            <th
              class="px-2 py-1.5 text-left cursor-pointer hover:text-text-primary"
              @click="toggleSort('player')">
              BUYER {{ sortIndicator('player') }}
            </th>
            <th
              class="px-2 py-1.5 text-left cursor-pointer hover:text-text-primary"
              @click="toggleSort('item')">
              ITEM {{ sortIndicator('item') }}
            </th>
            <th
              class="px-2 py-1.5 text-right cursor-pointer hover:text-text-primary"
              @click="toggleSort('quantity')">
              QTY {{ sortIndicator('quantity') }}
            </th>
            <th
              class="px-2 py-1.5 text-right cursor-pointer hover:text-text-primary"
              @click="toggleSort('price_unit')">
              UNIT {{ sortIndicator('price_unit') }}
            </th>
            <th
              class="px-2 py-1.5 text-right cursor-pointer hover:text-text-primary"
              @click="toggleSort('price_total')">
              TOTAL {{ sortIndicator('price_total') }}
            </th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="row in rows"
            :key="row.id"
            class="border-t border-border-default/40 hover:bg-surface-hover"
            :class="row.ignored ? 'opacity-35' : ''">
            <td class="px-2 py-1 text-center">
              <button
                type="button"
                class="text-text-secondary hover:text-text-primary"
                :title="row.ignored ? 'Click to un-mute' : 'Click to mute (excludes from stats)'"
                @click="handleToggleIgnored(row)">
                {{ row.ignored ? '○' : '⊘' }}
              </button>
            </td>
            <td class="px-2 py-1 text-text-secondary whitespace-nowrap">
              {{ row.event_timestamp }}
            </td>
            <td class="px-2 py-1 text-entity-player">{{ row.player }}</td>
            <td class="px-2 py-1">
              <ItemInline
                v-if="row.item"
                :reference="row.item" />
              <span
                v-else
                class="text-text-secondary">—</span>
            </td>
            <td class="px-2 py-1 text-right">{{ row.quantity }}</td>
            <td class="px-2 py-1 text-right">{{ formatPrice(row.price_unit) }}</td>
            <td class="px-2 py-1 text-right text-accent-gold">
              {{ formatPrice(row.price_total) }}
            </td>
          </tr>
          <tr v-if="rows.length === 0 && !loading">
            <td
              colspan="7"
              class="px-2 py-8 text-center text-text-secondary">
              No sales match the current filters.
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Load more -->
    <div
      v-if="rows.length < totalCount"
      class="flex-shrink-0 flex justify-center">
      <button
        type="button"
        class="px-4 py-1.5 text-xs bg-surface-elevated border border-border-default rounded hover:border-accent-gold/50 text-text-primary disabled:opacity-50"
        :disabled="loading"
        @click="loadMore">
        {{ loading ? 'Loading…' : `Load more (${(totalCount - rows.length).toLocaleString()} remaining)` }}
      </button>
    </div>

    <p
      v-if="error"
      class="text-xs text-red-400 flex-shrink-0">
      {{ error }}
    </p>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onBeforeUnmount } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { confirm } from '@tauri-apps/plugin-dialog'
import { useStallTrackerStore } from '../../stores/stallTrackerStore'
import SearchableSelect from '../Shared/SearchableSelect.vue'
import DatePicker from '../Shared/DatePicker.vue'
import ItemInline from '../Shared/Item/ItemInline.vue'
import StatCard from './StatCard.vue'
import type { StallEvent, StallEventsPage, StallEventsParams } from '../../types/stallTracker'

const store = useStallTrackerStore()

const PAGE_SIZE = 500

const rows = ref<StallEvent[]>([])
const totalCount = ref(0)
const loading = ref(false)
const error = ref<string | null>(null)

const filterDateFrom = ref<string>('')
const filterDateTo = ref<string>('')
const filterBuyer = ref<string | null>(null)
const filterItem = ref<string | null>(null)

const sortBy = ref<string>('event_at')
const sortDir = ref<'asc' | 'desc'>('desc')

// In-flight request token: every reload bumps it. Stale loadMore/reload
// responses (e.g. user clicked a sort header before the previous query
// returned) check the token and discard themselves rather than clobbering
// the visible state with rows from the wrong sort/filter set.
let reloadToken = 0

const hasActiveFilters = computed(
  () => !!(filterDateFrom.value || filterDateTo.value || filterBuyer.value || filterItem.value),
)

function sortIndicator(col: string): string {
  if (sortBy.value !== col) return ''
  return sortDir.value === 'asc' ? '▲' : '▼'
}

function toggleSort(col: string) {
  if (sortBy.value === col) {
    sortDir.value = sortDir.value === 'asc' ? 'desc' : 'asc'
  } else {
    sortBy.value = col
    // Numeric/date columns default desc, text defaults asc.
    sortDir.value = col === 'player' || col === 'item' ? 'asc' : 'desc'
  }
  void reload()
}

function buildParams(offset: number): StallEventsParams {
  return {
    owner: store.currentOwner,
    dateFrom: filterDateFrom.value || null,
    dateTo: filterDateTo.value || null,
    player: filterBuyer.value,
    item: filterItem.value,
    action: 'bought',
    includeIgnored: true,
    sortBy: sortBy.value,
    sortDir: sortDir.value,
    limit: PAGE_SIZE,
    offset,
  }
}

async function reload() {
  if (!store.currentOwner) {
    rows.value = []
    totalCount.value = 0
    error.value = null
    return
  }
  const token = ++reloadToken
  loading.value = true
  error.value = null
  try {
    const page = await invoke<StallEventsPage>('get_stall_events', {
      params: buildParams(0),
    })
    if (token !== reloadToken) return // superseded by a newer reload
    rows.value = page.rows
    totalCount.value = page.total_count
    // Keep stats header in sync with current filters.
    void store.loadStats({
      dateFrom: filterDateFrom.value || null,
      dateTo: filterDateTo.value || null,
      player: filterBuyer.value,
      item: filterItem.value,
    })
  } catch (e) {
    if (token === reloadToken) error.value = String(e)
    console.error('[StallSalesTab] reload failed:', e)
  } finally {
    if (token === reloadToken) loading.value = false
  }
}

async function loadMore() {
  if (!store.currentOwner || loading.value) return
  // Capture the current token. If a reload supersedes us mid-request,
  // we discard the appended rows so we don't mix sort orders.
  const token = reloadToken
  loading.value = true
  try {
    const page = await invoke<StallEventsPage>('get_stall_events', {
      params: buildParams(rows.value.length),
    })
    if (token !== reloadToken) return // a newer reload changed sort/filters
    rows.value.push(...page.rows)
    totalCount.value = page.total_count
  } catch (e) {
    if (token === reloadToken) error.value = String(e)
    console.error('[StallSalesTab] loadMore failed:', e)
  } finally {
    if (token === reloadToken) loading.value = false
  }
}

async function handleToggleIgnored(row: StallEvent) {
  const action = row.ignored ? 'un-mute' : 'mute'
  const ok = await confirm(
    `${action === 'mute' ? 'Mute' : 'Un-mute'} this sale?\n\n` +
      `${row.player} bought ${row.item ?? '(unknown)'} for ${formatPrice(row.price_total)}g.\n\n` +
      `${action === 'mute' ? 'Muted sales are excluded from stats but stay visible in the list.' : 'Un-muting will include this sale in stats again.'}`,
    { title: 'Stall Tracker', kind: 'info' },
  )
  if (!ok) return
  try {
    await store.toggleIgnored(row.id, !row.ignored)
    // dataVersion bump triggers reload via the watcher below.
  } catch (e) {
    error.value = String(e)
    console.error('[StallSalesTab] toggleIgnored failed:', e)
  }
}

function clearFilters() {
  filterDateFrom.value = ''
  filterDateTo.value = ''
  filterBuyer.value = null
  filterItem.value = null
}

function formatNumber(n: number): string {
  return n.toLocaleString()
}

function formatPrice(n: number | null): string {
  if (n === null) return '—'
  return Math.round(n).toLocaleString()
}

// Auto-swap inverted date range so users can't accidentally produce
// an empty result by entering "to" before "from".
watch([filterDateFrom, filterDateTo], ([from, to]) => {
  if (from && to && from > to) {
    filterDateFrom.value = to
    filterDateTo.value = from
  }
})

// Debounced reload on filter change (200ms) — avoids spamming invokes
// while the user is still typing in a date input.
let filterTimer: ReturnType<typeof setTimeout> | null = null
watch(
  [filterDateFrom, filterDateTo, filterBuyer, filterItem],
  () => {
    if (filterTimer) clearTimeout(filterTimer)
    filterTimer = setTimeout(() => void reload(), 200)
  },
)

// Refetch when the store says the data changed (live ingest, toggle, clear, import).
watch(() => store.dataVersion, () => void reload())

// Reset filter state on character switch — buyer "Alvida" from char A
// would otherwise carry over to char B and silently return zero rows.
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
