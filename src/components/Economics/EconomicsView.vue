<template>
  <PaneLayout screen-key="economics">
    <div class="flex flex-col gap-4 h-full">
      <!-- Market Prices (default) -->
      <template v-if="activeTab === 'market'">
        <MarketView />
      </template>

      <!-- Farming -->
      <template v-else-if="activeTab === 'farming'">
        <FarmingView />
      </template>

      <!-- Surveying -->
      <template v-else-if="activeTab === 'surveying'">
        <SurveyView />
      </template>

      <!-- Stall Tracker (stub) -->
      <template v-else-if="activeTab === 'stall-tracker'">
        <EmptyState primary="Stall Tracker" secondary="Coming soon." />
      </template>
    </div>
  </PaneLayout>
</template>

<script setup lang="ts">
import PaneLayout from '../Shared/PaneLayout.vue'
import EmptyState from '../Shared/EmptyState.vue'
import MarketView from '../Market/MarketView.vue'
import FarmingView from './EconomicsFarmingView.vue'
import SurveyView from './EconomicsSurveyView.vue'

defineProps<{
  activeTab: string;
}>();
</script>
