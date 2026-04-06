<template>
  <div class="flex flex-col gap-4">
    <!-- Global stat summary -->
    <div v-if="speedStats" class="grid grid-cols-2 xl:grid-cols-4 gap-2">
      <div class="bg-surface-card border border-border-default rounded px-3 py-2">
        <div class="text-[0.6rem] uppercase tracking-widest text-text-dim">Total Surveys</div>
        <div class="text-lg font-mono font-bold text-text-primary">{{ speedStats.total_surveys.toLocaleString() }}</div>
      </div>
      <div class="bg-surface-card border border-border-default rounded px-3 py-2">
        <div class="text-[0.6rem] uppercase tracking-widest text-text-dim">Speed Bonus Rate</div>
        <div class="text-lg font-mono font-bold text-[#c8b47e]">{{ speedStats.speed_bonus_rate.toFixed(1) }}%</div>
        <div class="text-[0.55rem] text-text-dim">{{ speedStats.speed_bonus_count.toLocaleString() }} bonuses</div>
      </div>
      <div class="bg-surface-card border border-border-default rounded px-3 py-2">
        <div class="text-[0.6rem] uppercase tracking-widest text-text-dim">Bonus Items</div>
        <div class="text-lg font-mono font-bold text-text-primary">{{ speedStats.total_bonus_items.toLocaleString() }}</div>
        <div class="text-[0.55rem] text-text-dim">{{ speedStats.unique_bonus_items }} unique types</div>
      </div>
      <div class="bg-surface-card border border-border-default rounded px-3 py-2">
        <div class="text-[0.6rem] uppercase tracking-widest text-text-dim">Category Split</div>
        <div class="flex items-center gap-3 mt-0.5">
          <span class="text-xs font-mono">
            <span class="text-[#7ec8e3] font-bold">{{ mineralTotal }}</span>
            <span class="text-text-dim"> mineral</span>
          </span>
          <span class="text-xs font-mono">
            <span class="text-[#c87e7e] font-bold">{{ miningTotal }}</span>
            <span class="text-text-dim"> mining</span>
          </span>
        </div>
      </div>
    </div>

    <!-- Two-column layout for tables -->
    <div class="grid grid-cols-1 xl:grid-cols-2 gap-4">
      <!-- Cross-Zone Comparison -->
      <div v-if="zones.length > 1" class="bg-surface-card border border-border-default rounded p-3">
        <div class="text-[0.65rem] uppercase tracking-widest text-[#7ec8e3] font-bold mb-2">
          Cross-Zone Comparison
        </div>
        <CrossZoneComparison :zones="zones" />
      </div>

      <!-- Survey Type Comparison -->
      <div class="bg-surface-card border border-border-default rounded p-3">
        <div class="text-[0.65rem] uppercase tracking-widest text-[#7ec8e3] font-bold mb-2">
          All Survey Types
        </div>
        <table v-if="allSurveyTypes.length > 0" class="text-xs w-full">
          <thead>
            <tr class="text-[0.6rem] uppercase tracking-wide text-text-muted font-bold">
              <th class="text-left py-1 px-2 font-bold">Survey Type</th>
              <th class="text-left py-1 px-2 font-bold">Zone</th>
              <th class="text-right py-1 px-2 font-bold">Done</th>
              <th class="text-right py-1 px-2 font-bold">Items</th>
              <th class="text-right py-1 px-2 font-bold">Cost</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="st in allSurveyTypes"
              :key="`${st.zone}-${st.survey_type}`"
              class="bg-black/20 border-b border-border-default hover:bg-black/30"
            >
              <td class="py-1 px-2 font-semibold"
                  :class="st.category === 'mineral' ? 'text-[#7ec8e3]' : 'text-[#c87e7e]'">
                {{ st.survey_type }}
              </td>
              <td class="py-1 px-2 text-text-secondary text-[0.65rem]">{{ formatZone(st.zone) }}</td>
              <td class="text-right py-1 px-2 font-mono text-text-primary">{{ st.total_completed }}</td>
              <td class="text-right py-1 px-2 font-mono text-text-secondary">{{ st.totalItems }}</td>
              <td class="text-right py-1 px-2 font-mono text-text-secondary">{{ formatGold(st.crafting_cost) }}</td>
            </tr>
          </tbody>
        </table>
        <div v-else class="text-text-dim italic text-xs">No survey type data available.</div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import type { SpeedBonusStats, ZoneAnalytics } from "../../../types/database";
import CrossZoneComparison from "./CrossZoneComparison.vue";

const props = defineProps<{
  zones: ZoneAnalytics[];
  speedStats: SpeedBonusStats | null;
}>();

const mineralTotal = computed(() => {
  let total = 0;
  for (const zone of props.zones) {
    const cat = zone.speed_bonus_stats.find(s => s.category === "mineral");
    if (cat) total += cat.total_surveys;
  }
  return total.toLocaleString();
});

const miningTotal = computed(() => {
  let total = 0;
  for (const zone of props.zones) {
    const cat = zone.speed_bonus_stats.find(s => s.category === "mining");
    if (cat) total += cat.total_surveys;
  }
  return total.toLocaleString();
});

interface SurveyTypeRow {
  survey_type: string;
  zone: string;
  category: string;
  total_completed: number;
  totalItems: number;
  crafting_cost: number;
}

const allSurveyTypes = computed<SurveyTypeRow[]>(() => {
  const rows: SurveyTypeRow[] = [];
  for (const zone of props.zones) {
    for (const st of zone.survey_type_stats) {
      const totalItems = st.item_stats.reduce((sum, i) => sum + i.total_quantity, 0);
      rows.push({
        survey_type: st.survey_type,
        zone: zone.zone,
        category: st.category,
        total_completed: st.total_completed,
        totalItems,
        crafting_cost: st.crafting_cost,
      });
    }
  }
  return rows.sort((a, b) => b.total_completed - a.total_completed);
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
