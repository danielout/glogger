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

    <!-- Sort controls -->
    <div v-if="zoneRows.length > 1" class="flex items-center gap-2">
      <span class="text-[0.6rem] text-text-muted uppercase tracking-wide">Sort by</span>
      <div class="flex gap-1 bg-surface-elevated border border-border-default rounded p-0.5">
        <button
          v-for="col in sortColumns"
          :key="col.key"
          @click="toggleSort(col.key)"
          :class="[
            'px-2 py-0.5 text-[0.6rem] rounded transition-all',
            sortBy === col.key
              ? 'bg-[#7ec8e3]/20 text-[#7ec8e3] font-semibold'
              : 'text-text-muted hover:text-text-secondary'
          ]"
        >
          {{ col.label }}
          <span v-if="sortBy === col.key">{{ sortAsc ? '↑' : '↓' }}</span>
        </button>
      </div>
    </div>

    <!-- Table -->
    <table v-if="zoneRows.length > 0" class="text-xs">
      <thead>
        <tr class="text-[0.6rem] uppercase tracking-wide text-text-muted font-bold">
          <th class="text-left py-1 px-2 font-bold">Zone</th>
          <th class="text-right py-1 px-2 font-bold">Surveys</th>
          <th class="text-right py-1 px-2 font-bold">Bonus Rate</th>
          <th class="text-right py-1 px-2 font-bold">Avg Bonus</th>
          <th class="text-right py-1 px-2 font-bold">Avg Cost</th>
          <th class="text-right py-1 px-2 font-bold">Types</th>
          <th class="text-right py-1 px-2 font-bold">Profit/Survey</th>
        </tr>
      </thead>
      <tbody>
        <tr
          v-for="row in zoneRows"
          :key="row.zone"
          class="bg-black/20 border-b border-border-default hover:bg-black/30"
        >
          <td class="py-1 px-2 text-text-primary font-semibold">{{ formatZone(row.zone) }}</td>
          <td class="text-right py-1 px-2 font-mono text-text-primary">{{ row.totalSurveys }}</td>
          <td class="text-right py-1 px-2 font-mono text-[#c8b47e]">{{ row.bonusRate.toFixed(1) }}%</td>
          <td class="text-right py-1 px-2 font-mono text-text-secondary">{{ row.avgBonusValue > 0 ? formatGold(row.avgBonusValue) : '-' }}</td>
          <td class="text-right py-1 px-2 font-mono text-text-secondary">{{ formatGold(row.avgCostPerSurvey) }}</td>
          <td class="text-right py-1 px-2 font-mono text-text-dim">{{ row.surveyTypeCount }}</td>
          <td class="text-right py-1 px-2 font-mono" :class="(row.profitIndicator ?? 0) >= 0 ? 'text-[#7ec87e]' : 'text-[#c87e7e]'">
            {{ row.profitIndicator !== null ? ((row.profitIndicator ?? 0) >= 0 ? '+' : '') + formatGold(row.profitIndicator ?? 0) : '-' }}
          </td>
        </tr>
      </tbody>
    </table>

    <div v-else class="text-text-dim italic text-xs">
      No {{ category }} survey data available across zones.
    </div>

    <div v-if="zoneRows.length > 0" class="text-[0.55rem] text-text-dim italic">
      Profit/Survey is estimated from average bonus value minus average crafting cost per survey.
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import type { ZoneAnalytics } from "../../../types/database";

const props = defineProps<{
  zones: ZoneAnalytics[];
}>();

const category = ref<"mineral" | "mining">("mineral");
const sortBy = ref<string>("totalSurveys");
const sortAsc = ref(false);

interface ZoneRow {
  zone: string;
  totalSurveys: number;
  bonusRate: number;
  avgBonusValue: number;
  avgCostPerSurvey: number;
  surveyTypeCount: number;
  profitIndicator: number | null;
}

const sortColumns = [
  { key: "totalSurveys", label: "Surveys" },
  { key: "bonusRate", label: "Bonus Rate" },
  { key: "avgCostPerSurvey", label: "Cost" },
  { key: "profitIndicator", label: "Profit" },
];

function toggleSort(key: string) {
  if (sortBy.value === key) {
    sortAsc.value = !sortAsc.value;
  } else {
    sortBy.value = key;
    sortAsc.value = false;
  }
}

const zoneRows = computed<ZoneRow[]>(() => {
  const rows: ZoneRow[] = [];

  for (const zone of props.zones) {
    const catStats = zone.speed_bonus_stats.find(s => s.category === category.value);
    if (!catStats || catStats.total_surveys === 0) continue;

    const catSurveyTypes = zone.survey_type_stats.filter(st => st.category === category.value);

    // Weighted average cost
    const totalCost = catSurveyTypes.reduce((sum, st) => sum + st.crafting_cost * st.total_completed, 0);
    const totalCompleted = catSurveyTypes.reduce((sum, st) => sum + st.total_completed, 0);
    const avgCost = totalCompleted > 0 ? totalCost / totalCompleted : 0;

    // Profit indicator: avg bonus value per proc * proc rate - avg cost
    // This gives an expected "bonus value per survey" minus cost
    const expectedBonusPerSurvey = catStats.avg_bonus_value * (catStats.speed_bonus_rate / 100);
    const profitIndicator = expectedBonusPerSurvey > 0 || avgCost > 0
      ? expectedBonusPerSurvey - avgCost
      : null;

    rows.push({
      zone: zone.zone,
      totalSurveys: catStats.total_surveys,
      bonusRate: catStats.speed_bonus_rate,
      avgBonusValue: catStats.avg_bonus_value,
      avgCostPerSurvey: avgCost,
      surveyTypeCount: catSurveyTypes.length,
      profitIndicator,
    });
  }

  // Sort
  rows.sort((a, b) => {
    const key = sortBy.value as keyof ZoneRow;
    const aVal = (a[key] as number) ?? 0;
    const bVal = (b[key] as number) ?? 0;
    return sortAsc.value ? aVal - bVal : bVal - aVal;
  });

  return rows;
});

function formatZone(zone: string): string {
  return zone.replace(/([a-z])([A-Z])/g, "$1 $2");
}

function formatGold(amount: number): string {
  const rounded = Math.round(amount);
  if (rounded >= 0) return rounded.toLocaleString() + "g";
  return "-" + Math.abs(rounded).toLocaleString() + "g";
}
</script>
