<template>
  <div class="flex flex-col gap-4">
    <div v-if="loading && rows.length === 0" class="text-text-dim italic text-sm">Loading shop log...</div>
    <div v-else-if="error" class="text-[#c87e7e] text-sm">{{ error }}</div>
    <EmptyState
      v-else-if="!loading && totalCount === 0"
      variant="panel"
      primary="No shop log entries"
      secondary="Open your shop log book in-game to import stall data." />

    <template v-else>
      <div class="flex items-center gap-3 flex-wrap">
        <SearchableSelect v-model="filterDateFrom" :options="store.filterOptions.dates" placeholder="From date" />
        <span class="text-text-dim text-xs">&ndash;</span>
        <SearchableSelect v-model="filterDateTo" :options="store.filterOptions.dates" placeholder="To date" />
        <SearchableSelect v-model="filterPlayer" :options="store.filterOptions.players" placeholder="All players" />
        <SearchableSelect v-model="filterAction" :options="store.filterOptions.actions" placeholder="All actions" />
        <SearchableSelect v-model="filterItem" :options="store.filterOptions.items" placeholder="All items" />
        <span class="text-xs text-text-muted">Showing {{ rows.length.toLocaleString() }} of {{ totalCount.toLocaleString() }} entries</span>
      </div>

      <div class="overflow-auto">
        <table class="w-full text-sm">
          <thead>
            <tr class="text-left text-text-muted text-xs uppercase tracking-wide border-b border-border-default">
              <th class="pb-2 w-8"></th>
              <th class="pb-2 pr-4 cursor-pointer hover:text-text-primary" @click="toggleSort('event_at')">Date {{ sortIcon('event_at') }}</th>
              <th class="pb-2 pr-4 cursor-pointer hover:text-text-primary" @click="toggleSort('player')">Player {{ sortIcon('player') }}</th>
              <th class="pb-2 pr-4 cursor-pointer hover:text-text-primary" @click="toggleSort('action')">Action {{ sortIcon('action') }}</th>
              <th class="pb-2 pr-4 cursor-pointer hover:text-text-primary" @click="toggleSort('item')">Item {{ sortIcon('item') }}</th>
              <th class="pb-2 pr-4 text-right cursor-pointer hover:text-text-primary" @click="toggleSort('quantity')">Qty {{ sortIcon('quantity') }}</th>
              <th class="pb-2 text-right cursor-pointer hover:text-text-primary" @click="toggleSort('price_total')">Gold {{ sortIcon('price_total') }}</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="entry in rows"
              :key="entry.id"
              class="border-b border-border-light hover:bg-[#1a1a2e] transition-colors"
              :class="{ 'opacity-35': entry.ignored }">
              <td class="py-1.5 pr-2 text-center">
                <button
                  class="text-text-dim hover:text-text-primary text-xs cursor-pointer"
                  :title="entry.ignored ? 'Include in stats' : 'Exclude from stats'"
                  @click="handleToggleIgnored(entry)">
                  {{ entry.ignored ? '\u25CB' : '\u2298' }}
                </button>
              </td>
              <td class="py-1.5 pr-4 text-text-dim text-xs whitespace-nowrap">{{ entry.event_timestamp }}</td>
              <td class="py-1.5 pr-4 text-entity-player">{{ entry.player }}</td>
              <td class="py-1.5 pr-4">
                <span class="inline-block px-1.5 py-0.5 rounded text-xs font-medium" :class="actionClass(entry.action)">
                  {{ entry.action }}
                </span>
              </td>
              <td class="py-1.5 pr-4"><ItemInline v-if="entry.item" :reference="entry.item" :show-icon="false" /></td>
              <td class="py-1.5 pr-4 text-right text-text-secondary">{{ entry.item ? entry.quantity : '' }}</td>
              <td class="py-1.5 text-right text-[#d4af37]">{{ formatGold(entry) }}</td>
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
import type { StallEvent } from '../../types/stallTracker'

interface StallEventsPage {
  rows: StallEvent[]
  total_count: number
}

type SortKey = 'event_at' | 'player' | 'action' | 'item' | 'quantity' | 'price_total'

const PAGE_SIZE = 500

const store = useStallTrackerStore()

const filterDateFrom = ref('')
const filterDateTo = ref('')
const filterPlayer = ref('')
const filterAction = ref('')
const filterItem = ref('')

const sortKey = ref<SortKey>('event_at')
const sortAsc = ref(false)

const rows = shallowRef<StallEvent[]>([])
const totalCount = ref(0)
const loading = ref(false)
const error = ref<string | null>(null)

function buildParams(offset: number) {
  return {
    owner: store.currentOwner,
    action: filterAction.value || null,
    player: filterPlayer.value || null,
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

async function reload() {
  loading.value = true
  error.value = null
  try {
    const page = await invoke<StallEventsPage>('get_stall_events', { params: buildParams(0) })
    rows.value = page.rows
    totalCount.value = page.total_count
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
    sortAsc.value = key === 'player' || key === 'item' || key === 'action'
  }
}

function sortIcon(key: string): string {
  if (sortKey.value !== key) return ''
  return sortAsc.value ? '\u25B2' : '\u25BC'
}

function actionClass(action: string): string {
  switch (action) {
    case 'bought': return 'bg-[#1a3a1a] text-[#7ec87e]'
    case 'added': return 'bg-[#1a2a3a] text-[#7eaac8]'
    case 'removed': return 'bg-[#3a1a1a] text-[#c87e7e]'
    case 'configured': return 'bg-[#3a3a1a] text-[#c8c87e]'
    case 'visible': return 'bg-[#2a1a3a] text-[#a87ec8]'
    case 'collected': return 'bg-[#2a3a1a] text-[#d4af37]'
    default: return 'bg-[#2a2a2a] text-text-dim'
  }
}

function formatGold(entry: StallEvent): string {
  if (entry.price_total != null) return entry.price_total.toLocaleString() + 'g'
  if (entry.price_unit != null) return Math.round(entry.price_unit).toLocaleString() + 'g/ea'
  return ''
}

async function handleToggleIgnored(event: StallEvent) {
  const action = event.ignored ? 'Include' : 'Exclude'
  const item = event.item ?? 'this event'
  const ok = await confirm(
    `${action} "${item}" (${event.action}, ${event.event_timestamp}) from stats?`,
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

watch([filterDateFrom, filterDateTo], ([from, to]) => {
  if (from && to && from > to) {
    filterDateFrom.value = to
    filterDateTo.value = from
  }
})

let reloadTimer: ReturnType<typeof setTimeout> | null = null
function scheduleReload() {
  if (reloadTimer) clearTimeout(reloadTimer)
  reloadTimer = setTimeout(() => reload(), 200)
}

onMounted(() => reload())

watch([filterDateFrom, filterDateTo, filterPlayer, filterAction, filterItem, sortKey, sortAsc], scheduleReload)
watch(() => store.dataVersion, () => reload())
</script>
