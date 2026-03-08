<script setup lang="ts">
import { ref, onMounted, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useSkillStore } from "./stores/skillStore";
import { useSurveyStore } from "./stores/surveyStore";
import { useSettingsStore } from "./stores/settingsStore";
import MenuBar, { type AppView } from "./components/MenuBar.vue";
import SkillGrid from "./components/SkillGrid.vue";
import SurveySessionCard from "./components/SurveySessionCard.vue";
import SurveyLog from "./components/SurveyLog.vue";
import DataBrowser from "./components/DataBrowser.vue";
import Settings from "./components/Settings.vue";

const skillStore = useSkillStore();
const surveyStore = useSurveyStore();
const settingsStore = useSettingsStore();

const logPath = ref(settingsStore.settings.logFilePath);
const error = ref("");
const watching = ref(false);
const parsing = ref(false);
const currentView = ref<AppView>("skills");

// Watch for changes to the log path in settings
watch(
  () => settingsStore.settings.logFilePath,
  (newPath) => {
    logPath.value = newPath;
  }
);

onMounted(async () => {
  await listen("skill-update", (event: any) => {
    skillStore.handleUpdate(event.payload);
    surveyStore.handleSkillUpdate(event.payload);
  });
  await listen("survey-event", (event: any) => {
    surveyStore.handleSurveyEvent(event.payload);
  });

  // Auto-watch on startup if enabled
  if (settingsStore.settings.autoWatchOnStartup && logPath.value) {
    await startWatching();
  }
});

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

<template>
  <div class="app">
    <MenuBar :currentView="currentView" @navigate="navigateToView" />

    <div class="content">
      <div class="view-content">
        <template v-if="currentView === 'skills'">
          <SkillGrid />
        </template>
        <template v-else-if="currentView === 'surveying'">
          <SurveySessionCard />
          <SurveyLog />
        </template>
        <template v-else-if="currentView === 'data-browser'">
          <DataBrowser />
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

<style>
* {
  box-sizing: border-box;
}
body {
  margin: 0;
  background: #111;
  color: #ccc;
  font-family: monospace;
}

.app {
  padding: 1rem;
  min-height: 100vh;
  display: flex;
  flex-direction: column;
}

.content {
  flex: 1;
  display: flex;
  flex-direction: column;
}

.view-content {
  flex: 1;
}

button {
  padding: 0.4rem 0.75rem;
  background: #222;
  color: #ccc;
  border: 1px solid #444;
  cursor: pointer;
  font-family: monospace;
  border-radius: 4px;
}
button:hover:not(:disabled) {
  background: #2a2a2a;
  border-color: #666;
}
button:disabled {
  opacity: 0.4;
  cursor: default;
}
</style>
