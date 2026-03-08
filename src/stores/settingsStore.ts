import { defineStore } from "pinia";
import { ref } from "vue";

export interface AppSettings {
  logFilePath: string;
  autoWatchOnStartup: boolean;
  gameDataPath: string; // Path to Elder Game\Project Gorgon folder
}

const STORAGE_KEY = "glogger-settings";

// Get the default game data path
function getDefaultGameDataPath(): string {
  // %APPDATA%\..\LocalLow\Elder Game\Project Gorgon\
  // In Tauri, we'll construct this from the typical Windows path
  // User will be able to customize this in settings
  if (typeof window !== "undefined") {
    // Try to get from environment or use typical Windows path structure
    return "C:\\Users\\%USERNAME%\\AppData\\LocalLow\\Elder Game\\Project Gorgon";
  }
  return "";
}

function getDefaultLogPath(): string {
  const basePath = getDefaultGameDataPath();
  return basePath ? basePath + "\\Player.log" : "";
}

function loadSettings(): AppSettings {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      const parsed = JSON.parse(stored);
      // Migrate old settings without new fields
      return {
        logFilePath: parsed.logFilePath || "",
        autoWatchOnStartup: parsed.autoWatchOnStartup ?? false,
        gameDataPath: parsed.gameDataPath || getDefaultGameDataPath(),
      };
    }
  } catch (e) {
    console.error("Failed to load settings:", e);
  }
  return {
    logFilePath: getDefaultLogPath(),
    autoWatchOnStartup: false,
    gameDataPath: getDefaultGameDataPath(),
  };
}

function saveSettings(settings: AppSettings) {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(settings));
  } catch (e) {
    console.error("Failed to save settings:", e);
  }
}

export const useSettingsStore = defineStore("settings", () => {
  const settings = ref<AppSettings>(loadSettings());

  function updateLogFilePath(path: string) {
    settings.value.logFilePath = path;
    saveSettings(settings.value);
  }

  function updateAutoWatchOnStartup(enabled: boolean) {
    settings.value.autoWatchOnStartup = enabled;
    saveSettings(settings.value);
  }

  function updateGameDataPath(path: string) {
    settings.value.gameDataPath = path;
    // Update log path if it was using the old game data path
    if (settings.value.logFilePath.includes("Player.log")) {
      settings.value.logFilePath = path + "\\Player.log";
    }
    saveSettings(settings.value);
  }

  function updateSettings(newSettings: Partial<AppSettings>) {
    settings.value = { ...settings.value, ...newSettings };
    saveSettings(settings.value);
  }

  function getPlayerLogPath(): string {
    return settings.value.gameDataPath + "\\Player.log";
  }

  function getChatLogsPath(): string {
    return settings.value.gameDataPath + "\\ChatLogs";
  }

  function getReportsPath(): string {
    return settings.value.gameDataPath + "\\Reports";
  }

  return {
    settings,
    updateLogFilePath,
    updateAutoWatchOnStartup,
    updateGameDataPath,
    updateSettings,
    getPlayerLogPath,
    getChatLogsPath,
    getReportsPath,
  };
});
