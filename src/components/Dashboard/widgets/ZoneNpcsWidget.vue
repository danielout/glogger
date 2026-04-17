<template>
  <div class="flex flex-col gap-1">
    <!-- Loading -->
    <div v-if="!areaName" class="text-text-dim text-sm italic">No zone data yet.</div>
    <div v-else-if="loading" class="text-text-dim text-xs italic">Loading NPCs...</div>

    <!-- No NPCs -->
    <div v-else-if="displayNpcs.length === 0" class="text-text-dim text-xs italic">
      No NPCs match filters in current zone.
    </div>

    <!-- NPC compact list -->
    <div
      v-else
      class="flex flex-col gap-1.5 overflow-y-auto max-h-80"
    >
      <div
        v-for="entry in displayNpcs"
        :key="entry.npc.key"
        class="flex flex-col gap-0 px-2 py-1 rounded bg-surface-elevated border border-border-default"
      >
        <!-- Line 1: Name, trained skills (centered), favor badge (right) -->
        <div class="flex items-center gap-1.5 min-w-0">
          <NpcInline :reference="entry.npc.key" :npc="entry.npc" class="shrink-0" />
          <span class="flex-1 flex items-center justify-center gap-1 min-w-0 overflow-hidden">
            <template v-if="config.showSkills">
              <span
                v-for="skill in entry.trainedSkills"
                :key="skill"
                class="text-[0.6rem] text-entity-skill shrink-0"
              >&#x2666;{{ skill }}</span>
            </template>
          </span>
          <span
            v-if="entry.favorTier"
            class="text-[0.6rem] px-1 py-0 rounded border shrink-0 leading-tight"
            :class="favorBadgeClasses(entry.favorTier)"
          >
            {{ tierDisplayName(entry.favorTier) }}
          </span>
        </div>

        <!-- Line 2: Gold (left), Storage (right) -->
        <div
          v-if="entry.hasSecondLine"
          class="flex items-center justify-between text-xs text-text-secondary pl-1"
        >
          <span v-if="entry.goldLabel" class="flex items-center gap-1">
            <span class="text-accent-gold">$</span>
            Gold: {{ entry.goldLabel }}
            <span v-if="entry.timerLabel" class="text-text-dim text-[0.6rem]">
              (resets ~{{ entry.timerLabel }})
            </span>
          </span>
          <span v-else />
          <span v-if="entry.storageLabel" class="flex items-center gap-1">
            <span class="text-cyan-400">&#x25A3;</span>
            Storage: {{ entry.storageLabel }}
          </span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useGameStateStore } from '../../../stores/gameStateStore'
import { useGameDataStore } from '../../../stores/gameDataStore'
import { favorBadgeClasses, tierDisplayName, tierIndex } from '../../../composables/useFavorTiers'
import { hasVendor, hasStorage, hasTraining, getTrainingService } from '../../../composables/useNpcServices'
import NpcInline from '../../Shared/NPC/NpcInline.vue'
import type { NpcInfo } from '../../../types/gameData'

const VENDOR_RESET_HOURS = 168
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

const config = ref<WidgetConfig>(loadConfig())

// Listen for config changes from the config panel
window.addEventListener('storage', (e) => {
  if (e.key === CONFIG_KEY) config.value = loadConfig()
})

// Also watch for same-tab updates via a custom event
window.addEventListener('zoneNpcsConfigChanged', () => {
  config.value = loadConfig()
})

const gameState = useGameStateStore()
const gameData = useGameDataStore()

const areaName = computed(() => gameState.world.area?.area_name ?? null)

const areaNpcs = ref<NpcInfo[]>([])
const loading = ref(false)

watch(areaName, async (name) => {
  if (!name) {
    areaNpcs.value = []
    return
  }
  loading.value = true
  try {
    areaNpcs.value = await gameData.getNpcsInArea(name)
  } catch (e) {
    console.error('[ZoneNpcsWidget] Failed to load NPCs for area:', e)
    areaNpcs.value = []
  } finally {
    loading.value = false
  }
}, { immediate: true })

interface NpcDisplayEntry {
  npc: NpcInfo
  favorTier: string | null
  trainedSkills: string[]
  storageLabel: string | null
  goldLabel: string | null
  timerLabel: string | null
  hasSecondLine: boolean
  hasStorageOrShop: boolean
  storageUsed: number | null
  storageTotalSlots: number | null
  goldAvailable: number | null
}

function buildEntry(npc: NpcInfo): NpcDisplayEntry {
  const favorTier = gameState.favorByNpc[npc.key]?.favor_tier ?? null

  // Training skills
  const training = getTrainingService(npc)
  const trainedSkills = training?.skills ?? []

  // Storage
  let storageLabel: string | null = null
  if (hasStorage(npc)) {
    const items = gameState.storageByVault[npc.key]
    const vault = gameState.storageVaultsByKey[npc.key]
    if (items != null && vault) {
      const used = items.length
      const total = gameState.getVaultUnlockedSlots(vault)
      if (total != null) {
        storageLabel = `${used}/${total}`
      }
    } else {
      storageLabel = 'available'
    }
  }

  // Gold
  let goldLabel: string | null = null
  let timerLabel: string | null = null
  const vendorData = gameState.vendorByNpc[npc.key]
  if (hasVendor(npc)) {
    if (vendorData?.vendor_gold_available != null) {
      goldLabel = vendorData.vendor_gold_available.toLocaleString()
      if (vendorData.vendor_gold_max != null) {
        goldLabel += `/${vendorData.vendor_gold_max.toLocaleString()}`
      }
    } else {
      goldLabel = 'no sell data recorded'
    }
    if (vendorData?.vendor_gold_timer_start) {
      const start = new Date(vendorData.vendor_gold_timer_start + 'Z')
      const resetAt = new Date(start.getTime() + VENDOR_RESET_HOURS * 60 * 60 * 1000)
      const now = new Date()
      const remaining = resetAt.getTime() - now.getTime()
      if (remaining > 0) {
        const hours = Math.floor(remaining / (1000 * 60 * 60))
        const days = Math.floor(hours / 24)
        const remainingHours = hours % 24
        timerLabel = days > 0 ? `${days}d ${remainingHours}h` : `${hours}h`
      }
    }
  }

  const hasSecondLine = storageLabel !== null || goldLabel !== null
  const hasStorageOrShop = hasStorage(npc) || hasVendor(npc)

  // Numeric values for sorting
  const storageItems = gameState.storageByVault[npc.key]
  const storageUsed = storageItems ? storageItems.length : null
  const vault = gameState.storageVaultsByKey[npc.key]
  const storageTotalSlots = vault ? gameState.getVaultUnlockedSlots(vault) : null
  const goldAvailable = vendorData?.vendor_gold_available ?? null

  return { npc, favorTier, trainedSkills, storageLabel, goldLabel, timerLabel, hasSecondLine, hasStorageOrShop, storageUsed, storageTotalSlots, goldAvailable }
}

const displayNpcs = computed(() => {
  let entries = areaNpcs.value.map(buildEntry)

  const cfg = config.value

  // Type filters — if ALL are off, show all (treat as no filter)
  const anyTypeFilter = cfg.showStorage || cfg.showShops || cfg.showTrainers
  // Only filter if not all three are checked (i.e., user deselected something)
  const allTypeChecked = cfg.showStorage && cfg.showShops && cfg.showTrainers
  if (anyTypeFilter && !allTypeChecked) {
    entries = entries.filter(e => {
      if (cfg.showStorage && hasStorage(e.npc)) return true
      if (cfg.showShops && hasVendor(e.npc)) return true
      if (cfg.showTrainers && hasTraining(e.npc)) return true
      return false
    })
  }

  // Favor rank filters
  if (cfg.minFavorRank) {
    const minIdx = tierIndex(cfg.minFavorRank)
    entries = entries.filter(e => {
      if (!e.favorTier) return false
      return tierIndex(e.favorTier) <= minIdx // lower index = higher rank
    })
  }
  if (cfg.maxFavorRank) {
    const maxIdx = tierIndex(cfg.maxFavorRank)
    entries = entries.filter(e => {
      if (!e.favorTier) return true // no data, don't exclude
      return tierIndex(e.favorTier) >= maxIdx
    })
  }

  // Giftable only
  if (cfg.showGiftableOnly) {
    entries = entries.filter(e => e.npc.gift_favor_tiers.length > 0)
  }

  // Sort by selected criteria
  entries.sort((a, b) => {
    // Push NPCs without storage or shops to bottom first if enabled
    if (cfg.pushEmptyToBottom) {
      if (a.hasStorageOrShop && !b.hasStorageOrShop) return -1
      if (!a.hasStorageOrShop && b.hasStorageOrShop) return 1
    }

    switch (cfg.sortBy) {
      case 'storage': {
        const aRemain = a.storageTotalSlots != null && a.storageUsed != null ? a.storageTotalSlots - a.storageUsed : -1
        const bRemain = b.storageTotalSlots != null && b.storageUsed != null ? b.storageTotalSlots - b.storageUsed : -1
        return bRemain - aRemain
      }
      case 'gold': {
        const aGold = a.goldAvailable ?? -1
        const bGold = b.goldAvailable ?? -1
        return bGold - aGold
      }
      case 'favor':
        return (tierIndex(a.favorTier ?? 'Neutral')) - (tierIndex(b.favorTier ?? 'Neutral'))
      case 'name':
      default:
        return a.npc.name.localeCompare(b.npc.name)
    }
  })

  return entries
})
</script>
