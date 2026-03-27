<template>
  <div class="flex flex-col gap-4 h-full">
    <div class="flex-1 min-h-0">
      <div v-if="activeTab === 'session'" class="flex flex-col gap-4 h-full">
        <FarmingSessionCard />
      </div>
      <HistoricalTab v-else-if="activeTab === 'historical'" />
    </div>
  </div>
</template>

<script setup lang="ts">
import FarmingSessionCard from "./FarmingSessionCard.vue";
import HistoricalTab from "./HistoricalTab.vue";

defineProps<{
  activeTab: string;
}>();
</script>
