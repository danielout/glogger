# Update Notifications

## Overview

Glogger checks for new releases on GitHub and notifies the user when a newer version is available. This is a lightweight "go download it" notification — it does not auto-update or download anything on the user's behalf.

## How It Works

### Version Check

- **Source:** GitHub Releases API for [danielout/glogger-release](https://github.com/danielout/glogger-release)
- **Endpoint:** `GET /repos/danielout/glogger-release/releases/latest`
- The Rust backend (`src-tauri/src/update_check.rs`) fetches the latest release, strips the `v` prefix from the tag, and does a semver comparison against the running app version from `tauri.conf.json`.
- Failures (no network, rate-limited, GitHub down) are silently ignored — the feature is best-effort.

### Polling Schedule

- **Initial check:** 5 seconds after app startup (avoids blocking the startup sequence)
- **Recurring:** Every 60 minutes while the app is running
- Managed by the frontend update store (`src/stores/updateStore.ts`)

### User Notification

Two notification surfaces:

1. **Toast (one-time):** When a new version is first detected, an info toast appears via the existing toast system: *"Glogger v0.5.0 is available!"*

2. **Bottom bar (persistent):** A small indicator appears on the right side of the app's bottom status bar showing the available version (e.g., "v0.5.0 available"). Clicking it opens the GitHub release page in the user's default browser. The indicator includes a pulsing dot to draw subtle attention.

The user can dismiss the bottom bar indicator via the ✕ button. It will automatically resurface after 5 hours to gently re-remind during long sessions.

## Key Files

| Layer | File | Purpose |
|-------|------|---------|
| Rust | `src-tauri/src/update_check.rs` | GitHub API call, semver comparison, `check_for_update` command |
| Store | `src/stores/updateStore.ts` | Polling lifecycle, update state, toast trigger |
| UI | `src/App.vue` | Bottom bar indicator, click-to-open handler |

## Design Decisions

- **No auto-update:** Avoids the complexity of code signing, delta patching, and Tauri's built-in updater. Appropriate for the alpha stage.
- **GitHub Releases API:** No custom infrastructure needed — releases are already published there.
- **Silent failure:** Network errors never surface to the user. The update check is strictly informational.
- **Dismiss resurfaces after 5 hours:** Dismissing the indicator hides it temporarily. It reappears after 5 hours to gently re-remind during long sessions without being annoying.
