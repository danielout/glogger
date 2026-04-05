<template>
  <div class="bg-surface-card border border-border-default rounded p-4">
    <div class="text-[0.65rem] uppercase tracking-widest text-[#7ec8e3] mb-3 font-bold">
      Item Cost Calculator
    </div>

    <!-- Controls -->
    <div class="flex items-center gap-3 mb-4 flex-wrap">
      <select
        v-model="selectedItem"
        class="bg-surface-elevated border border-border-default rounded px-3 py-1.5 text-sm text-text-primary min-w-[200px]"
      >
        <option value="">Select an item...</option>
        <option v-for="item in availableItems" :key="item" :value="item">{{ item }}</option>
      </select>

      <div class="flex items-center gap-1.5">
        <label class="text-[0.65rem] text-text-muted uppercase tracking-wide">Qty</label>
        <input
          v-model.number="desiredQty"
          type="number"
          min="1"
          placeholder="100"
          class="bg-surface-elevated border border-border-default rounded px-3 py-1.5 text-sm text-text-primary w-24 font-mono"
        />
      </div>

      <div class="flex items-center gap-1.5">
        <label class="text-[0.65rem] text-text-muted uppercase tracking-wide">Sell Price</label>
        <input
          v-model.number="sellPrice"
          type="number"
          min="0"
          placeholder="0g"
          class="bg-surface-elevated border border-border-default rounded px-3 py-1.5 text-sm text-text-primary w-24 font-mono"
        />
      </div>

      <div class="flex gap-1 bg-surface-elevated border border-border-default rounded p-0.5">
        <button
          @click="sortMode = 'cost'"
          :class="[
            'px-2.5 py-1 text-xs rounded transition-all',
            sortMode === 'cost'
              ? 'bg-[#7ec8e3]/20 text-[#7ec8e3] font-semibold'
              : 'text-text-muted hover:text-text-secondary'
          ]"
        >Sort by Cost</button>
        <button
          @click="sortMode = 'time'"
          :class="[
            'px-2.5 py-1 text-xs rounded transition-all',
            sortMode === 'time'
              ? 'bg-[#7ec8e3]/20 text-[#7ec8e3] font-semibold'
              : 'text-text-muted hover:text-text-secondary'
          ]"
        >Sort by Time</button>
        <button
          v-if="hasSellPrice"
          @click="sortMode = 'profit'"
          :class="[
            'px-2.5 py-1 text-xs rounded transition-all',
            sortMode === 'profit'
              ? 'bg-[#7ec8e3]/20 text-[#7ec8e3] font-semibold'
              : 'text-text-muted hover:text-text-secondary'
          ]"
        >Sort by Profit/hr</button>
      </div>
    </div>

    <!-- Loading -->
    <div v-if="loading" class="text-text-dim italic text-xs">Loading item data...</div>

    <!-- Error -->
    <div v-else-if="error" class="text-[#c87e7e] text-xs">{{ error }}</div>

    <!-- Results table -->
    <div v-else-if="selectedItem && (desiredQty ?? 0) > 0 && sortedResults.length > 0">
      <div class="flex flex-col gap-1">
        <!-- Header -->
        <div :class="gridCols" class="gap-3 px-3 py-1.5 text-[0.6rem] uppercase tracking-wide text-text-muted font-bold">
          <div>Survey Type</div>
          <div class="text-right">Zone</div>
          <div class="text-right">Avg Yield</div>
          <div class="text-right">Needed</div>
          <div class="text-right">Cost Each</div>
          <div class="text-right">Total Cost</div>
          <div class="text-right">Est. Time</div>
          <div v-if="hasSellPrice" class="text-right">Profit</div>
          <div v-if="hasSellPrice" class="text-right">Profit/hr</div>
        </div>

        <!-- Rows -->
        <div
          v-for="(r, idx) in sortedResults"
          :key="r.survey_type"
          class="group"
        >
          <div :class="gridCols" class="gap-3 px-3 py-1.5 text-xs bg-black/20 border border-border-default rounded hover:bg-black/30">
            <div class="min-w-0 truncate">
              <span class="text-text-primary font-semibold">{{ r.survey_type }}</span>
              <span v-if="idx === 0" class="ml-1.5 text-[0.55rem] text-[#7ec87e] uppercase tracking-wider font-bold">Best</span>
            </div>
            <div class="text-right text-text-secondary text-[0.65rem]">{{ formatZone(r.zone) }}</div>
            <div class="text-right font-mono text-text-primary">{{ r.effective_yield.toFixed(1) }}</div>
            <div class="text-right font-mono text-text-primary">{{ r.surveys_needed }}</div>
            <div class="text-right font-mono text-text-secondary">{{ formatGold(r.crafting_cost) }}</div>
            <div class="text-right font-mono" :class="idx === 0 && sortMode === 'cost' ? 'text-[#7ec87e] font-bold' : 'text-text-primary'">
              {{ formatGold(r.total_cost) }}
            </div>
            <div class="text-right font-mono" :class="[
              r.avg_seconds_per_survey <= 0 ? 'text-text-dim italic' : '',
              idx === 0 && sortMode === 'time' ? 'text-[#7ec87e] font-bold' : 'text-text-primary'
            ]">
              {{ r.avg_seconds_per_survey > 0 ? formatTime(r.total_time_seconds) : 'N/A' }}
            </div>
            <div v-if="hasSellPrice" class="text-right font-mono" :class="r.profit >= 0 ? 'text-[#7ec87e]' : 'text-[#c87e7e]'">
              {{ r.profit >= 0 ? '+' : '' }}{{ formatGold(r.profit) }}
            </div>
            <div v-if="hasSellPrice" class="text-right font-mono" :class="[
              r.avg_seconds_per_survey <= 0 ? 'text-text-dim italic' : '',
              idx === 0 && sortMode === 'profit' ? 'text-[#7ec87e] font-bold' : r.profit_per_hour >= 0 ? 'text-[#7ec87e]' : 'text-[#c87e7e]'
            ]">
              {{ r.avg_seconds_per_survey > 0 ? (r.profit_per_hour >= 0 ? '+' : '') + formatGold(r.profit_per_hour) + '/hr' : 'N/A' }}
            </div>
          </div>

          <!-- Yield breakdown sub-row -->
          <div class="px-3 py-1 text-[0.6rem] text-text-dim ml-4 flex gap-4">
            <span v-if="r.primary_avg > 0">
              Primary: {{ r.primary_avg.toFixed(1) }}/survey
              <span class="text-text-muted">({{ r.primary_times_seen }}/{{ r.total_completions }} surveys)</span>
            </span>
            <span v-if="r.bonus_per_completion > 0" class="text-[#c8b47e]">
              Speed Bonus: {{ r.bonus_avg_per_proc.toFixed(1) }}/proc
              <span class="text-text-muted">({{ r.bonus_times_seen }}/{{ r.total_completions }} surveys &middot; {{ r.bonus_per_completion.toFixed(2) }}/survey effective)</span>
            </span>
            <span class="text-text-muted">{{ r.total_completions }} completions recorded</span>
          </div>
        </div>
      </div>

      <div class="mt-2 text-[0.55rem] text-text-dim italic">
        Yield includes primary loot + expected speed bonus contribution. Time estimates are averaged from session data.
      </div>
    </div>

    <!-- No results for this item -->
    <div v-else-if="selectedItem && (desiredQty ?? 0) > 0 && sortedResults.length === 0" class="text-text-dim italic text-xs">
      No survey data found for this item.
    </div>

    <!-- Empty prompt -->
    <div v-else-if="!loading && availableItems.length > 0" class="text-text-dim italic text-xs">
      Select an item and enter a desired quantity to calculate acquisition costs.
    </div>

    <!-- No data at all -->
    <div v-else-if="!loading" class="text-text-dim italic text-xs">
      No survey loot data available yet. Complete some surveys to use the calculator.
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

const props = withDefaults(defineProps<{
  includeImports?: boolean;
}>(), {
  includeImports: false,
});

interface ItemSourceAnalysis {
  item_name: string;
  survey_type: string;
  zone: string;
  category: string;
  crafting_cost: number;
  total_completions: number;
  primary_total_qty: number;
  primary_times_seen: number;
  primary_avg_per_completion: number;
  bonus_total_qty: number;
  bonus_times_seen: number;
  bonus_avg_per_proc: number;
  speed_bonus_rate: number;
  avg_seconds_per_survey: number;
}

interface CalculatedResult {
  survey_type: string;
  zone: string;
  crafting_cost: number;
  total_completions: number;
  effective_yield: number;
  surveys_needed: number;
  total_cost: number;
  total_time_seconds: number;
  avg_seconds_per_survey: number;
  primary_avg: number;
  primary_times_seen: number;
  bonus_per_completion: number;
  bonus_avg_per_proc: number;
  bonus_times_seen: number;
  speed_bonus_rate: number;
  profit: number;
  profit_per_hour: number;
}

const loading = ref(false);
const error = ref("");
const allData = ref<ItemSourceAnalysis[]>([]);
const selectedItem = ref("");
const desiredQty = ref<number | null>(null);
const sellPrice = ref<number | null>(null);
const sortMode = ref<"cost" | "time" | "profit">("cost");

const hasSellPrice = computed(() => (sellPrice.value ?? 0) > 0);

const gridCols = computed(() =>
  hasSellPrice.value
    ? 'grid grid-cols-[1fr_80px_80px_70px_90px_90px_90px_90px_90px]'
    : 'grid grid-cols-[1fr_80px_80px_70px_90px_90px_90px]'
);

onMounted(() => {
  loadData();
});

watch(() => props.includeImports, () => {
  loadData();
});

async function loadData() {
  loading.value = true;
  error.value = "";
  try {
    allData.value = await invoke<ItemSourceAnalysis[]>("get_item_cost_analysis", {
      includeImports: props.includeImports,
    });
  } catch (e) {
    error.value = `Failed to load item analysis: ${e}`;
  } finally {
    loading.value = false;
  }
}

const availableItems = computed(() => {
  const items = new Set(allData.value.map(d => d.item_name));
  return [...items].sort();
});

const sortedResults = computed<CalculatedResult[]>(() => {
  if (!selectedItem.value || !desiredQty.value || desiredQty.value <= 0) return [];

  const qty = desiredQty.value;
  const itemData = allData.value.filter(d => d.item_name === selectedItem.value);

  // Group by survey type — an item might appear as both primary and bonus from the same survey
  // but our backend already provides one row per (survey_type, item_name) pair with both splits
  const results: CalculatedResult[] = itemData.map(d => {
    // effective yield per survey = total items obtained / total surveys completed
    // This accounts for the drop rate (not every survey yields this item)
    const primaryPerSurvey = d.total_completions > 0
      ? d.primary_total_qty / d.total_completions
      : 0;
    const bonusPerCompletion = d.total_completions > 0
      ? d.bonus_total_qty / d.total_completions
      : 0;
    const effectiveYield = primaryPerSurvey + bonusPerCompletion;

    const surveysNeeded = effectiveYield > 0 ? Math.ceil(qty / effectiveYield) : Infinity;
    const totalCost = surveysNeeded * d.crafting_cost;
    const totalTime = surveysNeeded * d.avg_seconds_per_survey;

    // Profit = revenue from selling desired qty - total survey cost
    const revenue = (sellPrice.value ?? 0) * qty;
    const profit = surveysNeeded === Infinity ? 0 : revenue - totalCost;
    const profitPerHour = totalTime > 0 ? (profit / totalTime) * 3600 : 0;

    return {
      survey_type: d.survey_type,
      zone: d.zone,
      crafting_cost: d.crafting_cost,
      total_completions: d.total_completions,
      effective_yield: effectiveYield,
      surveys_needed: surveysNeeded === Infinity ? 0 : surveysNeeded,
      total_cost: surveysNeeded === Infinity ? 0 : totalCost,
      total_time_seconds: surveysNeeded === Infinity ? 0 : totalTime,
      avg_seconds_per_survey: d.avg_seconds_per_survey,
      primary_avg: primaryPerSurvey,
      primary_times_seen: d.primary_times_seen,
      bonus_per_completion: bonusPerCompletion,
      bonus_avg_per_proc: d.bonus_avg_per_proc,
      bonus_times_seen: d.bonus_times_seen,
      speed_bonus_rate: d.speed_bonus_rate,
      profit,
      profit_per_hour: profitPerHour,
    };
  }).filter(r => r.effective_yield > 0);

  // Sort
  if (sortMode.value === "profit") {
    results.sort((a, b) => {
      if (a.avg_seconds_per_survey <= 0 && b.avg_seconds_per_survey <= 0) return b.profit - a.profit;
      if (a.avg_seconds_per_survey <= 0) return 1;
      if (b.avg_seconds_per_survey <= 0) return -1;
      return b.profit_per_hour - a.profit_per_hour;
    });
  } else if (sortMode.value === "time") {
    results.sort((a, b) => {
      // Items without time data go last
      if (a.avg_seconds_per_survey <= 0 && b.avg_seconds_per_survey <= 0) return a.total_cost - b.total_cost;
      if (a.avg_seconds_per_survey <= 0) return 1;
      if (b.avg_seconds_per_survey <= 0) return -1;
      return a.total_time_seconds - b.total_time_seconds;
    });
  } else {
    results.sort((a, b) => a.total_cost - b.total_cost);
  }

  return results;
});

function formatZone(zone: string): string {
  return zone.replace(/([a-z])([A-Z])/g, "$1 $2");
}

function formatGold(amount: number): string {
  const rounded = Math.round(amount);
  if (rounded >= 0) return rounded.toLocaleString() + "g";
  return "-" + Math.abs(rounded).toLocaleString() + "g";
}

function formatTime(seconds: number): string {
  if (seconds <= 0) return "N/A";
  const hrs = Math.floor(seconds / 3600);
  const mins = Math.floor((seconds % 3600) / 60);
  const secs = Math.round(seconds % 60);
  if (hrs > 0) return `${hrs}h ${mins}m`;
  if (mins > 0) return `${mins}m ${secs}s`;
  return `${secs}s`;
}
</script>
