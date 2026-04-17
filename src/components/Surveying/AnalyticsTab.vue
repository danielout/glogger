<template>
  <PaneLayout
    screen-key="surveying-analytics"
    :left-pane="{
      title: 'Analytics',
      defaultWidth: 240,
      minWidth: 200,
      maxWidth: 360,
    }"
    :right-pane="{
      title: 'Item Cost Calculator',
      defaultWidth: 420,
      minWidth: 340,
      maxWidth: 640,
      defaultCollapsed: false,
    }"
  >
    <!-- LEFT: view picker -->
    <template #left>
      <AnalyticsNav
        :zones="store.analytics?.zones ?? []"
        :survey-types="store.analytics?.survey_types ?? []"
        :view="view"
        @update:view="view = $event"
      />
    </template>

    <!-- CENTER: selected view -->
    <div class="h-full overflow-y-auto">
      <!-- Empty state: no analytics yet -->
      <div
        v-if="!hasData"
        class="h-full flex flex-col items-center justify-center gap-2 text-text-dim text-xs italic p-6 text-center"
      >
        <p>No survey data yet.</p>
        <p>Complete a survey on the Session tab and come back — analytics populate automatically.</p>
      </div>

      <div v-else class="p-4 flex flex-col gap-3">
        <!-- Header with refresh -->
        <header class="flex items-center justify-between">
          <div class="flex items-baseline gap-3">
            <h2 class="text-sm text-text-primary font-semibold">Survey Analytics</h2>
            <span class="text-[0.65rem] text-text-dim">
              {{ store.analytics?.total_uses.toLocaleString() }} surveys across
              {{ store.analytics?.zones.length }} zone{{ store.analytics?.zones.length === 1 ? '' : 's' }}
            </span>
          </div>
          <button
            class="px-2.5 py-1 rounded border border-border-default bg-surface-elevated text-xs text-text-secondary hover:text-text-primary hover:border-border-hover transition-colors"
            @click="refresh"
          >
            Refresh
          </button>
        </header>

        <!-- Views -->
        <AnalyticsOverviewView
          v-if="view.kind === 'overview' && store.analytics"
          :analytics="store.analytics"
          @select-zone="onSelectZone"
          @select-type="onSelectType"
        />
        <AnalyticsZoneView
          v-else-if="view.kind === 'zone' && currentZone && store.analytics"
          :zone="currentZone"
          :analytics="store.analytics"
        />
        <AnalyticsTypeView
          v-else-if="view.kind === 'type' && currentType && store.analytics"
          :type="currentType"
          :analytics="store.analytics"
        />
      </div>
    </div>

    <!-- RIGHT: calculator -->
    <template #right>
      <div class="h-full overflow-y-auto p-3">
        <ItemCostCalculator ref="calcRef" />
      </div>
    </template>
  </PaneLayout>
</template>

<script setup lang="ts">
// Survey Analytics — PaneLayout-based rebuild. The left pane holds
// Overview / Zone / Type navigation; the center pane renders the
// selected view; the right pane holds the Item Cost Calculator tool
// that lets users rank survey types by cost, time, or profit per hour
// for a given item and target quantity.
//
// Sub-components (built for this tab):
//   - AnalyticsNav              — left pane buttons
//   - AnalyticsOverviewView     — metric cards + cross-zone + all-types
//   - AnalyticsZoneView         — per-type cards for a single zone
//   - AnalyticsTypeView         — item breakdown for a single type
//   - CrossZoneComparison       — sortable per-zone rollup table
//   - ZoneRewardsCard           — reusable per-type item table
//   - ItemCostCalculator        — right-pane tool
//   - StatCard                  — standard label/value/sub tile
//
// The store's `analytics` cache is the single source of truth for the
// center views; the calculator has its own cache via a separate
// backend command (`survey_tracker_item_cost_analysis`).
import { computed, onMounted, ref } from 'vue'
import PaneLayout from '../Shared/PaneLayout.vue'
import AnalyticsNav, { type AnalyticsView } from './AnalyticsNav.vue'
import AnalyticsOverviewView from './AnalyticsOverviewView.vue'
import AnalyticsZoneView from './AnalyticsZoneView.vue'
import AnalyticsTypeView from './AnalyticsTypeView.vue'
import ItemCostCalculator from './ItemCostCalculator.vue'
import { useSurveyTrackerStore } from '../../stores/surveyTrackerStore'

const store = useSurveyTrackerStore()

const view = ref<AnalyticsView>({ kind: 'overview' })
const calcRef = ref<InstanceType<typeof ItemCostCalculator> | null>(null)

const hasData = computed(() => (store.analytics?.total_uses ?? 0) > 0)

const currentZone = computed(() => {
  const v = view.value
  if (v.kind !== 'zone') return null
  return store.analytics?.zones.find(z => z.area === v.area) ?? null
})

const currentType = computed(() => {
  const v = view.value
  if (v.kind !== 'type') return null
  return (
    store.analytics?.survey_types.find(
      t => t.map_internal_name === v.map && t.area === v.area,
    ) ?? null
  )
})

function onSelectZone(area: string) {
  view.value = { kind: 'zone', area }
}
function onSelectType(payload: { map: string; area: string | null }) {
  view.value = { kind: 'type', ...payload }
}

async function refresh() {
  await store.refreshAnalytics()
  // Calculator pulls from a different backend command, so refresh both.
  calcRef.value?.reload()
}

onMounted(async () => {
  await store.refreshAnalytics()
})
</script>
