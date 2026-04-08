<template>
  <div class="flex flex-col gap-4 h-full">
    <div class="flex items-center justify-between">
      <TabBar v-model="activeTab" :tabs="tabs" class="flex-1" />
      <button
        v-if="store.sales.length > 0 || store.shopLog.length > 0"
        class="text-xs text-text-dim hover:text-accent-red transition-colors px-2 py-1 shrink-0"
        @click="handleClear">
        Clear data
      </button>
    </div>
    <div class="flex-1 min-h-0 overflow-auto">
      <StallSalesTab v-if="activeTab === 'sales'" />
      <StallShopLogTab v-else-if="activeTab === 'shop-log'" />
      <StallRevenueTab v-else-if="activeTab === 'revenue'" />
      <StallInventoryTab v-else-if="activeTab === 'inventory'" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { confirm } from '@tauri-apps/plugin-dialog'
import TabBar from '../Shared/TabBar.vue'
import StallSalesTab from './StallSalesTab.vue'
import StallShopLogTab from './StallShopLogTab.vue'
import StallRevenueTab from './StallRevenueTab.vue'
import StallInventoryTab from './StallInventoryTab.vue'
import { useStallTrackerStore } from '../../stores/stallTrackerStore'

const tabs = [
  { id: 'sales', label: 'Sales' },
  { id: 'shop-log', label: 'Shop Log' },
  { id: 'revenue', label: 'Revenue' },
  { id: 'inventory', label: 'Inventory' },
]

const activeTab = ref('sales')
const store = useStallTrackerStore()

onMounted(() => {
  store.loadAll()
})

async function handleClear() {
  const ok = await confirm(
    'This will delete all stall tracker data (sales and shop log entries). You can re-import by opening your shop log books in-game.',
    { title: 'Clear Stall Data', kind: 'warning' },
  )
  if (ok) {
    await store.clearAll()
  }
}
</script>
