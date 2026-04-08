<template>
  <div class="flex flex-col gap-4">
    <div v-if="store.loading" class="text-text-dim italic text-sm">Loading shop log...</div>
    <div v-else-if="store.error" class="text-[#c87e7e] text-sm">{{ store.error }}</div>
    <EmptyState
      v-else-if="store.shopLog.length === 0"
      variant="panel"
      primary="No shop log entries"
      secondary="Open your shop log book in-game to import stall data." />

    <template v-else>
      <!-- Filters -->
      <div class="flex items-center gap-3 flex-wrap">
        <SearchableSelect v-model="filterDateFrom" :options="dateOptions" placeholder="From date" />
        <span class="text-text-dim text-xs">&ndash;</span>
        <SearchableSelect v-model="filterDateTo" :options="dateOptions" placeholder="To date" />
        <SearchableSelect v-model="filterPlayer" :options="playerOptions" placeholder="All players" />
        <SearchableSelect v-model="filterAction" :options="actionOptions" placeholder="All actions" />
        <SearchableSelect v-model="filterItem" :options="itemOptions" placeholder="All items" />
        <span class="text-xs text-text-muted">{{ sortedLog.length }} of {{ store.shopLog.length }} entries</span>
      </div>

      <div class="overflow-auto">
        <table class="w-full text-sm">
          <thead>
            <tr class="text-left text-text-muted text-xs uppercase tracking-wide border-b border-border-default">
              <th class="pb-2 pr-4 cursor-pointer hover:text-text-primary" @click="toggleSort('event_timestamp')">Date {{ sortIcon('event_timestamp') }}</th>
              <th class="pb-2 pr-4 cursor-pointer hover:text-text-primary" @click="toggleSort('player')">Player {{ sortIcon('player') }}</th>
              <th class="pb-2 pr-4 cursor-pointer hover:text-text-primary" @click="toggleSort('action')">Action {{ sortIcon('action') }}</th>
              <th class="pb-2 pr-4 cursor-pointer hover:text-text-primary" @click="toggleSort('item')">Item {{ sortIcon('item') }}</th>
              <th class="pb-2 pr-4 text-right cursor-pointer hover:text-text-primary" @click="toggleSort('quantity')">Qty {{ sortIcon('quantity') }}</th>
              <th class="pb-2 text-right cursor-pointer hover:text-text-primary" @click="toggleSort('gold')">Gold {{ sortIcon('gold') }}</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="entry in sortedLog"
              :key="entry.id"
              class="border-b border-border-light hover:bg-[#1a1a2e] transition-colors">
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
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import EmptyState from '../Shared/EmptyState.vue'
import ItemInline from '../Shared/Item/ItemInline.vue'
import SearchableSelect from '../Shared/SearchableSelect.vue'
import { useStallTrackerStore } from '../../stores/stallTrackerStore'
import type { StallEvent } from '../../types/stallTracker'
import { timestampToSortKey, timestampToDateKey, uniqueDates } from './stallTimestamp'

const store = useStallTrackerStore()

const filterDateFrom = ref('')
const filterDateTo = ref('')
const filterPlayer = ref('')
const filterAction = ref('')
const filterItem = ref('')

const dateOptions = computed(() => uniqueDates(store.shopLog.map(e => e.event_timestamp)))
const playerOptions = computed(() =>
  [...new Set(store.shopLog.map(e => e.player).filter(Boolean))].sort((a, b) => a.localeCompare(b))
)
const actionOptions = computed(() =>
  [...new Set(store.shopLog.map(e => e.action))].sort((a, b) => a.localeCompare(b))
)
const itemOptions = computed(() =>
  [...new Set(store.shopLog.map(e => e.item).filter((v): v is string => v != null))].sort((a, b) => a.localeCompare(b))
)

type SortKey = 'event_timestamp' | 'player' | 'action' | 'item' | 'quantity' | 'gold'
const sortKey = ref<SortKey>('event_timestamp')
const sortAsc = ref(false)

function goldValue(entry: StallEvent): number {
  return entry.price_total ?? entry.price_unit ?? 0
}

const sortedLog = computed(() => {
  const fp = filterPlayer.value
  const fa = filterAction.value
  const fi = filterItem.value
  const fromKey = filterDateFrom.value ? timestampToDateKey(filterDateFrom.value) : 0
  const toKey = filterDateTo.value ? timestampToDateKey(filterDateTo.value) : Infinity
  const list = store.shopLog.filter(e => {
    if (fp && e.player !== fp) return false
    if (fa && e.action !== fa) return false
    if (fi && e.item !== fi) return false
    if (fromKey || toKey < Infinity) {
      const dk = timestampToDateKey(e.event_timestamp)
      if (dk < fromKey || dk > toKey) return false
    }
    return true
  })
  const dir = sortAsc.value ? 1 : -1
  const key = sortKey.value
  list.sort((a, b) => {
    if (key === 'event_timestamp') return (timestampToSortKey(a.event_timestamp) - timestampToSortKey(b.event_timestamp)) * dir
    if (key === 'gold') return (goldValue(a) - goldValue(b)) * dir
    if (key === 'quantity') return (a.quantity - b.quantity) * dir
    const av = a[key as keyof StallEvent] as string | null
    const bv = b[key as keyof StallEvent] as string | null
    if (av == null && bv == null) return 0
    if (av == null) return 1
    if (bv == null) return -1
    return av.localeCompare(bv) * dir
  })
  return list
})

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
</script>
