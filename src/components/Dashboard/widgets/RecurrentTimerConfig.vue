<template>
  <div class="flex flex-col gap-3 min-w-56">
    <div class="text-xs font-bold text-text-secondary uppercase tracking-wide">Recurring Events</div>

    <div v-if="prefs.events.length === 0" class="text-xs text-text-dim italic">
      No events configured.
    </div>

    <div v-else class="overflow-y-auto max-h-48 flex flex-col gap-1.5">
      <div
        v-for="event in prefs.events"
        :key="event.id"
        class="flex flex-col gap-1 p-1.5 rounded border border-border-default/50 bg-surface-base/20">
        <div class="flex items-center justify-between gap-2">
          <span class="text-xs text-text-primary truncate">{{ event.label }}</span>
          <button
            class="text-xs text-red-400 hover:text-red-300 cursor-pointer shrink-0"
            @click="removeEvent(event.id)">
            Remove
          </button>
        </div>
        <div class="text-[10px] text-text-dim">
          {{ formatSchedule(event) }}
        </div>
      </div>
    </div>

    <div class="border-t border-border-default/30 pt-2">
      <div class="text-[10px] text-text-dim">
        Use the Add button on the widget to create new recurring events. Events are sorted by next occurrence.
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useViewPrefs } from '../../../composables/useViewPrefs'

interface RecurrentEvent {
  id: string
  label: string
  recurrence: 'daily' | 'weekly' | 'biweekly' | 'monthly'
  hour: number
  minute: number
  dayOfWeek?: number
  dayOfMonth?: number
  anchorDate?: string
}

interface RecurrentTimerPrefs {
  [key: string]: unknown
  events: RecurrentEvent[]
}

const DEFAULTS: RecurrentTimerPrefs = { events: [] }
const DAY_NAMES_FULL = ['Sunday', 'Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday']

const { prefs, update } = useViewPrefs<RecurrentTimerPrefs>('widget.recurrent-timer', DEFAULTS)

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

function removeEvent(id: string) {
  update({ events: prefs.value.events.filter(e => e.id !== id) })
}
</script>
