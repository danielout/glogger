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
      <div class="p-4 flex-1 flex flex-col">
        <MenuBar :currentView="currentView" @navigate="navigateToView" />

        <div class="flex-1 flex flex-col">
          <div class="flex-1">
            <template v-if="currentView === 'skills'">
              <SkillGrid />
            </template>
            <template v-else-if="currentView === 'surveying'">
              <SurveyView />
            </template>
            <template v-else-if="currentView === 'character'">
              <CharacterView />
            </template>
            <template v-else-if="currentView === 'inventory'">
              <InventoryView />
            </template>
            <template v-else-if="currentView === 'data-browser'">
              <DataBrowser :nav-target="entityNavTarget" />
            </template>
            <template v-else-if="currentView === 'chat'">
              <ChatView />
            </template>
            <template v-else-if="currentView === 'settings'">
              <Settings
                :watching="watching"
                :parsing="parsing"
                :error="error"
                :onStartWatching="startWatching"
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
import InventoryView from "./components/Character/InventoryView.vue";
import Settings from "./components/Settings.vue";
import StartupSplash from "./components/Startup/StartupSplash.vue";
import StartupLayout from "./components/Startup/StartupLayout.vue";
import StartupProgress from "./components/Startup/StartupProgress.vue";
import SetupPathStep from "./components/Startup/SetupPathStep.vue";
import SetupWatchersStep from "./components/Startup/SetupWatchersStep.vue";
import SetupCharacterStep from "./components/Startup/SetupCharacterStep.vue";
import CharacterSelect from "./components/Startup/CharacterSelect.vue";

const skillStore = useSkillStore();
const surveyStore = useSurveyStore();
const settingsStore = useSettingsStore();
const coordinator = useCoordinatorStore();
const characterStore = useCharacterStore();
const startup = useStartupStore();

const logPath = ref("");
const error = ref("");
const watching = ref(false);
const parsing = ref(false);
const currentView = ref<AppView>("skills");
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
  });
  await listen("survey-event", (event: any) => {
    surveyStore.handleSurveyEvent(event.payload);
  });

  // Auto-watch on startup if enabled (legacy)
  if (settingsStore.settings.autoWatchOnStartup && logPath.value) {
    await startWatching();
  }

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

async function startWatching() {
  error.value = "";
  skillStore.reset();
  surveyStore.reset();
  try {
    await invoke("start_watching", { path: logPath.value });
    watching.value = true;
  } catch (e) {
    error.value = String(e);
  }
}

async function parseLog() {
  error.value = "";
  skillStore.reset();
  surveyStore.reset();
  parsing.value = true;
  try {
    await invoke("parse_log", { path: logPath.value });
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
