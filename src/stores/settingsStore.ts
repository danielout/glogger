import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { WatchRule } from "../types/database";

export interface AppSettings {
  logFilePath: string;
  autoWatchOnStartup: boolean;
  gameDataPath: string;
  autoPurgeEnabled: boolean;
  autoPurgeDays: number;
  autoTailChat: boolean;
  autoTailPlayerLog: boolean;
  dbPath: string | null;
  excludedChatChannels: string[];
  chatRetentionDays: number | null;
  tellsRetentionDays: number | null;
  guildRetentionDays: number | null;
  devModeEnabled: boolean;
  autoCheckGameData: boolean;
  autoUpdateGameData: boolean;
  userDataAutoPurgeDays: number | null;
  watchRules: WatchRule[];
  setupCompleted: boolean;
  activeCharacterName: string | null;
  activeServerName: string | null;
  autoLoadLastCharacter: boolean;
  autoWatchReports: boolean;
  reportWatchIntervalSeconds: number;
  excludeMaxEnchantedRecipes: boolean;
  marketPriceMode: string;
  itemValuationMode: string;
  showRawJsonInDataBrowser: boolean;
  viewPreferences: Record<string, Record<string, unknown>>;
}

// Backend settings format (snake_case)
interface BackendSettings {
  log_file_path: string;
  auto_watch_on_startup: boolean;
  game_data_path: string;
  auto_purge_enabled: boolean;
  auto_purge_days: number;
  auto_tail_chat: boolean;
  auto_tail_player_log: boolean;
  db_path: string | null;
  excluded_chat_channels: string[];
  chat_retention_days: number | null;
  tells_retention_days: number | null;
  guild_retention_days: number | null;
  dev_mode_enabled: boolean;
  auto_check_game_data: boolean;
  auto_update_game_data: boolean;
  user_data_auto_purge_days: number | null;
  watch_rules: WatchRule[];
  setup_completed: boolean;
  active_character_name: string | null;
  active_server_name: string | null;
  auto_load_last_character: boolean;
  auto_watch_reports: boolean;
  report_watch_interval_seconds: number;
  exclude_max_enchanted_recipes: boolean;
  market_price_mode: string;
  item_valuation_mode: string;
  show_raw_json_in_data_browser: boolean;
  view_preferences: Record<string, Record<string, unknown>>;
}

// Convert frontend format to backend format
function toBackendSettings(settings: AppSettings): BackendSettings {
  return {
    log_file_path: settings.logFilePath,
    auto_watch_on_startup: settings.autoWatchOnStartup,
    game_data_path: settings.gameDataPath,
    auto_purge_enabled: settings.autoPurgeEnabled,
    auto_purge_days: settings.autoPurgeDays,
    auto_tail_chat: settings.autoTailChat,
    auto_tail_player_log: settings.autoTailPlayerLog,
    db_path: settings.dbPath,
    excluded_chat_channels: settings.excludedChatChannels,
    chat_retention_days: settings.chatRetentionDays,
    tells_retention_days: settings.tellsRetentionDays,
    guild_retention_days: settings.guildRetentionDays,
    dev_mode_enabled: settings.devModeEnabled,
    auto_check_game_data: settings.autoCheckGameData,
    auto_update_game_data: settings.autoUpdateGameData,
    user_data_auto_purge_days: settings.userDataAutoPurgeDays,
    watch_rules: settings.watchRules,
    setup_completed: settings.setupCompleted,
    active_character_name: settings.activeCharacterName,
    active_server_name: settings.activeServerName,
    auto_load_last_character: settings.autoLoadLastCharacter,
    auto_watch_reports: settings.autoWatchReports,
    report_watch_interval_seconds: settings.reportWatchIntervalSeconds,
    exclude_max_enchanted_recipes: settings.excludeMaxEnchantedRecipes,
    market_price_mode: settings.marketPriceMode,
    item_valuation_mode: settings.itemValuationMode,
    show_raw_json_in_data_browser: settings.showRawJsonInDataBrowser,
    view_preferences: settings.viewPreferences,
  };
}

const DEFAULT_EXCLUDED_CHANNELS = [
  "System", "Error", "Emotes", "Action Emotes", "NPC Chatter", "Status", "Combat"
];

// Convert backend format to frontend format
function fromBackendSettings(settings: BackendSettings): AppSettings {
  return {
    logFilePath: settings.log_file_path,
    autoWatchOnStartup: settings.auto_watch_on_startup,
    gameDataPath: settings.game_data_path,
    autoPurgeEnabled: settings.auto_purge_enabled,
    autoPurgeDays: settings.auto_purge_days,
    autoTailChat: settings.auto_tail_chat ?? false,
    autoTailPlayerLog: settings.auto_tail_player_log ?? false,
    dbPath: settings.db_path ?? null,
    excludedChatChannels: settings.excluded_chat_channels ?? DEFAULT_EXCLUDED_CHANNELS,
    chatRetentionDays: settings.chat_retention_days ?? null,
    tellsRetentionDays: settings.tells_retention_days ?? null,
    guildRetentionDays: settings.guild_retention_days ?? null,
    devModeEnabled: settings.dev_mode_enabled ?? false,
    autoCheckGameData: settings.auto_check_game_data ?? true,
    autoUpdateGameData: settings.auto_update_game_data ?? true,
    userDataAutoPurgeDays: settings.user_data_auto_purge_days ?? null,
    watchRules: settings.watch_rules ?? [],
    setupCompleted: settings.setup_completed ?? false,
    activeCharacterName: settings.active_character_name ?? null,
    activeServerName: settings.active_server_name ?? null,
    autoLoadLastCharacter: settings.auto_load_last_character ?? true,
    autoWatchReports: settings.auto_watch_reports ?? true,
    reportWatchIntervalSeconds: settings.report_watch_interval_seconds ?? 10,
    excludeMaxEnchantedRecipes: settings.exclude_max_enchanted_recipes ?? true,
    marketPriceMode: settings.market_price_mode ?? 'universal',
    itemValuationMode: settings.item_valuation_mode ?? 'highest_market_vendor',
    showRawJsonInDataBrowser: settings.show_raw_json_in_data_browser ?? false,
    viewPreferences: settings.view_preferences ?? {},
  };
}

// Default settings
function getDefaultSettings(): AppSettings {
  return {
    logFilePath: "",
    autoWatchOnStartup: false,
    gameDataPath: "",
    autoPurgeEnabled: false,
    autoPurgeDays: 90,
    autoTailChat: false,
    autoTailPlayerLog: false,
    dbPath: null,
    excludedChatChannels: DEFAULT_EXCLUDED_CHANNELS,
    chatRetentionDays: null,
    tellsRetentionDays: null,
    guildRetentionDays: null,
    devModeEnabled: false,
    autoCheckGameData: true,
    autoUpdateGameData: true,
    userDataAutoPurgeDays: null,
    watchRules: [],
    setupCompleted: false,
    activeCharacterName: null,
    activeServerName: null,
    autoLoadLastCharacter: true,
    autoWatchReports: true,
    reportWatchIntervalSeconds: 10,
    excludeMaxEnchantedRecipes: true,
    marketPriceMode: 'universal',
    itemValuationMode: 'highest_market_vendor',
    showRawJsonInDataBrowser: false,
    viewPreferences: {},
  };
}

async function loadSettings(): Promise<AppSettings> {
  try {
    const backendSettings = await invoke<BackendSettings>("load_settings");
    return fromBackendSettings(backendSettings);
  } catch (e) {
    console.error("Failed to load settings:", e);
    return getDefaultSettings();
  }
}

async function saveSettings(settings: AppSettings): Promise<void> {
  try {
    const backendSettings = toBackendSettings(settings);
    await invoke("save_settings", { settings: backendSettings });
  } catch (e) {
    console.error("Failed to save settings:", e);
  }
}

export const useSettingsStore = defineStore("settings", () => {
  const settings = ref<AppSettings>(getDefaultSettings());
  const settingsFilePath = ref<string>("");
  const isLoaded = ref(false);

  // Initialize settings on store creation
  async function initialize() {
    if (isLoaded.value) return;

    settings.value = await loadSettings();
    isLoaded.value = true;

    // Get settings file path for user reference
    try {
      settingsFilePath.value = await invoke<string>("get_settings_file_path");
    } catch (e) {
      console.error("Failed to get settings file path:", e);
    }
  }

  async function updateLogFilePath(path: string) {
    settings.value.logFilePath = path;
    await saveSettings(settings.value);
  }

  async function updateAutoWatchOnStartup(enabled: boolean) {
    settings.value.autoWatchOnStartup = enabled;
    await saveSettings(settings.value);
  }

  async function updateGameDataPath(path: string) {
    settings.value.gameDataPath = path;
    // Update log path if it was using the old game data path
    if (settings.value.logFilePath.includes("Player.log")) {
      settings.value.logFilePath = path + "\\Player.log";
    }
    await saveSettings(settings.value);
  }

  async function updateSettings(newSettings: Partial<AppSettings>) {
    settings.value = { ...settings.value, ...newSettings };
    await saveSettings(settings.value);
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

  async function updateAutoPurgeEnabled(enabled: boolean) {
    settings.value.autoPurgeEnabled = enabled;
    await saveSettings(settings.value);
  }

  async function updateAutoPurgeDays(days: number) {
    settings.value.autoPurgeDays = days;
    await saveSettings(settings.value);
  }

  async function updateAutoTailChat(enabled: boolean) {
    settings.value.autoTailChat = enabled;
    await saveSettings(settings.value);
  }

  return {
    settings,
    settingsFilePath,
    isLoaded,
    initialize,
    updateLogFilePath,
    updateAutoWatchOnStartup,
    updateGameDataPath,
    updateSettings,
    getPlayerLogPath,
    getChatLogsPath,
    getReportsPath,
    updateAutoPurgeEnabled,
    updateAutoPurgeDays,
    updateAutoTailChat,
  };
});
