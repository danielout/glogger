<template>
  <div class="flex flex-col gap-3">
    <!-- Header area with edit toggle -->
    <div class="flex items-center justify-between">
      <button
        class="text-xs text-text-dim hover:text-text-muted transition-colors"
        title="Edit watched NPCs"
        @click="editing = !editing"
      >
        {{ editing ? '✕ Done' : '⚙ Edit' }}
      </button>
      <span class="text-xs text-text-dim">
        {{ watchedNpcKeys.length }} NPC{{ watchedNpcKeys.length === 1 ? '' : 's' }} watched
      </span>
    </div>

    <!-- Edit mode: NPC search + selection -->
    <div v-if="editing" class="flex flex-col gap-2">
      <!-- Search input -->
      <div class="relative">
        <input
          v-model="searchQuery"
          type="text"
          placeholder="Search NPCs by name..."
          class="w-full bg-surface-elevated text-text text-sm rounded px-2 py-1.5 border border-border-dim focus:border-accent-blue focus:outline-none"
          @input="onSearchInput"
        />
        <!-- Search results dropdown -->
        <div
          v-if="searchResults.length > 0 && searchQuery.length >= 2"
          class="absolute z-10 top-full left-0 right-0 mt-1 bg-surface-elevated border border-border-dim rounded shadow-lg max-h-48 overflow-y-auto"
        >
          <button
            v-for="npc in searchResults"
            :key="npc.key"
            class="w-full text-left px-2 py-1.5 text-sm hover:bg-surface-hover transition-colors"
            :class="watchedNpcKeys.includes(npc.key) ? 'text-text-dim' : 'text-text'"
            @click="toggleNpc(npc.key)"
          >
            <span>{{ npc.name }}</span>
            <span v-if="npc.area_friendly_name" class="text-text-dim text-xs ml-1">({{ npc.area_friendly_name }})</span>
            <span v-if="watchedNpcKeys.includes(npc.key)" class="text-accent-green text-xs ml-1">watching</span>
          </button>
        </div>
      </div>

      <!-- Watched NPC chips -->
      <div v-if="watchedNpcKeys.length > 0" class="flex flex-wrap gap-1.5">
        <span
          v-for="key in watchedNpcKeys"
          :key="key"
          class="inline-flex items-center gap-1 bg-surface-elevated text-entity-npc text-xs rounded-full px-2 py-0.5 border border-entity-npc/20"
        >
          {{ resolveNpcName(key) }}
          <button
            class="text-text-dim hover:text-accent-red transition-colors"
            @click="removeNpc(key)"
          >×</button>
        </span>
      </div>

      <p v-else class="text-xs text-text-dim italic">
        Search and select NPCs to watch for gift-worthy items in your inventory.
      </p>
    </div>

    <!-- Main display: matches per NPC -->
    <template v-if="!editing">
      <div v-if="watchedNpcKeys.length === 0" class="text-xs text-text-dim italic">
        No NPCs watched. Click ⚙ Edit to add some.
      </div>

      <div v-else-if="loading" class="text-xs text-text-dim italic">
        Resolving items...
      </div>

      <div v-else class="flex flex-col gap-3">
        <div
          v-for="entry in npcMatches"
          :key="entry.npcKey"
          class="flex flex-col gap-1"
        >
          <NpcInline :reference="entry.npcKey" />

          <div v-if="entry.matches.length > 0" class="flex flex-col gap-0.5 pl-3">
            <div
              v-for="match in entry.matches"
              :key="match.itemName"
              class="flex items-center justify-between text-sm"
            >
              <ItemInline :reference="match.itemName" :show-icon="false" />
              <span class="text-xs font-mono text-text-muted shrink-0 ml-2">
                ×{{ match.count }}
                <span
                  class="ml-1"
                  :class="match.desire === 'Love' ? 'text-accent-red' : 'text-accent-gold'"
                >{{ match.desire === 'Love' ? '♥' : '♦' }}</span>
              </span>
            </div>
          </div>

          <div v-else class="text-xs text-text-dim italic pl-3">
            No matching items
          </div>
        </div>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { useGameDataStore } from '../../../stores/gameDataStore'
import { useGameStateStore } from '../../../stores/gameStateStore'
import { findGiftableItems } from '../../../composables/useNpcGiftMatching'
import type { NpcInfo } from '../../../types/gameData/npcs'
import type { ItemInfo } from '../../../types/gameData/items'
import NpcInline from '../../Shared/NPC/NpcInline.vue'
import ItemInline from '../../Shared/Item/ItemInline.vue'

const STORAGE_KEY = 'giftWatcher.watchedNpcs'

const gameData = useGameDataStore()
const gameState = useGameStateStore()

const editing = ref(false)
const searchQuery = ref('')
const searchResults = ref<NpcInfo[]>([])
const watchedNpcKeys = ref<string[]>([])
const resolvedItems = ref<Record<string, ItemInfo>>({})
const loading = ref(false)

// ── Persistence ──────────────────────────────────────────────────────────────

function loadWatchedNpcs() {
  try {
    const stored = localStorage.getItem(STORAGE_KEY)
    if (stored) watchedNpcKeys.value = JSON.parse(stored)
  } catch {
    watchedNpcKeys.value = []
  }
}

function saveWatchedNpcs() {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(watchedNpcKeys.value))
}

// ── NPC search ───────────────────────────────────────────────────────────────

let searchTimeout: ReturnType<typeof setTimeout> | null = null

function onSearchInput() {
  if (searchTimeout) clearTimeout(searchTimeout)
  if (searchQuery.value.length < 2) {
    searchResults.value = []
    return
  }
  searchTimeout = setTimeout(async () => {
    searchResults.value = await gameData.searchNpcs(searchQuery.value)
  }, 200)
}

function toggleNpc(key: string) {
  if (watchedNpcKeys.value.includes(key)) {
    removeNpc(key)
  } else {
    watchedNpcKeys.value = [...watchedNpcKeys.value, key]
    saveWatchedNpcs()
  }
  searchQuery.value = ''
  searchResults.value = []
}

function removeNpc(key: string) {
  watchedNpcKeys.value = watchedNpcKeys.value.filter(k => k !== key)
  saveWatchedNpcs()
}

function resolveNpcName(key: string): string {
  const npc = gameData.resolveNpcSync(key)
  return npc?.name ?? key
}

// ── Item resolution ──────────────────────────────────────────────────────────

// Stable fingerprint of inventory item names — avoids re-resolving when only
// stack sizes change (Object.keys is the same set of names).
const inventoryItemNames = computed(() => Object.keys(gameState.inventoryItemCounts))
const itemNamesFingerprint = computed(() => inventoryItemNames.value.slice().sort().join('\0'))

async function resolveInventoryItems(showLoading = true) {
  const names = inventoryItemNames.value
  if (names.length === 0) {
    resolvedItems.value = {}
    return
  }
  if (showLoading) loading.value = true
  try {
    resolvedItems.value = await gameData.resolveItemsBatch(names)
  } catch (e) {
    console.warn('GiftWatcher: failed to resolve items', e)
  } finally {
    loading.value = false
  }
}

// Only re-resolve when the actual set of item names changes, not on every
// stack size update.  Skip the loading flash on incremental updates.
watch(itemNamesFingerprint, () => resolveInventoryItems(false))

// ── Gift matching ────────────────────────────────────────────────────────────

interface NpcMatchEntry {
  npcKey: string
  matches: { itemName: string; count: number; desire: string }[]
}

const npcMatches = computed<NpcMatchEntry[]>(() => {
  const counts = gameState.inventoryItemCounts
  const items: { name: string; keywords: string[] }[] = []

  for (const [name, count] of Object.entries(counts)) {
    if (count <= 0) continue
    const resolved = resolvedItems.value[name]
    if (resolved) {
      items.push({ name: resolved.name, keywords: resolved.keywords })
    }
  }

  return watchedNpcKeys.value.map(key => {
    const npc = gameData.resolveNpcSync(key)
    if (!npc) return { npcKey: key, matches: [] }

    const giftable = findGiftableItems(npc, items)
    return {
      npcKey: key,
      matches: giftable.map(g => ({
        itemName: g.item.name,
        count: counts[g.item.name] ?? 0,
        desire: g.bestPref.desire,
      })),
    }
  })
})

// ── Lifecycle ────────────────────────────────────────────────────────────────

onMounted(async () => {
  loadWatchedNpcs()
  await gameData.loadAllNpcsMap()
  await resolveInventoryItems()

  // Open edit mode automatically if no NPCs are watched
  if (watchedNpcKeys.value.length === 0) editing.value = true
})
</script>
