<template>
  <div v-if="!store.sessionActive" class="py-4 flex flex-col items-center gap-4">
    <div class="text-text-dim italic">
      No active survey session. Start watching a log or parse a file.
    </div>
    <button @click="store.manualStart" class="px-4! py-2! text-sm! bg-[#2a3a2a]! border border-[#4a5a4a]! text-[#8ec88e]! rounded cursor-pointer transition-all font-medium hover:bg-[#3a4a3a] hover:border-[#5a7a5a] hover:text-[#aedaae]">
      Start Manual Session
    </button>
  </div>

  <div v-else-if="s" class="bg-[#1a1a2e] border border-border-light rounded-lg p-4 mb-4">
    <!-- Status bar -->
    <div class="flex justify-between items-center mb-2">
      <div class="flex items-center gap-2">
        <span
          :class="[
            'inline-block w-2 h-2 rounded-full',
            s.endTime ? 'bg-text-dim' : s.isPaused ? 'bg-[#c8b47e] animate-pulse' : 'bg-[#7ec87e] animate-pulse'
          ]" />
        <span class="text-base font-bold text-[#7ec8e3]">Survey Session</span>
        <span v-if="s.endTime" class="text-xs text-text-dim uppercase tracking-wide">(Ended)</span>
      </div>
      <div class="flex gap-2">
        <button
          v-if="!s.endTime"
          @click="store.togglePause"
          :class="[
            'px-3 py-1.5 text-xs bg-[#2a2a3e] border border-border-light rounded text-text-secondary cursor-pointer transition-all font-medium hover:bg-[#3a3a4e] hover:border-border-hover hover:text-text-primary',
            s.isPaused && 'bg-[#3a4a2a]! border-[#5a7a3a]! text-[#8ec88e]!'
          ]">
          {{ s.isPaused ? "Resume" : "Pause" }}
        </button>
        <button
          v-if="!s.endTime"
          @click="store.manualEnd"
          class="px-3 py-1.5 text-xs bg-[#3a2a2a]! border border-[#5a3a3a]! rounded text-[#c87e7e]! cursor-pointer transition-all font-medium hover:bg-[#4a3a3a] hover:border-[#6a4a4a]">
          End Session
        </button>
        <button @click="store.reset" class="px-3 py-1.5 text-xs bg-[#2a2a3a]! border border-[#4a4a5a]! rounded text-text-secondary cursor-pointer transition-all font-medium hover:bg-[#3a3a4e] hover:border-border-hover hover:text-text-primary">Reset</button>
      </div>
    </div>
    <div class="flex justify-between items-baseline mb-3">
      <span class="text-xs text-text-muted">
        Started {{ formatTs(s.startTime) }}
        <span v-if="s.endTime"> · Ended {{ formatTs(s.endTime) }}</span>
        · {{ store.elapsed }} elapsed
        <span v-if="s.isPaused" class="text-[#c8b47e] font-bold ml-2">(PAUSED)</span>
      </span>
      <span v-if="s.manualMode" class="text-[0.65rem] text-[#7ec8e3] uppercase tracking-wide">Manual Mode</span>
    </div>

    <!-- Two-column: Stats + XP | Economics -->
    <div class="grid grid-cols-1 lg:grid-cols-2 gap-4 mb-4">
      <!-- Left: Stats + XP -->
      <div>
        <div class="flex gap-4 mb-3">
          <div class="text-center flex-1">
            <div class="text-[0.65rem] text-text-muted uppercase tracking-wide">Maps Crafted</div>
            <div class="text-lg font-bold text-text-primary">{{ s.mapsStarted }}</div>
          </div>
          <div class="text-center flex-1">
            <div class="text-[0.65rem] text-text-muted uppercase tracking-wide">Completed</div>
            <div class="text-lg font-bold text-text-primary">{{ s.surveysCompleted }}</div>
          </div>
          <div class="text-center flex-1">
            <div class="text-[0.65rem] text-text-muted uppercase tracking-wide">Avg Time</div>
            <div class="text-lg font-bold text-text-primary">{{ store.avgSurveyTime }}</div>
          </div>
        </div>
        <div class="flex gap-2">
          <div class="flex-1 p-2 rounded text-center bg-[#1a2e1a] border border-[#3a5a3a]">
            <div class="text-[0.6rem] text-text-secondary uppercase">Surveying XP</div>
            <div class="text-sm font-bold text-[#7ec87e]">+{{ s.surveyingXpGained.toLocaleString() }}</div>
          </div>
          <div class="flex-1 p-2 rounded text-center bg-[#2e1a1a] border border-[#5a3a3a]">
            <div class="text-[0.6rem] text-text-secondary uppercase">Mining XP</div>
            <div class="text-sm font-bold text-[#c87e7e]">+{{ s.miningXpGained.toLocaleString() }}</div>
          </div>
          <div class="flex-1 p-2 rounded text-center bg-[#2e2a1a] border border-[#5a4a2a]">
            <div class="text-[0.6rem] text-text-secondary uppercase">Geology XP</div>
            <div class="text-sm font-bold text-[#c8b47e]">+{{ s.geologyXpGained.toLocaleString() }}</div>
          </div>
        </div>
      </div>

      <!-- Right: Economics -->
      <div v-if="store.totalValue > 0">
        <div class="text-[0.65rem] uppercase tracking-widest text-[#7ec8e3] mb-2 font-bold">Economics</div>
        <div class="flex gap-2 mb-2">
          <div class="flex-1 p-2 rounded text-center bg-[#1a2a1a]! border border-[#5a7a5a]!">
            <div class="text-[0.6rem] text-text-secondary uppercase">Revenue</div>
            <div class="text-sm font-bold text-[#d4af37]">{{ store.totalValue.toLocaleString() }}g</div>
          </div>
          <div class="flex-1 p-2 rounded text-center bg-[#2a1a1a]! border border-[#7a5a5a]!">
            <div class="text-[0.6rem] text-text-secondary uppercase">Cost</div>
            <div class="text-sm font-bold text-[#d4af37]">{{ store.totalCost.toLocaleString() }}g</div>
          </div>
          <div class="flex-1 p-2 rounded text-center bg-[#2a2a1a]! border border-[#7a7a5a]!">
            <div class="text-[0.6rem] text-text-secondary uppercase">Profit</div>
            <div :class="['text-sm font-bold', store.totalProfit < 0 ? 'text-[#c87e7e]!' : 'text-[#7ec87e]']">
              {{ store.totalProfit >= 0 ? '+' : '' }}{{ store.totalProfit.toLocaleString() }}g
            </div>
          </div>
        </div>
        <div class="flex gap-2">
          <div class="flex-1 p-2 rounded text-center bg-[#1a1a2e] border border-border-light">
            <div class="text-[0.6rem] text-text-secondary uppercase">Per Survey</div>
            <div :class="['text-sm! font-bold text-[#d4af37]', store.profitPerSurvey < 0 && 'text-[#c87e7e]!']">
              {{ store.profitPerSurvey >= 0 ? '+' : '' }}{{ store.profitPerSurvey.toLocaleString() }}g
            </div>
          </div>
          <div class="flex-1 p-2 rounded text-center bg-[#1a1a2e] border border-border-light">
            <div class="text-[0.6rem] text-text-secondary uppercase">Per Hour</div>
            <div :class="['text-sm! font-bold text-[#d4af37]', store.profitPerHour < 0 && 'text-[#c87e7e]!']">
              {{ store.profitPerHour >= 0 ? '+' : '' }}{{ store.profitPerHour.toLocaleString() }}g
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Survey Type Breakdown -->
    <div v-if="store.surveyTypeBreakdown.length > 0" class="border-t border-[#2a2a3e] pt-3 mb-4">
      <div class="text-[0.65rem] uppercase tracking-widest text-[#7ec8e3] mb-2 font-bold">Survey Type Breakdown</div>
      <div class="flex flex-col gap-1">
        <div class="grid grid-cols-[1fr_80px_90px_80px_90px_100px] gap-3 px-3 py-2 rounded text-xs items-center bg-[#1a1a2e] border border-border-light font-bold text-text-secondary text-[0.7rem] uppercase">
          <div class="text-left min-w-0 text-text-primary font-medium">Type</div>
          <div class="text-right min-w-0">Completed</div>
          <div class="text-right min-w-0">Revenue</div>
          <div class="text-right min-w-0">Cost</div>
          <div class="text-right min-w-0">Profit</div>
          <div class="text-right min-w-0">Profit/Survey</div>
        </div>
        <div
          v-for="entry in store.surveyTypeBreakdown"
          :key="entry.type"
          class="grid grid-cols-[1fr_80px_90px_80px_90px_100px] gap-3 px-3 py-2 rounded text-xs items-center bg-black/20 border border-border-default hover:bg-black/30 hover:border-border-light">
          <div class="text-left min-w-0 font-mono text-text-primary font-medium truncate">{{ entry.type }}</div>
          <div class="text-right min-w-0 font-mono">{{ entry.completed }}</div>
          <div class="text-right min-w-0 font-mono text-[#8ec88e]!">{{ entry.revenue.toLocaleString() }}g</div>
          <div class="text-right min-w-0 font-mono text-[#c87e7e]!">{{ entry.cost.toLocaleString() }}g</div>
          <div :class="['text-right min-w-0 font-mono text-[#7ec87e]! font-semibold', entry.profit < 0 && 'text-[#c87e7e]!']">
            {{ entry.profit >= 0 ? '+' : '' }}{{ entry.profit.toLocaleString() }}g
          </div>
          <div :class="['text-right min-w-0 font-mono text-[#7ec87e]! font-semibold', entry.profitPerSurvey < 0 && 'text-[#c87e7e]!']">
            {{ entry.profitPerSurvey >= 0 ? '+' : '' }}{{ entry.profitPerSurvey.toLocaleString() }}g
          </div>
        </div>
      </div>
    </div>

    <!-- Loot: Survey Rewards -->
    <div v-if="store.primaryLootSummary.length > 0" class="border-t border-[#2a2a3e] pt-3 mb-4">
      <SurveyLootGrid
        :items="store.primaryLootSummary"
        title="Survey Rewards"
        title-class="text-text-dim"
      />
    </div>

    <!-- Loot: Speed Bonus -->
    <div v-if="store.speedBonusLootSummary.length > 0" class="border-t border-[#2a2a3e] pt-3 mb-4">
      <SurveyLootGrid
        :items="store.speedBonusLootSummary"
        title="Speed Bonus"
        title-class="text-[#c8b47e]"
      />
    </div>

    <!-- Crafting Materials -->
    <div v-if="hasCraftingMaterials" class="border-t border-[#2a2a3e] pt-3">
      <div class="text-[0.65rem] uppercase tracking-widest text-text-dim mb-2 font-bold">Crafting Materials Consumed</div>
      <div class="flex flex-wrap gap-2">
        <div
          v-for="(qty, name) in s.craftingMaterials"
          :key="name"
          class="bg-black/20 border border-border-default rounded px-2.5 py-1.5 text-xs"
        >
          <span class="text-text-primary">{{ name }}</span>
          <span class="text-text-secondary ml-1">&times;{{ qty }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { useSurveyStore } from "../../stores/surveyStore";
import { formatTimeFull } from "../../composables/useTimestamp";
import SurveyLootGrid from "./SurveyLootGrid.vue";

function formatTs(ms: number | null): string {
  if (!ms) return '';
  return formatTimeFull(new Date(ms).toISOString());
}

const store = useSurveyStore();
const s = computed(() => store.session);

const hasCraftingMaterials = computed(() => {
  if (!s.value) return false;
  return Object.keys(s.value.craftingMaterials).length > 0;
});
</script>
