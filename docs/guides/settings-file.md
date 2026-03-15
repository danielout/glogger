# Settings System Guide

## Overview

Application settings are stored in a local JSON file managed by the Rust backend. The Vue frontend communicates with Rust via Tauri `invoke` commands to read and write settings. Settings are never stored in `localStorage`.

**Key properties:**
- **Portable** - Easy to back up or transfer between machines
- **Inspectable** - View and edit with any text editor
- **Rust-authoritative** - The Rust `SettingsManager` is the single source of truth; the Vue store is a synchronized mirror

## File Location

The settings file is created automatically at:

**Windows:**
```
%APPDATA%\com.glogger.dev\settings.json
```

Typical full path: `C:\Users\YourName\AppData\Roaming\com.glogger.dev\settings.json`

The exact path is displayed at the bottom of the Settings page in the app.

## File Format

```json
{
  "log_file_path": "C:\\Users\\YourName\\AppData\\LocalLow\\Elder Game\\Project Gorgon\\Player.log",
  "auto_watch_on_startup": false,
  "game_data_path": "C:\\Users\\YourName\\AppData\\LocalLow\\Elder Game\\Project Gorgon",
  "auto_purge_enabled": false,
  "auto_purge_days": 90,
  "auto_tail_chat": false,
  "auto_tail_player_log": false,
  "db_path": null
}
```

### Fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `log_file_path` | string | `""` | Path to the Player.log file (legacy, may deprecate) |
| `auto_watch_on_startup` | boolean | `false` | Auto-start watching Player.log on launch |
| `game_data_path` | string | auto-detected | Root Project Gorgon data folder |
| `auto_purge_enabled` | boolean | `false` | Enable automatic purging of old player data on startup |
| `auto_purge_days` | number | `90` | Delete player data older than this many days |
| `auto_tail_chat` | boolean | `false` | Auto-start chat log tailing on startup |
| `auto_tail_player_log` | boolean | `false` | Auto-start Player.log tailing on startup |
| `db_path` | string or null | `null` | Custom database path (`null` = default in app data dir) |

The file uses **snake_case** field names. The Vue frontend uses **camelCase** equivalents (see [Rust/Vue Communication](#rustvue-communication) below).

---

## Rust/Vue Communication

### Architecture

```
settings.json <--> SettingsManager (Rust) <--> Tauri Commands <--> settingsStore (Vue)
                    [source of truth]          [IPC bridge]        [reactive mirror]
```

### Rust Side (`src-tauri/src/settings.rs`)

The `SettingsManager` struct owns the settings lifecycle:

- **`SettingsManager::new(app_data_dir)`** - Loads `settings.json` from disk on startup, or creates defaults if the file doesn't exist. The `AppSettings` struct is held in an `Arc<RwLock<>>` for thread-safe access.
- **`SettingsManager::get()`** - Returns a clone of the current in-memory settings.
- **`SettingsManager::update(settings)`** - Writes new settings to both memory and disk atomically.
- **`SettingsManager::get_db_path()`** - Returns the custom `db_path` if set, otherwise `{app_data_dir}/glogger.db`.
- **`SettingsManager::get_player_log_path()`** - Derives `Player.log` path from `game_data_path`.
- **`SettingsManager::get_chat_logs_dir()`** - Derives `ChatLogs/` directory from `game_data_path`.

Three Tauri commands are exposed to the frontend:

| Tauri Command | Rust Function | Purpose |
|---------------|---------------|---------|
| `load_settings` | `load_settings()` | Returns the current `AppSettings` to the frontend |
| `save_settings` | `save_settings(settings)` | Accepts an `AppSettings` object, updates memory + writes to disk |
| `get_settings_file_path` | `get_settings_file_path()` | Returns the absolute path to `settings.json` for display |

### Vue Side (`src/stores/settingsStore.ts`)

The Pinia `settingsStore` mirrors the Rust settings as reactive state:

- **`initialize()`** - Called once on app startup. Invokes `load_settings` to hydrate the store, and `get_settings_file_path` to display the file location.
- **Individual updaters** (`updateLogFilePath`, `updateGameDataPath`, etc.) - Each updates the local reactive state and immediately calls `save_settings` to persist via Rust.
- **`updateSettings(partial)`** - Merges a partial settings object and saves.

### Case Conversion

The JSON file and Rust struct use **snake_case**. The Vue `AppSettings` interface uses **camelCase**. Two private functions in `settingsStore.ts` handle conversion:

- `toBackendSettings(settings)` - camelCase -> snake_case (before sending to Rust)
- `fromBackendSettings(settings)` - snake_case -> camelCase (after receiving from Rust)

When adding a new setting, you must update all three places:
1. `AppSettings` struct in `settings.rs` (with `#[serde(default)]` for backwards compatibility)
2. `BackendSettings` interface + `AppSettings` interface in `settingsStore.ts`
3. Both conversion functions (`toBackendSettings` / `fromBackendSettings`)

### Data Flow Example

When a user changes the game data path:

1. User types in the input or uses the Browse button in `GeneralSettings.vue`
2. Component calls `settingsStore.updateGameDataPath(newPath)`
3. Store updates `settings.value.gameDataPath` (reactive, UI updates immediately)
4. Store calls `saveSettings()` which converts to snake_case and invokes `save_settings` Tauri command
5. Rust `save_settings()` updates the in-memory `Arc<RwLock<AppSettings>>` and writes `settings.json` to disk

---

## Settings Page Structure

The Settings page (`src/components/Settings.vue`) is a tabbed container with two tabs. It receives props from its parent for log-watching state (`watching`, `parsing`, `error`) and action callbacks (`onStartWatching`, `onParseLog`).

The settings file path is displayed at the bottom of the page regardless of which tab is active.

### General Tab (`Settings/GeneralSettings.vue`)

Contains day-to-day configuration:

**Log File Management**
- Text input + Browse button for the Player.log path
- "Start Watching" and "Parse Log" buttons (disabled while already watching/parsing or if no path is set)
- "Use Default Player.log Location" button that derives the path from `game_data_path`

**Game Data Directory**
- Text input + Browse (directory) for the Project Gorgon data folder
- Defaults to `%APPDATA%\..\LocalLow\Elder Game\Project Gorgon\`
- Changing this also updates `log_file_path` if it contained "Player.log"

**Startup Behavior**
- Checkbox: "Start watching log file on startup" (`auto_watch_on_startup`)

**Game Data (CDN)**
- Shows cached vs. remote CDN version, update status, item/skill counts
- "Force Refresh CDN Data" button to re-download all game data from the Project: Gorgon CDN

### Advanced Tab (`Settings/AdvancedSettings.vue`)

Contains maintenance and data management operations:

**Parse Old Logs**
- Button to parse a log file from disk (uses the path selected in General tab)

**Database Management**
- Displays database statistics: total size, CDN data size, player data size
- Record counts for market prices, sales history, survey sessions, event log
- "Refresh Statistics" button to reload stats from Rust (`get_database_stats` command)

**Force CDN Table Rebuild**
- "Rebuild CDN Tables" button (styled as warning) to clear and repopulate all CDN-derived tables
- Player data is not affected by this operation

**Player Data Cleanup**
- Purge records older than N days (configurable input + button)
- "Purge ALL Player Data" button (styled as danger) with a required confirmation checkbox
- Shows results after purge: counts of deleted records per table

**Auto-Purge Settings**
- Checkbox: "Automatically purge old data on startup" (`auto_purge_enabled`)
- When enabled, shows a days input for `auto_purge_days` (default: 90)

### Component Conventions

- Both tab components import `settings-shared.css` for consistent styling
- Path inputs save on blur or Enter key, not on every keystroke
- Local `ref()` values shadow store values to avoid saving on every character typed; `watch()` syncs if the store changes externally
- Destructive operations (purge all, CDN rebuild) use visual warning/danger styling and require explicit user action
- Loading/progress states disable buttons and show "...ing" text

---

## Manual Editing

You can edit `settings.json` while the app is closed:

1. Close the application completely
2. Open `settings.json` in any text editor
3. Make changes (use double backslashes `\\` for Windows paths)
4. Save the file
5. Restart the application

New fields added in newer app versions use `#[serde(default)]` in Rust, so older settings files missing those fields will load fine with defaults filled in.

## Backup & Restore

Copy the `settings.json` file to back up. Replace it to restore. Restart the app after restoring.

```bash
# Windows backup
copy "%APPDATA%\com.glogger.dev\settings.json" "D:\Backups\"
```

When transferring to another machine, update `game_data_path` and `log_file_path` to match the new machine's directory structure.

## Troubleshooting

### Settings Not Saving
- Check file permissions on the app data directory
- Check disk space
- Check antivirus (some block writes to AppData)
- View console logs for error messages from `save_settings`

### Settings Reset on Startup
- File may be deleted by antivirus
- File may have JSON syntax errors (Rust will fail to parse and fall back to defaults)
- File permissions may prevent reading

### Can't Find Settings File
- Open the app, go to Settings -- the full path is displayed at the bottom
- Or open Windows Run (Win+R) and type: `%APPDATA%\com.glogger.dev`

### Invalid JSON Errors
- Validate at [JSONLint](https://jsonlint.com/)
- Or delete the file entirely to reset to defaults (the app will recreate it)

## See Also

- [Data Architecture](../plans/data-architecture.md)
