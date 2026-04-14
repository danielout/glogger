# Settings

The Settings screen provides configuration for all Glogger features, accessible via the gear icon in the top-right of the menu bar.

## Layout

Two-column layout with vertical tab navigation on the left and content area on the right. Active tab highlighted with gold text and left border accent. Settings auto-save on change; the settings file path is shown at the bottom.

## Components

- **`Settings.vue`** (`src/components/Settings.vue`) — Root component with tab navigation and conditional rendering of tab content.
- Tab components live in `src/components/Settings/`.

## Tabs

### General
**`GeneralSettings.vue`** — Core configuration: game data path, startup behavior, timestamp display format, crafting options.

### App Settings
**`AppSettingsTab.vue`** — Application-level preferences.

### Chat Logs
**`ChatLogsSettings.vue`** — Chat log file configuration and parsing options. Receives `parsing` and `error` props from the app root.

### Notifications
**`NotificationsSettings.vue`** — Notification preferences.

### User Data
**`UserDataSettings.vue`** — Character data management: JSON imports (character export, VIP inventory), data export/purge.

### Game Data
**`GameDataSettings.vue`** — CDN data configuration and cache management.

### Advanced
**`AdvancedSettings.vue`** — Power-user features: log reparsing, database statistics, CDN rebuild, data purging. See [advanced-settings.md](../advanced-settings.md) for details.

### Game State (Dev Only)
**`GameStateDebug.vue`** — Live game state inspection. Only visible when dev mode is enabled.

## Architecture

- Settings state managed by `settingsStore` (Pinia).
- Settings persisted to disk via Rust backend (`src-tauri/src/settings.rs`).
- See [settings-file.md](../../architecture/settings-file.md) for file format and storage details.
