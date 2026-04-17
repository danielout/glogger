<template>
  <div class="flex flex-col gap-4">
    <div class="flex-1">
      <SessionTab v-if="activeTab === 'session'" />
      <HistoricalTab v-else-if="activeTab === 'historical'" />
      <AnalyticsTab v-else-if="activeTab === 'analytics'" />
    </div>
  </div>
</template>

<script setup lang="ts">
import SessionTab from "./SessionTab.vue";
import HistoricalTab from "./HistoricalTab.vue";
import AnalyticsTab from "./AnalyticsTab.vue";

defineProps<{
  activeTab: string;
}>();
</script>
