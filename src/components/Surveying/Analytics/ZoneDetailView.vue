<template>
  <div class="flex flex-col gap-4">
    <!-- Zone summary stats -->
    <div class="grid grid-cols-2 xl:grid-cols-4 gap-2">
      <div class="bg-surface-card border border-border-default rounded px-3 py-2">
        <div class="text-[0.6rem] uppercase tracking-widest text-text-dim">Total Surveys</div>
        <div class="text-lg font-mono font-bold text-text-primary">{{ totalSurveys.toLocaleString() }}</div>
      </div>
      <div v-for="cat in zone.speed_bonus_stats" :key="cat.category"
           class="bg-surface-card border border-border-default rounded px-3 py-2">
        <div class="text-[0.6rem] uppercase tracking-widest"
             :class="cat.category === 'mineral' ? 'text-[#7ec8e3]' : 'text-[#c87e7e]'">
          {{ cat.category === 'mineral' ? 'Mineral' : 'Mining' }}
        </div>
        <div class="text-lg font-mono font-bold text-text-primary">{{ cat.total_surveys.toLocaleString() }}</div>
        <div class="text-[0.55rem] text-text-dim">
          {{ cat.speed_bonus_rate.toFixed(1) }}% bonus rate
        </div>
      </div>
      <div class="bg-surface-card border border-border-default rounded px-3 py-2">
        <div class="text-[0.6rem] uppercase tracking-widest text-text-dim">Survey Types</div>
        <div class="text-lg font-mono font-bold text-text-primary">{{ zone.survey_type_stats.length }}</div>
      </div>
    </div>

    <!-- Cards grid: survey type rewards + speed bonus + chart + profit -->
    <div class="grid grid-cols-1 xl:grid-cols-2 2xl:grid-cols-3 gap-3">
      <!-- Survey type rewards cards -->
      <ZoneRewardsCard
        v-for="st in zone.survey_type_stats"
        :key="st.survey_type"
        :survey-type="st"
      />

      <!-- Speed bonus cards (one per category with data) -->
      <ZoneSpeedBonusCard
        v-for="cat in categoriesWithBonusData"
        :key="cat.category"
        :category="cat"
      />

      <!-- Survey type distribution chart -->
      <SurveyTypeDistributionChart
        v-if="zone.survey_type_stats.length > 1"
        :survey-types="zone.survey_type_stats"
      />

      <!-- Profit rate card -->
      <ProfitRateCard :zone="zone" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import type { ZoneAnalytics } from "../../../types/database";
import ZoneRewardsCard from "./ZoneRewardsCard.vue";
import ZoneSpeedBonusCard from "./ZoneSpeedBonusCard.vue";
import SurveyTypeDistributionChart from "./SurveyTypeDistributionChart.vue";
import ProfitRateCard from "./ProfitRateCard.vue";

const props = defineProps<{
  zone: ZoneAnalytics;
}>();

const totalSurveys = computed(() =>
  props.zone.speed_bonus_stats.reduce((sum, c) => sum + c.total_surveys, 0)
);

const categoriesWithBonusData = computed(() =>
  props.zone.speed_bonus_stats.filter(c => c.item_stats.length > 0)
);
</script>
