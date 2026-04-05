<template>
  <div class="flex flex-col gap-4">
    <div class="flex justify-between items-center">
      <h3 class="text-lg text-[#7ec8e3] m-0">Survey Analytics</h3>
      <div class="flex items-center gap-2">
        <!-- Data source toggle (only show when imports exist) -->
        <div v-if="hasImports" class="flex rounded border border-border-default overflow-hidden text-xs">
          <button
            @click="includeImports = false"
            :class="[
              'px-3 py-1.5 transition-all',
              !includeImports
                ? 'bg-accent-gold/20 text-accent-gold border-r border-accent-gold/30'
                : 'bg-surface-elevated text-text-muted hover:text-text-secondary border-r border-border-default'
            ]"
          >My Data</button>
          <button
            @click="includeImports = true"
            :class="[
              'px-3 py-1.5 transition-all',
              includeImports
                ? 'bg-accent-gold/20 text-accent-gold'
                : 'bg-surface-elevated text-text-muted hover:text-text-secondary'
            ]"
          >All Data</button>
        </div>

        <button @click="handleExport" :disabled="exporting"
          class="px-3 py-1.5 text-sm bg-surface-elevated border border-border-default rounded text-text-secondary hover:text-text-primary hover:border-border-hover transition-all"
          title="Export your survey data to share with others">
          {{ exporting ? "Exporting..." : "Export" }}
        </button>
        <button @click="handleImport" :disabled="importing"
          class="px-3 py-1.5 text-sm bg-surface-elevated border border-border-default rounded text-text-secondary hover:text-text-primary hover:border-border-hover transition-all"
          title="Import survey data from another player">
          {{ importing ? "Importing..." : "Import" }}
        </button>
        <button v-if="hasImports" @click="showImportManager = true"
          class="px-3 py-1.5 text-sm bg-surface-elevated border border-border-default rounded text-text-secondary hover:text-text-primary hover:border-border-hover transition-all"
          title="Manage imported data sets">
          Imports ({{ importCount }})
        </button>
        <button @click="loadAll" :disabled="loading"
          class="px-3 py-1.5 text-sm bg-surface-elevated border border-border-default rounded text-text-secondary hover:text-text-primary hover:border-border-hover transition-all">
          {{ loading ? "Loading..." : "Refresh" }}
        </button>
      </div>
    </div>

    <div v-if="error" class="text-[#c87e7e] bg-[#2a1a1a] border border-[#5a3a3a] rounded p-3 text-sm">{{ error }}</div>

    <!-- Global Summary (quick overview) -->
    <div v-if="speedStats" class="bg-surface-card border border-border-default rounded p-4">
      <div class="text-[0.65rem] uppercase tracking-widest text-[#7ec8e3] mb-3 font-bold">
        {{ includeImports && hasImports ? 'Combined Overview (Your Data + Imports)' : 'All-Time Overview' }}
      </div>
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
          <div class="text-[0.65rem] text-text-muted uppercase tracking-wide">Zones Active</div>
          <div class="text-lg font-bold text-text-primary">{{ zones.length }}</div>
        </div>
      </div>
    </div>

    <!-- Item Cost Calculator -->
    <ItemCostCalculator v-if="speedStats" :include-imports="includeImports" />

    <!-- Zone Accordions -->
    <AccordionSection
      v-for="zone in zones"
      :key="zone.zone"
      :default-open="zones.length === 1"
    >
      <template #title>{{ formatZone(zone.zone) }}</template>
      <template #badge>
        <span class="text-[0.6rem] text-text-dim font-mono">
          {{ zoneTotalSurveys(zone) }} surveys
        </span>
      </template>

      <div class="flex flex-col gap-4 mt-2">
        <!-- Category sections (mineral / mining) -->
        <div
          v-for="cat in zone.speed_bonus_stats"
          :key="cat.category"
          class="bg-[#1a1a2e] border border-border-light rounded-lg p-4"
        >
          <div class="text-[0.65rem] uppercase tracking-widest mb-3 font-bold"
               :class="cat.category === 'mineral' ? 'text-[#7ec8e3]' : 'text-[#c87e7e]'">
            {{ cat.category === 'mineral' ? 'Mineral Surveys' : 'Mining Surveys' }}
            <span class="text-text-dim font-normal ml-2">
              {{ cat.total_surveys }} completed &middot; {{ cat.speed_bonus_count }} bonuses ({{ cat.speed_bonus_rate.toFixed(1) }}%)
            </span>
          </div>

          <!-- Speed Bonus Item Stats -->
          <div v-if="cat.item_stats.length > 0" class="mb-4">
            <div class="text-[0.6rem] uppercase tracking-widest text-[#c8b47e] mb-2 font-bold">
              Speed Bonus Items
              <span v-if="cat.avg_bonus_value > 0" class="text-text-dim font-normal ml-2">
                avg value per proc: {{ formatGold(cat.avg_bonus_value) }}
              </span>
            </div>
            <div class="flex flex-col gap-1">
              <div class="grid grid-cols-[1fr_60px_60px_60px_60px_60px_80px] gap-3 px-3 py-1.5 text-[0.6rem] uppercase tracking-wide text-text-muted font-bold">
                <div>Item</div>
                <div class="text-right">Total</div>
                <div class="text-right">Seen</div>
                <div class="text-right">Min</div>
                <div class="text-right">Max</div>
                <div class="text-right">Avg</div>
                <div class="text-right">Out of</div>
              </div>
              <div
                v-for="item in cat.item_stats"
                :key="item.item_name"
                class="grid grid-cols-[1fr_60px_60px_60px_60px_60px_80px] gap-3 px-3 py-1.5 text-xs bg-black/20 border border-border-default rounded hover:bg-black/30"
              >
                <div class="min-w-0"><ItemInline :reference="item.item_name" /></div>
                <div class="text-right font-mono text-[#c8b47e]">{{ item.total_quantity }}</div>
                <div class="text-right font-mono text-text-secondary">{{ item.times_seen }}</div>
                <div class="text-right font-mono">{{ item.min_per_proc }}</div>
                <div class="text-right font-mono">{{ item.max_per_proc }}</div>
                <div class="text-right font-mono text-text-primary">{{ item.avg_per_proc.toFixed(1) }}</div>
                <div class="text-right font-mono text-text-dim">{{ item.total_procs }} procs</div>
              </div>
            </div>
          </div>

          <!-- Per-Survey-Type Breakdown for this category -->
          <div v-for="st in surveyTypesForCategory(zone, cat.category)" :key="st.survey_type">
            <div class="flex items-center gap-3 mb-2 mt-3">
              <span class="text-xs font-semibold text-text-primary">{{ st.survey_type }}</span>
              <span class="text-[0.6rem] text-text-dim">
                {{ st.total_completed }} completed &middot; cost: {{ formatGold(st.crafting_cost) }}
              </span>
            </div>
            <div v-if="st.item_stats.length > 0" class="flex flex-col gap-1 ml-2">
              <div class="grid grid-cols-[1fr_60px_60px_60px_60px_60px] gap-3 px-3 py-1 text-[0.6rem] uppercase tracking-wide text-text-muted font-bold">
                <div>Item</div>
                <div class="text-right">Total</div>
                <div class="text-right">Seen</div>
                <div class="text-right">Min</div>
                <div class="text-right">Max</div>
                <div class="text-right">Avg</div>
              </div>
              <div
                v-for="item in st.item_stats"
                :key="item.item_name"
                class="grid grid-cols-[1fr_60px_60px_60px_60px_60px] gap-3 px-3 py-1.5 text-xs bg-black/10 border border-border-default rounded hover:bg-black/20"
              >
                <div class="min-w-0"><ItemInline :reference="item.item_name" /></div>
                <div class="text-right font-mono text-text-primary">{{ item.total_quantity }}</div>
                <div class="text-right font-mono text-text-secondary">{{ item.times_seen }}/{{ st.total_completed }}</div>
                <div class="text-right font-mono">{{ item.min_per_completion }}</div>
                <div class="text-right font-mono">{{ item.max_per_completion }}</div>
                <div class="text-right font-mono text-text-primary">{{ item.avg_per_completion.toFixed(1) }}</div>
              </div>
            </div>
            <div v-else class="text-text-dim italic text-xs ml-2">No loot data recorded.</div>
          </div>
        </div>
      </div>
    </AccordionSection>

    <!-- Speed Bonus Rates by Zone -->
    <AccordionSection v-if="zones.length > 1" :default-open="false">
      <template #title>Speed Bonus Rates by Zone</template>
      <template #badge>
        <span class="text-[0.6rem] text-text-dim font-mono">{{ zones.length }} zones</span>
      </template>
      <SpeedBonusChart :zones="zones" />
    </AccordionSection>

    <!-- Cross-Zone Comparison -->
    <AccordionSection v-if="zones.length > 1" :default-open="false">
      <template #title>Cross-Zone Comparison</template>
      <template #badge>
        <span class="text-[0.6rem] text-text-dim font-mono">{{ zones.length }} zones</span>
      </template>
      <CrossZoneComparison :zones="zones" />
    </AccordionSection>

    <!-- Empty state -->
    <EmptyState
      v-if="!loading && zones.length === 0 && !speedStats"
      variant="panel"
      primary="No survey data yet"
      secondary="Complete some surveys to see analytics here." />

    <!-- Import Manager Modal -->
    <SurveyImportManager
      v-if="showImportManager"
      @close="showImportManager = false"
      @deleted="handleImportDeleted"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { save, open } from "@tauri-apps/plugin-dialog";
import { useToast } from "../../composables/useToast";
import { useSettingsStore } from "../../stores/settingsStore";
import { useSurveyStore } from "../../stores/surveyStore";
import EmptyState from "../Shared/EmptyState.vue";
import AccordionSection from "../Shared/AccordionSection.vue";
import ItemInline from "../Shared/Item/ItemInline.vue";
import ItemCostCalculator from "./Analytics/ItemCostCalculator.vue";
import SpeedBonusChart from "./Analytics/SpeedBonusChart.vue";
import CrossZoneComparison from "./Analytics/CrossZoneComparison.vue";
import SurveyImportManager from "./SurveyImportManager.vue";

interface SpeedBonusStats {
  total_surveys: number;
  speed_bonus_count: number;
  speed_bonus_rate: number;
  total_bonus_items: number;
  unique_bonus_items: number;
}

interface SpeedBonusItemStats {
  item_name: string;
  total_quantity: number;
  times_seen: number;
  total_procs: number;
  min_per_proc: number;
  max_per_proc: number;
  avg_per_proc: number;
}

interface CategorySpeedBonusStats {
  category: string;
  total_surveys: number;
  speed_bonus_count: number;
  speed_bonus_rate: number;
  avg_bonus_value: number;
  item_stats: SpeedBonusItemStats[];
}

interface SurveyItemStats {
  item_name: string;
  total_quantity: number;
  times_seen: number;
  min_per_completion: number;
  max_per_completion: number;
  avg_per_completion: number;
}

interface SurveyTypeAnalytics {
  survey_type: string;
  category: string;
  crafting_cost: number;
  total_completed: number;
  item_stats: SurveyItemStats[];
}

interface ZoneAnalytics {
  zone: string;
  speed_bonus_stats: CategorySpeedBonusStats[];
  survey_type_stats: SurveyTypeAnalytics[];
}

interface SurveyImportInfo {
  id: number;
  label: string;
  source_player: string | null;
  session_count: number;
  event_count: number;
  imported_at: string;
}

const toast = useToast();
const settingsStore = useSettingsStore();
const surveyStore = useSurveyStore();
const loading = ref(false);
const exporting = ref(false);
const importing = ref(false);
const error = ref("");
const speedStats = ref<SpeedBonusStats | null>(null);
const zones = ref<ZoneAnalytics[]>([]);
const includeImports = ref(false);
const showImportManager = ref(false);
const importCount = ref(0);
const hasImports = ref(false);

onMounted(() => {
  loadAll();
  refreshImportCount();
});

watch(includeImports, () => {
  loadAll();
});

// Auto-reload when a survey session is finalized (auto-end or manual end)
watch(() => surveyStore.sessionFinalizedCounter, () => {
  loadAll();
});

async function refreshImportCount() {
  try {
    const imports = await invoke<SurveyImportInfo[]>("get_survey_imports");
    importCount.value = imports.length;
    hasImports.value = imports.length > 0;
  } catch {
    // ignore
  }
}

async function loadAll() {
  loading.value = true;
  error.value = "";
  try {
    const [speed, zoneData] = await Promise.all([
      invoke<SpeedBonusStats>("get_speed_bonus_stats", {
        sessionId: null,
        includeImports: includeImports.value,
      }),
      invoke<ZoneAnalytics[]>("get_zone_analytics", {
        includeImports: includeImports.value,
      }),
    ]);
    speedStats.value = speed;
    zones.value = zoneData;
  } catch (e) {
    error.value = `Failed to load analytics: ${e}`;
  } finally {
    loading.value = false;
  }
}

async function handleExport() {
  exporting.value = true;
  try {
    const filePath = await save({
      filters: [{ name: "Survey Data", extensions: ["glogger-survey"] }],
      defaultPath: "survey-data.glogger-survey",
    });
    if (!filePath) return;

    const json = await invoke<string>("export_survey_data", {
      exporterName: settingsStore.settings.activeCharacterName ?? null,
      serverName: settingsStore.settings.activeServerName ?? null,
    });
    await invoke("export_text_file", { filePath, content: json });
    toast.success("Survey data exported successfully");
  } catch (e) {
    toast.error(`Export failed: ${e}`);
  } finally {
    exporting.value = false;
  }
}

async function handleImport() {
  importing.value = true;
  try {
    const filePath = await open({
      filters: [{ name: "Survey Data", extensions: ["glogger-survey"] }],
      multiple: false,
    });
    if (!filePath) return;

    const result = await invoke<{
      import_id: number;
      label: string;
      sessions_imported: number;
      events_imported: number;
      loot_items_imported: number;
    }>("import_survey_data_from_file", { filePath, label: null });

    toast.success(
      `Imported "${result.label}": ${result.sessions_imported} sessions, ${result.events_imported} events`
    );
    await refreshImportCount();
    // If toggle is on "All Data", reload to show new data
    if (includeImports.value) {
      loadAll();
    }
  } catch (e) {
    toast.error(`Import failed: ${e}`);
  } finally {
    importing.value = false;
  }
}

function handleImportDeleted() {
  refreshImportCount();
  // Reload analytics if we're showing combined data
  if (includeImports.value) {
    loadAll();
  }
}

function zoneTotalSurveys(zone: ZoneAnalytics): number {
  return zone.speed_bonus_stats.reduce((sum, c) => sum + c.total_surveys, 0);
}

function surveyTypesForCategory(zone: ZoneAnalytics, category: string): SurveyTypeAnalytics[] {
  return zone.survey_type_stats.filter(st => st.category === category);
}

function formatZone(zone: string): string {
  // Convert camelCase zone names to spaced: "KurMountains" → "Kur Mountains"
  return zone.replace(/([a-z])([A-Z])/g, "$1 $2");
}

function formatGold(amount: number): string {
  const rounded = Math.round(amount);
  if (rounded >= 0) return rounded.toLocaleString() + "g";
  return "-" + Math.abs(rounded).toLocaleString() + "g";
}
</script>
