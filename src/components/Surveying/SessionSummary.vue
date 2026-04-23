<template>
  <div class="flex flex-col gap-4">
    <!-- Top stat cards: economics + duration -->
    <div class="grid grid-cols-2 xl:grid-cols-4 gap-2">
      <StatCard
        label="Duration"
        :value="durationDisplay"
        :sub="etaSub"
      />
      <StatCard
        label="Surveys"
        :value="usesDisplay"
        :sub="consumedSub"
      />
      <StatCard
        label="Profit"
        :value="formatGold(liveProf)"
        :sub="profitRateSub"
        :accent="profitAccent"
      />
      <StatCard
        label="Revenue"
        :value="formatGold(liveRev)"
        :sub="costSub"
      />
    </div>

    <!-- Price-gap notice when some items are unpriced -->
    <div
      v-if="liveItemsUnpriced > 0"
      class="text-[10px] text-text-dim italic px-1"
    >
      {{ liveItemsUnpriced }} item{{ liveItemsUnpriced === 1 ? '' : 's' }}
      have no market value set — revenue and profit are low-balled.
      Add prices in the Market Prices tab to improve accuracy.
    </div>

    <!-- Uses list -->
    <section v-if="detail.uses.length > 0" class="flex flex-col gap-1">
      <header class="flex items-baseline justify-between">
        <h4 class="text-[10px] uppercase tracking-widest text-text-secondary font-semibold">
          Uses
        </h4>
        <span class="text-[10px] text-text-dim">{{ detail.uses.length }} recorded</span>
      </header>
      <div class="flex flex-col gap-1 max-h-64 overflow-y-auto pr-1">
        <div
          v-for="u in detail.uses"
          :key="u.id"
          class="rounded border border-border-default bg-surface-elevated px-2 py-1.5"
        >
          <div class="flex items-start justify-between gap-2">
            <div class="flex-1 min-w-0">
              <div class="text-xs text-text-primary truncate">
                {{ u.map_display_name }}
              </div>
              <div class="text-[10px] text-text-secondary">
                {{ kindBadge(u.kind) }}
                · {{ formatTimeFull(u.used_at) }}
                <span v-if="u.area"> · <AreaInline :reference="u.area" /></span>
              </div>
            </div>
            <div class="text-right whitespace-nowrap tabular-nums">
              <div class="text-xs">
                <span class="text-accent-gold font-semibold">{{ u.loot_qty }}</span>
                <span class="text-text-dim"> loot</span>
              </div>
              <div class="text-[10px]" :class="statusColor(u.status)">
                {{ u.status.replace('_', ' ') }}
              </div>
            </div>
          </div>
        </div>
      </div>
    </section>

    <!-- Loot totals: grid of minicards with primary/bonus breakdown -->
    <SurveyLootGrid
      :rows="liveRows"
      title="Loot Totals"
      title-class="text-text-secondary"
      :subtitle="`${liveRows.length} unique`"
      show-unpriced-hint
    />
  </div>
</template>

<script setup lang="ts">
// Shared detail view for a survey session — used by both the Session tab's
// center panel (the active session) and the Session History tab's expanded
// row. Takes a `SurveySessionDetail` + an "is active?" flag and renders:
//   - stat cards (duration, surveys, profit, revenue with derived rates)
//   - per-use list with status
//   - per-item loot totals as ItemMinicards with primary/bonus breakdown
//
// Live duration/ETA ticks every second when isActive; ended sessions
// anchor on ended_at.
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import type { SurveySessionDetail, SurveyUseKind, SurveyUseStatus } from '../../stores/surveyTrackerStore'
import { formatTimeFull, formatDuration } from '../../composables/useTimestamp'
import { formatGold } from '../../composables/useRecipeCost'
import { liveEnrichedRows, liveRevenue, liveProfit as computeLiveProfit } from '../../composables/useLiveValuation'
import StatCard from './StatCard.vue'
import SurveyLootGrid from './SurveyLootGrid.vue'
import AreaInline from '../Shared/Area/AreaInline.vue'

const props = defineProps<{
  detail: SurveySessionDetail
  /** When true, duration ticks live and ETA is computed on the fly. */
  isActive?: boolean
}>()

// ── Reactive economics ─────────────────────────────────────────────────
// Re-derive revenue/profit from the loot_summary rows + live market data
// so changes to market prices immediately update the display. Cost comes
// from the backend snapshot (recipe-based, not market-dependent).

const liveRows = computed(() => liveEnrichedRows(props.detail.loot_summary))
const liveRev = computed(() => liveRevenue(liveRows.value))
const liveCost = computed(() => props.detail.economics.cost_total)
const liveProf = computed(() => computeLiveProfit(liveRows.value, liveCost.value))
const liveItemsUnpriced = computed(() =>
  liveRows.value.filter(r => r.unit_value === null).length,
)

// One-per-component interval; cheap (re-render of a computed once/sec).
const liveNow = ref<number>(Date.now())
let tickId: number | null = null
onMounted(() => {
  if (props.isActive) {
    tickId = window.setInterval(() => { liveNow.value = Date.now() }, 1000)
  }
})
onBeforeUnmount(() => {
  if (tickId !== null) window.clearInterval(tickId)
})

function parseTs(ts: string): number {
  // Backend timestamps are "YYYY-MM-DD HH:MM:SS" in UTC. Date.parse treats
  // that as local time on some browsers; append "Z" to force UTC.
  return Date.parse(ts.replace(' ', 'T') + 'Z')
}

const elapsedSeconds = computed(() => {
  const start = parseTs(props.detail.session.started_at)
  if (props.detail.session.ended_at) {
    const end = parseTs(props.detail.session.ended_at)
    return Math.max(0, Math.floor((end - start) / 1000))
  }
  return Math.max(0, Math.floor((liveNow.value - start) / 1000))
})

const durationDisplay = computed(() => formatDuration(elapsedSeconds.value))

// ETA: only meaningful when crafted_count is known, consumed < crafted,
// and we have >= 3 uses to derive a baseline avg-per-use from.
const etaSub = computed(() => {
  const s = props.detail.session
  if (!props.isActive) return ''
  if (s.crafted_count === null) return ''
  const remaining = s.crafted_count - s.consumed_count
  if (remaining <= 0) return 'finishing up'
  if (props.detail.uses.length < 3) return `${remaining} left`
  const avgSecs = elapsedSeconds.value / Math.max(1, s.consumed_count)
  const etaSecs = Math.round(avgSecs * remaining)
  return `~${formatDuration(etaSecs)} left (${remaining} maps)`
})

const usesDisplay = computed(() => String(props.detail.uses.length))
const consumedSub = computed(() => {
  const s = props.detail.session
  if (s.crafted_count === null) return `${s.consumed_count} consumed`
  return `${s.consumed_count} / ${s.crafted_count} maps`
})

// Profit per hour — only shown once 60s has elapsed so the number isn't
// dominated by the first use's noise.
const profitRateSub = computed(() => {
  if (elapsedSeconds.value < 60) return 'warming up…'
  const perHour = (liveProf.value / elapsedSeconds.value) * 3600
  return `${formatGold(perHour)} / hr`
})
const profitAccent = computed<'positive' | 'negative' | undefined>(() => {
  if (liveProf.value > 0) return 'positive'
  if (liveProf.value < 0) return 'negative'
  return undefined
})

const costSub = computed(() => `cost ${formatGold(liveCost.value)}`)

function kindBadge(kind: SurveyUseKind): string {
  switch (kind) {
    case 'basic': return 'Basic'
    case 'motherlode': return 'Motherlode'
    case 'multihit': return 'Multihit'
  }
}

function statusColor(status: SurveyUseStatus): string {
  switch (status) {
    case 'completed': return 'text-accent-green'
    case 'pending_loot': return 'text-accent-gold'
    case 'aborted': return 'text-accent-red'
    case 'unknown': return 'text-text-dim'
  }
}
</script>
