<template>
  <div class="flex flex-col h-full min-h-0">
    <!-- Empty states -->
    <EmptyState v-if="!gameState.storage.length" variant="panel" primary="No storage data" secondary="Import an inventory report to see stored items." />
    <EmptyState v-else-if="plan.moves.length === 0" variant="panel" primary="Nothing to consolidate" secondary="All your items are already in single locations." />

    <template v-else>
      <!-- Summary bar -->
      <div class="shrink-0 flex items-center gap-4 mb-3 flex-wrap">
        <div class="flex gap-5 text-sm">
          <div>
            <span class="text-accent-gold font-bold text-lg tabular-nums">{{ plan.slotsSaved }}</span>
            <span class="text-text-muted text-xs ml-1">slots saveable</span>
          </div>
          <div>
            <span class="text-text-primary font-bold text-lg tabular-nums">{{ plan.itemsToMove }}</span>
            <span class="text-text-muted text-xs ml-1">items to move</span>
          </div>
          <div>
            <span class="text-text-primary font-bold text-lg tabular-nums">{{ plan.zonesInvolved }}</span>
            <span class="text-text-muted text-xs ml-1">zones</span>
          </div>
        </div>

        <div class="ml-auto flex items-center gap-2">
          <template v-if="!consolidation.wizardActive.value">
            <button class="btn btn-primary text-xs" @click="consolidation.startWizard()">
              Start Wizard
            </button>
            <button
              class="btn btn-secondary text-xs"
              :disabled="planning"
              @click="planRoute">
              {{ planning ? 'Planning...' : 'Plan Route' }}
            </button>
          </template>
          <template v-else>
            <div class="flex items-center gap-2 text-xs">
              <div class="w-32 h-1.5 bg-surface-elevated rounded-full overflow-hidden">
                <div class="h-full bg-accent-gold rounded-full transition-all" :style="{ width: `${progressPct}%` }" />
              </div>
              <span class="text-text-muted tabular-nums">{{ consolidation.completedCount.value }}/{{ consolidation.totalCount.value }}</span>
            </div>
            <button class="btn btn-secondary text-xs" @click="consolidation.stopWizard()">Exit Wizard</button>
          </template>
        </div>
      </div>

      <!-- Route result banner -->
      <div v-if="route" class="shrink-0 mb-3 p-2 rounded bg-accent-gold/10 border border-accent-gold/30 flex items-center gap-3 text-xs">
        <span class="text-accent-gold font-semibold">Route:</span>
        <span class="text-text-primary">{{ route.total_hops }} hops across {{ plan.zonesInvolved }} zones</span>
        <button class="ml-auto text-text-muted hover:text-text-primary cursor-pointer" @click="showRouteSteps = !showRouteSteps">
          {{ showRouteSteps ? 'Hide' : 'Show steps' }}
        </button>
      </div>
      <div v-if="route && showRouteSteps" class="shrink-0 mb-3 border border-border-default rounded overflow-hidden max-h-32 overflow-y-auto">
        <div
          v-for="(step, i) in route.steps" :key="i"
          class="flex items-start gap-2 py-1 px-2 text-xs border-b border-border-default/30 last:border-b-0"
          :class="step.action === 'travel' ? 'bg-surface-elevated/30' : ''">
          <span class="shrink-0 w-4 text-text-muted text-[10px] text-right mt-0.5">{{ i + 1 }}</span>
          <span :class="step.action === 'travel' ? 'text-text-dim italic' : 'text-text-primary'">{{ step.details }}</span>
        </div>
      </div>
      <div v-if="routeError" class="shrink-0 mb-3 text-accent-red text-xs">{{ routeError }}</div>

      <!-- Wizard: current zone highlight -->
      <div v-if="consolidation.wizardActive.value && consolidation.currentZoneStop.value" class="shrink-0 mb-3 p-3 rounded-lg border-2 border-accent-gold/50 bg-accent-gold/5">
        <div class="flex items-center gap-2 mb-2">
          <span class="text-accent-gold text-xs font-bold uppercase tracking-wider">You are here</span>
          <span class="text-text-primary font-semibold">{{ consolidation.currentZoneStop.value.areaName }}</span>
        </div>
        <div class="grid grid-cols-2 gap-4">
          <!-- Pickups: items to carry OUT of this zone -->
          <div v-if="consolidation.currentZoneStop.value.pickups.length">
            <div class="micro-label mb-1 text-value-positive">Pick Up to Carry</div>
            <div class="flex flex-col gap-0.5">
              <label
                v-for="move in consolidation.currentZoneStop.value.pickups" :key="`p-${move.itemName}-${move.fromVaultKey}`"
                class="flex items-center gap-2 text-xs py-0.5 cursor-pointer hover:bg-surface-row-hover rounded px-1"
                :class="move.completed ? 'opacity-50' : ''">
                <input type="checkbox" :checked="move.completed" @change="consolidation.toggleMoveCompleted(move)" />
                <span class="flex-1 min-w-0"><ItemInline :reference="move.itemName" /></span>
                <span class="tabular-nums text-text-secondary shrink-0">x{{ move.quantity }}</span>
                <span class="text-text-dim text-[10px] shrink-0">{{ move.fromVaultName }}</span>
              </label>
            </div>
          </div>
          <!-- Dropoffs: items arriving FROM another zone -->
          <div v-if="consolidation.currentZoneStop.value.dropoffs.length">
            <div class="micro-label mb-1 text-value-negative">Drop Off (from travel)</div>
            <div class="flex flex-col gap-0.5">
              <label
                v-for="move in consolidation.currentZoneStop.value.dropoffs" :key="`d-${move.itemName}-${move.toVaultKey}`"
                class="flex items-center gap-2 text-xs py-0.5 cursor-pointer hover:bg-surface-row-hover rounded px-1"
                :class="move.completed ? 'opacity-50' : ''">
                <input type="checkbox" :checked="move.completed" @change="consolidation.toggleMoveCompleted(move)" />
                <span class="flex-1 min-w-0"><ItemInline :reference="move.itemName" /></span>
                <span class="tabular-nums text-text-secondary shrink-0">x{{ move.quantity }}</span>
                <span class="text-text-dim text-[10px] shrink-0">&rarr; {{ move.toVaultName }}</span>
              </label>
            </div>
          </div>
        </div>
        <!-- Local rearrangement: vault-to-vault within this zone -->
        <div v-if="consolidation.currentZoneStop.value.localMoves.length" class="mt-2 pt-2 border-t border-border-default/50">
          <div class="micro-label mb-1 text-accent-blue">Rearrange Locally</div>
          <div class="flex flex-col gap-0.5">
            <label
              v-for="move in consolidation.currentZoneStop.value.localMoves" :key="`l-${move.itemName}-${move.fromVaultKey}`"
              class="flex items-center gap-2 text-xs py-0.5 cursor-pointer hover:bg-surface-row-hover rounded px-1"
              :class="move.completed ? 'opacity-50' : ''">
              <input type="checkbox" :checked="move.completed" @change="consolidation.toggleMoveCompleted(move)" />
              <span class="flex-1 min-w-0"><ItemInline :reference="move.itemName" /></span>
              <span class="tabular-nums text-text-secondary shrink-0">x{{ move.quantity }}</span>
              <span class="text-text-dim text-[10px] shrink-0">{{ move.fromVaultName }} &rarr; {{ move.toVaultName }}</span>
            </label>
          </div>
        </div>
      </div>

      <!-- Zone-by-zone plan (main content) -->
      <div class="flex-1 min-h-0 overflow-y-auto pr-1">
        <div class="grid grid-cols-1 lg:grid-cols-2 xl:grid-cols-3 gap-3">
          <div
            v-for="zs in displayedZoneStops" :key="zs.areaKey"
            class="card p-0 overflow-hidden"
            :class="zs.completed ? 'opacity-40' : ''">
            <!-- Zone header -->
            <div class="flex items-center gap-2 px-3 py-2 bg-surface-base/50 border-b border-border-default">
              <div class="flex-1 min-w-0">
                <div class="text-xs font-semibold text-text-primary">{{ zs.areaName }}</div>
              </div>
              <span class="text-[10px] text-text-muted shrink-0">
                {{ zs.pickups.length + zs.dropoffs.length + zs.localMoves.length }} actions
              </span>
            </div>

            <div class="divide-y divide-border-default/30">
              <!-- Pickups: carry out of this zone -->
              <div v-if="zs.pickups.length" class="px-3 py-1.5">
                <div class="micro-label mb-1 text-value-positive">Pick Up to Carry</div>
                <label
                  v-for="move in zs.pickups" :key="`p-${move.itemName}-${move.fromVaultKey}`"
                  class="flex items-center gap-1.5 text-xs py-0.5 cursor-pointer"
                  :class="move.completed ? 'opacity-40 line-through' : ''">
                  <input type="checkbox" :checked="move.completed" @change="consolidation.toggleMoveCompleted(move)" />
                  <span class="flex-1 min-w-0 truncate"><ItemInline :reference="move.itemName" /></span>
                  <span class="tabular-nums text-text-secondary shrink-0">x{{ move.quantity }}</span>
                  <span class="text-text-dim text-[10px] shrink-0 truncate max-w-20">{{ move.fromVaultName }}</span>
                </label>
              </div>

              <!-- Dropoffs: deposit items from travel -->
              <div v-if="zs.dropoffs.length" class="px-3 py-1.5">
                <div class="micro-label mb-1 text-value-negative">Drop Off (from travel)</div>
                <label
                  v-for="move in zs.dropoffs" :key="`d-${move.itemName}-${move.toVaultKey}`"
                  class="flex items-center gap-1.5 text-xs py-0.5 cursor-pointer"
                  :class="move.completed ? 'opacity-40 line-through' : ''">
                  <input type="checkbox" :checked="move.completed" @change="consolidation.toggleMoveCompleted(move)" />
                  <span class="flex-1 min-w-0 truncate"><ItemInline :reference="move.itemName" /></span>
                  <span class="tabular-nums text-text-secondary shrink-0">x{{ move.quantity }}</span>
                  <span class="text-text-dim text-[10px] shrink-0 truncate max-w-20">&rarr; {{ move.toVaultName }}</span>
                </label>
              </div>

              <!-- Local: rearrange between vaults in this zone -->
              <div v-if="zs.localMoves.length" class="px-3 py-1.5">
                <div class="micro-label mb-1 text-accent-blue">Rearrange Locally</div>
                <label
                  v-for="move in zs.localMoves" :key="`l-${move.itemName}-${move.fromVaultKey}`"
                  class="flex items-center gap-1.5 text-xs py-0.5 cursor-pointer"
                  :class="move.completed ? 'opacity-40 line-through' : ''">
                  <input type="checkbox" :checked="move.completed" @change="consolidation.toggleMoveCompleted(move)" />
                  <span class="flex-1 min-w-0 truncate"><ItemInline :reference="move.itemName" /></span>
                  <span class="tabular-nums text-text-secondary shrink-0">x{{ move.quantity }}</span>
                  <span class="text-text-dim text-[10px] shrink-0 truncate max-w-20">{{ move.fromVaultName }} &rarr; {{ move.toVaultName }}</span>
                </label>
              </div>
            </div>
          </div>
        </div>
      </div>
    </template>
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

const planning = ref(false)
const routeError = ref('')
const showRouteSteps = ref(false)

interface PlannedRoute {
  steps: { zone: string; action: string; details: string }[]
  total_hops: number
}
const route = ref<PlannedRoute | null>(null)

onMounted(() => gameState.loadStorageVaults())

const plan = computed(() => consolidation.plan.value)

const progressPct = computed(() => {
  if (consolidation.totalCount.value === 0) return 0
  return Math.round((consolidation.completedCount.value / consolidation.totalCount.value) * 100)
})

/** In wizard mode, show current zone separately (highlighted above), so filter it out of the main grid */
const displayedZoneStops = computed(() => {
  if (!consolidation.wizardActive.value) return plan.value.zoneStops
  const currentArea = consolidation.currentZone.value
  return plan.value.zoneStops.filter(zs => zs.areaKey !== currentArea)
})

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
    showRouteSteps.value = false
  } catch (e) {
    routeError.value = String(e)
  } finally {
    planning.value = false
  }
}
</script>
