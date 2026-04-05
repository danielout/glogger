# Economics — Surveying

## Overview

Real-time session tracker for the Surveying skill. Detects survey crafting and completion from Player.log via the player-event-parser pipeline, tracks loot/XP/costs per session, and provides historical analytics with pre-computed session summaries. Supports cross-referencing with Chat.log to correct motherlode loot quantities.

The surveying feature has three sub-tabs, each documented separately:

- [Session](economics-surveying-session.md) — active session tracking with live loot/XP/profit
- [Historical](economics-surveying-historical.md) — browse and review past sessions
- [Analytics](economics-surveying-analytics.md) — all-time aggregate stats organized by zone

## Architecture

### Files

**Backend (Rust):**
- `src-tauri/src/survey_parser.rs` — survey event detection from PlayerEvents
- `src-tauri/src/survey_persistence.rs` — SurveySessionTracker: synchronous DB persistence
- `src-tauri/src/chat_status_parser.rs` — Status channel event parsing for loot corrections
- `src-tauri/src/replay.rs` — dual-log replay engine
- `src-tauri/src/db/survey_commands.rs` — survey type data
- `src-tauri/src/db/player_commands_survey_events.rs` — survey event logging, historical queries, analytics aggregation

**Frontend (Vue/TS):**
- `src/stores/surveyStore.ts` — session lifecycle, loot/profit tracking, XP baselines
- `src/components/Economics/EconomicsSurveyView.vue` — tab container (Session / Historical / Analytics)
- `src/components/Surveying/SessionTab.vue` — active session view
- `src/components/Surveying/SessionSidebar.vue` — stats, XP, economics sidebar
- `src/components/Surveying/SurveyTypeAccordion.vue` — per-survey-type breakdown table
- `src/components/Surveying/SurveyLootGrid.vue` — loot display with counts/percentages
- `src/components/Surveying/SurveyLog.vue` — activity log
- `src/components/Surveying/HistoricalTab.vue` — past sessions browser
- `src/components/Surveying/AnalyticsTab.vue` — aggregated analytics

## Event Pipeline

### Live Tailing

```
Player.log -> PlayerLogWatcher -> PlayerEventParser -> SurveyParser -> SurveySessionTracker -> DB
                                                                            |
                                                              Frontend (survey-event emit)

Chat.log -> ChatLogWatcher -> ChatStatusParser -> SurveySessionTracker.correct_loot_from_chat_status()
                                                        |
                                              Frontend (survey-loot-correction emit)
```

### Pipeline Stages

1. **PlayerEventParser** — parses raw `ProcessXxx` lines into structured `PlayerEvent`s
2. **SurveyParser** — consumes PlayerEvents to detect survey-specific events:
   - `MapCrafted` — detected when `ItemAdded` matches a known survey internal name; includes consumed ingredients
   - `SurveyUsed` — detected from `DelayLoopStarted` with "Using ... Survey/Map" label
   - `Completed` — detected from `ScreenText(ImportantInfo, "...collected!...")` with parsed loot items and speed bonus detection
   - `MotherlodeCompleted` — detected via motherlode mining interaction tracking
3. **ChatStatusParser** — stateless parser converting `[Status]` messages into `ChatStatusEvent`s (ItemGained, XpGained, LevelUp, etc.)
4. **SurveySessionTracker** — synchronous DB persistence: creates/updates sessions, logs events and loot, auto-ends sessions, corrects loot quantities from Chat.log

### Survey Parser State Machine

1. `UsingSurvey` pending state waits for: `ProcessMapFx` (locate), `ItemDeleted` for map (→ `AwaitingLoot`), `ScreenText` "collected!" (→ emit `Completed`), or timeout (15 lines)
2. `AwaitingLoot` pending state waits for: `ScreenText` "collected!" (→ emit `Completed`) or timeout
3. **Crafting window tracking** — opens on "Surveying" `DelayLoopStarted`, records all `ItemStackChanged`/`ItemDeleted` during window, drains when survey map `ItemAdded` detected

### Motherlode Handling

- Motherlode surveys end in "Map" (vs "Survey" for regular)
- They don't produce `Completed` events (spawn a mineable node instead)
- Excluded from auto-end count (`completable_maps` only counts non-motherlode)
- Speed bonuses never apply to motherlode surveys

## Loot Quantity Correction

Player.log's `ProcessAddItem` records quantity=1 for new item stacks (the real stack size is in `UpdateItemCode`). Chat.log's `[Status]` channel provides the real quantity: `"Gypsum x9 added to inventory."`.

### How It Works

1. ChatStatusParser parses `[Status]` messages into `ChatStatusEvent::ItemGained { item_name, quantity }`
2. SurveySessionTracker finds the most recent matching loot row where `quantity < chat_quantity` and updates it
3. A `LootCorrection` is emitted to the frontend as `survey-loot-correction`
4. The frontend's `handleLootCorrection` applies the delta to running totals

### Session Fallback

When a session auto-ends, `current_session_id` is cleared. But chat corrections for the last motherlode may still be in-flight. The tracker maintains `last_session_id` as a fallback.

### Name Matching Limitation

Player.log uses internal names (e.g., `MetalSlab7`) while Chat.log uses display names (e.g., `Astounding Metal Slab`). Corrections only work when both logs use the same name — works for some minerals (Gypsum, Paladium) but not metal slabs. Known limitation.

## Dual-Log Replay

The replay engine (`src-tauri/src/replay.rs`) processes archived Player.log and Chat.log files together, interleaving events by UTC timestamp. This enables cross-referencing features using historical logs.

### Interleaving Strategy

Events are stable-sorted by `(utc_second, source_order)`:

| source_order | Event Type | Rationale |
|:---:|---|---|
| 0 | ChatLogin | Timezone offset must be applied first |
| 1 | PlayerLine | Player.log loot rows must exist in DB before chat corrections |
| 2 | ChatMessage | Chat corrections reference loot rows from PlayerLine events |

### Timezone Handling

- Chat.log timestamps are UTC; Player.log timestamps are local time
- Chat login line provides timezone offset for UTC conversion
- Base date derived from chat log filename or first message

## Database Schema

### `survey_types` — CDN-derived reference data
Survey item metadata: name, zone, skill requirements, crafting cost, icon, category, motherlode flag.

### `survey_session_stats` — Pre-computed session summaries
Key columns: id, name, notes, start/end time, maps_started, surveys_completed, XP gains (surveying/mining/geology), revenue/cost/profit, elapsed_seconds, speed_bonus_count, survey_types_used, maps_used_summary.

### `survey_events` — Individual event log
Columns: id, timestamp, session_id (FK), event_type, map_type, survey_type, speed_bonus_earned.

### `survey_loot_items` — Individual loot drops
Columns: id, event_id (FK), item_id, item_name, quantity, is_speed_bonus, is_primary.

## Frontend Events

| Event | Payload | Purpose |
|-------|---------|---------|
| `survey-event` | SurveyEvent | Each parsed survey event |
| `survey-session-ended` | `session_id` | Backend session auto-ended |
| `survey-loot-correction` | LootCorrection | Chat.log corrected a loot quantity |
| `skill-update` | SkillUpdate | XP changes (Surveying/Mining/Geology) |

## Tauri Commands

| Command | Purpose |
|---------|---------|
| `get_all_survey_types` | List all survey types from CDN data |
| `patch_survey_session` | Patch frontend-known fields (elapsed, XP, manual) onto finalized session. Uses the larger of frontend-computed or DB-computed elapsed to prevent imported sessions from getting ~1s durations |
| `update_survey_session` | Update user-provided name and notes |
| `update_survey_session_times` | Update start/end timestamps and re-finalize economics (for correcting imported session timing) |
| `get_historical_sessions` | Query pre-computed session stats |
| `get_speed_bonus_stats` | Aggregate speed bonus metrics (optional session filter) |
| `get_loot_breakdown` | Aggregate loot by item for a session |
| `get_survey_type_metrics` | Per-survey-type stats |
| `get_zone_analytics` | Zone-grouped analytics with per-category speed bonus and per-survey-type loot stats |
| `replay_dual_logs` | Dual-log replay for offline analysis |

## Time Handling

Surveying follows the app-wide time standards defined in [time.md](../../../architecture/time.md).

### Backend

- All `survey_events.timestamp` and `survey_session_stats.start_time`/`end_time` values are stored as UTC strings (`YYYY-MM-DD HH:MM:SS`).
- Player.log timestamps are converted to UTC via `SurveySessionTracker::to_utc()`, which delegates to `to_utc_datetime()` in `parsers.rs`.
- Replay timestamps use the dual-log interleaving strategy (Chat.log UTC timestamps are authoritative; Player.log times are converted using the chat login timezone offset).

### Frontend

- **Active session timing** uses epoch milliseconds (`Date.now()`) for `startTime`, `endTime`, `pauseStartTime`, and `completionTimestamps`. This is internal-only state for live timer math — not persisted directly.
- **Elapsed display** uses `formatDuration()` from `useTimestamp.ts` with `alwaysShowSeconds: true` for the live timer.
- **Average survey time** also uses `formatDuration()` with `alwaysShowSeconds: true`.
- **Start/end timestamps** displayed in the sidebar and session card use `formatTimeFull()` (converts epoch ms → ISO → timezone-aware `HH:MM:SS`).
- **Historical sessions** display timestamps via `formatDateTimeShort()` and durations via `formatDuration()` (without seconds, since precision isn't needed for completed sessions).
- **Activity log** timestamps pass through `formatAnyTimestamp()` which handles both full UTC datetime strings (from DB events) and bare `HH:MM:SS` strings (from live Player.log events).

## Known Issues & Improvement Plans

### Planned Improvements
- Per-zone speed bonus analytics (min/max/avg per item type, value tracking, split by mineral/metal)
- XP-to-level estimates during active surveying (crafting + completion XP)
- Player ability to ignore incorrectly attributed loot from a session
- Internal→display name resolution for loot corrections (would fix metal slab corrections)
- Buffered corrections for timing edge cases (chat correction arrives before loot row persisted)
- Replay throttling for large log files (PostMessage queue limit)
