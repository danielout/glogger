# Dev Mode

## Overview

Dev Mode is an opt-in toggle in **Settings > Advanced** that reveals developer/debug tools and beta features throughout the app. It is off by default and intended for contributors, testers, and power users.

## Enabling Dev Mode

1. Go to **Settings > Advanced**
2. Check **Enable Developer Mode**

## What Dev Mode Enables

### Titlebar Link to Dev Panel

When dev mode is on, the "glogger" text in the top-left of the menu bar becomes clickable. Clicking it opens the **Dev Panel** — a separate OS-level window for debug/dev tools.

### Dev Panel Window

A standalone Tauri window (`src/dev-panel/`) that opens independently of the main app. It can be moved to a second monitor, kept open alongside the main app, etc.

**Tabs:**

- **Game State** — The game state inspector (moved from Settings > Game State). Shows all persisted game state domains (skills, inventory, equipment, recipes, favor, currencies, effects, storage) plus live in-memory data (inventory, event log, session skills). Includes an auto-refresh toggle.
- **Component Showcase** — Interactive preview of shared UI components: AccordionSection, StyledSelect, TabBar, EmptyState, ModalDialog (prompt, confirm, danger variants).
- **Testing Helpers** — Toast notification triggers. Can fire toasts in the main app window (via Tauri events) or preview them locally in the dev panel.

### Hidden Sections in Advanced Settings

When dev mode is **off**, the following sections in Settings > Advanced are hidden:
- Player.log Upload (manual log file parsing)
- Dual-Log Replay (interleaved Player.log + Chat.log replay)

These are revealed when dev mode is turned on.

### Game State Tab in Settings

When dev mode is **on**, the Game State tab is hidden from Settings (since it's accessible via the dev panel instead). When dev mode is off, it remains in Settings as a fallback.

## Architecture

### Multi-Window Setup

The dev panel is a second Vite entry point (`dev-panel.html`) with its own Vue app instance (`src/dev-panel/main.ts`). It shares component and store source files with the main app but runs in a separate Tauri webview window.

**Key files:**
- `dev-panel.html` — HTML entry point
- `src/dev-panel/main.ts` — Vue app bootstrap
- `src/dev-panel/DevPanel.vue` — Root component with tab navigation
- `src/dev-panel/tabs/` — Tab components (GameStateTab, ComponentShowcaseTab, TestingHelpersTab)
- `src/composables/useDevPanel.ts` — Window creation/focus composable

**Tauri config:**
- `src-tauri/capabilities/default.json` — Window creation permissions (`core:window:allow-create`, `core:webview:allow-create-webview-window`), dev-panel window included in capability scope
- `vite.config.ts` — Multi-page build with `rollupOptions.input`

### Cross-Window Communication

The dev panel's Testing Helpers tab can trigger toasts in the main window using Tauri's event system:
- Dev panel emits `dev-toast` events via `@tauri-apps/api/event`
- Main app (`App.vue`) listens for `dev-toast` and routes to the toast store
