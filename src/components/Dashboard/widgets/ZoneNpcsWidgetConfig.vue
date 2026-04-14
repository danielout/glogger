<template>
  <div class="flex flex-col gap-3">
    <!-- NPC Type Filters -->
    <div>
      <div class="text-xs font-bold text-text-secondary uppercase tracking-wide mb-1">NPC Types</div>
      <label class="flex items-center gap-2 cursor-pointer text-sm">
        <input type="checkbox" v-model="config.showStorage" class="accent-accent-gold" @change="save" />
        <span>Storage NPCs</span>
      </label>
      <label class="flex items-center gap-2 cursor-pointer text-sm">
        <input type="checkbox" v-model="config.showShops" class="accent-accent-gold" @change="save" />
        <span>Shop NPCs</span>
      </label>
      <label class="flex items-center gap-2 cursor-pointer text-sm">
        <input type="checkbox" v-model="config.showTrainers" class="accent-accent-gold" @change="save" />
        <span>Trainers</span>
      </label>
    </div>

    <!-- Favor Rank Filters -->
    <div>
      <div class="text-xs font-bold text-text-secondary uppercase tracking-wide mb-1">Favor Range</div>
      <div class="flex flex-col gap-1.5">
        <label class="flex items-center gap-2 text-xs text-text-secondary">
          <span class="w-8">Min:</span>
          <select
            v-model="config.minFavorRank"
            class="flex-1 px-2 py-1 bg-surface-base border border-border-default rounded text-xs text-text-primary cursor-pointer"
            @change="save"
          >
            <option value="">Any</option>
            <option v-for="tier in FAVOR_TIERS" :key="tier" :value="tier">
              {{ tierDisplayName(tier) }}
            </option>
          </select>
        </label>
        <label class="flex items-center gap-2 text-xs text-text-secondary">
          <span class="w-8">Max:</span>
          <select
            v-model="config.maxFavorRank"
            class="flex-1 px-2 py-1 bg-surface-base border border-border-default rounded text-xs text-text-primary cursor-pointer"
            @change="save"
          >
            <option value="">Any</option>
            <option v-for="tier in FAVOR_TIERS" :key="tier" :value="tier">
              {{ tierDisplayName(tier) }}
            </option>
          </select>
        </label>
      </div>
    </div>

    <!-- Sort -->
    <div>
      <div class="text-xs font-bold text-text-secondary uppercase tracking-wide mb-1">Sort By</div>
      <select
        v-model="config.sortBy"
        class="w-full px-2 py-1 bg-surface-base border border-border-default rounded text-xs text-text-primary cursor-pointer"
        @change="save"
      >
        <option value="name">Name</option>
        <option value="storage">Storage Remaining</option>
        <option value="gold">Gold Remaining</option>
        <option value="favor">Favor Level</option>
      </select>
    </div>

    <!-- Other Options -->
    <div>
      <div class="text-xs font-bold text-text-secondary uppercase tracking-wide mb-1">Options</div>
      <label class="flex items-center gap-2 cursor-pointer text-sm">
        <input type="checkbox" v-model="config.showSkills" class="accent-accent-gold" @change="save" />
        <span>Show trained skills</span>
      </label>
      <label class="flex items-center gap-2 cursor-pointer text-sm mt-1">
        <input type="checkbox" v-model="config.showGiftableOnly" class="accent-accent-gold" @change="save" />
        <span>Show giftable only</span>
      </label>
      <label class="flex items-center gap-2 cursor-pointer text-sm mt-1">
        <input type="checkbox" v-model="config.pushEmptyToBottom" class="accent-accent-gold" @change="save" />
        <span>Push NPCs without storage/shops to bottom</span>
      </label>
    </div>
  </div>
</template>

<script setup lang="ts">
import { reactive } from 'vue'
import { FAVOR_TIERS, tierDisplayName } from '../../../composables/useFavorTiers'

const CONFIG_KEY = 'zoneNpcsWidget.config'

interface WidgetConfig {
  showStorage: boolean
  showShops: boolean
  showTrainers: boolean
  minFavorRank: string
  maxFavorRank: string
  showGiftableOnly: boolean
  pushEmptyToBottom: boolean
  showSkills: boolean
  sortBy: 'name' | 'storage' | 'gold' | 'favor'
}

const defaultConfig: WidgetConfig = {
  showStorage: true,
  showShops: true,
  showTrainers: true,
  minFavorRank: '',
  maxFavorRank: '',
  showGiftableOnly: false,
  pushEmptyToBottom: false,
  showSkills: true,
  sortBy: 'name',
}

function loadConfig(): WidgetConfig {
  try {
    const raw = localStorage.getItem(CONFIG_KEY)
    if (raw) return { ...defaultConfig, ...JSON.parse(raw) }
  } catch { /* ignore */ }
  return { ...defaultConfig }
}

const config = reactive<WidgetConfig>(loadConfig())

function save() {
  localStorage.setItem(CONFIG_KEY, JSON.stringify(config))
  // Notify the widget in the same tab
  window.dispatchEvent(new Event('zoneNpcsConfigChanged'))
}
</script>
