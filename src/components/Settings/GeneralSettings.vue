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
            placeholder="Path to Project Gorgon data folder..."
            class="input flex-1" />
          <button @click="browseGameDataFolder" class="btn btn-secondary whitespace-nowrap">
            Browse
          </button>
        </div>
        <p class="mt-2 text-text-muted text-xs leading-relaxed">
          Contains ChatLogs, Reports, and other game data.<br />
          Windows: <code>%APPDATA%\..\LocalLow\Elder Game\Project Gorgon\</code><br />
          macOS: <code>~/Library/Application Support/unity.Elder Game.Project Gorgon/</code>
        </p>
      </div>

      <div class="mb-4">
        <label for="player-log-path" class="block text-text-secondary mb-2 text-sm">Player.log File</label>
        <div class="flex gap-2">
          <input
            id="player-log-path"
            v-model="localPlayerLogPath"
            @blur="handlePlayerLogPathInput"
            @keyup.enter="handlePlayerLogPathInput"
            placeholder="Path to Player.log..."
            class="input flex-1" />
          <button @click="browsePlayerLogFile" class="btn btn-secondary whitespace-nowrap">
            Browse
          </button>
        </div>
        <p class="mt-2 text-text-muted text-xs leading-relaxed">
          On macOS this is separate from the data folder:<br />
          <code>~/Library/Logs/Elder Game/Project Gorgon/Player.log</code>
        </p>
      </div>

      <div class="flex gap-2 mt-2">
        <button @click="useDefaultGameDataPath" class="btn btn-secondary text-xs">
          Use Default Game Data Path
        </button>
        <button @click="useDefaultPlayerLog" class="btn btn-secondary text-xs">
          Use Default Player.log Location
        </button>
        <button @click="useDefaultsBoth" class="btn btn-secondary text-xs">
          Reset Both to Defaults
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

    <div class="settings-section">
      <h3>Timestamp Display</h3>

      <div class="mb-4">
        <label class="block text-text-secondary mb-2 text-sm">Display timestamps in</label>
        <div class="flex gap-2">
          <button
            v-for="option in timestampOptions"
            :key="option.value"
            @click="handleTimestampModeChange(option.value)"
            :class="[
              'px-3 py-1.5 rounded text-sm border transition-colors',
              timestampMode === option.value
                ? 'bg-accent/20 border-accent text-accent'
                : 'border-border text-text-secondary hover:border-text-muted'
            ]">
            {{ option.label }}
          </button>
        </div>
        <p class="mt-2 text-text-muted text-xs leading-relaxed">
          <span v-if="timestampMode === 'local'">
            Times are shown in your computer's local timezone.
          </span>
          <span v-else-if="timestampMode === 'server'">
            Times are shown in the game server's timezone
            <template v-if="serverOffsetLabel"> ({{ serverOffsetLabel }})</template>.
            <template v-if="!hasServerOffset">
              <br />Server timezone not yet detected — start chat log tailing to auto-detect from a login line.
            </template>
          </span>
          <span v-else>
            Times are shown in UTC (Coordinated Universal Time).
          </span>
        </p>
      </div>

      <div>
        <label class="block text-text-secondary mb-2 text-sm">Hour format</label>
        <div class="flex gap-2">
          <button
            v-for="option in hourFormatOptions"
            :key="String(option.value)"
            @click="handleHourFormatChange(option.value)"
            :class="[
              'px-3 py-1.5 rounded text-sm border transition-colors',
              use24Hour === option.value
                ? 'bg-accent/20 border-accent text-accent'
                : 'border-border text-text-secondary hover:border-text-muted'
            ]">
            {{ option.label }}
          </button>
        </div>
        <p class="mt-2 text-text-muted text-xs leading-relaxed">
          <span v-if="use24Hour">Times are shown in 24-hour format (e.g. 14:30).</span>
          <span v-else>Times are shown in 12-hour format with AM/PM (e.g. 2:30 PM).</span>
        </p>
      </div>
    </div>

    <div class="settings-section">
      <h3>Crafting</h3>

      <div>
        <label class="flex items-center gap-2 cursor-pointer text-text-primary">
          <input
            type="checkbox"
            v-model="excludeMaxEnchanted"
            @change="handleExcludeMaxEnchantedToggle"
            class="size-5 cursor-pointer" />
          <span>Exclude "Max-Enchanted" recipes from automated selection</span>
        </label>
        <p class="mt-2 text-text-muted text-xs leading-relaxed">
          When enabled, Max-Enchanted recipe variants are excluded from the leveling optimizer,
          work order matching, and intermediate craft resolution. These recipes use extremely rare
          ingredients and are unlikely choices for leveling or work orders. You can still add them
          to projects manually.
        </p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { useSettingsStore } from "../../stores/settingsStore";

const settingsStore = useSettingsStore();
const localGameDataPath = ref(settingsStore.settings.gameDataPath);
const localPlayerLogPath = ref(settingsStore.settings.logFilePath);
const autoTailChat = ref(settingsStore.settings.autoTailChat);
const autoTailPlayerLog = ref(settingsStore.settings.autoTailPlayerLog);
const excludeMaxEnchanted = ref(settingsStore.settings.excludeMaxEnchantedRecipes);
const timestampMode = ref(settingsStore.settings.timestampDisplayMode);
const use24Hour = ref(settingsStore.settings.use24HourTime);

const timestampOptions = [
  { value: 'local' as const, label: 'Local Time' },
  { value: 'server' as const, label: 'Server Time' },
  { value: 'utc' as const, label: 'UTC' },
];

const hourFormatOptions = [
  { value: true, label: '24-hour' },
  { value: false, label: '12-hour' },
];

const hasServerOffset = computed(() => {
  return settingsStore.settings.manualTimezoneOverride != null
    || settingsStore.settings.timezoneOffsetSeconds != null;
});

const serverOffsetLabel = computed(() => {
  const offset = settingsStore.settings.manualTimezoneOverride
    ?? settingsStore.settings.timezoneOffsetSeconds;
  if (offset == null) return '';
  const sign = offset >= 0 ? '+' : '-';
  const abs = Math.abs(offset);
  const h = Math.floor(abs / 3600);
  const m = Math.floor((abs % 3600) / 60);
  return `UTC${sign}${h}${m > 0 ? ':' + String(m).padStart(2, '0') : ''}`;
});

watch(
  () => settingsStore.settings.gameDataPath,
  (newPath) => {
    localGameDataPath.value = newPath;
  }
);

watch(
  () => settingsStore.settings.logFilePath,
  (newPath) => {
    localPlayerLogPath.value = newPath;
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

watch(
  () => settingsStore.settings.excludeMaxEnchantedRecipes,
  (val) => { excludeMaxEnchanted.value = val; }
);

watch(
  () => settingsStore.settings.timestampDisplayMode,
  (val) => { timestampMode.value = val; }
);

watch(
  () => settingsStore.settings.use24HourTime,
  (val) => { use24Hour.value = val; }
);

async function browseGameDataFolder() {
  const selected = await open({
    directory: true,
    multiple: false,
    defaultPath: settingsStore.settings.gameDataPath || undefined,
  });
  if (selected) {
    localGameDataPath.value = selected;
    settingsStore.updateGameDataPath(selected);
  }
}

function handleGameDataPathInput() {
  settingsStore.updateGameDataPath(localGameDataPath.value);
}

async function useDefaultGameDataPath() {
  try {
    const defaultPath = await invoke<string>("get_default_game_data_path_command");
    if (defaultPath) {
      localGameDataPath.value = defaultPath;
      settingsStore.updateGameDataPath(defaultPath);
    }
  } catch (e) {
    console.error("Failed to get default game data path:", e);
  }
}

async function useDefaultPlayerLog() {
  try {
    const defaultPath = await invoke<string>("get_default_player_log_path_command");
    if (defaultPath) {
      localPlayerLogPath.value = defaultPath;
      settingsStore.updateLogFilePath(defaultPath);
    }
  } catch (e) {
    console.error("Failed to get default Player.log path:", e);
  }
}

async function useDefaultsBoth() {
  try {
    const [dataPath, logPath] = await Promise.all([
      invoke<string>("get_default_game_data_path_command"),
      invoke<string>("get_default_player_log_path_command"),
    ]);
    if (dataPath) {
      localGameDataPath.value = dataPath;
      settingsStore.updateGameDataPath(dataPath);
    }
    if (logPath) {
      localPlayerLogPath.value = logPath;
      settingsStore.updateLogFilePath(logPath);
    }
  } catch (e) {
    console.error("Failed to reset both paths to defaults:", e);
  }
}

function handlePlayerLogPathInput() {
  settingsStore.updateLogFilePath(localPlayerLogPath.value);
}

async function browsePlayerLogFile() {
  const selected = await open({
    multiple: false,
    defaultPath: settingsStore.settings.logFilePath || settingsStore.settings.gameDataPath || undefined,
    filters: [{ name: "Player.log", extensions: ["log"] }],
  });
  if (selected) {
    localPlayerLogPath.value = selected;
    settingsStore.updateLogFilePath(selected);
  }
}

function handleAutoTailChatToggle() {
  settingsStore.updateSettings({ autoTailChat: autoTailChat.value });
}

function handleAutoTailPlayerLogToggle() {
  settingsStore.updateSettings({ autoTailPlayerLog: autoTailPlayerLog.value });
}

function handleExcludeMaxEnchantedToggle() {
  settingsStore.updateSettings({ excludeMaxEnchantedRecipes: excludeMaxEnchanted.value });
}

function handleTimestampModeChange(mode: 'local' | 'server' | 'utc') {
  timestampMode.value = mode;
  settingsStore.updateSettings({ timestampDisplayMode: mode });
}

function handleHourFormatChange(value: boolean) {
  use24Hour.value = value;
  settingsStore.updateSettings({ use24HourTime: value });
}
</script>
