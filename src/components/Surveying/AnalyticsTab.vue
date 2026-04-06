<template>
  <PaneLayout
    screen-key="survey-analytics"
    :left-pane="{ title: 'Views', defaultWidth: 220, minWidth: 160, maxWidth: 350 }"
    :right-pane="{ title: 'Item Calculator', defaultWidth: 400, minWidth: 280, maxWidth: 600 }"
  >
    <!-- Left pane: View navigation -->
    <template #left>
      <AnalyticsViewNav
        :zones="zones"
        :selected-view="selectedView"
        @update:selected-view="setView"
      />
    </template>

    <!-- Center: Header + active view -->
    <div class="flex flex-col h-full min-h-0">
      <AnalyticsHeader
        :speed-stats="speedStats"
        :zone-count="zones.length"
        :include-imports="includeImports"
        :has-imports="hasImports"
        :import-count="importCount"
        :loading="loading"
        :exporting="exporting"
        :importing="importing"
        @update:include-imports="includeImports = $event"
        @export="handleExport"
        @import="handleImport"
        @manage-imports="showImportManager = true"
        @refresh="loadAll"
      />

      <div v-if="error" class="text-[#c87e7e] bg-[#2a1a1a] border border-[#5a3a3a] rounded p-2 text-xs mx-3 mt-2">
        {{ error }}
      </div>

      <div class="flex-1 min-h-0 overflow-y-auto p-3">
        <!-- Overview -->
        <OverviewView
          v-if="viewType === 'overview'"
          :zones="zones"
          :speed-stats="speedStats"
        />

        <!-- Zone detail -->
        <ZoneDetailView
          v-else-if="viewType === 'zone' && selectedZone"
          :zone="selectedZone"
        />

        <!-- Survey type detail -->
        <SurveyTypeDetailView
          v-else-if="viewType === 'surveytype' && selectedSurveyTypeName"
          :survey-type-name="selectedSurveyTypeName"
          :zones="zones"
        />

        <!-- Empty state -->
        <EmptyState
          v-else-if="!loading && zones.length === 0 && !speedStats"
          variant="panel"
          primary="No survey data yet"
          secondary="Complete some surveys to see analytics here."
        />
      </div>
    </div>

    <!-- Right pane: Item Cost Calculator -->
    <template #right>
      <div class="p-2">
        <ItemCostCalculator :include-imports="includeImports" />
      </div>
    </template>
  </PaneLayout>

  <!-- Import Manager Modal -->
  <SurveyImportManager
    v-if="showImportManager"
    @close="showImportManager = false"
    @deleted="handleImportDeleted"
  />
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { save, open } from "@tauri-apps/plugin-dialog";
import { useToast } from "../../composables/useToast";
import { useViewPrefs } from "../../composables/useViewPrefs";
import { useSettingsStore } from "../../stores/settingsStore";
import { useSurveyStore } from "../../stores/surveyStore";
import type {
  SpeedBonusStats,
  ZoneAnalytics,
  SurveyImportInfo,
} from "../../types/database";
import PaneLayout from "../Shared/PaneLayout.vue";
import EmptyState from "../Shared/EmptyState.vue";
import AnalyticsViewNav from "./Analytics/AnalyticsViewNav.vue";
import AnalyticsHeader from "./Analytics/AnalyticsHeader.vue";
import ItemCostCalculator from "./Analytics/ItemCostCalculator.vue";
import OverviewView from "./Analytics/OverviewView.vue";
import ZoneDetailView from "./Analytics/ZoneDetailView.vue";
import SurveyTypeDetailView from "./Analytics/SurveyTypeDetailView.vue";
import SurveyImportManager from "./SurveyImportManager.vue";

const toast = useToast();
const settingsStore = useSettingsStore();
const surveyStore = useSurveyStore();

const { prefs: viewPrefs, update: updateViewPrefs } = useViewPrefs("survey-analytics.view", {
  selectedView: "overview",
});

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

const selectedView = ref(viewPrefs.value.selectedView as string);

// Parse view type and param from selectedView
const viewType = computed(() => {
  const v = selectedView.value;
  if (v.startsWith("zone:")) return "zone";
  if (v.startsWith("surveytype:")) return "surveytype";
  return "overview";
});

const selectedZone = computed(() => {
  if (viewType.value !== "zone") return null;
  const zoneName = selectedView.value.slice("zone:".length);
  return zones.value.find(z => z.zone === zoneName) ?? null;
});

const selectedSurveyTypeName = computed(() => {
  if (viewType.value !== "surveytype") return null;
  return selectedView.value.slice("surveytype:".length);
});

function setView(view: string) {
  selectedView.value = view;
  updateViewPrefs({ selectedView: view });
}

// Fallback to overview if persisted view no longer exists in data
watch(zones, () => {
  if (viewType.value === "zone" && !selectedZone.value) {
    setView("overview");
  }
  if (viewType.value === "surveytype" && selectedSurveyTypeName.value) {
    const exists = zones.value.some(z =>
      z.survey_type_stats.some(st => st.survey_type === selectedSurveyTypeName.value)
    );
    if (!exists) setView("overview");
  }
});

onMounted(() => {
  loadAll();
  refreshImportCount();
});

watch(includeImports, () => {
  loadAll();
});

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
  if (includeImports.value) {
    loadAll();
  }
}
</script>
