<template>
  <div>
    <div class="settings-section">
      <h3>Game Data Directory</h3>

      <div class="mb-4">
        <label for="game-data-path" class="block text-text-secondary mb-2 text-sm">Project Gorgon Data Folder</label>
        <div class="flex gap-2">
          <input
            id="game-data-path"
            v-model="localGameDataPath"
            @blur="handleGameDataPathInput"
            @keyup.enter="handleGameDataPathInput"
            placeholder="Path to Elder Game\Project Gorgon folder..."
            class="input flex-1" />
          <button @click="browseGameDataFolder" class="btn btn-secondary whitespace-nowrap">
            Browse
          </button>
        </div>
        <p class="mt-2 text-text-muted text-xs leading-relaxed">
          Default location: %APPDATA%\..\LocalLow\Elder Game\Project Gorgon\
          <br />
          This folder contains Player.log, ChatLogs, and Reports subfolders.
          Auto-detection works on Windows.
        </p>
      </div>

      <div>
        <button @click="useDefaultPlayerLog" class="btn btn-secondary">
          Use Default Player.log Location
        </button>
      </div>
    </div>

    <div class="settings-section">
      <h3>Startup Behavior</h3>

      <div class="mb-4">
        <label class="flex items-center gap-2 cursor-pointer text-text-primary">
          <input
            type="checkbox"
            v-model="autoTailChat"
            @change="handleAutoTailChatToggle"
            class="size-5 cursor-pointer" />
          <span>Automatically start chat log watching on startup</span>
        </label>
        <p class="mt-2 text-text-muted text-xs leading-relaxed">
          When enabled, the app will automatically begin tailing chat log files when it launches.
        </p>
      </div>

      <div>
        <label class="flex items-center gap-2 cursor-pointer text-text-primary">
          <input
            type="checkbox"
            v-model="autoTailPlayerLog"
            @change="handleAutoTailPlayerLogToggle"
            class="size-5 cursor-pointer" />
          <span>Automatically start Player.log watching on startup</span>
        </label>
        <p class="mt-2 text-text-muted text-xs leading-relaxed">
          When enabled, the app will automatically begin tailing Player.log for skill updates,
          survey events, and other game data when it launches.
        </p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { useSettingsStore } from "../../stores/settingsStore";

const settingsStore = useSettingsStore();
const localGameDataPath = ref(settingsStore.settings.gameDataPath);
const autoTailChat = ref(settingsStore.settings.autoTailChat);
const autoTailPlayerLog = ref(settingsStore.settings.autoTailPlayerLog);

watch(
  () => settingsStore.settings.gameDataPath,
  (newPath) => {
    localGameDataPath.value = newPath;
  }
);

watch(
  () => settingsStore.settings.autoTailChat,
  (val) => { autoTailChat.value = val; }
);

watch(
  () => settingsStore.settings.autoTailPlayerLog,
  (val) => { autoTailPlayerLog.value = val; }
);

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

function handleGameDataPathInput() {
  settingsStore.updateGameDataPath(localGameDataPath.value);
}

function useDefaultPlayerLog() {
  const playerLogPath = settingsStore.getPlayerLogPath();
  settingsStore.updateLogFilePath(playerLogPath);
}

function handleAutoTailChatToggle() {
  settingsStore.updateSettings({ autoTailChat: autoTailChat.value });
}

function handleAutoTailPlayerLogToggle() {
  settingsStore.updateSettings({ autoTailPlayerLog: autoTailPlayerLog.value });
}
</script>
