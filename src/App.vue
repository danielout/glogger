<template>
  <div class="min-h-screen flex flex-col">
    <!-- Startup phases -->
    <StartupSplash v-if="startup.phase === 'splash'" />

    <StartupLayout v-else-if="startup.isSetupWizard" :current-step="startup.setupStepIndex">
      <SetupPathStep v-if="startup.phase === 'setup-path'" />
      <SetupWatchersStep v-else-if="startup.phase === 'setup-watchers'" />
      <SetupCharacterStep v-else-if="startup.phase === 'setup-character'" />
    </StartupLayout>

    <CharacterSelect v-else-if="startup.phase === 'select-character'" />

    <StartupProgress v-else-if="startup.phase === 'loading'" :tasks="startup.startupTasks" />

    <!-- Main app -->
    <template v-else-if="startup.phase === 'ready'">
      <div class="flex-1 flex flex-col">
        <MenuBar :currentView="currentView" @navigate="navigateToView" />

        <div class="flex-1 flex flex-col p-4 pt-20">
          <div class="flex-1">
            <template v-if="currentView === 'dashboard'">
              <DashboardView />
            </template>
            <template v-else-if="currentView === 'skills'">
              <SkillGrid />
            </template>
            <template v-else-if="currentView === 'surveying'">
              <SurveyView />
            </template>
            <template v-else-if="currentView === 'character'">
              <CharacterView />
            </template>
            <template v-else-if="currentView === 'inventory'">
              <InventoryWrapper />
            </template>
            <template v-else-if="currentView === 'data-browser'">
              <DataBrowser :nav-target="entityNavTarget" />
            </template>
            <template v-else-if="currentView === 'chat'">
              <ChatView />
            </template>
            <template v-else-if="currentView === 'gourmand'">
              <GourmandView />
            </template>
            <template v-else-if="currentView === 'farming'">
              <FarmingView />
            </template>
            <template v-else-if="currentView === 'crafting'">
              <CraftingView />
            </template>
            <template v-else-if="currentView === 'settings'">
              <Settings
                :parsing="parsing"
                :error="error"
                :onParseLog="parseLog" />
            </template>
          </div>
        </div>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useSkillStore } from "./stores/skillStore";
import { useSurveyStore } from "./stores/surveyStore";
import { useSettingsStore } from "./stores/settingsStore";
import { useCoordinatorStore } from "./stores/coordinatorStore";
import { useCharacterStore } from "./stores/characterStore";
import { useStartupStore } from "./stores/startupStore";
import { provideEntityNavigation, type EntityNavigationTarget } from "./composables/useEntityNavigation";
import MenuBar, { type AppView } from "./components/MenuBar.vue";
import SkillGrid from "./components/Shared/SkillGrid.vue";
import SurveyView from "./components/Surveying/SurveyView.vue";
import DataBrowser from "./components/DataBrowser/DataBrowser.vue";
import ChatView from "./components/Chat/ChatView.vue";
import CharacterView from "./components/Character/CharacterView.vue";
import InventoryWrapper from "./components/Inventory/InventoryWrapper.vue";
import { useInventoryStore } from "./stores/inventoryStore";
import GourmandView from "./components/Gourmand/GourmandView.vue";
import FarmingView from "./components/Farming/FarmingView.vue";
import CraftingView from "./components/Crafting/CraftingView.vue";
import DashboardView from "./components/Dashboard/DashboardView.vue";
import Settings from "./components/Settings.vue";
import StartupSplash from "./components/Startup/StartupSplash.vue";
import StartupLayout from "./components/Startup/StartupLayout.vue";
import StartupProgress from "./components/Startup/StartupProgress.vue";
import SetupPathStep from "./components/Startup/SetupPathStep.vue";
import SetupWatchersStep from "./components/Startup/SetupWatchersStep.vue";
import SetupCharacterStep from "./components/Startup/SetupCharacterStep.vue";
import CharacterSelect from "./components/Startup/CharacterSelect.vue";

import { useFarmingStore } from "./stores/farmingStore";
import type { PlayerEvent } from "./types/playerEvents";

const skillStore = useSkillStore();
const surveyStore = useSurveyStore();
const farmingStore = useFarmingStore();
const settingsStore = useSettingsStore();
const coordinator = useCoordinatorStore();
const characterStore = useCharacterStore();
// Instantiate to activate event listeners (side-effect only)
useInventoryStore();
const startup = useStartupStore();

const logPath = ref("");
const error = ref("");
const parsing = ref(false);
const currentView = ref<AppView>("dashboard");
const entityNavTarget = ref<EntityNavigationTarget | null>(null);

provideEntityNavigation((target) => {
  currentView.value = "data-browser";
  entityNavTarget.value = { ...target };
});

// When the startup flow reaches 'ready', initialize the main app
watch(
  () => startup.phase,
  async (newPhase) => {
    if (newPhase === "ready") {
      await initializeMainApp();
    }
  }
);

onMounted(async () => {
  await startup.initialize();
});

async function initializeMainApp() {
  logPath.value = settingsStore.settings.logFilePath || settingsStore.getPlayerLogPath();

  await listen("skill-update", (event: any) => {
    skillStore.handleUpdate(event.payload);
    surveyStore.handleSkillUpdate(event.payload);
    farmingStore.handleSkillUpdate(event.payload);
  });
  await listen("survey-event", (event: any) => {
    console.log("[survey-event] Received:", event.payload);
    surveyStore.handleSurveyEvent(event.payload);
  });
  await listen<number>("survey-session-ended", (event) => {
    console.log("[survey-session-ended] Session finalized:", event.payload);
    surveyStore.handleSessionEnded(event.payload);
  });
  await listen<PlayerEvent>("player-event", (event) => {
    farmingStore.handlePlayerEvent(event.payload);
  });

  // Start coordinator polling for log watchers
  coordinator.startPolling(1500);

  // Auto-start coordinator watchers if enabled
  if (settingsStore.settings.autoTailPlayerLog && settingsStore.settings.gameDataPath) {
    try {
      await coordinator.startPlayerTailing();
    } catch (e) {
      console.error("Failed to auto-start player log tailing:", e);
    }
  }

  if (settingsStore.settings.autoTailChat && settingsStore.settings.gameDataPath) {
    try {
      await coordinator.startChatTailing();
    } catch (e) {
      console.error("Failed to auto-start chat log tailing:", e);
    }
  }

  // Start report folder watching
  characterStore.startReportWatching();
}

async function parseLog() {
  error.value = "";
  skillStore.reset();
  surveyStore.reset();
  parsing.value = true;
  try {
    // Use the latest path from settings (may have been updated by file picker)
    const path = settingsStore.settings.logFilePath || logPath.value;
    await invoke("parse_log", { path });
  } catch (e) {
    error.value = String(e);
  } finally {
    parsing.value = false;
  }
}

function navigateToView(view: AppView) {
  currentView.value = view;
}
</script>
