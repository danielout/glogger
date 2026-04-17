<template>
  <section class="bg-surface-card border border-border-default rounded p-3 flex flex-col gap-2 h-full overflow-hidden">
    <header class="shrink-0">
      <h3 class="text-[0.65rem] uppercase tracking-widest text-text-secondary font-semibold">
        Time
      </h3>
    </header>

    <div class="flex flex-col gap-1.5 text-xs tabular-nums">
      <!-- Start / End -->
      <div class="flex justify-between">
        <span class="text-text-secondary">Start</span>
        <span class="text-text-primary">{{ effectiveStart ? formatTimeFull(effectiveStart) : '—' }}</span>
      </div>
      <div class="flex justify-between">
        <span class="text-text-secondary">End</span>
        <span class="text-text-primary">
          {{ effectiveEnd ? formatTimeFull(effectiveEnd) : (isActive ? 'in progress' : '—') }}
        </span>
      </div>

      <!-- Total duration -->
      <div class="flex justify-between border-t border-border-default pt-1.5">
        <span class="text-text-secondary">Total</span>
        <span class="text-text-primary font-semibold">{{ totalDurationDisplay }}</span>
      </div>

      <!-- Craft duration -->
      <div class="flex justify-between">
        <span class="text-text-dim">Crafting</span>
        <span class="text-text-muted">{{ craftDurationDisplay }}</span>
      </div>

      <!-- Prep time -->
      <div class="flex justify-between">
        <span class="text-text-dim">Prep time</span>
        <span class="text-text-muted">{{ prepTimeDisplay }}</span>
      </div>

      <!-- Survey completion -->
      <div class="flex justify-between">
        <span class="text-text-dim">Surveying</span>
        <span class="text-text-muted">{{ surveyDurationDisplay }}</span>
      </div>

      <!-- Averages -->
      <div v-if="session.consumed_count > 0" class="flex flex-col gap-1 border-t border-border-default pt-1.5">
        <div class="flex justify-between">
          <span class="text-text-secondary">Avg / survey (total)</span>
          <span class="text-text-primary">{{ avgTotalDisplay }}</span>
        </div>
        <div class="flex justify-between">
          <span class="text-text-secondary">Avg / survey (completion)</span>
          <span class="text-text-primary">{{ avgCompletionDisplay }}</span>
        </div>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
// Time breakdown panel for the session center view. Shows start/end
// (using user-adjusted times when set), total duration, craft time,
// prep time, survey completion time, and per-survey averages.
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import type { SurveySession } from '../../stores/surveyTrackerStore'
import { formatTimeFull, formatDuration } from '../../composables/useTimestamp'

const props = defineProps<{
  session: SurveySession
  isActive: boolean
}>()

// Live tick for active sessions.
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
  return Date.parse(ts.replace(' ', 'T') + 'Z')
}

function tsDiffSeconds(a: string | null, b: string | null): number | null {
  if (!a || !b) return null
  const diff = (parseTs(b) - parseTs(a)) / 1000
  return diff >= 0 ? diff : null
}

const effectiveStart = computed(() =>
  props.session.user_started_at ?? props.session.started_at,
)
const effectiveEnd = computed(() =>
  props.session.user_ended_at ?? props.session.ended_at,
)

const totalSeconds = computed(() => {
  const start = effectiveStart.value
  if (!start) return null
  const end = effectiveEnd.value
  if (end) return tsDiffSeconds(start, end)
  if (props.isActive) {
    return Math.max(0, Math.floor((liveNow.value - parseTs(start)) / 1000))
  }
  return null
})

const totalDurationDisplay = computed(() => {
  if (totalSeconds.value === null) return '—'
  return formatDuration(totalSeconds.value)
})

// Craft duration: last_craft_at - first_craft_at
const craftDurationDisplay = computed(() => {
  const secs = tsDiffSeconds(props.session.first_craft_at, props.session.last_craft_at)
  if (secs === null) return 'N/A'
  return formatDuration(secs)
})

// Prep time: first_loot_at - last_craft_at
const prepTimeDisplay = computed(() => {
  const secs = tsDiffSeconds(props.session.last_craft_at, props.session.first_loot_at)
  if (secs === null) return 'N/A'
  return formatDuration(secs)
})

// Survey completion: last_loot_at - first_loot_at
const surveySeconds = computed(() =>
  tsDiffSeconds(props.session.first_loot_at, props.session.last_loot_at),
)
const surveyDurationDisplay = computed(() => {
  if (surveySeconds.value === null) return 'N/A'
  return formatDuration(surveySeconds.value)
})

// Averages
const avgTotalDisplay = computed(() => {
  if (totalSeconds.value === null || props.session.consumed_count === 0) return '—'
  return formatDuration(Math.round(totalSeconds.value / props.session.consumed_count))
})

const avgCompletionDisplay = computed(() => {
  if (surveySeconds.value === null || props.session.consumed_count === 0) return '—'
  return formatDuration(Math.round(surveySeconds.value / props.session.consumed_count))
})
</script>
