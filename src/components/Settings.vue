<script setup lang="ts">
import { ref, watch, defineProps } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { useSettingsStore } from "../stores/settingsStore";
import { useGameDataStore } from "../stores/gameDataStore";

const props = defineProps<{
  watching: boolean;
  parsing: boolean;
  error: string;
  onStartWatching: () => void;
  onParseLog: () => void;
}>();

const settingsStore = useSettingsStore();
const gameDataStore = useGameDataStore();
const localLogPath = ref(settingsStore.settings.logFilePath);
const localGameDataPath = ref(settingsStore.settings.gameDataPath);
const autoWatchOnStartup = ref(settingsStore.settings.autoWatchOnStartup);

watch(
  () => settingsStore.settings.logFilePath,
  (newPath) => {
    localLogPath.value = newPath;
  }
);

watch(
  () => settingsStore.settings.gameDataPath,
  (newPath) => {
    localGameDataPath.value = newPath;
  }
);

watch(
  () => settingsStore.settings.autoWatchOnStartup,
  (newValue) => {
    autoWatchOnStartup.value = newValue;
  }
);

async function browseLogFile() {
  const selected = await open({
    multiple: false,
    filters: [{ name: "Log Files", extensions: ["log", "txt"] }],
  });
  if (selected) {
    localLogPath.value = selected;
    settingsStore.updateLogFilePath(selected);
  }
}

async function browseGameDataFolder() {
  const selected = await open({
    directory: true,
    multiple: false,
  });
  if (selected) {
    localGameDataPath.value = selected;
    settingsStore.updateGameDataPath(selected);
  }
}

function handleLogPathInput() {
  settingsStore.updateLogFilePath(localLogPath.value);
}

function handleGameDataPathInput() {
  settingsStore.updateGameDataPath(localGameDataPath.value);
}

function handleAutoWatchToggle() {
  settingsStore.updateAutoWatchOnStartup(autoWatchOnStartup.value);
}

function useDefaultPlayerLog() {
  const playerLogPath = settingsStore.getPlayerLogPath();
  localLogPath.value = playerLogPath;
  settingsStore.updateLogFilePath(playerLogPath);
}

const refreshing = ref(false);
const refreshError = ref<string | null>(null);

async function forceRefreshCDN() {
  refreshing.value = true;
  refreshError.value = null;
  try {
    await gameDataStore.forceRefreshCdn();
  } catch (e: any) {
    refreshError.value = e.toString();
  } finally {
    refreshing.value = false;
  }
}
</script>

<template>
  <div class="settings">
    <h2>Settings</h2>

    <div class="settings-section">
      <h3>Log File Management</h3>

      <div class="setting-item">
        <label for="log-path">Current Log File</label>
        <div class="path-input-group">
          <input
            id="log-path"
            v-model="localLogPath"
            @blur="handleLogPathInput"
            @keyup.enter="handleLogPathInput"
            placeholder="Path to log file..."
            class="path-input" />
          <button @click="browseLogFile" class="browse-button">Browse</button>
        </div>
        <p class="setting-description">
          The log file currently being monitored or to be parsed.
        </p>
      </div>

      <div class="log-controls">
        <button
          @click="props.onStartWatching"
          :disabled="props.watching || props.parsing || !localLogPath"
          class="control-button">
          {{ props.watching ? "Watching…" : "Start Watching" }}
        </button>
        <button
          @click="props.onParseLog"
          :disabled="props.watching || props.parsing || !localLogPath"
          class="control-button">
          {{ props.parsing ? "Parsing…" : "Parse Log" }}
        </button>
      </div>

      <div v-if="props.error" class="error-box">{{ props.error }}</div>
    </div>

    <div class="settings-section">
      <h3>Game Data Directory</h3>

      <div class="setting-item">
        <label for="game-data-path">Project Gorgon Data Folder</label>
        <div class="path-input-group">
          <input
            id="game-data-path"
            v-model="localGameDataPath"
            @blur="handleGameDataPathInput"
            @keyup.enter="handleGameDataPathInput"
            placeholder="Path to Elder Game\Project Gorgon folder..."
            class="path-input" />
          <button @click="browseGameDataFolder" class="browse-button">
            Browse
          </button>
        </div>
        <p class="setting-description">
          Default location: %APPDATA%\..\LocalLow\Elder Game\Project Gorgon\
          <br />
          This folder contains Player.log, ChatLogs, and Reports subfolders.
        </p>
      </div>

      <div class="setting-item">
        <button @click="useDefaultPlayerLog" class="secondary-button">
          Use Default Player.log Location
        </button>
      </div>
    </div>

    <div class="settings-section">
      <h3>Startup Behavior</h3>

      <div class="setting-item">
        <label class="checkbox-label">
          <input
            type="checkbox"
            v-model="autoWatchOnStartup"
            @change="handleAutoWatchToggle"
            class="checkbox" />
          <span>Start watching log file on startup</span>
        </label>
        <p class="setting-description">
          When enabled, the app will automatically start watching the
          Player.log file when it launches.
        </p>
      </div>
    </div>

    <div class="settings-section">
      <h3>Game Data (CDN)</h3>

      <div class="setting-item">
        <div v-if="gameDataStore.cacheStatus" class="cdn-status">
          <div class="status-row">
            <span class="status-label">Cached Version:</span>
            <span class="status-value">{{ gameDataStore.cacheStatus.cached_version || 'None' }}</span>
          </div>
          <div class="status-row">
            <span class="status-label">Remote Version:</span>
            <span class="status-value">{{ gameDataStore.cacheStatus.remote_version || 'Unknown' }}</span>
          </div>
          <div class="status-row">
            <span class="status-label">Status:</span>
            <span class="status-value" :class="{ 'status-ok': gameDataStore.cacheStatus.up_to_date }">
              {{ gameDataStore.cacheStatus.up_to_date ? 'Up to date' : 'Update available' }}
            </span>
          </div>
          <div class="status-row">
            <span class="status-label">Items:</span>
            <span class="status-value">{{ gameDataStore.cacheStatus.item_count.toLocaleString() }}</span>
          </div>
          <div class="status-row">
            <span class="status-label">Skills:</span>
            <span class="status-value">{{ gameDataStore.cacheStatus.skill_count.toLocaleString() }}</span>
          </div>
        </div>

        <button
          @click="forceRefreshCDN"
          :disabled="refreshing"
          class="control-button cdn-refresh-btn">
          {{ refreshing ? 'Refreshing…' : 'Force Refresh CDN Data' }}
        </button>

        <p class="setting-description">
          Downloads the latest game data from the Project: Gorgon CDN.
          This will re-download all items, skills, recipes, and other game data.
        </p>

        <div v-if="refreshError" class="error-box">{{ refreshError }}</div>
      </div>
    </div>

    <div class="settings-info">
      <p>Settings are automatically saved to local storage.</p>
    </div>
  </div>
</template>

<style scoped>
.settings {
  max-width: 800px;
  padding: 1rem;
}

h2 {
  color: #e0c060;
  margin-top: 0;
  margin-bottom: 1.5rem;
  font-size: 1.5rem;
}

.settings-section {
  background: #1a1a1a;
  border: 1px solid #333;
  border-radius: 4px;
  padding: 1.5rem;
  margin-bottom: 1rem;
}

.settings-section h3 {
  color: #ccc;
  margin-top: 0;
  margin-bottom: 1rem;
  font-size: 1.1rem;
}

.setting-item {
  margin-bottom: 1rem;
}

.setting-item:last-child {
  margin-bottom: 0;
}

.setting-item label {
  display: block;
  color: #aaa;
  margin-bottom: 0.5rem;
  font-size: 0.95rem;
}

.path-input-group {
  display: flex;
  gap: 0.5rem;
}

.path-input {
  flex: 1;
  padding: 0.5rem;
  background: #222;
  color: #ccc;
  border: 1px solid #444;
  border-radius: 4px;
  font-family: monospace;
  font-size: 0.9rem;
}

.path-input:focus {
  outline: none;
  border-color: #e0c060;
}

.browse-button {
  padding: 0.5rem 1rem;
  background: #222;
  color: #ccc;
  border: 1px solid #444;
  border-radius: 4px;
  cursor: pointer;
  font-family: monospace;
  white-space: nowrap;
}

.browse-button:hover {
  background: #2a2a2a;
  border-color: #666;
}

.control-button {
  padding: 0.5rem 1rem;
  background: #2a2a2a;
  color: #ccc;
  border: 1px solid #444;
  border-radius: 4px;
  cursor: pointer;
  font-family: monospace;
  margin-right: 0.5rem;
}

.control-button:hover:not(:disabled) {
  background: #333;
  border-color: #666;
}

.control-button:disabled {
  opacity: 0.4;
  cursor: default;
}

.secondary-button {
  padding: 0.5rem 1rem;
  background: #222;
  color: #aaa;
  border: 1px solid #444;
  border-radius: 4px;
  cursor: pointer;
  font-family: monospace;
  font-size: 0.9rem;
}

.secondary-button:hover {
  background: #2a2a2a;
  border-color: #666;
  color: #ccc;
}

.log-controls {
  display: flex;
  gap: 0.5rem;
  margin-top: 1rem;
}

.setting-description {
  margin: 0.5rem 0 0 0;
  color: #666;
  font-size: 0.85rem;
  line-height: 1.4;
}

.checkbox-label {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  cursor: pointer;
  color: #ccc;
}

.checkbox {
  width: 1.2rem;
  height: 1.2rem;
  cursor: pointer;
}

.error-box {
  margin-top: 1rem;
  padding: 0.75rem;
  background: #2a1a1a;
  border: 1px solid #662222;
  border-radius: 4px;
  color: #f66;
  font-size: 0.9rem;
}

.settings-info {
  background: #1a1a1a;
  border: 1px solid #333;
  border-radius: 4px;
  padding: 1rem;
  color: #888;
  font-size: 0.9rem;
}

.settings-info p {
  margin: 0;
}

.cdn-status {
  background: #0d0d0d;
  border: 1px solid #333;
  border-radius: 4px;
  padding: 0.75rem;
  margin-bottom: 1rem;
  font-family: monospace;
  font-size: 0.85rem;
}

.status-row {
  display: flex;
  justify-content: space-between;
  padding: 0.25rem 0;
}

.status-label {
  color: #888;
}

.status-value {
  color: #ccc;
  font-weight: bold;
}

.status-value.status-ok {
  color: #8c8;
}

.cdn-refresh-btn {
  margin-top: 0.5rem;
}
</style>
