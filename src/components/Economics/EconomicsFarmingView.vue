<template>
  <div class="flex flex-col gap-4 h-full">
    <TabBar v-model="activeTab" :tabs="tabs" />
    <div class="flex-1 min-h-0">
      <div v-if="activeTab === 'session'" class="flex flex-col gap-4 h-full">
        <FarmingSessionCard />
      </div>
      <HistoricalTab v-else-if="activeTab === 'historical'" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import TabBar from '../Shared/TabBar.vue'
import FarmingSessionCard from '../Farming/FarmingSessionCard.vue'
import HistoricalTab from '../Farming/HistoricalTab.vue'

const tabs = [
  { id: 'session', label: 'Active Session' },
  { id: 'historical', label: 'Session History' },
]

const activeTab = ref('session')
</script>
