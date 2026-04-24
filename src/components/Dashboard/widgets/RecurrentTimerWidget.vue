<template>
  <div class="flex flex-col h-full min-h-0 gap-1.5 text-sm">
    <!-- Quick-add row -->
    <form class="flex gap-1.5 shrink-0" @submit.prevent="handleAdd">
      <input
        v-model="newLabel"
        type="text"
        placeholder="Event label"
        class="flex-1 min-w-0 px-2 py-1 bg-surface-base border border-border-default rounded text-xs text-text-primary placeholder-text-muted focus:outline-none focus:border-accent-gold/50" />
      <button
        type="button"
        class="px-2 py-1 bg-accent-gold/20 text-accent-gold border border-accent-gold/40 rounded text-xs hover:bg-accent-gold/30 transition-colors cursor-pointer shrink-0"
        @click="showForm = !showForm">
        {{ showForm ? 'Cancel' : 'Add' }}
      </button>
    </form>

    <!-- Add/edit form (expandable) -->
    <div v-if="showForm" class="flex flex-col gap-1.5 shrink-0 p-2 rounded border border-border-default/50 bg-surface-base/30">
      <div class="flex gap-1.5">
        <select
          v-model="newRecurrence"
          class="flex-1 px-2 py-1 bg-surface-base border border-border-default rounded text-xs text-text-primary focus:outline-none focus:border-accent-gold/50">
          <option value="daily">Daily</option>
          <option value="weekly">Weekly</option>
          <option value="biweekly">Biweekly</option>
          <option value="monthly">Monthly</option>
        </select>

        <!-- Day of week (for weekly/biweekly) -->
        <select
          v-if="newRecurrence === 'weekly' || newRecurrence === 'biweekly'"
          v-model="newDayOfWeek"
          class="px-2 py-1 bg-surface-base border border-border-default rounded text-xs text-text-primary focus:outline-none focus:border-accent-gold/50">
          <option v-for="(name, idx) in DAY_NAMES" :key="idx" :value="idx">{{ name }}</option>
        </select>

        <!-- Day of month (for monthly) -->
        <select
          v-if="newRecurrence === 'monthly'"
          v-model="newDayOfMonth"
          class="w-14 px-2 py-1 bg-surface-base border border-border-default rounded text-xs text-text-primary focus:outline-none focus:border-accent-gold/50">
          <option v-for="d in 31" :key="d" :value="d">{{ d }}</option>
        </select>
      </div>

      <div class="flex gap-1.5 items-center">
        <input
          v-model="newTime"
          type="time"
          class="flex-1 px-2 py-1 bg-surface-base border border-border-default rounded text-xs text-text-primary focus:outline-none focus:border-accent-gold/50" />

        <!-- Biweekly anchor date -->
        <input
          v-if="newRecurrence === 'biweekly'"
          v-model="newAnchorDate"
          type="date"
          title="Anchor date (a date this event occurs on)"
          class="flex-1 px-2 py-1 bg-surface-base border border-border-default rounded text-xs text-text-primary focus:outline-none focus:border-accent-gold/50" />

        <button
          type="button"
          :disabled="!canAdd"
          class="px-2 py-1 bg-accent-gold/20 text-accent-gold border border-accent-gold/40 rounded text-xs hover:bg-accent-gold/30 transition-colors cursor-pointer disabled:opacity-40 disabled:cursor-not-allowed shrink-0"
          @click="handleAdd">
          Save
        </button>
      </div>
    </div>

    <!-- Event list -->
    <div class="flex-1 overflow-y-auto min-h-0">
      <div v-if="sortedEvents.length === 0" class="text-xs text-text-dim italic">
        No recurring events. Click Add to create one.
      </div>

      <div class="flex flex-col gap-1">
        <div
          v-for="event in sortedEvents"
          :key="event.id"
          class="flex flex-col gap-0.5 px-2 py-1 rounded border"
          :class="event.isImminent
            ? 'border-accent-gold/50 bg-accent-gold/10'
            : 'border-border-default/30 bg-surface-base/20'">
          <div class="flex items-center gap-2">
            <span class="text-xs text-text-primary truncate flex-1 min-w-0">{{ event.label }}</span>
            <span
              class="text-xs font-mono whitespace-nowrap shrink-0"
              :class="event.isImminent ? 'text-accent-gold font-bold' : 'text-text-secondary'">
              {{ event.countdownText }}
            </span>
            <button
              class="p-0.5 text-text-dim hover:text-red-400 transition-colors cursor-pointer shrink-0"
              title="Remove"
              @click="removeEvent(event.id)">
              <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16" fill="currentColor" class="w-3 h-3">
                <path d="M5.28 4.22a.75.75 0 0 0-1.06 1.06L6.94 8l-2.72 2.72a.75.75 0 1 0 1.06 1.06L8 9.06l2.72 2.72a.75.75 0 1 0 1.06-1.06L9.06 8l2.72-2.72a.75.75 0 0 0-1.06-1.06L8 6.94 5.28 4.22Z" />
              </svg>
            </button>
          </div>
          <div class="text-[10px] text-text-dim">
            {{ event.scheduleText }}
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useViewPrefs } from '../../../composables/useViewPrefs'

// --- Types ---

type RecurrenceType = 'daily' | 'weekly' | 'biweekly' | 'monthly'

interface RecurrentEvent {
  id: string
  label: string
  recurrence: RecurrenceType
  /** Hour (0-23) in local time */
  hour: number
  /** Minute (0-59) in local time */
  minute: number
  /** Day of week (0=Sun..6=Sat) — used for weekly/biweekly */
  dayOfWeek?: number
  /** Day of month (1-31) — used for monthly */
  dayOfMonth?: number
  /** ISO date string anchor for biweekly calculation */
  anchorDate?: string
}

interface RecurrentTimerPrefs {
  [key: string]: unknown
  events: RecurrentEvent[]
}

const DEFAULTS: RecurrentTimerPrefs = { events: [] }

const DAY_NAMES = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat']
const DAY_NAMES_FULL = ['Sunday', 'Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday']

// --- State ---

const { prefs, update } = useViewPrefs<RecurrentTimerPrefs>('widget.recurrent-timer', DEFAULTS)

const now = ref(Date.now())
let tickInterval: ReturnType<typeof setInterval> | null = null

// Form state
const showForm = ref(false)
const newLabel = ref('')
const newRecurrence = ref<RecurrenceType>('weekly')
const newDayOfWeek = ref(3) // Wednesday
const newDayOfMonth = ref(1)
const newTime = ref('10:00')
const newAnchorDate = ref(todayISO())

// --- Helpers ---

function generateId(): string {
  return Date.now().toString(36) + Math.random().toString(36).slice(2, 6)
}

function todayISO(): string {
  const d = new Date()
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`
}

function parseTime(t: string): { hour: number; minute: number } {
  const [h, m] = t.split(':').map(Number)
  return { hour: h || 0, minute: m || 0 }
}

/**
 * Compute the next occurrence of a recurrent event from the current time.
 */
function getNextOccurrence(event: RecurrentEvent, fromMs: number): Date {
  const from = new Date(fromMs)

  if (event.recurrence === 'daily') {
    // Today at the specified time, or tomorrow if already past
    const candidate = new Date(from)
    candidate.setHours(event.hour, event.minute, 0, 0)
    if (candidate.getTime() <= fromMs) {
      candidate.setDate(candidate.getDate() + 1)
    }
    return candidate
  }

  if (event.recurrence === 'weekly') {
    const dow = event.dayOfWeek ?? 0
    const candidate = new Date(from)
    candidate.setHours(event.hour, event.minute, 0, 0)
    const currentDow = candidate.getDay()
    let daysAhead = (dow - currentDow + 7) % 7
    if (daysAhead === 0 && candidate.getTime() <= fromMs) {
      daysAhead = 7
    }
    candidate.setDate(candidate.getDate() + daysAhead)
    return candidate
  }

  if (event.recurrence === 'biweekly') {
    const dow = event.dayOfWeek ?? 0
    // Use anchor date to determine which weeks are "on" weeks
    const anchor = event.anchorDate ? new Date(event.anchorDate + 'T00:00:00') : new Date(from)
    anchor.setHours(0, 0, 0, 0)

    // Find the next occurrence of the correct day of week
    const candidate = new Date(from)
    candidate.setHours(event.hour, event.minute, 0, 0)
    const currentDow = candidate.getDay()
    let daysAhead = (dow - currentDow + 7) % 7
    if (daysAhead === 0 && candidate.getTime() <= fromMs) {
      daysAhead = 7
    }
    candidate.setDate(candidate.getDate() + daysAhead)

    // Check if this candidate is on an "on" week relative to the anchor
    const msPerWeek = 7 * 24 * 60 * 60 * 1000
    const weeksDiff = Math.round((candidate.getTime() - anchor.getTime()) / msPerWeek)
    // "on" weeks are when weeksDiff is even
    if (weeksDiff % 2 !== 0) {
      candidate.setDate(candidate.getDate() + 7)
    }
    return candidate
  }

  if (event.recurrence === 'monthly') {
    const dom = event.dayOfMonth ?? 1
    const candidate = new Date(from.getFullYear(), from.getMonth(), dom, event.hour, event.minute, 0, 0)
    if (candidate.getTime() <= fromMs) {
      candidate.setMonth(candidate.getMonth() + 1)
    }
    // Handle months that don't have enough days (e.g., day 31 in Feb)
    // The Date constructor handles this by rolling over, but we clamp instead
    if (candidate.getDate() !== dom) {
      // Rolled over — go to next month's dom
      candidate.setDate(1)
      candidate.setMonth(candidate.getMonth() + 1)
      candidate.setDate(Math.min(dom, new Date(candidate.getFullYear(), candidate.getMonth() + 1, 0).getDate()))
    }
    return candidate
  }

  // Fallback
  return new Date(fromMs)
}

function formatCountdown(ms: number): string {
  if (ms <= 0) return 'now!'
  const totalSecs = Math.floor(ms / 1000)
  const days = Math.floor(totalSecs / 86400)
  const hours = Math.floor((totalSecs % 86400) / 3600)
  const mins = Math.floor((totalSecs % 3600) / 60)
  const secs = totalSecs % 60

  if (days > 0) {
    return `${days}d ${hours}h ${mins}m`
  }
  if (hours > 0) {
    return `${hours}:${String(mins).padStart(2, '0')}:${String(secs).padStart(2, '0')}`
  }
  return `${mins}:${String(secs).padStart(2, '0')}`
}

function formatSchedule(event: RecurrentEvent): string {
  const timeStr = `${String(event.hour).padStart(2, '0')}:${String(event.minute).padStart(2, '0')}`
  switch (event.recurrence) {
    case 'daily':
      return `Daily at ${timeStr}`
    case 'weekly':
      return `Every ${DAY_NAMES_FULL[event.dayOfWeek ?? 0]} at ${timeStr}`
    case 'biweekly':
      return `Every other ${DAY_NAMES_FULL[event.dayOfWeek ?? 0]} at ${timeStr}`
    case 'monthly':
      return `${ordinal(event.dayOfMonth ?? 1)} of each month at ${timeStr}`
  }
}

function ordinal(n: number): string {
  const s = ['th', 'st', 'nd', 'rd']
  const v = n % 100
  return n + (s[(v - 20) % 10] || s[v] || s[0])
}

// --- Computed ---

const canAdd = computed(() => {
  return newLabel.value.trim().length > 0 && newTime.value.length > 0
})

const sortedEvents = computed(() => {
  const currentMs = now.value
  return prefs.value.events
    .map(event => {
      const next = getNextOccurrence(event, currentMs)
      const msUntil = next.getTime() - currentMs
      return {
        ...event,
        nextOccurrence: next,
        msUntil,
        countdownText: formatCountdown(msUntil),
        scheduleText: formatSchedule(event),
        isImminent: msUntil < 3600_000, // less than 1 hour
      }
    })
    .sort((a, b) => a.msUntil - b.msUntil)
})

// --- Actions ---

function handleAdd() {
  if (!canAdd.value) return
  const { hour, minute } = parseTime(newTime.value)
  const event: RecurrentEvent = {
    id: generateId(),
    label: newLabel.value.trim(),
    recurrence: newRecurrence.value,
    hour,
    minute,
  }
  if (newRecurrence.value === 'weekly' || newRecurrence.value === 'biweekly') {
    event.dayOfWeek = newDayOfWeek.value
  }
  if (newRecurrence.value === 'biweekly') {
    event.anchorDate = newAnchorDate.value
  }
  if (newRecurrence.value === 'monthly') {
    event.dayOfMonth = newDayOfMonth.value
  }

  update({ events: [...prefs.value.events, event] })
  newLabel.value = ''
  showForm.value = false
}

function removeEvent(id: string) {
  update({ events: prefs.value.events.filter(e => e.id !== id) })
}

// --- Lifecycle ---

onMounted(() => {
  tickInterval = setInterval(() => {
    now.value = Date.now()
  }, 1000)
})

onUnmounted(() => {
  if (tickInterval) {
    clearInterval(tickInterval)
    tickInterval = null
  }
})
</script>
