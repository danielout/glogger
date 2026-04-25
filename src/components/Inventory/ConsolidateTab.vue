<template>
  <div class="flex flex-col gap-4 h-full">
    <!-- Header -->
    <div class="flex items-center justify-between">
      <div>
        <h2 class="screen-title m-0">Consolidate Storage</h2>
        <p class="text-xs text-text-muted mt-1 m-0">Find items scattered across vaults and plan pickup routes to consolidate them.</p>
      </div>
    </div>

    <!-- Empty state -->
    <EmptyState
      v-if="!gameState.storage.length"
      variant="panel"
      primary="No storage data"
      secondary="Import an inventory report to see stored items." />

    <template v-else>
      <!-- Summary + Controls -->
      <div class="flex items-center gap-4 flex-wrap">
        <div class="flex gap-4 text-sm">
          <div class="flex gap-1.5 items-baseline">
            <span class="text-text-muted">Candidates:</span>
            <span class="text-text-primary font-medium">{{ consolidation.candidateCount.value }}</span>
          </div>
          <div class="flex gap-1.5 items-baseline">
            <span class="text-text-muted">Across:</span>
            <span class="text-text-primary font-medium">{{ consolidation.totalLocationCount.value }} locations</span>
          </div>
        </div>

        <div class="flex items-center gap-2 ml-auto">
          <label class="text-xs text-text-muted">Target:</label>
          <select
            v-model="consolidation.targetStrategy.value"
            class="input text-xs py-1 w-44">
            <option value="most_items">Most items at location</option>
            <option value="specific_vault">Specific vault...</option>
          </select>
          <select
            v-if="consolidation.targetStrategy.value === 'specific_vault'"
            v-model="consolidation.specificVaultKey.value"
            class="input text-xs py-1 w-52">
            <option :value="null" disabled>Choose a vault...</option>
            <option v-for="v in consolidation.availableVaults.value" :key="v.key" :value="v.key">
              {{ v.displayName }} ({{ v.areaName ?? 'Unknown' }})
            </option>
          </select>
        </div>
      </div>

      <!-- Filter -->
      <div class="flex items-center gap-3">
        <div class="relative">
          <input
            v-model="search"
            type="text"
            placeholder="Filter items..."
            class="input text-xs py-1 w-48 pr-7" />
          <button
            v-if="search"
            class="absolute right-1.5 top-1/2 -translate-y-1/2 text-text-muted hover:text-text-primary text-xs cursor-pointer"
            @click="search = ''">&times;</button>
        </div>
        <span class="text-xs text-text-muted">{{ filteredCandidates.length }} items</span>
        <div class="ml-auto flex gap-2">
          <button class="text-xs text-text-muted hover:text-text-primary cursor-pointer" @click="consolidation.selectAll()">Select All</button>
          <button class="text-xs text-text-muted hover:text-text-primary cursor-pointer" @click="consolidation.deselectAll()">Clear</button>
        </div>
      </div>

      <!-- No candidates -->
      <EmptyState
        v-if="consolidation.candidateCount.value === 0"
        variant="compact"
        primary="No items to consolidate"
        secondary="All items are in single locations." />

      <!-- Candidate list -->
      <div v-else class="flex-1 min-h-0 overflow-y-auto border border-border-default rounded-lg">
        <table class="w-full border-collapse text-xs">
          <thead class="sticky top-0 z-10 bg-surface-base border-b border-border-default">
            <tr>
              <th class="w-8 px-2 py-1.5">
                <input
                  type="checkbox"
                  :checked="allSelected"
                  :indeterminate="someSelected && !allSelected"
                  @change="allSelected ? consolidation.deselectAll() : consolidation.selectAll()" />
              </th>
              <th class="text-[10px] uppercase tracking-wider text-text-muted font-semibold px-3 py-1.5 text-left">Item</th>
              <th class="text-[10px] uppercase tracking-wider text-text-muted font-semibold px-3 py-1.5 text-right tabular-nums">Total Qty</th>
              <th class="text-[10px] uppercase tracking-wider text-text-muted font-semibold px-3 py-1.5 text-right">Locations</th>
              <th class="text-[10px] uppercase tracking-wider text-text-muted font-semibold px-3 py-1.5 text-left">Target</th>
            </tr>
          </thead>
          <tbody>
            <template v-for="candidate in filteredCandidates" :key="candidate.itemName">
              <tr
                class="border-b border-border-default/50 hover:bg-surface-row-hover cursor-pointer"
                :class="consolidation.isSelected(candidate.itemName) ? 'bg-accent-gold/5' : ''"
                @click="consolidation.toggleItem(candidate.itemName)">
                <td class="px-2 py-2">
                  <input type="checkbox" :checked="consolidation.isSelected(candidate.itemName)" @click.stop />
                </td>
                <td class="px-3 py-2">
                  <ItemInline :reference="candidate.itemName" />
                </td>
                <td class="px-3 py-2 text-right tabular-nums font-medium text-text-primary">{{ candidate.totalQuantity.toLocaleString() }}</td>
                <td class="px-3 py-2 text-right">
                  <button
                    class="text-text-muted hover:text-text-primary cursor-pointer text-xs"
                    @click.stop="toggleExpand(candidate.itemName)">
                    {{ candidate.locations.length }} {{ candidate.locations.length === 1 ? 'loc' : 'locs' }}
                    <span class="ml-0.5">{{ expandedItems.has(candidate.itemName) ? '\u25BC' : '\u25B6' }}</span>
                  </button>
                </td>
                <td class="px-3 py-2 text-text-muted text-xs">{{ candidate.targetDisplayName }}</td>
              </tr>
              <!-- Expanded per-location breakdown -->
              <template v-if="expandedItems.has(candidate.itemName)">
                <tr v-for="loc in candidate.locations" :key="`${candidate.itemName}-${loc.vaultKey}`" class="bg-surface-inset/30">
                  <td></td>
                  <td class="px-3 py-1 pl-8 text-text-muted">
                    {{ loc.displayName }}
                    <span v-if="loc.vaultKey === candidate.targetVaultKey" class="text-accent-gold text-[10px] ml-1">(target)</span>
                  </td>
                  <td class="px-3 py-1 text-right tabular-nums text-text-secondary">{{ loc.quantity.toLocaleString() }}</td>
                  <td colspan="2"></td>
                </tr>
              </template>
            </template>
          </tbody>
        </table>
      </div>

      <!-- Action bar -->
      <div v-if="consolidation.selectedItems.value.size > 0" class="flex items-center gap-4 border-t border-border-default pt-3">
        <span class="text-xs text-text-secondary">
          {{ consolidation.selectedItems.value.size }} items selected
        </span>

        <button
          class="btn btn-primary text-xs ml-auto"
          :disabled="planning || consolidation.selectedItems.value.size === 0"
          @click="planRoute">
          {{ planning ? 'Planning...' : 'Plan Route' }}
        </button>
      </div>

      <!-- Route display -->
      <div v-if="route" class="border border-border-default rounded-lg p-3">
        <div class="flex items-center justify-between mb-2">
          <h3 class="section-heading m-0">Planned Route</h3>
          <div class="flex gap-3 text-xs text-text-muted">
            <span>{{ route.steps.length }} steps</span>
            <span>{{ route.total_hops }} hop{{ route.total_hops !== 1 ? 's' : '' }}</span>
          </div>
        </div>
        <div class="flex flex-col gap-0.5 max-h-48 overflow-y-auto pr-1">
          <div
            v-for="(step, i) in route.steps"
            :key="i"
            class="flex items-start gap-2 py-1 px-1.5 rounded text-xs"
            :class="step.action === 'travel' ? 'bg-surface-elevated/50' : 'bg-accent-gold/5'">
            <span class="shrink-0 w-4 text-text-muted text-[10px] text-right mt-0.5">{{ i + 1 }}</span>
            <span :class="step.action === 'travel' ? 'text-text-dim italic' : 'text-text-primary'">
              {{ step.details }}
            </span>
          </div>
        </div>
      </div>

      <div v-if="routeError" class="text-accent-red text-xs">{{ routeError }}</div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useGameStateStore } from '../../stores/gameStateStore'
import { useStorageConsolidation } from '../../composables/useStorageConsolidation'
import EmptyState from '../Shared/EmptyState.vue'
import ItemInline from '../Shared/Item/ItemInline.vue'

const gameState = useGameStateStore()
const consolidation = useStorageConsolidation()

const search = ref('')
const expandedItems = reactive(new Set<string>())
const planning = ref(false)
const routeError = ref('')

interface PlannedRoute {
  steps: { zone: string; action: string; details: string }[]
  total_hops: number
}
const route = ref<PlannedRoute | null>(null)

// Initialize selection on mount
onMounted(() => {
  consolidation.selectAll()
  gameState.loadStorageVaults()
})

// Filtered candidates
const filteredCandidates = computed(() => {
  const q = search.value.trim().toLowerCase()
  if (!q) return consolidation.allCandidates.value
  return consolidation.allCandidates.value.filter(c => c.itemName.toLowerCase().includes(q))
})

// Selection state for header checkbox
const allSelected = computed(() =>
  filteredCandidates.value.length > 0 &&
  filteredCandidates.value.every(c => consolidation.isSelected(c.itemName))
)
const someSelected = computed(() =>
  filteredCandidates.value.some(c => consolidation.isSelected(c.itemName))
)

function toggleExpand(itemName: string) {
  if (expandedItems.has(itemName)) expandedItems.delete(itemName)
  else expandedItems.add(itemName)
}

async function planRoute() {
  planning.value = true
  routeError.value = ''
  route.value = null

  try {
    // Default start zone — use Serbule as fallback
    const areaObj = gameState.world?.area as { area_name?: string } | null
    const startZone = areaObj?.area_name ?? 'AreaSerbule'

    // Load travel config from localStorage (same key as trip planner widget)
    const configStr = localStorage.getItem('tripPlannerWidget.config')
    const config = configStr ? JSON.parse(configStr) : {}

    const travelConfig = {
      primaryBind: config.primaryBind ?? null,
      secondaryBind: config.secondaryBind ?? null,
      mushroomCircle1: config.mushroomCircle ?? null,
      mushroomCircle2: null,
      useTpMachine: config.useTpMachine ?? false,
      casinoPortal: null,
    }

    // Convert our route stops to the format plan_trip expects
    const stops = consolidation.routeStops.value.map(s => ({
      zone: s.zone,
      purpose: s.purpose,
      details: s.details,
    }))

    if (stops.length === 0) {
      routeError.value = 'No route stops generated — check that selected items have valid vault locations.'
      return
    }

    route.value = await invoke<PlannedRoute>('plan_trip', {
      startZone,
      stops,
      travelConfig,
    })
  } catch (e) {
    routeError.value = String(e)
  } finally {
    planning.value = false
  }
}
</script>
