<template>
  <div class="bg-surface-card border border-border-default rounded p-3">
    <div class="text-[0.65rem] uppercase tracking-widest text-[#7ec8e3] font-bold mb-2">
      Profit Estimates
    </div>

    <div v-if="categories.length > 0" class="flex flex-col gap-2">
      <div
        v-for="cat in categories"
        :key="cat.category"
        class="bg-black/20 rounded px-3 py-2"
      >
        <div class="flex items-center justify-between mb-1">
          <span class="text-xs font-bold"
                :class="cat.category === 'mineral' ? 'text-[#7ec8e3]' : 'text-[#c87e7e]'">
            {{ cat.category === 'mineral' ? 'Mineral' : 'Mining' }}
          </span>
          <span class="text-[0.6rem] text-text-dim">{{ cat.totalSurveys }} surveys</span>
        </div>

        <div class="grid grid-cols-3 gap-2 text-xs font-mono">
          <div>
            <div class="text-[0.55rem] text-text-dim uppercase">Avg Cost</div>
            <div class="text-text-secondary">{{ formatGold(cat.avgCost) }}</div>
          </div>
          <div>
            <div class="text-[0.55rem] text-text-dim uppercase">Expected Bonus</div>
            <div class="text-[#c8b47e]">{{ formatGold(cat.expectedBonusPerSurvey) }}/survey</div>
          </div>
          <div>
            <div class="text-[0.55rem] text-text-dim uppercase">Net/Survey</div>
            <div :class="cat.netPerSurvey >= 0 ? 'text-[#7ec87e]' : 'text-[#c87e7e]'">
              {{ cat.netPerSurvey >= 0 ? '+' : '' }}{{ formatGold(cat.netPerSurvey) }}
            </div>
          </div>
        </div>
      </div>
    </div>

    <div v-else class="text-text-dim italic text-xs">No profit data available.</div>

    <div v-if="categories.length > 0" class="text-[0.55rem] text-text-dim italic mt-2">
      Net = expected speed bonus value minus average crafting cost per survey.
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import type { ZoneAnalytics } from "../../../types/database";

const props = defineProps<{
  zone: ZoneAnalytics;
}>();

interface CategoryProfit {
  category: string;
  totalSurveys: number;
  avgCost: number;
  expectedBonusPerSurvey: number;
  netPerSurvey: number;
}

const categories = computed<CategoryProfit[]>(() => {
  const results: CategoryProfit[] = [];

  for (const cat of props.zone.speed_bonus_stats) {
    if (cat.total_surveys === 0) continue;

    // Weighted average cost across survey types in this category
    const catTypes = props.zone.survey_type_stats.filter(st => st.category === cat.category);
    const totalCost = catTypes.reduce((sum, st) => sum + st.crafting_cost * st.total_completed, 0);
    const totalCompleted = catTypes.reduce((sum, st) => sum + st.total_completed, 0);
    const avgCost = totalCompleted > 0 ? totalCost / totalCompleted : 0;

    const expectedBonusPerSurvey = cat.avg_bonus_value * (cat.speed_bonus_rate / 100);
    const netPerSurvey = expectedBonusPerSurvey - avgCost;

    results.push({
      category: cat.category,
      totalSurveys: cat.total_surveys,
      avgCost,
      expectedBonusPerSurvey,
      netPerSurvey,
    });
  }

  return results;
});

function formatGold(amount: number): string {
  const rounded = Math.round(amount);
  if (rounded >= 0) return rounded.toLocaleString() + "g";
  return "-" + Math.abs(rounded).toLocaleString() + "g";
}
</script>
