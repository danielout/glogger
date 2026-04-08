<template>
  <div class="flex flex-col gap-4 h-full">
    <TabBar v-model="activeTab" :tabs="tabs" />
    <div class="flex-1 min-h-0 overflow-auto">
      <StallSalesTab v-if="activeTab === 'sales'" />
      <StallShopLogTab v-else-if="activeTab === 'shop-log'" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import TabBar from '../Shared/TabBar.vue'
import StallSalesTab from './StallSalesTab.vue'
import StallShopLogTab from './StallShopLogTab.vue'
import { useStallTrackerStore } from '../../stores/stallTrackerStore'

const tabs = [
  { id: 'sales', label: 'Sales' },
  { id: 'shop-log', label: 'Shop Log' },
]

const activeTab = ref('sales')
const store = useStallTrackerStore()

onMounted(() => {
  store.loadAll()
})
</script>
