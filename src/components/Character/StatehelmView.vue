<template>
  <div class="flex flex-col gap-3 h-full min-h-0 overflow-y-auto">
    <!-- Header -->
    <div class="flex items-center justify-between">
      <h2 class="text-lg font-semibold text-text-primary">Statehelm Reputation</h2>
      <div v-if="filteredStatuses.length > 0" class="flex items-center gap-3">
        <span class="text-sm text-text-muted">
          {{ totalGiftsGiven }} / {{ totalGiftsMax }} gifts this week
        </span>
        <span class="text-xs text-text-dim">
          Resets {{ weekResetLabel }}
        </span>
      </div>
    </div>

    <!-- Filters -->
    <div v-if="npcStatuses.length > 0" class="flex items-center gap-4 flex-wrap text-xs">
      <!-- Hide above favor tier -->
      <label class="flex items-center gap-1.5 text-text-secondary">
        Hide above
        <select
          v-model="hideAboveTier"
          class="bg-surface-elevated border border-border-default rounded px-1.5 py-0.5 text-xs text-text-primary cursor-pointer">
          <option value="">Off</option>
          <option v-for="tier in filterableTiers" :key="tier" :value="tier">
            {{ tierDisplayName(tier) }}
          </option>
        </select>
      </label>

      <!-- Show only filters -->
      <label class="flex items-center gap-1 text-text-secondary cursor-pointer">
        <input type="checkbox" v-model="showOnlyTrainers" class="accent-accent-gold" />
        Trainers
      </label>
      <label class="flex items-center gap-1 text-text-secondary cursor-pointer">
        <input type="checkbox" v-model="showOnlyVendors" class="accent-accent-gold" />
        Vendors
      </label>
      <label class="flex items-center gap-1 text-text-secondary cursor-pointer">
        <input type="checkbox" v-model="showOnlyStorage" class="accent-accent-gold" />
        Storage
      </label>

      <span class="text-border-default">|</span>

      <!-- Hide maxed gifts -->
      <label class="flex items-center gap-1 text-text-secondary cursor-pointer">
        <input type="checkbox" v-model="hideMaxedGifts" class="accent-accent-gold" />
        Hide maxed gifts
      </label>
    </div>

    <EmptyState
      v-if="!loading && npcStatuses.length === 0"
      primary="No giftable Statehelm NPCs found."
      secondary="NPC data may still be loading, or no Statehelm NPCs have gift preferences configured." />

    <EmptyState
      v-else-if="npcStatuses.length > 0 && filteredStatuses.length === 0"
      primary="All NPCs hidden by filters."
      secondary="Adjust the filter controls above to show NPCs." />

    <!-- NPC Cards -->
    <div v-if="filteredStatuses.length > 0" class="grid grid-cols-2 xl:grid-cols-3 2xl:grid-cols-4 min-[1800px]:grid-cols-5 gap-2">
      <div
        v-for="status in filteredStatuses"
        :key="status.npc.key"
        class="bg-surface-elevated rounded border px-2.5 py-2 flex flex-col gap-1.5"
        :class="status.giftsThisWeek >= status.maxGifts
          ? 'border-green-700/40'
          : 'border-border-default'">

        <!-- NPC Header -->
        <div class="flex items-center justify-between gap-1">
          <NpcInline :reference="status.npc.key" />
          <span
            v-if="status.favorTier"
            class="text-[0.65rem] px-1.5 py-0.5 rounded border shrink-0"
            :class="favorBadgeClasses(status.favorTier)">
            {{ tierDisplayName(status.favorTier) }}
          </span>
        </div>

        <!-- Gift Progress -->
        <div class="flex flex-col gap-1">
          <div class="flex items-center justify-between text-xs">
            <span class="text-text-secondary">Gifts</span>
            <div class="flex items-center gap-1.5">
              <span :class="status.giftsThisWeek >= status.maxGifts ? 'text-green-400' : 'text-text-primary'">
                {{ status.giftsThisWeek }} / {{ status.maxGifts }}
              </span>
              <div class="flex items-center gap-0.5">
                <button
                  class="w-4 h-4 flex items-center justify-center rounded text-[0.6rem] text-text-muted hover:text-text-primary hover:bg-surface-default transition-colors"
                  :class="{ 'opacity-30 pointer-events-none': status.giftsThisWeek <= 0 }"
                  title="Remove a gift"
                  @click="removeGift(status.npc.key)">
                  -
                </button>
                <button
                  class="w-4 h-4 flex items-center justify-center rounded text-[0.6rem] text-text-muted hover:text-text-primary hover:bg-surface-default transition-colors"
                  :class="{ 'opacity-30 pointer-events-none': status.giftsThisWeek >= status.maxGifts }"
                  title="Add a gift"
                  @click="addGift(status.npc.key, status.npc.name)">
                  +
                </button>
              </div>
            </div>
          </div>
          <!-- Gift dots -->
          <div class="flex items-center gap-1">
            <div
              v-for="i in status.maxGifts"
              :key="i"
              class="w-2 h-2 rounded-full"
              :class="i <= status.giftsThisWeek
                ? 'bg-green-500'
                : 'bg-surface-default border border-border-default'" />
          </div>
        </div>

        <!-- Preferences -->
        <div v-if="sortedPreferences(status.npc).length" class="flex flex-col gap-0.5">
          <div class="text-[0.6rem] uppercase tracking-widest text-text-dim border-b border-surface-card pb-0.5 mb-0.5">
            Preferences
          </div>
          <div
            v-for="(pref, i) in sortedPreferences(status.npc)"
            :key="i"
            class="flex items-center gap-1.5 px-1.5 py-0.5 text-xs bg-[#151515] rounded">
            <span
              class="text-[0.6rem] px-1 py-0.5 rounded border min-w-8 text-center shrink-0"
              :class="desireBadgeClasses(pref.desire)">
              {{ pref.desire }}
            </span>
            <span class="text-text-secondary flex-1 truncate">
              {{ pref.name ?? pref.keywords.join(', ') }}
            </span>
            <span
              class="text-[0.6rem] shrink-0 font-mono"
              :class="pref.pref > 0 ? 'text-green-400' : 'text-red-400'">
              {{ pref.pref > 0 ? '+' : '' }}{{ pref.pref }}
            </span>
          </div>
        </div>

        <!-- Services summary -->
        <div v-if="status.services.length > 0" class="flex flex-col gap-1">
          <!-- Training -->
          <div v-for="training in status.trainingServices" :key="'t'" class="flex items-center gap-1.5 text-xs">
            <span class="text-entity-skill">&#x2726;</span>
            <span class="text-text-secondary">Trains:</span>
            <span class="flex flex-wrap gap-1">
              <SkillInline
                v-for="skill in training.skills"
                :key="skill"
                :reference="skill"
                :show-icon="false" />
            </span>
          </div>

          <!-- Storage -->
          <div v-for="storage in status.storageServices" :key="'s'" class="flex items-start gap-1.5 text-xs">
            <span class="text-cyan-400 mt-0.5">&#x25A3;</span>
            <div class="flex flex-col">
              <span class="text-text-secondary">
                Storage
                <span class="text-text-dim text-[0.6rem]">({{ tierDisplayName(storage.favor) }}+)</span>
              </span>
              <span v-if="storage.spaceIncreases.length" class="text-[0.6rem] text-text-dim">
                +space at:
                <span v-for="(tier, j) in storage.spaceIncreases" :key="tier">
                  <span :class="favorColor(tier)">{{ tierDisplayName(tier) }}</span><span v-if="j < storage.spaceIncreases.length - 1">, </span>
                </span>
              </span>
            </div>
          </div>

          <!-- Vendor -->
          <div v-for="store in status.storeServices" :key="'v'" class="flex items-start gap-1.5 text-xs">
            <span class="text-accent-gold mt-0.5">$</span>
            <div class="flex flex-col">
              <span class="text-text-secondary">
                Vendor
                <span class="text-text-dim text-[0.6rem]">({{ tierDisplayName(store.favor) }}+)</span>
              </span>
              <div v-if="store.capIncreases.length" class="flex flex-col text-[0.6rem] text-text-dim">
                <span v-for="cap in store.capIncreases" :key="cap.tier">
                  <span :class="favorColor(cap.tier)">{{ tierDisplayName(cap.tier) }}</span>:
                  <span class="text-accent-gold">{{ cap.maxGold.toLocaleString() }}</span>c
                  <span v-if="cap.itemTypes.length"> — {{ cap.itemTypes.join(', ') }}</span>
                </span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useStatehelmTracker } from '../../composables/useStatehelmTracker'
import { favorColor, favorBadgeClasses, tierDisplayName, isTierAtOrAbove, FAVOR_TIERS } from '../../composables/useFavorTiers'
import {
  parseServices,
  type StoreService,
  type TrainingService,
  type StorageService,
} from '../../types/npcServices'
import EmptyState from '../Shared/EmptyState.vue'
import NpcInline from '../Shared/NPC/NpcInline.vue'
import SkillInline from '../Shared/Skill/SkillInline.vue'
import type { NpcInfo, NpcPreference } from '../../types/gameData/npcs'

const {
  npcStatuses,
  totalGiftsGiven,
  totalGiftsMax,
  loading,
  loadGiftLog,
  addGift,
  removeGift,
  weekStart,
} = useStatehelmTracker()

// Filter state
const hideAboveTier = ref('')
const showOnlyTrainers = ref(false)
const showOnlyVendors = ref(false)
const showOnlyStorage = ref(false)
const hideMaxedGifts = ref(false)

// Tiers available for the "hide above" dropdown (exclude Despised — hiding above Despised hides everything)
const filterableTiers = FAVOR_TIERS.filter(t => t !== 'Despised')

// Enrich statuses with parsed services
const enrichedStatuses = computed(() => {
  return npcStatuses.value.map(status => {
    const services = parseServices(status.npc.services)
    return {
      ...status,
      services,
      trainingServices: services.filter((s): s is TrainingService => s.type === 'Training'),
      storeServices: services.filter((s): s is StoreService => s.type === 'Store'),
      storageServices: services.filter((s): s is StorageService => s.type === 'Storage'),
      hasTraining: services.some(s => s.type === 'Training'),
      hasVendor: services.some(s => s.type === 'Store'),
      hasStorage: services.some(s => s.type === 'Storage'),
    }
  })
})

const filteredStatuses = computed(() => {
  return enrichedStatuses.value.filter(status => {
    // Hide above tier filter
    if (hideAboveTier.value && status.favorTier) {
      if (isTierAtOrAbove(status.favorTier, hideAboveTier.value)) return false
    }

    // Service filters (additive — if any are checked, NPC must match at least one)
    const anyServiceFilter = showOnlyTrainers.value || showOnlyVendors.value || showOnlyStorage.value
    if (anyServiceFilter) {
      const matches =
        (showOnlyTrainers.value && status.hasTraining) ||
        (showOnlyVendors.value && status.hasVendor) ||
        (showOnlyStorage.value && status.hasStorage)
      if (!matches) return false
    }

    // Hide maxed gifts
    if (hideMaxedGifts.value && status.giftsThisWeek >= status.maxGifts) return false

    return true
  })
})

const weekResetLabel = computed(() => {
  const next = new Date(weekStart.value)
  next.setUTCDate(next.getUTCDate() + 7)
  const now = new Date()
  const diffMs = next.getTime() - now.getTime()
  const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24))
  const diffHours = Math.floor((diffMs % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60))
  if (diffDays > 0) return `in ${diffDays}d ${diffHours}h`
  return `in ${diffHours}h`
})

function sortedPreferences(npc: NpcInfo): NpcPreference[] {
  return [...(npc.preferences ?? [])].sort((a, b) => b.pref - a.pref)
}

function desireBadgeClasses(desire: string): string {
  switch (desire.toLowerCase()) {
    case 'love':
      return 'bg-red-900/30 border-red-700/40 text-red-300'
    case 'like':
      return 'bg-green-900/30 border-green-700/40 text-green-300'
    case 'hate':
      return 'bg-red-900/40 border-red-600/50 text-red-400'
    default:
      return 'bg-surface-elevated border-border-default text-text-muted'
  }
}

onMounted(() => {
  loadGiftLog()
})
</script>
