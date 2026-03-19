# Surveying Tracker — Architecture Summary

## Overview

A real-time session tracker for the Surveying skill. Detects survey crafting and completion from Player.log via the player-event-parser pipeline, tracks loot/XP/costs per session, and provides historical analytics with pre-computed session summaries.

---

## Architecture

### Event Pipeline

```
Player.log → PlayerLogWatcher → PlayerEventParser → SurveyParser → SurveySessionTracker → DB
                                                                         ↓
                                                              Frontend (survey-event emit)
```

1. **PlayerEventParser** (`player_event_parser.rs`) — Parses raw ProcessXxx lines into structured `PlayerEvent`s (ItemAdded, SkillsLoaded, DelayLoopStarted, ScreenText, etc.)
2. **SurveyParser** (`survey_parser.rs`) — Consumes PlayerEvents + raw lines to detect survey-specific events:
   - `SessionStart` — detected from `DelayLoopStarted` with "Using ... Survey/Map" label
   - `Completed` — detected from `ScreenText(ImportantInfo, "...collected!...")` with parsed loot
3. **SurveySessionTracker** (`survey_persistence.rs`) — Synchronous DB persistence:
   - Creates/updates `survey_session_stats` rows
   - Logs events to `survey_events` and loot to `survey_loot_items`
   - Auto-ends sessions when all completable maps are used
   - **Finalizes sessions** on auto-end: computes and stores revenue, cost, profit, speed bonus count, survey types, and maps used

### Motherlode Handling

- Motherlode surveys end in "Map" (vs "Survey" for regular)
- They don't produce `Completed` events (spawn a mineable node instead)
- Excluded from auto-end count (`completable_maps` only counts non-motherlode)
- Speed bonuses never apply to motherlode surveys

---

## Database Schema

### `survey_session_stats` — Pre-computed session summaries

| Column | Type | Source |
|--------|------|--------|
| `id` | INTEGER PK | Auto |
| `start_time` | TIMESTAMP | First SessionStart event |
| `end_time` | TIMESTAMP | Last Completed event |
| `maps_started` | INTEGER | Count of SessionStart events |
| `surveys_completed` | INTEGER | Count of Completed events |
| `surveying_xp_gained` | INTEGER | Frontend (XP delta tracking) |
| `mining_xp_gained` | INTEGER | Frontend (XP delta tracking) |
| `geology_xp_gained` | INTEGER | Frontend (XP delta tracking) |
| `total_revenue` | INTEGER | Finalized: SUM(loot qty × item value) |
| `total_cost` | INTEGER | Finalized: SUM(survey_types.crafting_cost) |
| `total_profit` | INTEGER | Finalized: revenue - cost |
| `profit_per_hour` | INTEGER | Finalized: profit / hours |
| `elapsed_seconds` | INTEGER | Frontend (with pause accounting) |
| `is_manual` | BOOLEAN | Frontend |
| `speed_bonus_count` | INTEGER | Finalized: COUNT(speed_bonus_earned) |
| `survey_types_used` | TEXT | Finalized: comma-separated names |
| `maps_used_summary` | TEXT | Finalized: "MapA x3, MapB x1" |

**Finalization split:** The backend computes revenue/cost/profit/bonuses from DB data on session auto-end. The frontend patches in elapsed time (accounting for pauses), XP gains, and manual flag via `patch_survey_session`.

### `survey_events` — Individual event log

| Column | Type | Notes |
|--------|------|-------|
| `id` | INTEGER PK | |
| `timestamp` | TIMESTAMP | |
| `session_id` | INTEGER FK | Links to survey_session_stats |
| `event_type` | TEXT | 'session_start' or 'completed' |
| `map_type` | TEXT | Map name (on session_start) |
| `survey_type` | TEXT | Survey name (on completed) |
| `speed_bonus_earned` | BOOLEAN | |

### `survey_loot_items` — Individual loot drops

| Column | Type | Notes |
|--------|------|-------|
| `id` | INTEGER PK | |
| `event_id` | INTEGER FK | Links to survey_events |
| `item_id` | INTEGER FK | Nullable, links to items |
| `item_name` | TEXT | Always stored |
| `quantity` | INTEGER | |
| `is_speed_bonus` | BOOLEAN | |
| `is_primary` | BOOLEAN | |

### `survey_types` — CDN-derived reference data

Populated during CDN refresh. Contains survey item metadata: name, zone, skill requirements, crafting cost, icon, category (mineral/mining), and motherlode flag.

---

## Frontend

### Store (`surveyStore.ts`)

- Manages active session display state (not persistence — that's all Rust-side)
- Tracks XP baselines and computes deltas (only frontend knows these)
- Tracks pause durations (only frontend knows these)
- On session end (auto or manual), calls `patch_survey_session` to write elapsed/XP/manual to DB

### Views

- **SessionTab** — Live session: controls, metrics, survey type breakdown, loot grid, event log
- **HistoricalTab** — Past sessions list with pre-computed summary data, expandable loot details
- **AnalyticsTab** — All-time speed bonus stats, per-type metrics, loot breakdown (queries raw events)

### Events

| Event | Payload | Purpose |
|-------|---------|---------|
| `survey-event` | SurveyEvent | Each parsed survey event (SessionStart/Completed) |
| `survey-session-ended` | session_id: number | Backend session auto-ended, frontend should patch elapsed/XP |
| `skill-update` | SkillUpdate | XP changes (store tracks Surveying/Mining/Geology) |

---

## Tauri Commands

| Command | Purpose |
|---------|---------|
| `get_all_survey_types` | List all survey types from CDN data |
| `save_survey_session_stats` | Create a manual session entry |
| `patch_survey_session` | Patch frontend-known fields (elapsed, XP, manual) onto a finalized session |
| `get_historical_sessions` | Simple SELECT from pre-computed survey_session_stats |
| `log_survey_event` | Manual event logging |
| `get_survey_events` | Query events by session |
| `log_survey_loot_item` | Manual loot logging |
| `get_survey_loot_items` | Query loot by event |
| `get_speed_bonus_stats` | Aggregate speed bonus metrics (all-time or per-session) |
| `get_loot_breakdown` | Aggregate loot by item (all-time or per-session) |
| `get_survey_type_metrics` | Per-survey-type stats (all-time or per-session) |
