<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import { useSkillStore } from "./stores/skillStore";
import { useSurveyStore } from "./stores/surveyStore";
import SkillGrid from "./components/SkillGrid.vue";
import SurveySessionCard from "./components/SurveySessionCard.vue";
import SurveyLog from "./components/SurveyLog.vue";
import ItemSearch from "./components/ItemSearch.vue";

const skillStore = useSkillStore();
const surveyStore = useSurveyStore();

const logPath = ref("");
const error = ref("");
const watching = ref(false);
const parsing = ref(false);
const activeTab = ref<"skills" | "surveying" | "items">("skills");

onMounted(async () => {
  await listen("skill-update", (event: any) => {
    skillStore.handleUpdate(event.payload);
    surveyStore.handleSkillUpdate(event.payload);
  });
  await listen("survey-event", (event: any) => {
    surveyStore.handleSurveyEvent(event.payload);
  });
});

async function pickFile() {
  const selected = await open({
    multiple: false,
    filters: [{ name: "Log Files", extensions: ["log", "txt"] }],
  });
  if (selected) logPath.value = selected;
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
</script>

<template>
  <div class="app">
    <div class="toolbar">
      <span class="app-title">Glogger</span>
      <input
        v-model="logPath"
        placeholder="Pick a log file..."
        class="path-input"
        readonly />
      <button @click="pickFile" :disabled="watching || parsing">Browse</button>
      <button
        @click="startWatching"
        :disabled="watching || parsing || !logPath">
        {{ watching ? "Watching…" : "Start Watching" }}
      </button>
      <button @click="parseLog" :disabled="watching || parsing || !logPath">
        {{ parsing ? "Parsing…" : "Parse Log" }}
      </button>
    </div>

    <div v-if="error" class="error">{{ error }}</div>

    <div class="tabs">
      <button
        class="tab"
        :class="{ active: activeTab === 'skills' }"
        @click="activeTab = 'skills'">
        Skills
      </button>
      <button
        class="tab"
        :class="{ active: activeTab === 'surveying' }"
        @click="activeTab = 'surveying'">
        Surveying
      </button>
      <button
        class="tab"
        :class="{ active: activeTab === 'items' }"
        @click="activeTab = 'items'">
        Items
      </button>
    </div>

    <div class="tab-content">
      <template v-if="activeTab === 'skills'">
        <SkillGrid />
      </template>
      <template v-if="activeTab === 'surveying'">
        <SurveySessionCard />
        <SurveyLog />
      </template>
      <template v-if="activeTab === 'items'">
        <ItemSearch />
      </template>
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
}

.toolbar {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-bottom: 0.75rem;
  flex-wrap: wrap;
}
.app-title {
  color: #e0c060;
  font-weight: bold;
  font-size: 1.1rem;
  margin-right: 0.5rem;
}
.path-input {
  flex: 1;
  min-width: 200px;
  padding: 0.4rem;
  background: #222;
  color: #ccc;
  border: 1px solid #444;
}

.error {
  color: #f66;
  margin-bottom: 0.75rem;
  font-size: 0.85rem;
}

.tabs {
  display: flex;
  gap: 0;
  margin-bottom: 1rem;
  border-bottom: 1px solid #333;
}
.tab {
  padding: 0.4rem 1rem;
  background: none;
  border: none;
  border-bottom: 2px solid transparent;
  color: #666;
  cursor: pointer;
  font-family: monospace;
  font-size: 0.9rem;
}
.tab:hover {
  color: #aaa;
}
.tab.active {
  color: #e0c060;
  border-bottom-color: #e0c060;
}

button {
  padding: 0.4rem 0.75rem;
  background: #222;
  color: #ccc;
  border: 1px solid #444;
  cursor: pointer;
  font-family: monospace;
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
