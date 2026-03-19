<template>
  <div class="flex flex-col gap-4">
    <div class="flex justify-between items-center">
      <h3 class="text-lg text-[#7ec8e3] m-0">Survey Analytics</h3>
      <button @click="loadAll" :disabled="loading"
        class="px-3 py-1.5 text-sm bg-surface-elevated border border-border-default rounded text-text-secondary hover:text-text-primary hover:border-border-hover transition-all">
        {{ loading ? "Loading..." : "Refresh" }}
      </button>
    </div>

    <div v-if="error" class="text-[#c87e7e] bg-[#2a1a1a] border border-[#5a3a3a] rounded p-3 text-sm">{{ error }}</div>

    <!-- Speed Bonus Stats -->
    <div v-if="speedStats" class="bg-surface-card border border-border-default rounded p-4">
      <div class="text-[0.65rem] uppercase tracking-widest text-[#7ec8e3] mb-3 font-bold">Speed Bonus Stats (All Time)</div>
      <div class="grid grid-cols-5 gap-4">
        <div class="text-center">
          <div class="text-[0.65rem] text-text-muted uppercase tracking-wide">Total Surveys</div>
          <div class="text-lg font-bold text-text-primary">{{ speedStats.total_surveys }}</div>
        </div>
        <div class="text-center">
          <div class="text-[0.65rem] text-text-muted uppercase tracking-wide">Bonuses Earned</div>
          <div class="text-lg font-bold text-[#c8b47e]">{{ speedStats.speed_bonus_count }}</div>
        </div>
        <div class="text-center">
          <div class="text-[0.65rem] text-text-muted uppercase tracking-wide">Bonus Rate</div>
          <div class="text-lg font-bold text-[#c8b47e]">{{ speedStats.speed_bonus_rate.toFixed(1) }}%</div>
        </div>
        <div class="text-center">
          <div class="text-[0.65rem] text-text-muted uppercase tracking-wide">Bonus Items</div>
          <div class="text-lg font-bold text-text-primary">{{ speedStats.total_bonus_items }}</div>
        </div>
        <div class="text-center">
          <div class="text-[0.65rem] text-text-muted uppercase tracking-wide">Unique Bonus Items</div>
          <div class="text-lg font-bold text-text-primary">{{ speedStats.unique_bonus_items }}</div>
        </div>
      </div>
    </div>

    <!-- Survey Type Metrics -->
    <div v-if="typeMetrics.length > 0" class="bg-surface-card border border-border-default rounded p-4">
      <div class="text-[0.65rem] uppercase tracking-widest text-[#7ec8e3] mb-3 font-bold">Survey Type Breakdown (All Time)</div>
      <div class="flex flex-col gap-1">
        <div class="grid grid-cols-[1fr_80px_100px_80px_80px_100px] gap-4 px-3 py-2 rounded text-[0.7rem] items-center bg-[#1a1a2e] border border-border-light font-bold text-text-secondary uppercase">
          <div class="text-left">Type</div>
          <div class="text-right">Completed</div>
          <div class="text-right">Speed Bonuses</div>
          <div class="text-right">Bonus Rate</div>
          <div class="text-right">Total Items</div>
          <div class="text-right">Avg Items/Survey</div>
        </div>
        <div
          v-for="metric in typeMetrics"
          :key="metric.survey_type"
          class="grid grid-cols-[1fr_80px_100px_80px_80px_100px] gap-4 px-3 py-2 rounded text-xs items-center bg-black/20 border border-border-default hover:bg-black/30 hover:border-border-light">
          <div class="text-left font-medium text-text-primary">{{ metric.survey_type }}</div>
          <div class="text-right font-mono">{{ metric.total_completed }}</div>
          <div class="text-right font-mono text-[#c8b47e]">{{ metric.speed_bonus_count }}/{{ metric.total_completed }}</div>
          <div class="text-right font-mono text-[#c8b47e]">{{ metric.speed_bonus_rate.toFixed(1) }}%</div>
          <div class="text-right font-mono">{{ metric.total_items }}</div>
          <div class="text-right font-mono">{{ metric.avg_items_per_survey.toFixed(1) }}</div>
        </div>
      </div>
    </div>

    <!-- All-Time Loot Breakdown -->
    <div v-if="lootBreakdown.length > 0" class="bg-surface-card border border-border-default rounded p-4">
      <div class="text-[0.65rem] uppercase tracking-widest text-[#7ec8e3] mb-3 font-bold">Loot Breakdown (All Time)</div>
      <div class="flex flex-col gap-1">
        <div class="grid grid-cols-[1fr_80px_80px_80px_80px] gap-4 px-3 py-2 rounded text-[0.7rem] items-center bg-[#1a1a2e] border border-border-light font-bold text-text-secondary uppercase">
          <div class="text-left">Item</div>
          <div class="text-right">Total</div>
          <div class="text-right">Primary</div>
          <div class="text-right">Bonus</div>
          <div class="text-right">Times Found</div>
        </div>
        <div
          v-for="loot in lootBreakdown"
          :key="loot.item_name"
          class="grid grid-cols-[1fr_80px_80px_80px_80px] gap-4 px-3 py-2 rounded text-xs items-center bg-black/20 border border-border-default hover:bg-black/30 hover:border-border-light">
          <div class="text-left min-w-0">
            <ItemInline :name="loot.item_name" />
          </div>
          <div class="text-right font-mono font-medium text-text-primary">{{ loot.total_quantity }}</div>
          <div class="text-right font-mono text-[#8ec88e]">{{ loot.primary_quantity }}</div>
          <div class="text-right font-mono text-[#c8b47e]">{{ loot.bonus_quantity }}</div>
          <div class="text-right font-mono text-text-secondary">{{ loot.times_received }}</div>
        </div>
      </div>
    </div>

    <!-- Empty state -->
    <div v-if="!loading && !speedStats && typeMetrics.length === 0 && lootBreakdown.length === 0"
      class="text-text-dim italic text-center p-8">
      No survey data yet. Complete some surveys to see analytics here.
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import ItemInline from "../Shared/Item/ItemInline.vue";

interface SpeedBonusStats {
  total_surveys: number;
  speed_bonus_count: number;
  speed_bonus_rate: number;
  total_bonus_items: number;
  unique_bonus_items: number;
}

interface SurveyTypeMetrics {
  survey_type: string;
  total_completed: number;
  speed_bonus_count: number;
  speed_bonus_rate: number;
  total_items: number;
  total_bonus_items: number;
  avg_items_per_survey: number;
}

interface LootBreakdownEntry {
  item_name: string;
  item_id: number | null;
  total_quantity: number;
  primary_quantity: number;
  bonus_quantity: number;
  times_received: number;
}

const loading = ref(false);
const error = ref("");
const speedStats = ref<SpeedBonusStats | null>(null);
const typeMetrics = ref<SurveyTypeMetrics[]>([]);
const lootBreakdown = ref<LootBreakdownEntry[]>([]);

onMounted(() => {
  loadAll();
});

async function loadAll() {
  loading.value = true;
  error.value = "";
  try {
    const [speed, metrics, loot] = await Promise.all([
      invoke<SpeedBonusStats>("get_speed_bonus_stats", { sessionId: null }),
      invoke<SurveyTypeMetrics[]>("get_survey_type_metrics", { sessionId: null }),
      invoke<LootBreakdownEntry[]>("get_loot_breakdown", { sessionId: null, limit: 100 }),
    ]);
    speedStats.value = speed;
    typeMetrics.value = metrics;
    lootBreakdown.value = loot;
  } catch (e) {
    error.value = `Failed to load analytics: ${e}`;
  } finally {
    loading.value = false;
  }
}
</script>
