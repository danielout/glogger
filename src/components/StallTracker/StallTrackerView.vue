<template>
  <div class="flex flex-col gap-4 h-full min-h-0">
    <!-- Tab bar + action row -->
    <div class="flex items-end justify-between gap-4 flex-shrink-0">
      <TabBar
        v-model="activeTab"
        :tabs="tabs" />

      <div class="flex items-center gap-3 pb-2">
        <span
          v-if="store.currentOwner"
          class="text-sm text-text-secondary"
          :title="`All Stall Tracker data is scoped to the active character (${store.currentOwner}). Switch character to view another stall.`">
          Viewing: <span class="text-entity-player">{{ store.currentOwner }}</span>
        </span>
      </div>
    </div>

    <!-- Active tab placeholder — real tab components land in Phases 6–9 -->
    <div class="flex-1 min-h-0 overflow-auto">
      <EmptyState
        v-if="!store.currentOwner"
        primary="No active character"
        secondary="Set an active character in Settings to view Stall Tracker data." />
      <EmptyState
        v-else-if="!store.hasData"
        primary="No sales recorded"
        secondary="Open your shop log book in-game to start tracking, or use Import to load an exported book file." />
      <div
        v-else
        class="p-4 text-text-secondary text-sm">
        <p class="mb-2">
          <strong class="text-text-primary">{{ activeTab }}</strong> tab — coming in Phase {{ phaseForTab[activeTab] }}.
        </p>
        <p>
          Stats: {{ store.stats?.total_sales ?? 0 }} sales,
          {{ formatNumber(store.stats?.total_revenue ?? 0) }}g revenue,
          {{ store.stats?.unique_buyers ?? 0 }} buyers,
          {{ store.stats?.unique_items ?? 0 }} items.
        </p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import TabBar, { type Tab } from '../Shared/TabBar.vue'
import EmptyState from '../Shared/EmptyState.vue'
import { useStallTrackerStore } from '../../stores/stallTrackerStore'

const store = useStallTrackerStore()

const tabs: Tab[] = [
  { id: 'sales', label: 'Sales' },
  { id: 'revenue', label: 'Revenue' },
  { id: 'inventory', label: 'Inventory' },
]
const activeTab = ref<string>('sales')

const phaseForTab: Record<string, number> = {
  sales: 6,
  revenue: 7,
  inventory: 8,
}

function formatNumber(n: number): string {
  return n.toLocaleString()
}

// Lazy: stats and filter options were preloaded at startup, but if the user
// somehow lands here before that's done, fire one refresh.
onMounted(() => {
  if (store.stats === null && store.currentOwner) {
    void store.loadStats()
    void store.loadFilterOptions()
  }
})
</script>
