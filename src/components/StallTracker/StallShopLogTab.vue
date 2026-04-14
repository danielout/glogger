<template>
  <div class="flex flex-col gap-3 h-full min-h-0">
    <!-- Filter row -->
    <div class="flex items-center gap-2 flex-wrap flex-shrink-0">
      <input
        v-model="filterDateFrom"
        type="date"
        class="bg-surface-elevated border border-border-default rounded px-2 py-1 text-xs text-text-primary focus:outline-none focus:border-accent-gold/50" />
      <span class="text-text-secondary text-xs">–</span>
      <input
        v-model="filterDateTo"
        type="date"
        class="bg-surface-elevated border border-border-default rounded px-2 py-1 text-xs text-text-primary focus:outline-none focus:border-accent-gold/50" />
      <SearchableSelect
        v-model="filterPlayer"
        :options="store.filterOptions.players"
        all-label="All players" />
      <SearchableSelect
        v-model="filterAction"
        :options="actionOptions"
        all-label="All actions" />
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
        Showing {{ rows.length.toLocaleString() }} of {{ totalCount.toLocaleString() }} entries
      </span>
    </div>

    <!-- Table -->
    <div class="flex-1 min-h-0 overflow-auto border border-border-default rounded">
      <table class="w-full text-xs">
        <thead class="sticky top-0 bg-surface-elevated z-10">
          <tr class="text-text-secondary">
            <th class="px-2 py-1.5 text-center w-8"></th>
            <th class="px-2 py-1.5 text-left">DATE</th>
            <th class="px-2 py-1.5 text-left">PLAYER</th>
            <th class="px-2 py-1.5 text-left">ACTION</th>
            <th class="px-2 py-1.5 text-left">ITEM</th>
            <th class="px-2 py-1.5 text-right">QTY</th>
            <th class="px-2 py-1.5 text-right">GOLD</th>
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
                :title="row.ignored ? 'Click to un-mute' : 'Click to mute'"
                @click="handleToggleIgnored(row)">
                {{ row.ignored ? '○' : '⊘' }}
              </button>
            </td>
            <td class="px-2 py-1 text-text-secondary whitespace-nowrap">
              {{ row.event_timestamp }}
            </td>
            <td class="px-2 py-1 text-entity-player">{{ row.player }}</td>
            <td class="px-2 py-1">
              <span
                class="px-1.5 py-0.5 rounded text-[10px] uppercase font-medium"
                :class="actionBadgeClass(row.action)">
                {{ row.action }}
              </span>
            </td>
            <td class="px-2 py-1 text-text-primary">{{ row.item ?? '—' }}</td>
            <td class="px-2 py-1 text-right tabular-nums">{{ row.quantity || '' }}</td>
            <td class="px-2 py-1 text-right text-accent-gold tabular-nums">
              {{ formatGold(row.price_total) }}
            </td>
          </tr>
          <tr v-if="rows.length === 0 && !loading">
            <td
              colspan="7"
              class="px-2 py-8 text-center text-text-secondary">
              No entries match the current filters.
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
import { ref, computed, watch, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { confirm } from '@tauri-apps/plugin-dialog'
import { useStallTrackerStore } from '../../stores/stallTrackerStore'
import SearchableSelect from '../Shared/SearchableSelect.vue'
import type { StallEvent, StallEventsPage, StallEventsParams } from '../../types/stallTracker'

const store = useStallTrackerStore()

const PAGE_SIZE = 500

/** All seven action buckets, including `unknown` so the maintenance view can
 * surface unparseable entries (Sales tab hides them by default). */
const actionOptions: string[] = [
  'bought',
  'added',
  'removed',
  'configured',
  'visible',
  'collected',
  'unknown',
]

const rows = ref<StallEvent[]>([])
const totalCount = ref(0)
const loading = ref(false)
const error = ref<string | null>(null)

const filterDateFrom = ref<string>('')
const filterDateTo = ref<string>('')
const filterPlayer = ref<string | null>(null)
const filterAction = ref<string | null>(null)
const filterItem = ref<string | null>(null)

let reloadToken = 0

const hasActiveFilters = computed(
  () =>
    !!(
      filterDateFrom.value ||
      filterDateTo.value ||
      filterPlayer.value ||
      filterAction.value ||
      filterItem.value
    ),
)

function buildParams(offset: number): StallEventsParams {
  return {
    owner: store.currentOwner,
    dateFrom: filterDateFrom.value || null,
    dateTo: filterDateTo.value || null,
    player: filterPlayer.value,
    item: filterItem.value,
    action: filterAction.value,
    // The Shop Log shows ALL actions including 'unknown'. When the user
    // explicitly filters to 'unknown', `forceAction` overrides the backend's
    // default `action != 'unknown'` filter so unparseable entries surface.
    forceAction: filterAction.value === 'unknown' ? 'unknown' : null,
    includeIgnored: true,
    sortBy: 'event_at',
    sortDir: 'desc',
    limit: PAGE_SIZE,
    offset,
  }
}

async function reload() {
  if (!store.currentOwner) {
    rows.value = []
    totalCount.value = 0
    return
  }
  const token = ++reloadToken
  loading.value = true
  error.value = null
  try {
    const page = await invoke<StallEventsPage>('get_stall_events', {
      params: buildParams(0),
    })
    if (token !== reloadToken) return
    rows.value = page.rows
    totalCount.value = page.total_count
  } catch (e) {
    if (token === reloadToken) error.value = String(e)
    console.error('[StallShopLogTab] reload failed:', e)
  } finally {
    if (token === reloadToken) loading.value = false
  }
}

async function loadMore() {
  if (!store.currentOwner || loading.value) return
  const token = reloadToken
  loading.value = true
  try {
    const page = await invoke<StallEventsPage>('get_stall_events', {
      params: buildParams(rows.value.length),
    })
    if (token !== reloadToken) return
    rows.value.push(...page.rows)
    totalCount.value = page.total_count
  } catch (e) {
    if (token === reloadToken) error.value = String(e)
    console.error('[StallShopLogTab] loadMore failed:', e)
  } finally {
    if (token === reloadToken) loading.value = false
  }
}

async function handleToggleIgnored(row: StallEvent) {
  const ok = await confirm(
    `${row.ignored ? 'Un-mute' : 'Mute'} this ${row.action} entry?\n\n${row.raw_message}`,
    { title: 'Stall Tracker', kind: 'info' },
  )
  if (!ok) return
  try {
    await store.toggleIgnored(row.id, !row.ignored)
  } catch (e) {
    error.value = String(e)
    console.error('[StallShopLogTab] toggleIgnored failed:', e)
  }
}

function clearFilters() {
  filterDateFrom.value = ''
  filterDateTo.value = ''
  filterPlayer.value = null
  filterAction.value = null
  filterItem.value = null
}

function formatGold(n: number | null): string {
  if (n === null || n === 0) return ''
  return n.toLocaleString()
}

function actionBadgeClass(action: string): string {
  switch (action) {
    case 'bought':
      return 'bg-green-500/15 text-green-400'
    case 'added':
      return 'bg-blue-500/15 text-blue-400'
    case 'removed':
      return 'bg-red-500/15 text-red-400'
    case 'configured':
      return 'bg-amber-500/15 text-amber-400'
    case 'visible':
      return 'bg-cyan-500/15 text-cyan-400'
    case 'collected':
      return 'bg-accent-gold/15 text-accent-gold'
    default:
      return 'bg-surface-elevated text-text-secondary'
  }
}

watch([filterDateFrom, filterDateTo], ([from, to]) => {
  if (from && to && from > to) {
    filterDateFrom.value = to
    filterDateTo.value = from
  }
})

let filterTimer: ReturnType<typeof setTimeout> | null = null
watch(
  [filterDateFrom, filterDateTo, filterPlayer, filterAction, filterItem],
  () => {
    if (filterTimer) clearTimeout(filterTimer)
    filterTimer = setTimeout(() => void reload(), 200)
  },
)

watch(() => store.dataVersion, () => void reload())
watch(
  () => store.currentOwner,
  () => {
    clearFilters()
    void reload()
  },
)

onMounted(reload)
</script>
