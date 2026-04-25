<template>
  <div class="flex flex-col h-full min-h-0">
    <!-- Top bar: controls -->
    <div class="shrink-0 flex items-center gap-3 mb-3 flex-wrap">
      <div class="relative">
        <input v-model="search" type="text" placeholder="Filter items..." class="input text-xs py-1 w-40 pr-7" />
        <button v-if="search" class="absolute right-1.5 top-1/2 -translate-y-1/2 text-text-muted hover:text-text-primary text-xs cursor-pointer" @click="search = ''">&times;</button>
      </div>

      <div class="flex items-center gap-1.5">
        <label class="text-xs text-text-muted">Consolidate to:</label>
        <select v-model="consolidation.targetStrategy.value" class="input text-xs py-1 w-36">
          <option value="most_items">Most at location</option>
          <option value="specific_vault">Specific vault</option>
        </select>
        <select v-if="consolidation.targetStrategy.value === 'specific_vault'" v-model="consolidation.specificVaultKey.value" class="input text-xs py-1 w-40">
          <option :value="null" disabled>Choose...</option>
          <option v-for="v in consolidation.availableVaults.value" :key="v.key" :value="v.key">{{ v.displayName }}</option>
        </select>
      </div>

      <div class="flex gap-2 text-xs">
        <button class="text-text-muted hover:text-text-primary cursor-pointer" @click="consolidation.selectAll()">All</button>
        <button class="text-text-muted hover:text-text-primary cursor-pointer" @click="consolidation.deselectAll()">None</button>
      </div>

      <div class="ml-auto flex items-center gap-3">
        <span class="text-xs text-text-muted">{{ consolidation.selectedItems.value.size }} items &middot; {{ uniqueStopCount }} stops</span>
        <button
          class="btn btn-primary text-xs"
          :disabled="planning || consolidation.selectedItems.value.size === 0"
          @click="planRoute">
          {{ planning ? 'Planning...' : 'Plan Route' }}
        </button>
      </div>
    </div>

    <!-- Empty states -->
    <EmptyState v-if="!gameState.storage.length" variant="panel" primary="No storage data" secondary="Import an inventory report to see stored items." />
    <EmptyState v-else-if="consolidation.candidateCount.value === 0" variant="panel" primary="Nothing to consolidate" secondary="All items are in single locations." />

    <!-- Main content: vault-grouped action plan -->
    <div v-else class="flex-1 min-h-0 overflow-y-auto pr-1">
      <!-- Route result banner -->
      <div v-if="route" class="mb-3 p-2 rounded bg-accent-gold/10 border border-accent-gold/30 flex items-center gap-3 text-xs">
        <span class="text-accent-gold font-semibold">Route planned:</span>
        <span class="text-text-primary">{{ route.steps.length }} steps</span>
        <span class="text-text-muted">&middot;</span>
        <span class="text-text-primary">{{ route.total_hops }} hops</span>
        <button class="ml-auto text-text-muted hover:text-text-primary cursor-pointer" @click="showRouteDetail = !showRouteDetail">
          {{ showRouteDetail ? 'Hide steps' : 'Show steps' }}
        </button>
      </div>

      <!-- Expandable route detail -->
      <div v-if="route && showRouteDetail" class="mb-3 border border-border-default rounded-lg overflow-hidden">
        <div
          v-for="(step, i) in route.steps"
          :key="i"
          class="flex items-start gap-2 py-1 px-2 text-xs border-b border-border-default/30 last:border-b-0"
          :class="step.action === 'travel' ? 'bg-surface-elevated/30' : ''">
          <span class="shrink-0 w-4 text-text-muted text-[10px] text-right mt-0.5">{{ i + 1 }}</span>
          <span :class="step.action === 'travel' ? 'text-text-dim italic' : 'text-text-primary'">{{ step.details }}</span>
        </div>
      </div>

      <div v-if="routeError" class="mb-3 text-accent-red text-xs">{{ routeError }}</div>

      <!-- Vault groups: the main content -->
      <div class="grid grid-cols-1 lg:grid-cols-2 gap-3">
        <div v-for="group in vaultGroups" :key="group.vaultKey" class="card p-0 overflow-hidden">
          <!-- Vault header -->
          <div class="flex items-center gap-2 px-3 py-2 bg-surface-base/50 border-b border-border-default">
            <input
              type="checkbox"
              :checked="isVaultFullySelected(group)"
              :indeterminate="isVaultPartiallySelected(group) && !isVaultFullySelected(group)"
              @change="toggleVaultSelection(group)" />
            <div class="flex-1 min-w-0">
              <div class="text-xs font-semibold text-text-primary truncate">{{ group.displayName }}</div>
              <div v-if="group.areaName" class="text-[10px] text-text-dim">{{ group.areaName }}</div>
            </div>
            <span class="text-[10px] text-text-muted shrink-0">{{ group.items.length }} items</span>
          </div>

          <!-- Items table -->
          <div class="divide-y divide-border-default/30">
            <div
              v-for="item in group.items"
              :key="item.itemName"
              class="flex items-center gap-2 px-3 py-1 text-xs hover:bg-surface-row-hover cursor-pointer"
              :class="consolidation.isSelected(item.itemName) ? 'bg-accent-gold/5' : ''"
              @click="consolidation.toggleItem(item.itemName)">
              <input type="checkbox" :checked="consolidation.isSelected(item.itemName)" class="shrink-0" @click.stop />
              <span class="flex-1 min-w-0 truncate"><ItemInline :reference="item.itemName" /></span>
              <span class="tabular-nums text-text-secondary shrink-0 w-12 text-right">x{{ item.quantity }}</span>
              <span class="text-text-dim shrink-0">&rarr;</span>
              <span class="text-text-muted truncate max-w-28 shrink-0 text-right">{{ item.targetName }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useGameStateStore } from '../../stores/gameStateStore'
import { useStorageConsolidation } from '../../composables/useStorageConsolidation'
import EmptyState from '../Shared/EmptyState.vue'
import ItemInline from '../Shared/Item/ItemInline.vue'

const gameState = useGameStateStore()
const consolidation = useStorageConsolidation()

const search = ref('')
const planning = ref(false)
const routeError = ref('')
const showRouteDetail = ref(false)

interface PlannedRoute {
  steps: { zone: string; action: string; details: string }[]
  total_hops: number
}
const route = ref<PlannedRoute | null>(null)

onMounted(() => {
  consolidation.selectAll()
  gameState.loadStorageVaults()
})

// ── Vault-grouped view ─────────────────────────────────────────────────────

interface VaultGroup {
  vaultKey: string
  displayName: string
  areaKey: string | null
  areaName: string | null
  items: { itemName: string; quantity: number; targetName: string }[]
}

const vaultGroups = computed<VaultGroup[]>(() => {
  const q = search.value.trim().toLowerCase()
  const groups = new Map<string, VaultGroup>()

  for (const candidate of consolidation.allCandidates.value) {
    if (q && !candidate.itemName.toLowerCase().includes(q)) continue

    for (const loc of candidate.locations) {
      if (loc.vaultKey === candidate.targetVaultKey) continue

      if (!groups.has(loc.vaultKey)) {
        groups.set(loc.vaultKey, {
          vaultKey: loc.vaultKey,
          displayName: loc.displayName,
          areaKey: loc.areaKey,
          areaName: loc.areaName,
          items: [],
        })
      }
      groups.get(loc.vaultKey)!.items.push({
        itemName: candidate.itemName,
        quantity: loc.quantity,
        targetName: candidate.targetDisplayName,
      })
    }
  }

  return [...groups.values()].sort((a, b) => b.items.length - a.items.length)
})

function isVaultFullySelected(group: VaultGroup): boolean {
  return group.items.every(i => consolidation.isSelected(i.itemName))
}

function isVaultPartiallySelected(group: VaultGroup): boolean {
  return group.items.some(i => consolidation.isSelected(i.itemName))
}

function toggleVaultSelection(group: VaultGroup) {
  const allSelected = isVaultFullySelected(group)
  for (const item of group.items) {
    const selected = consolidation.isSelected(item.itemName)
    if (allSelected && selected) consolidation.toggleItem(item.itemName)
    else if (!allSelected && !selected) consolidation.toggleItem(item.itemName)
  }
}

const uniqueStopCount = computed(() => {
  const zones = new Set<string>()
  for (const stop of consolidation.routeStops.value) zones.add(stop.zone)
  return zones.size
})

// ── Route planning ─────────────────────────────────────────────────────────

async function planRoute() {
  planning.value = true
  routeError.value = ''
  route.value = null

  try {
    const areaObj = gameState.world?.area as { area_name?: string } | null
    const startZone = areaObj?.area_name ?? 'AreaSerbule'

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

    const stops = consolidation.routeStops.value.map(s => ({
      zone: s.zone, purpose: s.purpose, details: s.details,
    }))

    if (stops.length === 0) {
      routeError.value = 'No routable stops — items may be in portable storage or unknown zones.'
      return
    }

    route.value = await invoke<PlannedRoute>('plan_trip', { startZone, stops, travelConfig })
    showRouteDetail.value = true
  } catch (e) {
    routeError.value = String(e)
  } finally {
    planning.value = false
  }
}
</script>
