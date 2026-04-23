<template>
  <div class="flex flex-col gap-3 h-full overflow-hidden">
    <!-- Top summary row -->
    <div class="flex items-center justify-between text-xs px-2">
      <div class="flex items-center gap-4 text-text-muted">
        <span>
          <span class="text-text-primary font-semibold">{{ store.historical.length }}</span>
          sessions
        </span>
        <span>
          <span class="text-accent-gold font-semibold">{{ totalUses }}</span>
          surveys
        </span>
        <span>
          <span class="text-accent-gold font-semibold">{{ totalLoot.toLocaleString() }}</span>
          items
        </span>
        <span>
          <span
            class="font-semibold"
            :class="totalProfit >= 0 ? 'text-accent-green' : 'text-accent-red'"
          >
            {{ formatGold(totalProfit) }}
          </span>
          total profit
        </span>
      </div>
      <button
        class="px-2.5 py-1 rounded border border-border-default bg-surface-elevated text-xs text-text-secondary hover:text-text-primary hover:border-border-hover transition-colors"
        @click="store.refreshHistorical()"
      >
        Refresh
      </button>
    </div>

    <!-- Empty state -->
    <div
      v-if="store.historical.length === 0"
      class="flex-1 flex items-center justify-center text-text-dim text-xs italic"
    >
      No sessions yet. The Session tab will populate this list as you play.
    </div>

    <!-- Session list -->
    <div v-else class="flex-1 min-h-0 overflow-y-auto pr-1">
      <div class="flex flex-col gap-2">
        <div
          v-for="row in store.historical"
          :key="row.session.id"
          class="rounded border border-border-default bg-surface-card"
        >
          <!-- Header row — clickable to expand -->
          <button
            class="w-full flex items-center justify-between px-3 py-2 text-left hover:bg-surface-elevated transition-colors"
            @click="toggleExpanded(row.session.id)"
          >
            <div class="flex items-center gap-3 min-w-0">
              <span class="text-text-dim text-xs">#{{ row.session.id }}</span>
              <span class="text-xs text-text-primary font-semibold">
                {{ formatTimeFull(row.session.started_at) }}
              </span>
              <span
                v-if="row.session.ended_at === null"
                class="text-[10px] px-1.5 py-0.5 rounded bg-accent-green/20 text-accent-green uppercase tracking-wider font-semibold"
              >
                Active
              </span>
              <span class="text-[10px] text-text-muted">
                {{ row.session.start_trigger }}
              </span>
            </div>
            <div class="flex items-center gap-4 text-xs whitespace-nowrap tabular-nums">
              <span>
                <span class="text-text-muted">surveys:</span>
                <span class="text-text-primary font-semibold ml-1">{{ row.total_uses }}</span>
              </span>
              <span>
                <span class="text-text-muted">loot:</span>
                <span class="text-accent-gold font-semibold ml-1">{{ row.total_loot_qty }}</span>
              </span>
              <span>
                <span class="text-text-muted">profit:</span>
                <span
                  class="font-semibold ml-1"
                  :class="rowProfit(row) >= 0 ? 'text-accent-green' : 'text-accent-red'"
                >
                  {{ formatGold(rowProfit(row)) }}
                </span>
              </span>
              <span class="text-text-muted">
                {{ row.duration_seconds !== null ? formatDuration(row.duration_seconds) : '—' }}
              </span>
              <span class="text-text-dim text-xs w-3 text-center">
                {{ expandedId === row.session.id ? '▾' : '▸' }}
              </span>
            </div>
          </button>

          <!-- Expanded body -->
          <div
            v-if="expandedId === row.session.id"
            class="border-t border-border-default px-3 py-3 flex flex-col gap-3 bg-surface-base"
          >
            <!-- Detail loads lazily on first expand. Once loaded it's the
                 same SessionSummary the Session tab renders for the active
                 session — so the two views stay visually aligned. -->
            <div v-if="loadedDetailId !== row.session.id">
              <button
                class="text-xs px-3 py-1.5 rounded border border-border-default bg-surface-elevated text-text-secondary hover:text-text-primary hover:border-border-hover transition-colors"
                @click="loadDetail(row.session.id)"
              >
                Load detail
              </button>
            </div>
            <SessionSummary
              v-else-if="store.openDetail"
              :detail="store.openDetail"
              :is-active="row.session.ended_at === null"
            />

            <!-- Notes editor -->
            <div>
              <div class="text-[10px] uppercase tracking-wider text-text-secondary font-semibold mb-1">
                Notes
              </div>
              <textarea
                v-model="notesDraft"
                class="w-full text-xs px-2 py-1.5 rounded border border-border-default bg-surface-elevated text-text-primary placeholder:text-text-dim focus:outline-none focus:border-accent-gold/50"
                rows="2"
                placeholder="Optional notes about this session…"
                @blur="saveNotes(row.session.id)"
              />
            </div>

            <!-- Danger zone -->
            <div class="flex justify-end">
              <button
                class="text-xs px-2.5 py-1 rounded border border-accent-red/50 text-accent-red hover:bg-accent-red/10 transition-colors"
                @click="pendingDeleteId = row.session.id; showDeleteConfirm = true"
              >
                Delete session
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <ModalDialog
      :show="showDeleteConfirm"
      title="Delete Session"
      type="confirm"
      :message="`Delete session #${pendingDeleteId}? This cannot be undone.`"
      confirm-label="Delete"
      :danger="true"
      @update:show="showDeleteConfirm = $event"
      @confirm="handleDeleteConfirmed"
    />
  </div>
</template>

<script setup lang="ts">
// Historical tab — expandable session rows with economics inline + a
// SessionSummary drill-in that matches the active-session layout. Reads
// the `survey_tracker_historical_sessions` + `_session_detail` + notes +
// delete commands from the Phase 5 aggregator.
import { ref, computed, onMounted } from 'vue'
import { useSurveyTrackerStore, type HistoricalSessionRow } from '../../stores/surveyTrackerStore'
import { formatTimeFull, formatDuration } from '../../composables/useTimestamp'
import { formatGold } from '../../composables/useRecipeCost'
import { liveEnrichedRows, liveProfit as computeLiveProfit } from '../../composables/useLiveValuation'
import ModalDialog from '../Shared/ModalDialog.vue'
import SessionSummary from './SessionSummary.vue'

const store = useSurveyTrackerStore()

// One session expanded at a time — keeps the UI compact and detail loads
// lazy (only fetched when a row is opened).
const expandedId = ref<number | null>(null)
const loadedDetailId = ref<number | null>(null)
const notesDraft = ref('')

const totalUses = computed(() =>
  store.historical.reduce((sum, r) => sum + r.total_uses, 0),
)
const totalLoot = computed(() =>
  store.historical.reduce((sum, r) => sum + r.total_loot_qty, 0),
)

/** Reactively compute profit for a single row using live market data. */
function rowProfit(row: HistoricalSessionRow): number {
  const enriched = liveEnrichedRows(row.loot_summary)
  return computeLiveProfit(enriched, row.economics.cost_total)
}

const totalProfit = computed(() =>
  store.historical.reduce((sum, r) => sum + rowProfit(r), 0),
)

onMounted(async () => {
  await store.refreshHistorical()
})

async function toggleExpanded(id: number) {
  if (expandedId.value === id) {
    expandedId.value = null
    return
  }
  expandedId.value = id
  const row = store.historical.find(r => r.session.id === id)
  notesDraft.value = row?.session.notes ?? ''
  loadedDetailId.value = null
  store.clearOpenDetail()
}

async function loadDetail(id: number) {
  await store.loadSessionDetail(id)
  loadedDetailId.value = id
}

async function saveNotes(id: number) {
  // Only PUT if the draft differs from what's persisted.
  const row = store.historical.find(r => r.session.id === id)
  const current = row?.session.notes ?? ''
  if (notesDraft.value !== current) {
    await store.updateSessionNotes(id, notesDraft.value)
  }
}

const showDeleteConfirm = ref(false)
const pendingDeleteId = ref<number | null>(null)

async function handleDeleteConfirmed() {
  const id = pendingDeleteId.value
  if (id === null) return
  pendingDeleteId.value = null
  const ok = await store.deleteSession(id)
  if (ok && expandedId.value === id) {
    expandedId.value = null
    loadedDetailId.value = null
    notesDraft.value = ''
  }
}
</script>
