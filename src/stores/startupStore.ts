import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useSettingsStore } from "./settingsStore";

export type StartupPhase =
  | "splash"
  | "setup-path"
  | "setup-watchers"
  | "setup-character"
  | "select-character"
  | "loading"
  | "ready";

export interface GameDataPathValidation {
  path_exists: boolean;
  player_log_found: boolean;
  chat_logs_found: boolean;
  reports_found: boolean;
}

export interface DiscoveredCharacter {
  character_name: string;
  server_name: string;
  report_count: number;
  latest_report_time: string | null;
}

export interface UserCharacter {
  id: number;
  character_name: string;
  server_name: string;
  source: string;
  is_active: boolean;
  latest_report_time: string | null;
  last_login_time: string | null;
}

export interface StartupTask {
  label: string;
  status: "pending" | "running" | "done" | "error";
  detail?: string;
}

export const useStartupStore = defineStore("startup", () => {
  const phase = ref<StartupPhase>("splash");
  const pathValidation = ref<GameDataPathValidation | null>(null);
  const discoveredCharacters = ref<DiscoveredCharacter[]>([]);
  const userCharacters = ref<UserCharacter[]>([]);
  const serverList = ref<string[]>([]);
  const startupTasks = ref<StartupTask[]>([]);
  const error = ref<string | null>(null);

  const isSetupWizard = computed(() => phase.value.startsWith("setup-"));

  const setupStepIndex = computed(() => {
    switch (phase.value) {
      case "setup-path": return 0;
      case "setup-watchers": return 1;
      case "setup-character": return 2;
      default: return -1;
    }
  });

  async function initialize() {
    const settingsStore = useSettingsStore();

    // Load settings first
    await settingsStore.initialize();

    // Load server list
    try {
      serverList.value = await invoke<string[]>("get_server_list");
    } catch (e) {
      console.error("Failed to load server list:", e);
      serverList.value = ["Dreva"];
    }

    const settings = settingsStore.settings;

    if (!settings.setupCompleted || !settings.gameDataPath) {
      // First-time setup
      phase.value = "setup-path";
    } else if (settings.autoLoadLastCharacter && settings.activeCharacterName) {
      // Auto-load last character
      phase.value = "loading";
      await runStartupTasks();
    } else {
      // Show character selection
      await loadUserCharacters();
      phase.value = "select-character";
    }
  }

  async function validatePath(path: string): Promise<GameDataPathValidation> {
    const result = await invoke<GameDataPathValidation>("validate_game_data_path", { path });
    pathValidation.value = result;
    return result;
  }

  async function scanForCharacters(path: string) {
    try {
      discoveredCharacters.value = await invoke<DiscoveredCharacter[]>(
        "scan_reports_for_characters",
        { path }
      );
    } catch (e) {
      console.error("Failed to scan for characters:", e);
      discoveredCharacters.value = [];
    }
  }

  async function loadUserCharacters() {
    try {
      userCharacters.value = await invoke<UserCharacter[]>("get_user_characters");
    } catch (e) {
      console.error("Failed to load user characters:", e);
      userCharacters.value = [];
    }
  }

  async function saveCharacter(characterName: string, serverName: string, source: string) {
    await invoke("save_user_character", {
      characterName,
      serverName,
      source,
    });
  }

  async function selectCharacter(characterName: string, serverName: string) {
    await invoke("set_active_character", { characterName, serverName });

    const settingsStore = useSettingsStore();
    settingsStore.settings.activeCharacterName = characterName;
    settingsStore.settings.activeServerName = serverName;

    phase.value = "loading";
    await runStartupTasks();
  }

  async function completeSetup() {
    await invoke("complete_setup");

    const settingsStore = useSettingsStore();
    settingsStore.settings.setupCompleted = true;

    phase.value = "loading";
    await runStartupTasks();
  }

  async function runStartupTasks() {
    startupTasks.value = [
      { label: "Initializing application", status: "running" },
      { label: "Checking game data", status: "pending" },
    ];

    // Brief pause so the user sees the loading screen
    await new Promise((r) => setTimeout(r, 300));

    startupTasks.value[0].status = "done";
    startupTasks.value[1].status = "running";

    // CDN loads in the background (non-blocking), just mark as done
    startupTasks.value[1].status = "done";

    phase.value = "ready";
  }

  function goToPhase(newPhase: StartupPhase) {
    phase.value = newPhase;
  }

  return {
    phase,
    pathValidation,
    discoveredCharacters,
    userCharacters,
    serverList,
    startupTasks,
    error,
    isSetupWizard,
    setupStepIndex,
    initialize,
    validatePath,
    scanForCharacters,
    loadUserCharacters,
    saveCharacter,
    selectCharacter,
    completeSetup,
    runStartupTasks,
    goToPhase,
  };
});
