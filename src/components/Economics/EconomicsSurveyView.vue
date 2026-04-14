<template>
  <div class="flex flex-col gap-4 h-full">
    <div class="p-3 bg-yellow-900/20 border border-yellow-700/40 rounded text-sm text-yellow-300">
      Surveying tracking is under active development. Loot counts may not be 100% accurate, and Povus/Vidaria surveys are not yet trackable.
    </div>
    <TabBar v-model="activeTab" :tabs="tabs" />
    <div class="flex-1 min-h-0">
      <SessionTab v-if="activeTab === 'session'" />
      <HistoricalTab v-else-if="activeTab === 'historical'" />
      <AnalyticsTab v-else-if="activeTab === 'analytics'" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import TabBar from '../Shared/TabBar.vue'
import SessionTab from '../Surveying/SessionTab.vue'
import HistoricalTab from '../Surveying/HistoricalTab.vue'
import AnalyticsTab from '../Surveying/AnalyticsTab.vue'

const tabs = [
  { id: 'session', label: 'Session' },
  { id: 'historical', label: 'Session History' },
  { id: 'analytics', label: 'Analytics' },
]

const activeTab = ref('session')
</script>
