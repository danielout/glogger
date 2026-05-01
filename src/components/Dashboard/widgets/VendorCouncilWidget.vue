<template>
  <div class="flex flex-col gap-2 overflow-y-auto max-h-96">
    <!-- View mode toggle + item type filter -->
    <div class="flex items-center gap-1.5 px-1 flex-wrap">
      <button
        class="px-2 py-0.5 text-[10px] rounded cursor-pointer transition-colors"
        :class="viewMode === 'character'
          ? 'bg-accent-gold/20 border border-accent-gold/40 text-text-primary'
          : 'text-text-muted hover:text-text-secondary'"
        @click="viewMode = 'character'"
      >
        Active Character
      </button>
      <button
        class="px-2 py-0.5 text-[10px] rounded cursor-pointer transition-colors"
        :class="viewMode === 'all'
          ? 'bg-accent-gold/20 border border-accent-gold/40 text-text-primary'
          : 'text-text-muted hover:text-text-secondary'"
        @click="switchToAll"
      >
        All Characters
      </button>
      <select
        v-model="itemTypeFilter"
        class="ml-auto px-1.5 py-0.5 text-[10px] bg-surface-base border border-border-default rounded text-text-primary cursor-pointer focus:outline-none focus:border-accent-gold/50"
      >
        <option value="">All Categories</option>
        <optgroup label="Categories">
          <option v-for="cat in CATEGORY_ORDER" :key="cat" :value="'cat:' + cat">{{ cat }}</option>
        </optgroup>
        <optgroup label="Item Types">
          <option v-for="t in allItemTypes" :key="t" :value="t">{{ t }}</option>
        </optgroup>
      </select>
    </div>

    <!-- Loading state for aggregate -->
    <div v-if="viewMode === 'all' && aggregateLoading" class="text-text-dim text-xs italic px-2">
      Loading cross-character data...
    </div>

    <!-- Empty state -->
    <div v-else-if="vendorEntries.length === 0" class="text-text-dim text-xs italic px-2">
      No vendors found in CDN data.
    </div>

    <template v-else>
      <!-- Grand total -->
      <div class="flex items-center justify-between px-2 py-1.5 bg-surface-elevated rounded border border-border-default">
        <span class="text-xs text-text-muted">Total Councils Remaining</span>
        <span class="text-sm font-bold font-mono" :class="grandTotalColorClass">
          {{ grandTotalAvailable.toLocaleString() }}
          <span v-if="grandTotalMax > 0" class="text-text-dim font-normal">
            / {{ grandTotalMax.toLocaleString() }}
          </span>
        </span>
      </div>

      <!-- Flat list when filtered -->
      <template v-if="itemTypeFilter">
        <div class="flex flex-col gap-0.5 px-1">
          <VendorRow
            v-for="v in vendorEntries"
            :key="v.npcKey"
            :entry="v"
            :view-mode="viewMode"
            :quick-edit-key="quickEditKey"
            :quick-edit-value="quickEditValue"
            @set-to-cap="setToCap"
            @start-quick-edit="startQuickEdit"
            @save-quick-edit="saveQuickEdit"
            @cancel-quick-edit="quickEditKey = null"
            @update:quick-edit-value="quickEditValue = $event"
          />
        </div>
      </template>

      <!-- Category groups when unfiltered -->
      <template v-else>
        <div
          v-for="cat in categories"
          :key="cat.name"
          class="flex flex-col"
        >
          <button
            class="flex items-center justify-between px-1.5 py-1 cursor-pointer hover:bg-surface-elevated/50 rounded transition-colors"
            @click="toggleCategory(cat.name)"
          >
            <span class="flex items-center gap-1.5">
              <span
                class="text-[10px] transition-transform"
                :class="expandedCategories.has(cat.name) ? 'rotate-90' : ''"
              >&#x25B6;</span>
              <span class="text-xs font-semibold text-text-secondary">{{ cat.name }}</span>
              <span class="text-[10px] text-text-dim">({{ cat.vendors.length }})</span>
            </span>
            <span class="text-xs font-mono" :class="categoryColorClass(cat)">
              {{ cat.totalAvailable.toLocaleString() }}
              <span v-if="cat.totalMax > 0" class="text-text-dim">
                / {{ cat.totalMax.toLocaleString() }}
              </span>
            </span>
          </button>

          <div
            v-if="expandedCategories.has(cat.name)"
            class="flex flex-col gap-0.5 pl-4 mt-0.5"
          >
            <VendorRow
              v-for="v in cat.vendors"
              :key="v.npcKey"
              :entry="v"
              :view-mode="viewMode"
              :quick-edit-key="quickEditKey"
              :quick-edit-value="quickEditValue"
              @set-to-cap="setToCap"
              @start-quick-edit="startQuickEdit"
              @save-quick-edit="saveQuickEdit"
              @cancel-quick-edit="quickEditKey = null"
              @update:quick-edit-value="quickEditValue = $event"
            />
          </div>
        </div>
      </template>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useGameStateStore } from '../../../stores/gameStateStore'
import { useGameDataStore } from '../../../stores/gameDataStore'
import { useCharacterStore } from '../../../stores/characterStore'
import { useSettingsStore } from '../../../stores/settingsStore'
import { hasBuyCapacity, getStoreService, goldCapAtTier } from '../../../composables/useNpcServices'
import VendorRow from './VendorRow.vue'

const VENDOR_RESET_HOURS = 168

// ── Category mapping ────────────────────────────────────────────

const CATEGORY_MAP: Record<string, string> = {
  Jewelry: 'Jewelry & Gems',
  Gem: 'Jewelry & Gems',
  Crystal: 'Jewelry & Gems',
  Seashell: 'Jewelry & Gems',
  Weapon: 'Weapons',
  Armor: 'Armor & Shields',
  Shield: 'Armor & Shields',
  ClothArmor: 'Armor & Shields',
  Skin: 'Skins & Trophies',
  Skinning: 'Skins & Trophies',
  Fur: 'Skins & Trophies',
  CorpseTrophy: 'Skins & Trophies',
  Skull: 'Skins & Trophies',
  Potion: 'Potions & Alchemy',
  AlchemyIngredient: 'Potions & Alchemy',
  Food: 'Food & Cooking',
  Edible: 'Food & Cooking',
  PreparedFood: 'Food & Cooking',
  CookingIngredient: 'Food & Cooking',
  Cheese: 'Food & Cooking',
  Butter: 'Food & Cooking',
  BreadDish: 'Food & Cooking',
  BrewingIngredient: 'Food & Cooking',
  BrewingRelated: 'Food & Cooking',
}

const CATEGORY_ORDER = [
  'Armor & Shields',
  'Weapons',
  'Jewelry & Gems',
  'Skins & Trophies',
  'Potions & Alchemy',
  'Food & Cooking',
  'Other',
]

function getCategoryName(itemType: string): string {
  return CATEGORY_MAP[itemType] ?? 'Other'
}

// ── Stores ──────────────────────────────────────────────────────

const gameState = useGameStateStore()
const gameData = useGameDataStore()
const characterStore = useCharacterStore()
const settingsStore = useSettingsStore()

// ── View mode ───────────────────────────────────────────────────

const viewMode = ref<'character' | 'all'>('character')

// ── Aggregate data ──────────────────────────────────────────────

interface AggregateVendorEntry {
  npc_key: string
  characters: {
    character_name: string
    vendor_gold_available: number | null
    vendor_gold_max: number | null
    vendor_gold_timer_start: string | null
    favor_tier: string | null
  }[]
}

const aggregateData = ref<AggregateVendorEntry[]>([])
const aggregateLoading = ref(false)

async function loadAggregate() {
  const serverName = settingsStore.settings.activeServerName
  if (!serverName) return
  aggregateLoading.value = true
  try {
    aggregateData.value = await invoke<AggregateVendorEntry[]>('get_aggregate_vendor', { serverName })
  } catch (e) {
    console.error('[VendorCouncilWidget] Failed to load aggregate vendor data:', e)
    aggregateData.value = []
  } finally {
    aggregateLoading.value = false
  }
}

function switchToAll() {
  viewMode.value = 'all'
  loadAggregate()
}

// Reload aggregate when vendor data changes while in aggregate mode
watch(() => gameState.vendorByNpc, () => {
  if (viewMode.value === 'all') loadAggregate()
})

// ── Collapsed/expanded state ────────────────────────────────────

const expandedCategories = ref(new Set<string>())

function toggleCategory(name: string) {
  if (expandedCategories.value.has(name)) {
    expandedCategories.value.delete(name)
  } else {
    expandedCategories.value.add(name)
  }
}

// ── Quick edit ──────────────────────────────────────────────────

const quickEditKey = ref<string | null>(null)
const quickEditValue = ref(0)

function startQuickEdit(v: VendorEntry) {
  quickEditKey.value = v.npcKey
  quickEditValue.value = v.goldMax ?? 0
}

async function setToCap(v: VendorEntry) {
  if (v.goldMax == null) return
  await saveVendorGold(v.npcKey, v.goldMax, v.goldMax)
}

async function saveQuickEdit(v: VendorEntry) {
  const max = v.goldMax ?? quickEditValue.value
  await saveVendorGold(v.npcKey, quickEditValue.value, max)
  quickEditKey.value = null
}

async function saveVendorGold(npcKey: string, goldAvailable: number, goldMax: number) {
  const characterName = settingsStore.settings.activeCharacterName
  const serverName = settingsStore.settings.activeServerName
  if (!characterName || !serverName) return
  try {
    await invoke('set_manual_vendor_gold', { characterName, serverName, npcKey, goldAvailable, goldMax })
    await gameState.refreshDomain('vendor')
  } catch (e) {
    console.error('[VendorCouncilWidget] Failed to save vendor gold:', e)
  }
}

// ── Reset detection ─────────────────────────────────────────────

/** Check if timer has expired (>168h since timer started), meaning gold has reset to cap. */
function hasTimerExpired(timerStart: string | null): boolean {
  if (!timerStart) return false
  const start = new Date(timerStart + 'Z')
  if (isNaN(start.getTime())) return false
  const resetAt = start.getTime() + VENDOR_RESET_HOURS * 60 * 60 * 1000
  return Date.now() >= resetAt
}

// ── Vendor entry building ───────────────────────────────────────

interface CharacterBreakdownEntry {
  characterName: string
  goldAvailable: number | null
  goldMax: number | null
}

interface VendorEntry {
  npcKey: string
  npcName: string
  area: string | null
  goldAvailable: number | null
  goldMax: number | null
  resetLabel: string | null
  assumedReset: boolean
  itemCategories: string[]
  rawItemTypes: string[]
  characterBreakdown: CharacterBreakdownEntry[] | null
}

function resolvePlayerTier(npcKey: string): string {
  const gsFavor = gameState.favorByNpc[npcKey]
  if (gsFavor?.favor_tier) return gsFavor.favor_tier
  const snap = characterStore.npcFavor.find(f => f.npc_key === npcKey)
  if (snap?.favor_level) return snap.favor_level
  return 'Neutral'
}

function computeTimerLabel(timerStart: string | null): string | null {
  if (!timerStart) return null
  const start = new Date(timerStart + 'Z')
  const resetAt = new Date(start.getTime() + VENDOR_RESET_HOURS * 60 * 60 * 1000)
  const now = new Date()
  const remaining = resetAt.getTime() - now.getTime()
  if (remaining <= 0) return null
  const hours = Math.floor(remaining / (1000 * 60 * 60))
  const days = Math.floor(hours / 24)
  const remainingHours = hours % 24
  return days > 0 ? `${days}d ${remainingHours}h` : `${hours}h`
}

function getRawItemTypes(npcKey: string): string[] {
  const npc = gameData.npcsByKey[npcKey]
  if (!npc) return []
  const store = getStoreService(npc)
  if (!store) return []
  const allItemTypes = new Set<string>()
  for (const cap of store.capIncreases) {
    for (const t of cap.itemTypes) allItemTypes.add(t)
  }
  return [...allItemTypes]
}

function getItemCategories(npcKey: string): string[] {
  const raw = getRawItemTypes(npcKey)
  if (raw.length === 0) return ['Other']
  return [...new Set(raw.map(getCategoryName))]
}

// ── Item type filter ────────────────────────────────────────────

const itemTypeFilter = ref('')

/** All unique raw item types across all vendors with buy capacity, sorted */
const allItemTypes = computed(() => {
  const types = new Set<string>()
  for (const [key, npc] of Object.entries(gameData.npcsByKey)) {
    if (!hasBuyCapacity(npc)) continue
    for (const t of getRawItemTypes(key)) types.add(t)
  }
  return [...types].sort()
})

// Active character vendor entries — includes ALL vendors with buy capacity
const characterEntries = computed<VendorEntry[]>(() => {
  const entries: VendorEntry[] = []

  for (const [key, npc] of Object.entries(gameData.npcsByKey)) {
    if (!hasBuyCapacity(npc)) continue

    const vendorState = gameState.vendorByNpc[key]
    const playerTier = resolvePlayerTier(key)
    const capAtTier = goldCapAtTier(npc, playerTier)
    const maxGold = vendorState?.vendor_gold_max ?? capAtTier?.maxGold ?? null

    // Determine effective gold: if timer has expired, assume reset to cap
    let goldAvailable = vendorState?.vendor_gold_available ?? null
    let assumedReset = false
    let resetLabel: string | null = null

    if (vendorState && goldAvailable != null) {
      if (hasTimerExpired(vendorState.vendor_gold_timer_start)) {
        // Timer expired — vendor has reset to cap
        goldAvailable = maxGold
        assumedReset = true
        resetLabel = '(reset)'
      } else {
        resetLabel = computeTimerLabel(vendorState.vendor_gold_timer_start)
        if (resetLabel) resetLabel = `(${resetLabel})`
      }
    }

    entries.push({
      npcKey: key,
      npcName: npc.name,
      area: npc.area_friendly_name ?? null,
      goldAvailable,
      goldMax: maxGold,
      resetLabel,
      assumedReset,
      itemCategories: getItemCategories(key),
      rawItemTypes: getRawItemTypes(key),
      characterBreakdown: null,
    })
  }

  // Sort: tracked vendors first, then untracked
  entries.sort((a, b) => {
    if (a.goldAvailable != null && b.goldAvailable == null) return -1
    if (a.goldAvailable == null && b.goldAvailable != null) return 1
    return a.npcName.localeCompare(b.npcName)
  })

  return entries
})

// Aggregate vendor entries (all characters) — includes ALL CDN vendors with buy capacity
const aggregateEntries = computed<VendorEntry[]>(() => {
  // Index aggregate DB data by npc_key for lookup
  const aggByNpc = new Map<string, AggregateVendorEntry>()
  for (const agg of aggregateData.value) {
    aggByNpc.set(agg.npc_key, agg)
  }

  const entries: VendorEntry[] = []

  for (const [key, npc] of Object.entries(gameData.npcsByKey)) {
    if (!hasBuyCapacity(npc)) continue

    const agg = aggByNpc.get(key)
    // For untracked vendors, use active character's favor tier as best guess
    const fallbackTier = resolvePlayerTier(key)
    const fallbackCap = goldCapAtTier(npc, fallbackTier)?.maxGold ?? null

    if (agg) {
      // Has DB data — resolve per-character with their own favor tier for cap
      const resolvedCharacters = agg.characters.map(c => {
        let goldAvail = c.vendor_gold_available
        // Use per-character favor tier from DB for accurate cap, fall back to active character's tier
        const charTier = c.favor_tier ?? fallbackTier
        const charCap = goldCapAtTier(npc, charTier)?.maxGold ?? null
        const goldMax = c.vendor_gold_max ?? charCap
        if (goldAvail != null && hasTimerExpired(c.vendor_gold_timer_start)) {
          goldAvail = goldMax
        }
        return { ...c, vendor_gold_available: goldAvail, vendor_gold_max: goldMax }
      })

      const totalAvailable = resolvedCharacters.reduce((sum, c) => sum + (c.vendor_gold_available ?? 0), 0)
      const totalMax = resolvedCharacters.reduce((sum, c) => sum + (c.vendor_gold_max ?? 0), 0)

      // Find earliest active (non-expired) timer
      let earliestTimer: string | null = null
      for (const c of agg.characters) {
        if (c.vendor_gold_timer_start && !hasTimerExpired(c.vendor_gold_timer_start)) {
          if (!earliestTimer || c.vendor_gold_timer_start < earliestTimer) {
            earliestTimer = c.vendor_gold_timer_start
          }
        }
      }
      const timerLabel = computeTimerLabel(earliestTimer)

      entries.push({
        npcKey: key,
        npcName: npc.name,
        area: npc.area_friendly_name ?? null,
        goldAvailable: totalAvailable > 0 || resolvedCharacters.some(c => c.vendor_gold_available != null) ? totalAvailable : null,
        goldMax: totalMax > 0 ? totalMax : null,
        resetLabel: timerLabel ? `(${timerLabel})` : null,
        assumedReset: false,
        itemCategories: getItemCategories(key),
        rawItemTypes: getRawItemTypes(key),
        characterBreakdown: resolvedCharacters
          .filter(c => c.vendor_gold_available != null)
          .map(c => ({
            characterName: c.character_name,
            goldAvailable: c.vendor_gold_available,
            goldMax: c.vendor_gold_max,
          })),
      })
    } else {
      // No DB data — untracked vendor
      entries.push({
        npcKey: key,
        npcName: npc.name,
        area: npc.area_friendly_name ?? null,
        goldAvailable: null,
        goldMax: fallbackCap,
        resetLabel: null,
        assumedReset: false,
        itemCategories: getItemCategories(key),
        rawItemTypes: getRawItemTypes(key),
        characterBreakdown: null,
      })
    }
  }

  // Sort: tracked first, then untracked
  entries.sort((a, b) => {
    if (a.goldAvailable != null && b.goldAvailable == null) return -1
    if (a.goldAvailable == null && b.goldAvailable != null) return 1
    return a.npcName.localeCompare(b.npcName)
  })

  return entries
})

// Switch based on view mode, then apply item type filter
const vendorEntries = computed(() => {
  const base = viewMode.value === 'all' ? aggregateEntries.value : characterEntries.value
  const filter = itemTypeFilter.value
  if (!filter) return base

  if (filter.startsWith('cat:')) {
    // Filter by preset category — show vendors whose mapped categories include this one
    const catName = filter.slice(4)
    return base.filter(v => v.itemCategories.includes(catName))
  }

  // Filter by specific raw item type
  return base.filter(v => v.rawItemTypes.includes(filter))
})

// ── Category grouping ───────────────────────────────────────────

interface CategoryGroup {
  name: string
  vendors: VendorEntry[]
  totalAvailable: number
  totalMax: number
}

const categories = computed<CategoryGroup[]>(() => {
  const catMap = new Map<string, VendorEntry[]>()

  for (const entry of vendorEntries.value) {
    for (const cat of entry.itemCategories) {
      if (!catMap.has(cat)) catMap.set(cat, [])
      catMap.get(cat)!.push(entry)
    }
  }

  const groups: CategoryGroup[] = []
  for (const name of CATEGORY_ORDER) {
    const vendors = catMap.get(name)
    if (!vendors || vendors.length === 0) continue
    // Sort: tracked first (by gold desc), then untracked alphabetically
    vendors.sort((a, b) => {
      if (a.goldAvailable != null && b.goldAvailable == null) return -1
      if (a.goldAvailable == null && b.goldAvailable != null) return 1
      if (a.goldAvailable != null && b.goldAvailable != null) return b.goldAvailable - a.goldAvailable
      return a.npcName.localeCompare(b.npcName)
    })
    groups.push({
      name,
      vendors,
      totalAvailable: vendors.reduce((sum, v) => sum + (v.goldAvailable ?? 0), 0),
      totalMax: vendors.reduce((sum, v) => sum + (v.goldMax ?? 0), 0),
    })
  }

  // Add any uncategorized
  const covered = new Set(CATEGORY_ORDER)
  for (const [name, vendors] of catMap) {
    if (covered.has(name)) continue
    vendors.sort((a, b) => {
      if (a.goldAvailable != null && b.goldAvailable == null) return -1
      if (a.goldAvailable == null && b.goldAvailable != null) return 1
      if (a.goldAvailable != null && b.goldAvailable != null) return b.goldAvailable - a.goldAvailable
      return a.npcName.localeCompare(b.npcName)
    })
    groups.push({
      name,
      vendors,
      totalAvailable: vendors.reduce((sum, v) => sum + (v.goldAvailable ?? 0), 0),
      totalMax: vendors.reduce((sum, v) => sum + (v.goldMax ?? 0), 0),
    })
  }

  return groups
})

// ── Grand totals (deduped — each vendor counted once) ───────────

const grandTotalAvailable = computed(() =>
  vendorEntries.value.reduce((sum, v) => sum + (v.goldAvailable ?? 0), 0)
)

const grandTotalMax = computed(() =>
  vendorEntries.value.reduce((sum, v) => sum + (v.goldMax ?? 0), 0)
)

// ── Color helpers ───────────────────────────────────────────────

const grandTotalColorClass = computed(() => {
  if (grandTotalMax.value === 0) return 'text-text-secondary'
  const ratio = grandTotalAvailable.value / grandTotalMax.value
  if (ratio >= 0.7) return 'text-value-positive'
  if (ratio >= 0.3) return 'text-yellow-400'
  return 'text-value-negative'
})

function categoryColorClass(cat: CategoryGroup): string {
  if (cat.totalMax === 0) return 'text-text-secondary'
  const ratio = cat.totalAvailable / cat.totalMax
  if (ratio >= 0.7) return 'text-value-positive'
  if (ratio >= 0.3) return 'text-yellow-400'
  return 'text-value-negative'
}

</script>
