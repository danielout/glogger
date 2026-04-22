<template>
  <div class="flex flex-col gap-3 p-1 min-w-56">
    <div class="text-xs text-text-primary font-medium">Travel Binds</div>

    <div class="text-[0.6rem] text-text-dim">
      Bind pad locations are auto-detected when you open your Teleportation skill info in-game.
      You can also set them manually here.
    </div>

    <!-- Primary Bind -->
    <div>
      <label class="text-[0.6rem] text-text-dim uppercase tracking-wide">Primary Bind</label>
      <select
        v-model="config.primaryBind"
        class="w-full px-2 py-1 rounded bg-surface-elevated border border-border-light text-xs text-text-primary"
        @change="save">
        <option :value="null">None</option>
        <option v-for="z in ALL_LOCATIONS" :key="z.name" :value="z.name">{{ z.name }}</option>
      </select>
    </div>

    <!-- Secondary Bind -->
    <div>
      <label class="text-[0.6rem] text-text-dim uppercase tracking-wide">Secondary Bind</label>
      <select
        v-model="config.secondaryBind"
        class="w-full px-2 py-1 rounded bg-surface-elevated border border-border-light text-xs text-text-primary"
        @change="save">
        <option :value="null">None</option>
        <option v-for="z in ALL_LOCATIONS" :key="z.name" :value="z.name">{{ z.name }}</option>
      </select>
    </div>

    <div class="border-t border-border-light pt-2">
      <div class="text-xs text-text-primary font-medium mb-1">Mushroom Circles</div>
      <div class="text-[0.6rem] text-text-dim mb-2">
        Circle attunements can't be auto-detected. Set them manually.
      </div>

      <div>
        <label class="text-[0.6rem] text-text-dim uppercase tracking-wide">Mushroom Circle</label>
        <select
          v-model="config.mushroomCircle"
          class="w-full px-2 py-1 rounded bg-surface-elevated border border-border-light text-xs text-text-primary"
          @change="save">
          <option :value="null">None</option>
          <option v-for="z in CIRCLE_LOCATIONS" :key="z.name" :value="z.name">{{ z.name }}</option>
        </select>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { reactive } from 'vue'

const CONFIG_KEY = 'tripPlannerWidget.config'

interface TripPlannerConfig {
  primaryBind: string | null
  secondaryBind: string | null
  mushroomCircle: string | null
}

const defaultConfig: TripPlannerConfig = {
  primaryBind: null,
  secondaryBind: null,
  mushroomCircle: null,
}

// Bind pads can be anywhere — overworld zones and some sub-zones
const ALL_LOCATIONS = [
  { name: 'Anagoge Island' },
  { name: 'Serbule' },
  { name: 'Serbule Hills' },
  { name: 'Eltibule' },
  { name: 'Sun Vale' },
  { name: 'Kur Mountains' },
  { name: 'Red Wing Casino' },
  { name: 'Ilmari' },
  { name: 'Rahu' },
  { name: 'Gazluk' },
  { name: 'Fae Realm' },
  { name: 'Povus' },
  { name: 'Vidaria' },
  { name: 'Statehelm' },
  { name: 'Winter Nexus' },
  { name: 'Caves Beneath Gazluk' },
  { name: 'Caves Under Serbule' },
  { name: 'Rahu Sewers' },
  { name: 'Statehelm Undercity' },
]

// Mushroom circles only exist in certain overworld zones
const CIRCLE_LOCATIONS = [
  { name: 'Serbule' },
  { name: 'Serbule Hills' },
  { name: 'Eltibule' },
  { name: 'Sun Vale' },
  { name: 'Kur Mountains' },
  { name: 'Ilmari' },
  { name: 'Rahu' },
  { name: 'Gazluk' },
  { name: 'Fae Realm' },
  { name: 'Povus' },
  { name: 'Vidaria' },
  { name: 'Statehelm' },
]

function loadConfig(): TripPlannerConfig {
  try {
    const raw = localStorage.getItem(CONFIG_KEY)
    if (raw) return { ...defaultConfig, ...JSON.parse(raw) }
  } catch { /* ignore */ }
  return { ...defaultConfig }
}

const config = reactive<TripPlannerConfig>(loadConfig())

function save() {
  localStorage.setItem(CONFIG_KEY, JSON.stringify(config))
  window.dispatchEvent(new Event('tripPlannerConfigChanged'))
}
</script>
