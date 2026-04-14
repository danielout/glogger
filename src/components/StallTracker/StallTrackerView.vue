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

    <!-- Active tab content -->
    <div class="flex-1 min-h-0 overflow-hidden">
      <EmptyState
        v-if="!store.currentOwner"
        primary="No active character"
        secondary="Set an active character in Settings to view Stall Tracker data." />
      <template v-else>
        <StallSalesTab v-if="activeTab === 'sales'" />
        <StallRevenueTab v-else-if="activeTab === 'revenue'" />
        <StallInventoryTab v-else-if="activeTab === 'inventory'" />
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import TabBar, { type Tab } from '../Shared/TabBar.vue'
import EmptyState from '../Shared/EmptyState.vue'
import StallSalesTab from './StallSalesTab.vue'
import StallRevenueTab from './StallRevenueTab.vue'
import StallInventoryTab from './StallInventoryTab.vue'
import { useStallTrackerStore } from '../../stores/stallTrackerStore'

const store = useStallTrackerStore()

const tabs: Tab[] = [
  { id: 'sales', label: 'Sales' },
  { id: 'revenue', label: 'Revenue' },
  { id: 'inventory', label: 'Inventory' },
]
const activeTab = ref<string>('sales')

// Lazy: stats and filter options were preloaded at startup, but if the user
// somehow lands here before that's done, fire one refresh.
onMounted(() => {
  if (store.stats === null && store.currentOwner) {
    void store.loadStats()
    void store.loadFilterOptions()
  }
})
</script>
