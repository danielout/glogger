<template>
  <div>
    <div class="settings-section">
      <h3>Developer Mode</h3>

      <div class="mb-4">
        <label class="flex items-center gap-2 cursor-pointer text-text-primary">
          <input
            type="checkbox"
            v-model="devMode"
            @change="saveDevMode"
            class="size-5 cursor-pointer" />
          <span>Enable Developer Mode</span>
        </label>
        <p class="mt-2 text-text-muted text-xs leading-relaxed">
          Reveals beta features, experimental tools, and additional debugging information
          throughout the app. Off by default.
        </p>
      </div>
    </div>

    <div class="settings-section">
      <h3>Character Report Watching</h3>

      <div class="mb-4">
        <label class="flex items-center gap-2 cursor-pointer text-text-primary">
          <input
            type="checkbox"
            v-model="autoWatchReports"
            @change="saveReportWatchSettings"
            class="size-5 cursor-pointer" />
          <span>Auto-ingest new character reports</span>
        </label>
        <p class="mt-2 text-text-muted text-xs leading-relaxed">
          Watches the Reports folder for new character exports and automatically imports them.
        </p>
      </div>

      <div v-if="autoWatchReports" class="mb-4">
        <label class="text-sm text-text-secondary mb-1 block">Check interval (seconds)</label>
        <input
          type="number"
          v-model.number="reportWatchInterval"
          @change="saveReportWatchSettings"
          min="5"
          max="300"
          class="bg-surface-elevated border border-border-default rounded px-3 py-1.5 text-sm text-text-primary w-24" />
        <p class="mt-1 text-text-muted text-xs">
          How often to check for new reports (5–300 seconds).
        </p>
      </div>
    </div>

    <div class="settings-section">
      <h3>Player.log Upload</h3>

      <div class="mb-4">
        <button
          @click="browseAndParsePlayerLog"
          :disabled="props.parsing"
          class="btn btn-secondary">
          {{ props.parsing ? "Parsing..." : "Upload Player.log File" }}
        </button>

        <p class="mt-2 text-text-muted text-xs leading-relaxed">
          Upload a Player.log file for manual parsing. Use this if you have test data or
          historical log files you want to process (e.g., from another machine or a backup).
        </p>

        <div v-if="parseResult" class="success-box">{{ parseResult }}</div>
        <div v-if="parseError" class="error-box">{{ parseError }}</div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { useSettingsStore } from "../../stores/settingsStore";
import { useCharacterStore } from "../../stores/characterStore";

const props = defineProps<{
  parsing: boolean;
  error: string;
  onParseLog: () => void;
}>();

const settingsStore = useSettingsStore();
const characterStore = useCharacterStore();

// Dev mode
const devMode = ref(settingsStore.settings.devModeEnabled);

function saveDevMode() {
  settingsStore.updateSettings({ devModeEnabled: devMode.value });
}

// Report watching
const autoWatchReports = ref(settingsStore.settings.autoWatchReports);
const reportWatchInterval = ref(settingsStore.settings.reportWatchIntervalSeconds);

function saveReportWatchSettings() {
  const interval = Math.max(5, Math.min(300, reportWatchInterval.value));
  reportWatchInterval.value = interval;
  settingsStore.updateSettings({
    autoWatchReports: autoWatchReports.value,
    reportWatchIntervalSeconds: interval,
  });
  // Restart watching with new settings
  characterStore.stopReportWatching();
  if (autoWatchReports.value) {
    characterStore.startReportWatching();
  }
}

// Player.log upload
const parseResult = ref<string | null>(null);
const parseError = ref<string | null>(null);

async function browseAndParsePlayerLog() {
  const selected = await open({
    multiple: false,
    filters: [{ name: "Log Files", extensions: ["log", "txt"] }],
  });
  if (selected) {
    parseResult.value = null;
    parseError.value = null;

    // Update the log file path and trigger parsing
    settingsStore.updateLogFilePath(selected);
    try {
      props.onParseLog();
      parseResult.value = `Parsing Player.log: ${selected}`;
    } catch (e: any) {
      parseError.value = e.toString();
    }
  }
}
</script>
