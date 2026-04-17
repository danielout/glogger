<template>
  <div class="flex flex-col gap-4">
    <!-- Summary stats -->
    <div v-if="matchingEntries.length > 0" class="grid grid-cols-2 xl:grid-cols-4 gap-2">
      <div class="bg-surface-card border border-border-default rounded px-3 py-2">
        <div class="text-[0.6rem] uppercase tracking-widest text-text-dim">Survey Type</div>
        <div class="text-sm font-bold"
             :class="firstEntry!.category === 'mineral' ? 'text-[#7ec8e3]' : 'text-[#c87e7e]'">
          {{ surveyTypeName }}
        </div>
      </div>
      <div class="bg-surface-card border border-border-default rounded px-3 py-2">
        <div class="text-[0.6rem] uppercase tracking-widest text-text-dim">Total Completed</div>
        <div class="text-lg font-mono font-bold text-text-primary">{{ totalCompleted.toLocaleString() }}</div>
        <div v-if="matchingEntries.length > 1" class="text-[0.55rem] text-text-dim">
          across {{ matchingEntries.length }} zones
        </div>
      </div>
      <div class="bg-surface-card border border-border-default rounded px-3 py-2">
        <div class="text-[0.6rem] uppercase tracking-widest text-text-dim">Crafting Cost</div>
        <div class="text-lg font-mono font-bold text-text-secondary">{{ formatGold(firstEntry!.crafting_cost) }}</div>
      </div>
      <div class="bg-surface-card border border-border-default rounded px-3 py-2">
        <div class="text-[0.6rem] uppercase tracking-widest text-text-dim">Unique Items</div>
        <div class="text-lg font-mono font-bold text-text-primary">{{ uniqueItemCount }}</div>
      </div>
    </div>

    <!-- Cards grid -->
    <div class="grid grid-cols-1 xl:grid-cols-2 2xl:grid-cols-3 gap-3">
      <!-- Per-zone item rewards -->
      <div
        v-for="entry in matchingEntries"
        :key="entry.zone"
        class="bg-surface-card border border-border-default rounded p-3"
      >
        <div class="flex items-center gap-2 mb-2">
          <span class="text-xs font-bold text-text-primary">{{ formatZone(entry.zone) }}</span>
          <span class="text-[0.6rem] text-text-dim">{{ entry.st.total_completed }} completed</span>
        </div>

        <table v-if="entry.st.item_stats.length > 0" class="text-xs w-full">
          <thead>
            <tr class="text-[0.6rem] uppercase tracking-wide text-text-muted font-bold">
              <th class="text-left py-0.5 px-2 font-bold">Item</th>
              <th class="text-right py-0.5 px-2 font-bold">Total</th>
              <th class="text-right py-0.5 px-2 font-bold">Seen</th>
              <th class="text-right py-0.5 px-2 font-bold">Min</th>
              <th class="text-right py-0.5 px-2 font-bold">Max</th>
              <th class="text-right py-0.5 px-2 font-bold">Avg</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="item in entry.st.item_stats"
              :key="item.item_name"
              class="bg-black/10 border-b border-border-default hover:bg-black/20"
            >
              <td class="py-0.5 px-2"><ItemInline :reference="item.item_name" /></td>
              <td class="text-right py-0.5 px-2 font-mono text-text-primary">{{ item.total_quantity }}</td>
              <td class="text-right py-0.5 px-2 font-mono text-text-secondary">{{ item.times_seen }}/{{ entry.st.total_completed }}</td>
              <td class="text-right py-0.5 px-2 font-mono">{{ item.min_per_completion }}</td>
              <td class="text-right py-0.5 px-2 font-mono">{{ item.max_per_completion }}</td>
              <td class="text-right py-0.5 px-2 font-mono text-text-primary">{{ item.avg_per_completion.toFixed(1) }}</td>
            </tr>
          </tbody>
        </table>
        <div v-else class="text-text-dim italic text-xs">No loot data recorded.</div>
      </div>

      <!-- Speed bonus context -->
      <div
        v-for="entry in entriesWithBonusData"
        :key="`bonus-${entry.zone}`"
        class="bg-surface-card border border-border-default rounded p-3"
      >
        <div class="flex items-center gap-2 mb-2">
          <span class="text-[0.65rem] uppercase tracking-widest font-bold text-[#c8b47e]">
            Speed Bonus — {{ formatZone(entry.zone) }}
          </span>
          <span class="text-[0.6rem] text-text-dim">
            {{ entry.bonusStats.speed_bonus_count }}/{{ entry.bonusStats.total_surveys }}
            ({{ entry.bonusStats.speed_bonus_rate.toFixed(1) }}%)
          </span>
        </div>

        <table v-if="entry.bonusStats.item_stats.length > 0" class="text-xs w-full">
          <thead>
            <tr class="text-[0.6rem] uppercase tracking-wide text-text-muted font-bold">
              <th class="text-left py-0.5 px-2 font-bold">Item</th>
              <th class="text-right py-0.5 px-2 font-bold">Total</th>
              <th class="text-right py-0.5 px-2 font-bold">Seen</th>
              <th class="text-right py-0.5 px-2 font-bold">Avg/Proc</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="item in entry.bonusStats.item_stats"
              :key="item.item_name"
              class="bg-black/10 border-b border-border-default hover:bg-black/20"
            >
              <td class="py-0.5 px-2"><ItemInline :reference="item.item_name" /></td>
              <td class="text-right py-0.5 px-2 font-mono text-[#c8b47e]">{{ item.total_quantity }}</td>
              <td class="text-right py-0.5 px-2 font-mono text-text-secondary">{{ item.times_seen }}/{{ item.total_procs }}</td>
              <td class="text-right py-0.5 px-2 font-mono text-text-primary">{{ item.avg_per_proc.toFixed(1) }}</td>
            </tr>
          </tbody>
        </table>
      </div>

      <!-- Item quantity donut chart -->
      <div v-if="itemChartDataset.length > 1" class="bg-surface-card border border-border-default rounded p-3">
        <div class="text-[0.65rem] uppercase tracking-widest text-[#7ec8e3] font-bold mb-2">
          Item Quantities
        </div>
        <div class="h-70">
          <VueUiDonut :dataset="itemChartDataset" :config="donutConfig" />
        </div>
      </div>
    </div>

    <!-- Empty state -->
    <div v-if="matchingEntries.length === 0" class="text-text-dim italic text-xs">
      No data found for this survey type.
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { VueUiDonut } from "vue-data-ui";
import type { VueUiDonutConfig, VueUiDonutDatasetItem } from "vue-data-ui";
import type { ZoneAnalytics, SurveyTypeAnalytics, CategorySpeedBonusStats } from "../../../types/database";
import { useGameDataStore } from "../../../stores/gameDataStore";
import ItemInline from "../../Shared/Item/ItemInline.vue";

const props = defineProps<{
  surveyTypeName: string;
  zones: ZoneAnalytics[];
}>();

const gameData = useGameDataStore();
const resolvedNames = ref<Record<string, string>>({});

interface MatchingEntry {
  zone: string;
  st: SurveyTypeAnalytics;
}

const matchingEntries = computed<MatchingEntry[]>(() => {
  const entries: MatchingEntry[] = [];
  for (const zone of props.zones) {
    const st = zone.survey_type_stats.find(s => s.survey_type === props.surveyTypeName);
    if (st) entries.push({ zone: zone.zone, st });
  }
  return entries.sort((a, b) => b.st.total_completed - a.st.total_completed);
});

const firstEntry = computed(() => matchingEntries.value.length > 0 ? matchingEntries.value[0].st : null);

const totalCompleted = computed(() =>
  matchingEntries.value.reduce((sum, e) => sum + e.st.total_completed, 0)
);

const uniqueItemCount = computed(() => {
  const items = new Set<string>();
  for (const entry of matchingEntries.value) {
    for (const item of entry.st.item_stats) {
      items.add(item.item_name);
    }
  }
  return items.size;
});

interface BonusEntry {
  zone: string;
  bonusStats: CategorySpeedBonusStats;
}

const entriesWithBonusData = computed<BonusEntry[]>(() => {
  if (!firstEntry.value) return [];
  const category = firstEntry.value.category;
  const entries: BonusEntry[] = [];
  for (const zone of props.zones) {
    const hasSurveyType = zone.survey_type_stats.some(s => s.survey_type === props.surveyTypeName);
    if (!hasSurveyType) continue;
    const bonusStats = zone.speed_bonus_stats.find(c => c.category === category);
    if (bonusStats && bonusStats.item_stats.length > 0) {
      entries.push({ zone: zone.zone, bonusStats });
    }
  }
  return entries;
});

// Resolve item names for chart labels
const allItemNames = computed(() => {
  const names = new Set<string>();
  for (const entry of matchingEntries.value) {
    for (const item of entry.st.item_stats) {
      names.add(item.item_name);
    }
  }
  return [...names];
});

watch(allItemNames, async (names) => {
  if (names.length === 0) return;
  const resolved = await gameData.resolveItemsBatch(names);
  const map: Record<string, string> = {};
  for (const [key, info] of Object.entries(resolved)) {
    map[key] = info.name;
  }
  resolvedNames.value = map;
}, { immediate: true });

function displayName(internalName: string): string {
  return resolvedNames.value[internalName] ?? internalName;
}

// Item quantity donut chart — aggregate across zones
const itemChartData = computed(() => {
  const map = new Map<string, number>();
  for (const entry of matchingEntries.value) {
    for (const item of entry.st.item_stats) {
      map.set(item.item_name, (map.get(item.item_name) ?? 0) + item.total_quantity);
    }
  }
  return [...map.entries()]
    .map(([name, quantity]) => ({ name, quantity }))
    .sort((a, b) => b.quantity - a.quantity);
});

const chartPalette = [
  "#7ec8e3", "#c87e7e", "#6366f1", "#f59e0b", "#10b981",
  "#ef4444", "#8b5cf6", "#ec4899", "#14b8a6", "#f97316",
];

const itemChartDataset = computed<VueUiDonutDatasetItem[]>(() => {
  const total = itemChartData.value.reduce((sum, d) => sum + d.quantity, 0);
  const threshold = total * 0.02;
  const result: VueUiDonutDatasetItem[] = [];
  let otherQty = 0;
  let colorIdx = 0;

  for (const item of itemChartData.value) {
    if (item.quantity >= threshold) {
      result.push({
        name: displayName(item.name),
        color: chartPalette[colorIdx % chartPalette.length],
        values: [item.quantity],
      });
      colorIdx++;
    } else {
      otherQty += item.quantity;
    }
  }

  if (otherQty > 0) {
    result.push({
      name: "Other",
      color: "#52525b",
      values: [otherQty],
    });
  }

  return result;
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
              text: "Items",
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

function formatZone(zone: string): string {
  return zone.replace(/([a-z])([A-Z])/g, "$1 $2");
}

function formatGold(amount: number): string {
  const rounded = Math.round(amount);
  if (rounded >= 0) return rounded.toLocaleString() + "g";
  return "-" + Math.abs(rounded).toLocaleString() + "g";
}
</script>
