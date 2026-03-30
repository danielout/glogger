# Economics — Surveying Sessions

## Overview

Real-time session tracker for the Surveying skill. Detects survey crafting and completion from Player.log via the player-event-parser pipeline, tracks loot/XP/costs per session, and provides historical analytics with pre-computed session summaries. Supports cross-referencing with Chat.log to correct motherlode loot quantities.

## Architecture

### Files

**Backend (Rust):**
- `src-tauri/src/survey_parser.rs` — survey event detection from PlayerEvents
- `src-tauri/src/survey_persistence.rs` — SurveySessionTracker: synchronous DB persistence
- `src-tauri/src/chat_status_parser.rs` — Status channel event parsing for loot corrections
- `src-tauri/src/replay.rs` — dual-log replay engine
- `src-tauri/src/db/survey_commands.rs` — survey type data
- `src-tauri/src/db/player_commands_survey_events.rs` — survey event logging

**Frontend (Vue/TS):**
- `src/stores/surveyStore.ts` — session lifecycle, loot/profit tracking, XP baselines
- `src/components/Surveying/SessionTab.vue` — active session view
- `src/components/Surveying/SessionSidebar.vue` — stats, XP, economics
- `src/components/Surveying/SurveyTypeAccordion.vue` — per-survey-type breakdown
- `src/components/Surveying/SurveyLootGrid.vue` — loot display with counts/percentages
- `src/components/Surveying/SurveyLog.vue` — activity log
- `src/components/Surveying/HistoricalTab.vue` — past sessions
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

## How It Works

### Active Session

- **Manual or auto-start** — sessions can be started manually or triggered by backend survey events
- **Pause/Resume/End** controls with name and notes editing
- **Auto-end** triggered by backend when all completable maps are used

### Session Sidebar (Stats)

- Maps crafted and completed counts
- Average time per survey
- XP gained by skill (Surveying, Mining, Geology)
- Estimated surveys to next level (based on running XP averages)
- Economics: total value, total cost, total profit, per-survey profit, per-hour profit

### Per-Survey-Type Breakdown (Accordion)

Each survey type gets its own expandable section showing:
- Maps started vs. completed
- Revenue (dynamically from current market/vendor prices)
- Cost (from crafting materials)
- Profit and per-survey profit
- Primary loot summary with counts, drop percentages, per-hour rates
- Speed bonus loot (separate section)

### Loot Tracking

- **Primary loot** — standard drops from surveys
- **Speed bonus loot** — extra drops from fast completions
- Item values reactively update when market prices change

### Sub-Tabs

- **Session** — active session view with sidebar, type breakdown, and activity log
- **Historical** — browse and review past sessions with pre-computed summaries
- **Analytics** — all-time speed bonus stats, per-type metrics, loot breakdown

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
| `patch_survey_session` | Patch frontend-known fields (elapsed, XP, manual) onto finalized session |
| `update_survey_session` | Update user-provided name and notes |
| `get_historical_sessions` | Query pre-computed session stats |
| `get_speed_bonus_stats` | Aggregate speed bonus metrics |
| `get_loot_breakdown` | Aggregate loot by item |
| `get_survey_type_metrics` | Per-survey-type stats |
| `replay_dual_logs` | Dual-log replay for offline analysis |

## Known Issues & Improvement Plans

### Bugs
- Elapsed time calculation broken — shows "0m 1s" for long sessions (likely timezone mismatch in elapsed computation)
- No "New Session" button after a session ends

### Planned Improvements
- Per-zone speed bonus analytics (min/max/avg per item type, value tracking, split by mineral/metal)
- XP-to-level estimates during active surveying (crafting + completion XP)
- Player ability to ignore incorrectly attributed loot from a session
- Internal→display name resolution for loot corrections (would fix metal slab corrections)
- Buffered corrections for timing edge cases (chat correction arrives before loot row persisted)
- Replay throttling for large log files (PostMessage queue limit)
