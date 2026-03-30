# Economics — Farming Sessions

## Overview

A general-purpose session tracker that lets users manually start/stop "farming sessions" to measure efficiency of any in-game activity. Tracks XP gains, item changes, NPC favor, and vendor gold during a session, with live rate-per-hour metrics.

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

The farming calculator subscribes to **existing event channels** — no new Rust-side parsing needed:

```
Player.log → PlayerEventParser → player-event channel ─┐
                                 skill-update channel ──┤
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

### Item Tracking Design

- `ItemStackChanged.delta` is the primary quantity source (every inventory change manifests as a stack change)
- `ItemDeleted` with `StorageTransfer` or `VendorSale` context is **excluded** from net loss — those are intentional player actions
- `ItemDeleted` with `Consumed` or `Unknown` context counts as a farming loss

### Active Session

- **Start/Pause/Resume/End** controls with editable session name and notes
- **Elapsed timer** with pause accounting (paused time excluded from per-hour rates)
- **Three-column live display:**
  - **Skills panel** — XP progress bars per skill, level-ups, XP/hour rates
  - **Items grid** — net item quantities with per-hour rates; ability to ignore specific items from tracking
  - **Activity log** — chronological event feed (item changes, XP gains, favor changes, vendor sales)

### Session Lifecycle

1. User clicks **Start Session** (optionally names it)
2. Store begins intercepting `player-event` and `skill-update` events
3. Live metrics update as events arrive (XP/hr, items/hr, etc.)
4. User can **Pause/Resume** — paused time excluded from rate calculations
5. User can edit session **name** and **notes** inline
6. User clicks **End Session** — summary saved to SQLite
7. Session appears in **History** tab with expandable detail view

### Historical Sessions

- Browse past sessions with full detail (skills, items, favors)
- Edit session name and notes after the fact
- Delete sessions

## Database Tables

- **`farming_sessions`** — session summary (name, notes, timing, vendor gold)
- **`farming_session_skills`** — XP gained per skill per session
- **`farming_session_items`** — net item quantity changes per session
- **`farming_session_favors`** — favor changes per NPC per session

All child tables cascade on delete from `farming_sessions`.

## Tauri Commands

- `save_farming_session(input) → session_id`
- `get_farming_sessions(limit?) → Vec<FarmingSession>`
- `update_farming_session(session_id, name, notes)`
- `delete_farming_session(session_id)`
