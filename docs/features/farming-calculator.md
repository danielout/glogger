# Farming Calculator

## Overview

A general-purpose session tracker that lets users manually start/stop "farming sessions" to measure efficiency of any in-game activity. Tracks XP gains, item changes, NPC favor, and vendor gold during a session, with live rate-per-hour metrics.

## How It Works

The farming calculator subscribes to **existing event channels** — no new Rust-side parsing was needed:

- **`player-event`** — item stack changes, item deletions, favor changes, vendor sales
- **`skill-update`** — XP gains per skill with level-up detection

When a session is active, the store intercepts these events and accumulates deltas. When paused, events are ignored. On session end, the summary is persisted to SQLite.

## What Gets Tracked

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

### Database Tables

- **`farming_sessions`** — session summary (name, notes, timing, vendor gold)
- **`farming_session_skills`** — XP gained per skill per session
- **`farming_session_items`** — net item quantity changes per session
- **`farming_session_favors`** — favor changes per NPC per session

All child tables cascade on delete from `farming_sessions`.

### Data Flow

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

## Session Lifecycle

1. User clicks **Start Session** (optionally names it)
2. Store begins intercepting `player-event` and `skill-update` events
3. Live metrics update as events arrive (XP/hr, items/hr, etc.)
4. User can **Pause/Resume** — paused time excluded from rate calculations
5. User can edit session **name** and **notes** inline
6. User clicks **End Session** — summary saved to SQLite
7. Session appears in **History** tab with expandable detail view

## Patterns Followed

- Session state management mirrors `surveyStore.ts` (pause/resume, elapsed time, baseline XP tracking)
- UI component hierarchy follows `Surveying/` pattern (tabbed view, session card, activity log)
- Database persistence follows `player_commands_survey_events.rs` (composite input, single save call)
- Uses shared inline components (`ItemInline`, `SkillInline`, `NpcInline`) for entity display
