<template>
  <div v-if="store.surveyTypeBreakdown.length > 0" class="bg-[#1a1a2e] border border-border-light rounded-lg p-4">
    <div class="text-[0.65rem] uppercase tracking-widest text-[#7ec8e3] mb-3 font-bold">Survey Type Breakdown</div>

    <!-- Header row -->
    <div class="grid grid-cols-[1fr_60px_80px_70px_80px_90px] gap-2 px-3 py-2 rounded text-[0.65rem] items-center bg-[#1a1a2e] border border-border-light font-bold text-text-secondary uppercase">
      <div class="text-left">Type</div>
      <div class="text-right">Done</div>
      <div class="text-right">Revenue</div>
      <div class="text-right">Cost</div>
      <div class="text-right">Profit</div>
      <div class="text-right">Profit/ea</div>
    </div>

    <!-- Type rows with accordion -->
    <div v-for="entry in store.surveyTypeBreakdown" :key="entry.type" class="mt-1">
      <!-- Summary row -->
      <button
        @click="toggle(entry.type)"
        class="w-full grid grid-cols-[1fr_60px_80px_70px_80px_90px] gap-2 px-3 py-2 rounded text-xs items-center bg-black/20 border border-border-default cursor-pointer transition-all hover:bg-black/30 hover:border-border-light"
      >
        <div class="text-left min-w-0 flex items-center gap-1.5">
          <span class="text-text-secondary text-xs">{{ expanded[entry.type] ? '▼' : '▶' }}</span>
          <span class="font-mono text-text-primary font-medium truncate">{{ entry.type }}</span>
        </div>
        <div class="text-right min-w-0 font-mono">{{ entry.completed }}</div>
        <div class="text-right min-w-0 font-mono text-[#8ec88e]!">{{ entry.revenue.toLocaleString() }}g</div>
        <div class="text-right min-w-0 font-mono text-[#c87e7e]!">{{ entry.cost.toLocaleString() }}g</div>
        <div :class="['text-right min-w-0 font-mono font-semibold', entry.profit < 0 ? 'text-[#c87e7e]!' : 'text-[#7ec87e]!']">
          {{ entry.profit >= 0 ? '+' : '' }}{{ entry.profit.toLocaleString() }}g
        </div>
        <div :class="['text-right min-w-0 font-mono font-semibold', entry.profitPerSurvey < 0 ? 'text-[#c87e7e]!' : 'text-[#7ec87e]!']">
          {{ entry.profitPerSurvey >= 0 ? '+' : '' }}{{ entry.profitPerSurvey.toLocaleString() }}g
        </div>
      </button>

      <!-- Expanded loot section (primary rewards only — speed bonus is area-wide, shown separately) -->
      <div v-if="expanded[entry.type]" class="ml-4 mt-1 mb-2">
        <SurveyLootGrid
          v-if="entry.primaryLoot.length > 0"
          :items="entry.primaryLoot"
          title="Rewards"
          title-class="text-text-dim"
        />
        <div v-else class="text-text-dim italic text-xs py-2 pl-2">No rewards yet</div>
      </div>
    </div>
  </div>

  <!-- Fallback when no types yet but session active -->
  <div v-else class="bg-[#1a1a2e] border border-border-light rounded-lg p-4">
    <div class="text-text-dim italic text-sm text-center py-4">
      No survey data yet. Craft a survey map to begin.
    </div>
  </div>
</template>

<script setup lang="ts">
import { reactive } from "vue";
import { useSurveyStore } from "../../stores/surveyStore";
import SurveyLootGrid from "./SurveyLootGrid.vue";

const store = useSurveyStore();
const expanded = reactive<Record<string, boolean>>({});

function toggle(type: string) {
  expanded[type] = !expanded[type];
}
</script>
