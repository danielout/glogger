<template>
  <div class="flex flex-col gap-3">
    <!-- Category toggle -->
    <div class="flex gap-1 bg-surface-elevated border border-border-default rounded p-0.5 w-fit">
      <button
        @click="category = 'mineral'"
        :class="[
          'px-2.5 py-1 text-xs rounded transition-all',
          category === 'mineral'
            ? 'bg-[#7ec8e3]/20 text-[#7ec8e3] font-semibold'
            : 'text-text-muted hover:text-text-secondary'
        ]"
      >Mineral</button>
      <button
        @click="category = 'mining'"
        :class="[
          'px-2.5 py-1 text-xs rounded transition-all',
          category === 'mining'
            ? 'bg-[#c87e7e]/20 text-[#c87e7e] font-semibold'
            : 'text-text-muted hover:text-text-secondary'
        ]"
      >Mining</button>
    </div>

    <!-- Chart -->
    <div v-if="chartData.length > 0" class="h-[250px]">
      <VueUiXy :dataset="chartDataset" :config="chartConfig" />
    </div>

    <div v-else class="text-text-dim italic text-xs">
      No {{ category }} survey data available across zones.
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import { VueUiXy } from "vue-data-ui";
import type { VueUiXyDatasetItem, VueUiXyConfig } from "vue-data-ui";

interface CategorySpeedBonusStats {
  category: string;
  total_surveys: number;
  speed_bonus_count: number;
  speed_bonus_rate: number;
  avg_bonus_value: number;
  item_stats: unknown[];
}

interface ZoneAnalytics {
  zone: string;
  speed_bonus_stats: CategorySpeedBonusStats[];
  survey_type_stats: unknown[];
}

const props = defineProps<{
  zones: ZoneAnalytics[];
}>();

const category = ref<"mineral" | "mining">("mineral");

interface ChartZoneData {
  zone: string;
  bonusRate: number;
  totalSurveys: number;
}

const chartData = computed<ChartZoneData[]>(() => {
  const result: ChartZoneData[] = [];
  for (const zone of props.zones) {
    const cat = zone.speed_bonus_stats.find(s => s.category === category.value);
    if (cat && cat.total_surveys > 0) {
      result.push({
        zone: zone.zone,
        bonusRate: cat.speed_bonus_rate,
        totalSurveys: cat.total_surveys,
      });
    }
  }
  return result.sort((a, b) => b.bonusRate - a.bonusRate);
});

const chartDataset = computed<VueUiXyDatasetItem[]>(() => [{
  name: "Speed Bonus Rate",
  series: chartData.value.map(d => d.bonusRate),
  type: "bar",
  color: "#c8b47e",
  dataLabels: true,
  suffix: "%",
}]);

const chartConfig = computed<VueUiXyConfig>(() => ({
  responsive: true,
  useCssAnimation: true,
  chart: {
    fontFamily: "inherit",
    backgroundColor: "transparent",
    color: "#a1a1aa",
    height: 250,
    padding: { top: 24, right: 24, bottom: 48, left: 48 },
    grid: {
      stroke: "#27272a",
      showHorizontalLines: true,
      showVerticalLines: false,
      labels: {
        color: "#a1a1aa",
        show: true,
        fontSize: 10,
        yAxis: {
          commonScaleSteps: 5,
          labelWidth: 40,
          rounding: 1,
        },
        xAxisLabels: {
          color: "#a1a1aa",
          show: true,
          values: chartData.value.map(d => formatZone(d.zone)),
          fontSize: 10,
          rotation: chartData.value.length > 4 ? -30 : 0,
        },
      },
    },
    labels: {
      fontSize: 10,
      suffix: "%",
    },
    legend: { show: false },
    title: { show: false },
    tooltip: {
      show: true,
      backgroundColor: "#27272a",
      color: "#d4d4d8",
      borderColor: "#3f3f46",
      borderWidth: 1,
      borderRadius: 4,
      fontSize: 12,
    },
  },
  userOptions: { show: false },
  table: { show: false } as Record<string, unknown>,
} as VueUiXyConfig));

function formatZone(zone: string): string {
  return zone.replace(/([a-z])([A-Z])/g, "$1 $2");
}
</script>
