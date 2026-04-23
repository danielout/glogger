<template>
  <section class="bg-surface-card border border-border-default rounded p-3 flex flex-col gap-2 h-full overflow-hidden">
    <header class="shrink-0">
      <h3 class="text-[10px] uppercase tracking-widest text-text-secondary font-semibold">
        Time
      </h3>
    </header>

    <div class="flex flex-col gap-1.5 text-xs tabular-nums">
      <!-- Start -->
      <div class="flex justify-between items-center">
        <span class="text-text-secondary">Start</span>
        <div class="flex items-center gap-1">
          <template v-if="editingField === 'start'">
            <input
              ref="startInput"
              class="bg-surface-elevated border border-accent-gold/50 rounded px-1.5 py-0.5 text-xs text-text-primary w-[130px] text-right focus:outline-none"
              :value="editValue"
              @input="editValue = ($event.target as HTMLInputElement).value"
              @blur="commitEdit('start')"
              @keydown.enter="commitEdit('start')"
              @keydown.escape="cancelEdit"
              placeholder="YYYY-MM-DD HH:MM:SS"
            />
          </template>
          <template v-else>
            <span class="text-text-primary">{{ effectiveStart ? formatStartEnd(effectiveStart) : '—' }}</span>
            <button
              v-if="!isActive"
              class="text-text-dim hover:text-accent-gold text-[10px] px-0.5"
              title="Edit start time"
              @click="startEditing('start')"
            >
              &#9998;
            </button>
          </template>
          <button
            v-if="!editingField && session.user_started_at && !isActive"
            class="text-text-dim hover:text-accent-red text-[10px] px-0.5"
            title="Reset to auto-detected start"
            @click="resetTime('start')"
          >
            &#10005;
          </button>
        </div>
      </div>

      <!-- End -->
      <div class="flex justify-between items-center">
        <span class="text-text-secondary">End</span>
        <div class="flex items-center gap-1">
          <template v-if="editingField === 'end'">
            <input
              ref="endInput"
              class="bg-surface-elevated border border-accent-gold/50 rounded px-1.5 py-0.5 text-xs text-text-primary w-[130px] text-right focus:outline-none"
              :value="editValue"
              @input="editValue = ($event.target as HTMLInputElement).value"
              @blur="commitEdit('end')"
              @keydown.enter="commitEdit('end')"
              @keydown.escape="cancelEdit"
              placeholder="YYYY-MM-DD HH:MM:SS"
            />
          </template>
          <template v-else>
            <span class="text-text-primary">
              {{ effectiveEnd ? formatStartEnd(effectiveEnd) : (isActive ? 'in progress' : '—') }}
            </span>
            <button
              v-if="!isActive"
              class="text-text-dim hover:text-accent-gold text-[10px] px-0.5"
              title="Edit end time"
              @click="startEditing('end')"
            >
              &#9998;
            </button>
          </template>
          <button
            v-if="!editingField && session.user_ended_at && !isActive"
            class="text-text-dim hover:text-accent-red text-[10px] px-0.5"
            title="Reset to auto-detected end"
            @click="resetTime('end')"
          >
            &#10005;
          </button>
        </div>
      </div>

      <!-- User-override hint -->
      <div
        v-if="(session.user_started_at || session.user_ended_at) && !editingField"
        class="text-[10px] text-text-dim italic"
      >
        Manually adjusted
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
// Start/end times are editable for ended sessions via inline inputs.
import { computed, nextTick, onBeforeUnmount, onMounted, ref } from 'vue'
import type { SurveySession } from '../../stores/surveyTrackerStore'
import { useSurveyTrackerStore } from '../../stores/surveyTrackerStore'
import { formatTimeFull, formatDuration, formatDateTimeShort } from '../../composables/useTimestamp'

const props = defineProps<{
  session: SurveySession
  isActive: boolean
}>()

const store = useSurveyTrackerStore()

// ── Inline editing ────────────────────────────────────────────────────
const editingField = ref<'start' | 'end' | null>(null)
const editValue = ref('')
const startInput = ref<HTMLInputElement | null>(null)
const endInput = ref<HTMLInputElement | null>(null)

function startEditing(field: 'start' | 'end') {
  editingField.value = field
  if (field === 'start') {
    editValue.value = props.session.user_started_at ?? props.session.started_at
  } else {
    editValue.value = props.session.user_ended_at ?? props.session.ended_at ?? ''
  }
  nextTick(() => {
    const input = field === 'start' ? startInput.value : endInput.value
    input?.focus()
    input?.select()
  })
}

function cancelEdit() {
  editingField.value = null
  editValue.value = ''
}

function commitEdit(field: 'start' | 'end') {
  const val = editValue.value.trim()
  if (!val || !isValidTimestamp(val)) {
    cancelEdit()
    return
  }
  const newStart = field === 'start' ? val : (props.session.user_started_at ?? null)
  const newEnd = field === 'end' ? val : (props.session.user_ended_at ?? null)
  store.updateSessionTimes(props.session.id, newStart, newEnd)
  cancelEdit()
}

function resetTime(field: 'start' | 'end') {
  const newStart = field === 'start' ? null : (props.session.user_started_at ?? null)
  const newEnd = field === 'end' ? null : (props.session.user_ended_at ?? null)
  store.updateSessionTimes(props.session.id, newStart, newEnd)
}

function isValidTimestamp(ts: string): boolean {
  // Accept YYYY-MM-DD HH:MM:SS format
  return /^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}$/.test(ts)
    && !isNaN(Date.parse(ts.replace(' ', 'T') + 'Z'))
}

// ── Live tick for active sessions ─────────────────────────────────────
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

// Show date+time when start and end are on different calendar days.
const spansMultipleDays = computed(() => {
  const s = effectiveStart.value
  const e = effectiveEnd.value
  if (!s || !e) return false
  return s.substring(0, 10) !== e.substring(0, 10)
})

function formatStartEnd(ts: string): string {
  return spansMultipleDays.value ? formatDateTimeShort(ts) : formatTimeFull(ts)
}

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
