<template>
  <div class="bg-surface-card border border-border-default rounded p-3">
    <div class="text-[0.65rem] uppercase tracking-widest text-[#7ec8e3] font-bold mb-2">
      Survey Type Distribution
    </div>
    <div v-if="chartDataset.length > 0" class="h-70">
      <VueUiDonut :dataset="chartDataset" :config="donutConfig" />
    </div>
    <div v-else class="text-text-dim italic text-xs">No survey type data to chart.</div>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { VueUiDonut } from "vue-data-ui";
import type { VueUiDonutConfig, VueUiDonutDatasetItem } from "vue-data-ui";
import type { SurveyTypeAnalytics } from "../../../types/database";

const props = defineProps<{
  surveyTypes: SurveyTypeAnalytics[];
}>();

const chartDataset = computed<VueUiDonutDatasetItem[]>(() => {
  const sorted = [...props.surveyTypes]
    .filter(st => st.total_completed > 0)
    .sort((a, b) => b.total_completed - a.total_completed);

  return sorted.map((st) => ({
    name: st.survey_type.replace(/\s*Survey$/, ""),
    color: st.category === "mineral" ? "#7ec8e3" : "#c87e7e",
    values: [st.total_completed],
  }));
});

const donutConfig = computed<VueUiDonutConfig>(() => ({
  responsive: true,
  useCssAnimation: true,
  useBlurOnHover: false,
  style: {
    fontFamily: "inherit",
    chart: {
      backgroundColor: "transparent",
      color: "#a1a1aa",
      layout: {
        labels: {
          dataLabels: {
            show: true,
            hideUnderValue: 3,
          },
          percentage: {
            show: true,
            color: "#a1a1aa",
            bold: true,
            fontSize: 10,
            rounding: 1,
          },
          name: {
            show: true,
            color: "#d4d4d8",
            bold: false,
            fontSize: 10,
          },
          value: {
            show: false,
          },
          hollow: {
            show: true,
            total: {
              show: true,
              bold: true,
              fontSize: 14,
              color: "#d4d4d8",
              text: "Surveys",
              value: {
                color: "#e4e4e7",
                fontSize: 16,
                bold: true,
                rounding: 0,
              },
            },
            average: { show: false },
          },
        },
        donut: {
          strokeWidth: 64,
          borderWidth: 1,
          useShadow: false,
        },
      },
      legend: {
        show: false,
      },
      title: {
        text: "",
      },
      tooltip: {
        show: true,
        showValue: true,
        showPercentage: true,
        roundingValue: 0,
        roundingPercentage: 1,
        backgroundColor: "#27272a",
        color: "#d4d4d8",
        borderColor: "#3f3f46",
        borderWidth: 1,
        borderRadius: 4,
        fontSize: 12,
      },
    },
  },
  userOptions: { show: false },
  table: { show: false },
}));
</script>
