<template>
  <div class="flex flex-col gap-2">
    <div class="text-xs font-bold text-text-secondary uppercase tracking-wide mb-1">Show Sections</div>
    <label v-for="opt in options" :key="opt.key" class="flex items-center gap-2 cursor-pointer">
      <input
        type="checkbox"
        :checked="prefs[opt.key]"
        :disabled="opt.parent && !prefs[opt.parent]"
        class="accent-accent-gold"
        @change="update({ [opt.key]: ($event.target as HTMLInputElement).checked })">
      <span :class="{ 'text-text-dim': opt.parent && !prefs[opt.parent] }">{{ opt.label }}</span>
    </label>
  </div>
</template>

<script setup lang="ts">
import { useViewPrefs } from '../../composables/useViewPrefs'

type PrefKey = 'showTime' | 'showLocalTime' | 'showMoon' | 'showWeather' | 'showCombat' | 'showCurrencies'

const { prefs, update } = useViewPrefs('widget.context-bar', {
  showTime: true,
  showLocalTime: false,
  showMoon: true,
  showWeather: true,
  showCombat: true,
  showCurrencies: true,
})

const options: { key: PrefKey; label: string; parent?: PrefKey }[] = [
  { key: 'showTime', label: 'Server / Game Time' },
  { key: 'showLocalTime', label: 'Local Time', parent: 'showTime' },
  { key: 'showMoon', label: 'Moon Phase' },
  { key: 'showWeather', label: 'Weather' },
  { key: 'showCombat', label: 'Combat / Mount' },
  { key: 'showCurrencies', label: 'Currencies' },
]
</script>
