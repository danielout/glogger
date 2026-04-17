<template>
  <button
    class="w-full text-left rounded border px-2.5 py-2 transition-colors flex flex-col gap-1"
    :class="cardClass"
    @click="$emit('select')"
  >
    <!-- Top row: name + badge -->
    <div class="flex items-center justify-between gap-2">
      <span class="text-xs text-text-primary font-semibold truncate">
        {{ displayName }}
      </span>
      <span
        v-if="isActive"
        class="text-[0.6rem] px-1.5 py-0.5 rounded bg-accent-green/20 text-accent-green uppercase tracking-wider font-semibold shrink-0"
      >
        Active
      </span>
      <span v-else class="text-[0.6rem] text-text-dim shrink-0">
        ended
      </span>
    </div>

    <!-- Zones -->
    <div v-if="row.zones.length > 0" class="flex flex-wrap gap-1">
      <span
        v-for="zone in row.zones"
        :key="zone"
        class="text-[0.6rem] text-text-secondary"
      >
        <AreaInline :reference="zone" />
      </span>
    </div>

    <!-- Stats row: surveys, profit/hr, date, duration -->
    <div class="flex items-center gap-3 text-[0.65rem] text-text-secondary tabular-nums">
      <span>
        <span class="text-text-primary font-semibold">{{ row.total_uses }}</span>
        surveys
      </span>
      <span
        v-if="row.total_uses > 0"
        class="font-semibold"
        :class="profitValue >= 0 ? 'text-accent-green' : 'text-accent-red'"
      >
        {{ formatGold(profitPerHour) }}/hr
      </span>
    </div>

    <!-- Date + duration -->
    <div class="text-[0.6rem] text-text-dim tabular-nums">
      {{ formatTimeFull(row.session.started_at) }}
      <span v-if="row.duration_seconds !== null">
        · {{ formatDuration(row.duration_seconds) }}
      </span>
    </div>
  </button>
</template>

<script setup lang="ts">
// Compact session card for the unified left panel. Shows key metrics
// at a glance; clicking selects the session for detail view in the
// center + right panels.
import { computed } from 'vue'
import type { HistoricalSessionRow } from '../../stores/surveyTrackerStore'
import { formatTimeFull, formatDuration } from '../../composables/useTimestamp'
import { formatGold } from '../../composables/useRecipeCost'
import { liveEnrichedRows, liveRevenue } from '../../composables/useLiveValuation'
import AreaInline from '../Shared/Area/AreaInline.vue'

const props = defineProps<{
  row: HistoricalSessionRow
  isActive: boolean
  isSelected: boolean
}>()

defineEmits<{
  select: []
}>()

const displayName = computed(() =>
  props.row.session.name ?? `Session #${props.row.session.id}`,
)

const cardClass = computed(() => {
  if (props.isSelected) return 'border-accent-gold/60 bg-accent-gold/10'
  if (props.isActive) return 'border-accent-green/40 bg-accent-green/5 hover:bg-accent-green/10'
  return 'border-border-default bg-surface-card hover:bg-surface-elevated'
})

// Reactive profit via live market data.
const enrichedRows = computed(() => liveEnrichedRows(props.row.loot_summary))
const revenue = computed(() => liveRevenue(enrichedRows.value))
const profitValue = computed(() => revenue.value - props.row.economics.cost_total)
const profitPerHour = computed(() => {
  const secs = props.row.duration_seconds
  if (!secs || secs < 60) return 0
  return (profitValue.value / secs) * 3600
})
</script>
