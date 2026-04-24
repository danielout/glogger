<template>
  <div class="flex flex-col h-full min-h-0 gap-1.5 text-sm">
    <!-- Add timer row -->
    <form class="flex gap-1.5 shrink-0" @submit.prevent="handleAdd">
      <input
        v-model="newLabel"
        type="text"
        placeholder="Label"
        class="flex-1 min-w-0 px-2 py-1 bg-surface-base border border-border-default rounded text-xs text-text-primary placeholder-text-muted focus:outline-none focus:border-accent-gold/50" />
      <input
        v-model="newDuration"
        type="text"
        placeholder="30m / 1:30"
        title="Duration: minutes (e.g. 90) or hours:minutes (e.g. 1:30)"
        class="w-16 px-2 py-1 bg-surface-base border border-border-default rounded text-xs text-text-primary placeholder-text-muted focus:outline-none focus:border-accent-gold/50 font-mono" />
      <button
        type="submit"
        :disabled="!canAdd"
        class="px-2 py-1 bg-accent-gold/20 text-accent-gold border border-accent-gold/40 rounded text-xs hover:bg-accent-gold/30 transition-colors cursor-pointer disabled:opacity-40 disabled:cursor-not-allowed shrink-0">
        Add
      </button>
    </form>

    <!-- Presets row (compact) -->
    <div v-if="store.presets.length > 0" class="flex gap-1 flex-wrap shrink-0">
      <button
        v-for="preset in store.presets"
        :key="preset.id"
        class="px-1.5 py-0.5 text-[10px] rounded bg-surface-elevated text-text-muted hover:text-text-primary hover:bg-surface-elevated/80 transition-colors cursor-pointer border border-border-default/50"
        :title="`Start ${preset.label} (${formatDurationShort(preset.durationSecs)})`"
        @click="store.addTimer(preset.label, preset.durationSecs, preset.id)">
        {{ preset.label }}
      </button>
    </div>

    <!-- Timer list -->
    <div class="flex-1 overflow-y-auto min-h-0">
      <div v-if="store.timersWithRemaining.length === 0" class="text-xs text-text-dim italic">
        No timers. Add one above or click a preset.
      </div>

      <div class="flex flex-col gap-1">
        <div
          v-for="timer in store.timersWithRemaining"
          :key="timer.id"
          class="flex flex-col gap-0.5 px-2 py-1 rounded border"
          :class="timer.isExpired
            ? 'border-red-500/50 bg-red-500/10 animate-pulse'
            : timer.isPaused
              ? 'border-border-default/50 bg-surface-base/30'
              : 'border-border-default/30 bg-surface-base/20'">
          <!-- Top row: label + time + actions -->
          <div class="flex items-center gap-2">
            <span class="text-xs text-text-primary truncate flex-1 min-w-0">{{ timer.label }}</span>
            <span
              class="text-xs font-mono whitespace-nowrap shrink-0"
              :class="timer.isExpired ? 'text-red-400 font-bold' : timer.isPaused ? 'text-text-dim' : 'text-accent-gold'">
              {{ timer.isExpired ? 'Expired!' : formatRemaining(timer.remaining) }}
            </span>
            <div class="flex gap-0.5 shrink-0">
              <!-- Pause / Resume -->
              <button
                v-if="timer.isRunning && !timer.isExpired"
                class="p-0.5 text-text-dim hover:text-text-secondary transition-colors cursor-pointer"
                title="Pause"
                @click="store.pauseTimer(timer.id)">
                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16" fill="currentColor" class="w-3 h-3">
                  <path d="M4.5 2a.5.5 0 0 0-.5.5v11a.5.5 0 0 0 .5.5h2a.5.5 0 0 0 .5-.5v-11a.5.5 0 0 0-.5-.5h-2Zm5 0a.5.5 0 0 0-.5.5v11a.5.5 0 0 0 .5.5h2a.5.5 0 0 0 .5-.5v-11a.5.5 0 0 0-.5-.5h-2Z" />
                </svg>
              </button>
              <button
                v-if="timer.isPaused"
                class="p-0.5 text-text-dim hover:text-green-400 transition-colors cursor-pointer"
                title="Resume"
                @click="store.resumeTimer(timer.id)">
                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16" fill="currentColor" class="w-3 h-3">
                  <path d="M4 3.5a.5.5 0 0 1 .778-.416l8 5.333a.5.5 0 0 1 0 .833l-8 5.333A.5.5 0 0 1 4 14.167V3.5Z" />
                </svg>
              </button>
              <!-- Restart -->
              <button
                class="p-0.5 text-text-dim hover:text-accent-gold transition-colors cursor-pointer"
                title="Restart"
                @click="store.restartTimer(timer.id)">
                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16" fill="currentColor" class="w-3 h-3">
                  <path fill-rule="evenodd" d="M13.836 2.477a.75.75 0 0 1 .75.75v3.182a.75.75 0 0 1-.75.75h-3.182a.75.75 0 0 1 0-1.5h1.37A5.007 5.007 0 0 0 8 3.5a5 5 0 1 0 4.546 7.07.75.75 0 0 1 1.362.628A6.5 6.5 0 1 1 13.61 3.83V2.727a.75.75 0 0 1 .75-.75h-.524Z" clip-rule="evenodd" />
                </svg>
              </button>
              <!-- Remove -->
              <button
                class="p-0.5 text-text-dim hover:text-red-400 transition-colors cursor-pointer"
                title="Remove"
                @click="store.removeTimer(timer.id)">
                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16" fill="currentColor" class="w-3 h-3">
                  <path d="M5.28 4.22a.75.75 0 0 0-1.06 1.06L6.94 8l-2.72 2.72a.75.75 0 1 0 1.06 1.06L8 9.06l2.72 2.72a.75.75 0 1 0 1.06-1.06L9.06 8l2.72-2.72a.75.75 0 0 0-1.06-1.06L8 6.94 5.28 4.22Z" />
                </svg>
              </button>
            </div>
          </div>
          <!-- Progress bar -->
          <div v-if="!timer.isExpired" class="w-full h-1 bg-surface-base rounded-full overflow-hidden">
            <div
              class="h-full rounded-full transition-all duration-1000"
              :class="timer.isPaused ? 'bg-text-dim/40' : 'bg-accent-gold/60'"
              :style="{ width: progressPercent(timer) + '%' }" />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useTimerStore } from '../../../stores/timerStore'

const store = useTimerStore()

const newLabel = ref('')
const newDuration = ref('')

/** Parse duration string: accepts "90" (minutes), "1:30" (h:m), "90m", "1h30m", "1h" */
function parseDuration(input: string): number | null {
  const trimmed = input.trim()
  if (!trimmed) return null

  // h:m format
  const hmMatch = trimmed.match(/^(\d+):(\d{1,2})$/)
  if (hmMatch) {
    const h = parseInt(hmMatch[1], 10)
    const m = parseInt(hmMatch[2], 10)
    if (m >= 60) return null
    return (h * 60 + m) * 60
  }

  // "1h30m", "1h", "30m" format
  const hmsMatch = trimmed.match(/^(?:(\d+)h)?(?:(\d+)m)?$/i)
  if (hmsMatch && (hmsMatch[1] || hmsMatch[2])) {
    const h = parseInt(hmsMatch[1] || '0', 10)
    const m = parseInt(hmsMatch[2] || '0', 10)
    return (h * 60 + m) * 60
  }

  // Plain number = minutes
  const num = parseFloat(trimmed)
  if (!isNaN(num) && num > 0) {
    return Math.round(num * 60)
  }

  return null
}

const parsedDuration = computed(() => parseDuration(newDuration.value))
const canAdd = computed(() => newLabel.value.trim().length > 0 && parsedDuration.value != null && parsedDuration.value > 0)

function handleAdd() {
  if (!canAdd.value || parsedDuration.value == null) return
  store.addTimer(newLabel.value.trim(), parsedDuration.value)
  newLabel.value = ''
  newDuration.value = ''
}

function formatRemaining(seconds: number): string {
  if (seconds <= 0) return '0:00'
  const h = Math.floor(seconds / 3600)
  const m = Math.floor((seconds % 3600) / 60)
  const s = seconds % 60
  if (h > 0) {
    return `${h}:${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`
  }
  return `${m}:${s.toString().padStart(2, '0')}`
}

function formatDurationShort(secs: number): string {
  const h = Math.floor(secs / 3600)
  const m = Math.floor((secs % 3600) / 60)
  if (h > 0 && m > 0) return `${h}h${m}m`
  if (h > 0) return `${h}h`
  return `${m}m`
}

function progressPercent(timer: { remaining: number; durationSecs: number; isPaused: boolean }): number {
  if (timer.durationSecs <= 0) return 0
  const elapsed = timer.durationSecs - Math.max(0, timer.remaining)
  return Math.min(100, (elapsed / timer.durationSecs) * 100)
}

onMounted(() => {
  store.startTicking()
})

onUnmounted(() => {
  if (!store.hasActiveTimers) {
    store.stopTicking()
  }
})
</script>
