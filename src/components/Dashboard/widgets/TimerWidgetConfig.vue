<template>
  <div class="flex flex-col gap-3 min-w-56">
    <!-- Current presets -->
    <div class="flex flex-col min-h-0">
      <div class="text-xs font-bold text-text-secondary uppercase tracking-wide mb-1 shrink-0">Presets</div>
      <div v-if="store.presets.length === 0" class="text-xs text-text-dim italic">No presets configured.</div>
      <div v-else class="overflow-y-auto max-h-32">
        <div
          v-for="preset in store.presets"
          :key="preset.id"
          class="flex items-center justify-between gap-2 py-0.5">
          <div class="flex items-center gap-1.5 min-w-0">
            <span class="text-xs text-text-primary truncate">{{ preset.label }}</span>
            <span class="text-[10px] text-text-dim font-mono shrink-0">{{ formatDuration(preset.durationSecs) }}</span>
          </div>
          <button
            class="text-xs text-red-400 hover:text-red-300 cursor-pointer shrink-0"
            @click="store.removePreset(preset.id)">
            Remove
          </button>
        </div>
      </div>
    </div>

    <!-- Add preset -->
    <div class="shrink-0">
      <div class="text-xs font-bold text-text-secondary uppercase tracking-wide mb-1">Add Preset</div>
      <form class="flex flex-col gap-1" @submit.prevent="handleAddPreset">
        <input
          v-model="presetLabel"
          type="text"
          placeholder="Preset name"
          class="w-full px-2 py-1 bg-surface-base border border-border-default rounded text-xs text-text-primary placeholder-text-muted focus:outline-none focus:border-accent-gold/50" />
        <div class="flex gap-1">
          <input
            v-model="presetDuration"
            type="text"
            placeholder="Duration (e.g. 90m, 1:30)"
            class="flex-1 px-2 py-1 bg-surface-base border border-border-default rounded text-xs text-text-primary placeholder-text-muted focus:outline-none focus:border-accent-gold/50 font-mono" />
          <button
            type="submit"
            :disabled="!canAddPreset"
            class="px-2 py-1 bg-accent-gold/20 text-accent-gold border border-accent-gold/40 rounded text-xs hover:bg-accent-gold/30 transition-colors cursor-pointer disabled:opacity-40 disabled:cursor-not-allowed shrink-0">
            Add
          </button>
        </div>
      </form>
    </div>

    <!-- Reset defaults -->
    <button
      class="text-xs text-text-dim hover:text-text-secondary cursor-pointer self-start"
      @click="store.resetPresets()">
      Reset to defaults
    </button>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useTimerStore } from '../../../stores/timerStore'

const store = useTimerStore()

const presetLabel = ref('')
const presetDuration = ref('')

function parseDuration(input: string): number | null {
  const trimmed = input.trim()
  if (!trimmed) return null

  const hmMatch = trimmed.match(/^(\d+):(\d{1,2})$/)
  if (hmMatch) {
    const h = parseInt(hmMatch[1], 10)
    const m = parseInt(hmMatch[2], 10)
    if (m >= 60) return null
    return (h * 60 + m) * 60
  }

  const hmsMatch = trimmed.match(/^(?:(\d+)h)?(?:(\d+)m)?$/i)
  if (hmsMatch && (hmsMatch[1] || hmsMatch[2])) {
    const h = parseInt(hmsMatch[1] || '0', 10)
    const m = parseInt(hmsMatch[2] || '0', 10)
    return (h * 60 + m) * 60
  }

  const num = parseFloat(trimmed)
  if (!isNaN(num) && num > 0) return Math.round(num * 60)

  return null
}

function formatDuration(secs: number): string {
  const h = Math.floor(secs / 3600)
  const m = Math.floor((secs % 3600) / 60)
  if (h > 0 && m > 0) return `${h}h ${m}m`
  if (h > 0) return `${h}h`
  return `${m}m`
}

const parsedPresetDuration = computed(() => parseDuration(presetDuration.value))
const canAddPreset = computed(() => presetLabel.value.trim().length > 0 && parsedPresetDuration.value != null && parsedPresetDuration.value > 0)

function handleAddPreset() {
  if (!canAddPreset.value || parsedPresetDuration.value == null) return
  store.addPreset(presetLabel.value.trim(), parsedPresetDuration.value)
  presetLabel.value = ''
  presetDuration.value = ''
}
</script>
