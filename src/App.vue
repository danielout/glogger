<template>
  <div class="h-screen flex flex-col overflow-hidden">
    <!-- Startup phases -->
    <StartupSplash v-if="startup.phase === 'splash'" />

    <StartupLayout v-else-if="startup.isSetupWizard" :current-step="startup.setupStepIndex">
      <SetupPathStep v-if="startup.phase === 'setup-path'" />
      <SetupWatchersStep v-else-if="startup.phase === 'setup-watchers'" />
      <SetupCharacterStep v-else-if="startup.phase === 'setup-character'" />
    </StartupLayout>

    <CharacterSelect v-else-if="startup.phase === 'select-character'" />

    <StartupProgress v-else-if="startup.phase === 'loading'" :tasks="startup.startupTasks" :error-message="startup.error" />

    <!-- Main app -->
    <template v-else-if="startup.phase === 'ready'">
      <div class="flex-1 flex flex-col min-h-0 overflow-hidden">
        <MenuBar
          ref="menuBarRef"
          :currentView="currentView"
          @navigate="navigateToView"
          @update:sub-tab="onSubTabChange"
        />

        <div class="flex-1 flex flex-col p-4 min-h-0 transition-[padding] duration-250 ease-out" :class="hasSubTabs ? 'pt-28' : 'pt-20'">
          <div class="flex-1 min-h-0">
            <div v-if="visited.has('dashboard')" v-show="currentView === 'dashboard'" class="h-full overflow-y-auto">
              <DashboardView />
            </div>
            <div v-if="visited.has('character')" v-show="currentView === 'character'" class="h-full">
              <CharacterView :active-tab="activeSubTab" />
            </div>
            <div v-if="visited.has('inventory')" v-show="currentView === 'inventory'" class="h-full">
              <InventoryWrapper :active-tab="activeSubTab" />
            </div>
            <div v-if="visited.has('crafting')" v-show="currentView === 'crafting'" class="h-full">
              <CraftingView :active-tab="activeSubTab" />
            </div>
            <div v-if="visited.has('economics')" v-show="currentView === 'economics'" class="h-full">
              <EconomicsView :active-tab="activeSubTab" />
            </div>
            <div v-if="visited.has('chat')" v-show="currentView === 'chat'" class="h-full overflow-y-auto">
              <ChatView :active-tab="activeSubTab" />
            </div>
            <div v-if="visited.has('data-browser')" v-show="currentView === 'data-browser'" class="h-full">
              <DataBrowser :nav-target="entityNavTarget" :active-tab="activeSubTab" />
            </div>
            <div v-if="visited.has('search')" v-show="currentView === 'search'" class="h-full overflow-y-auto">
              <EmptyState primary="Search" secondary="Coming soon." />
            </div>
            <div v-if="visited.has('settings')" v-show="currentView === 'settings'" class="h-full overflow-y-auto">
              <Settings
                :parsing="parsing"
                :error="error"
                :onParseLog="parseLog" />
            </div>
            <div v-if="visited.has('help')" v-show="currentView === 'help'" class="h-full overflow-y-auto">
              <EmptyState primary="Help" secondary="Coming soon." />
            </div>
          </div>
        </div>
      </div>

      <!-- Bottom bar -->
      <div class="shrink-0 h-6 bg-surface-base border-t border-border-default flex items-center px-3">
        <span class="text-text-dim text-[0.6rem]">glogger v0.1 DEV</span>
      </div>

      <ToastContainer />
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useSettingsStore } from "./stores/settingsStore";
import { useStartupStore } from "./stores/startupStore";
import { useGameStateStore } from "./stores/gameStateStore";
import { useSurveyStore } from "./stores/surveyStore";
import { provideEntityNavigation, type EntityNavigationTarget } from "./composables/useEntityNavigation";
import MenuBar, { type AppView } from "./components/MenuBar.vue";
import DashboardView from "./components/Dashboard/DashboardView.vue";
import CharacterView from "./components/Character/CharacterView.vue";
import InventoryWrapper from "./components/Inventory/InventoryWrapper.vue";
import CraftingView from "./components/Crafting/CraftingView.vue";
import EconomicsView from "./components/Economics/EconomicsView.vue";
import ChatView from "./components/Chat/ChatView.vue";
import DataBrowser from "./components/DataBrowser/DataBrowser.vue";
import EmptyState from "./components/Shared/EmptyState.vue";
import Settings from "./components/Settings.vue";
import StartupSplash from "./components/Startup/StartupSplash.vue";
import StartupLayout from "./components/Startup/StartupLayout.vue";
import StartupProgress from "./components/Startup/StartupProgress.vue";
import SetupPathStep from "./components/Startup/SetupPathStep.vue";
import SetupWatchersStep from "./components/Startup/SetupWatchersStep.vue";
import SetupCharacterStep from "./components/Startup/SetupCharacterStep.vue";
import CharacterSelect from "./components/Startup/CharacterSelect.vue";
import ToastContainer from "./components/Shared/ToastContainer.vue";

const settingsStore = useSettingsStore();
const startup = useStartupStore();

const error = ref("");
const parsing = ref(false);
const currentView = ref<AppView>("dashboard");
const entityNavTarget = ref<EntityNavigationTarget | null>(null);
const visited = reactive(new Set<AppView>(["dashboard"]));
const menuBarRef = ref<InstanceType<typeof MenuBar> | null>(null);
const activeSubTab = ref("");

const hasSubTabs = computed(() => menuBarRef.value?.hasTabs ?? false);

function onSubTabChange(tab: string) {
  activeSubTab.value = tab;
}

provideEntityNavigation((target) => {
  visited.add("data-browser");
  currentView.value = "data-browser";
  entityNavTarget.value = { ...target };
  const entityTypeToTab: Record<string, string> = {
    item: "items", skill: "skills", ability: "abilities", recipe: "recipes",
    quest: "quests", npc: "npcs", effect: "effects", title: "titles",
  };
  const tab = entityTypeToTab[target.type];
  if (tab && menuBarRef.value) {
    menuBarRef.value.activeSubTabs["data-browser"] = tab;
    activeSubTab.value = tab;
  }
});

onMounted(async () => {
  await startup.initialize();
});

async function parseLog() {
  error.value = "";
  const gameStateStore = useGameStateStore();
  const surveyStore = useSurveyStore();
  gameStateStore.resetSessionSkills();
  surveyStore.reset();
  parsing.value = true;
  try {
    const path = settingsStore.settings.logFilePath || settingsStore.getPlayerLogPath();
    await invoke("parse_log", { path });
  } catch (e) {
    error.value = String(e);
  } finally {
    parsing.value = false;
  }
}

function navigateToView(view: AppView) {
  visited.add(view);
  currentView.value = view;
}
</script>
