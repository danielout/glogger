# Economics — Farming Sessions

## Overview

A general-purpose session tracker that lets users manually start/stop "farming sessions" to measure efficiency of any in-game activity. Tracks XP gains, item changes, NPC favor, vendor gold, and **enemy kills** during a session, with live rate-per-hour metrics.

## Architecture

### Files

**Backend (Rust):**
- `src-tauri/src/db/migrations.rs` — migration_v5_farming (4 tables)
- `src-tauri/src/db/farming_commands.rs` — save, get, delete commands

**Frontend (Vue/TS):**
- `src/types/farming.ts` — all TypeScript interfaces
- `src/stores/farmingStore.ts` — Pinia store with event handlers, session controls, computed metrics
- `src/components/Farming/FarmingView.vue` — tabbed container (Session / History)
- `src/components/Farming/FarmingSessionCard.vue` — live session display with metrics
- `src/components/Farming/FarmingLog.vue` — chronological activity feed
- `src/components/Farming/HistoricalTab.vue` — past sessions from DB

## How It Works

### Event Pipeline

The farming calculator subscribes to multiple event channels:

```
Player.log → PlayerEventParser → player-events-batch channel ─┐
                                skill-update channel ─────────┤
Chat.log  → ChatCombatParser  → enemy-killed channel ─────────┤
                                                              ▼
                                                    farmingStore (if session active)
                                                              │
                                                    accumulate deltas + push log
                                                              │
                                                    on endSession() → invoke("save_farming_session")
```

### What Gets Tracked

| Category | Source Event | Details |
|----------|-------------|---------|
| Skill XP | `skill-update` | Baseline delta per skill, level-ups detected, XP/hr computed |
| Items Gained | `ItemStackChanged` (positive delta) | Net quantity per item, items/hr |
| Items Lost | `ItemStackChanged` (negative delta), `ItemDeleted` (Consumed/Unknown) | Storage transfers and vendor sales excluded |
| NPC Favor | `FavorChanged` | Accumulated per NPC |
| Vendor Gold | `VendorSold` | Total gold from vendor sales |
| Enemy Kills | `enemy-killed` (chat combat) | Kill count per enemy type, kills/hr, loot attributed via CorpseSearch provenance |

### Item Tracking Design

- `ItemStackChanged.delta` is the primary quantity source (every inventory change manifests as a stack change)
- `ItemDeleted` with `StorageTransfer` or `VendorSale` context is **excluded** from net loss — those are intentional player actions
- `ItemDeleted` with `Consumed` or `Unknown` context counts as a farming loss

### Active Session

- **Start/Pause/Resume/End** controls with editable session name and notes
- **Elapsed timer** with pause accounting (paused time excluded from per-hour rates)
- **Three-column live display:**
  - **Skills panel** — XP progress bars per skill, level-ups, XP/hour rates; favor changes; kill summary with per-creature loot attribution
  - **Items grid** — net item quantities with per-hour rates; ability to ignore specific items from tracking
  - **Activity log** — chronological event feed (item changes, XP gains, favor changes, vendor sales, kills)

### Session Lifecycle

1. User clicks **Start Session** (optionally names it)
2. Store begins intercepting `player-event` and `skill-update` events
3. Live metrics update as events arrive (XP/hr, items/hr, etc.)
4. User can **Pause/Resume** — paused time excluded from rate calculations
5. User can edit session **name** and **notes** inline
6. User clicks **End Session** — summary saved to SQLite
7. Session appears in **History** tab with expandable detail view

### Historical Sessions

- Browse past sessions with full detail (skills, items, favors, kills)
- Edit session name and notes after the fact
- Delete sessions

## Kill Tracking

Kill tracking operates at two levels:

### Standalone Kill Database

Every enemy kill is persisted to the `enemy_kills` table regardless of whether a farming session is active. When the player loots a corpse, items with `CorpseSearch` provenance are linked to the kill via entity_id matching and stored in `enemy_kill_loot`. This builds a drop-rate database over time.

The kill-to-loot link uses a short-lived in-memory map (`recent_kills`) that maps `enemy_entity_id -> kill_row_id`. When items arrive with `CorpseSearch { entity_id }` provenance, the coordinator looks up the matching kill and inserts into `enemy_kill_loot`. Entries expire after 120 seconds.

### Farming Session Integration

During an active farming session, the farming store:
1. Listens for `enemy-killed` events and increments per-enemy-type kill counts
2. Uses `CorpseSearch` provenance on `ItemStackChanged` events to attribute loot to specific enemy types
3. Displays kills/hr and per-creature loot breakdown in the session card
4. Persists aggregated kill counts to `farming_session_kills` when the session ends

## Database Tables

- **`farming_sessions`** — session summary (name, notes, timing, vendor gold)
- **`farming_session_skills`** — XP gained per skill per session
- **`farming_session_items`** — net item quantity changes per session
- **`farming_session_favors`** — favor changes per NPC per session
- **`farming_session_kills`** — kill counts per enemy type per session
- **`enemy_kills`** — standalone record of every enemy kill (not session-scoped)
- **`enemy_kill_loot`** — items attributed to specific kills via entity_id matching

`farming_session_*` tables cascade on delete from `farming_sessions`. `enemy_kill_loot` cascades on delete from `enemy_kills`.

## Time Handling

Farming follows the app-wide time standards defined in [time.md](../../../architecture/time.md).

### Active Sessions

- Session `startTime`, `endTime`, and `pauseStartTime` are stored as `HH:MM:SS` strings in the in-memory `FarmingSession` object. These are generated by `getCurrentTimestamp()` which uses `formatTimeFull()` from `useTimestamp.ts`.
- **Elapsed calculation** converts these strings to seconds via `tsToSeconds()` for pause-aware duration math. The computed `elapsed` display uses `formatDuration()` with `alwaysShowSeconds: true` for the live timer.
- **Per-hour rates** (XP/hr, items/hr) divide accumulated totals by `getActiveSeconds() / 3600`.
- **Activity log** entries carry the raw timestamp from the Player.log event and are displayed via `formatAnyTimestamp()`.

### Persistence

- When a session ends, `start_time` and `end_time` strings and the computed `elapsed_seconds` integer are saved to the database.
- Historical sessions display timestamps via `formatDateTimeShort()` and durations via `formatDuration()` (without seconds for completed sessions).

## Tauri Commands

- `save_farming_session(input) → session_id`
- `get_farming_sessions(limit?) → Vec<FarmingSession>`
- `update_farming_session(session_id, name, notes)`
- `delete_farming_session(session_id)`
