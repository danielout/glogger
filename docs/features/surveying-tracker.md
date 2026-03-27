# Surveying Tracker

Real-time session tracker for the Surveying skill. Detects survey crafting and completion from Player.log via the player-event-parser pipeline, tracks loot/XP/costs per session, and provides historical analytics with pre-computed session summaries. Supports cross-referencing with Chat.log to correct motherlode loot quantities.

---

## Event Pipeline

### Live Tailing (Coordinator)

```
Player.log -> PlayerLogWatcher -> PlayerEventParser -> SurveyParser -> SurveySessionTracker -> DB
                                                                            |
                                                                  Frontend (survey-event emit)

Chat.log -> ChatLogWatcher -> ChatStatusParser -> SurveySessionTracker.correct_loot_from_chat_status()
                                                        |
                                              Frontend (survey-loot-correction emit)
```

### Dual-Log Replay

```
Player.log ─┐
             ├─ merge by (utc_second, source_order) ─> same pipeline as live tailing
Chat.log  ──┘
```

The replay engine (`src-tauri/src/replay.rs`) interleaves both logs by UTC timestamp for offline analysis. See [Dual-Log Replay](#dual-log-replay) below.

### Pipeline Stages

1. **PlayerEventParser** (`src-tauri/src/player_event_parser.rs`) — Parses raw `ProcessXxx` lines into structured `PlayerEvent`s
2. **SurveyParser** (`src-tauri/src/survey_parser.rs`) — Consumes PlayerEvents to detect survey-specific events:
   - `MapCrafted` — detected when an `ItemAdded` event matches a known survey internal name; includes consumed ingredients from the crafting window
   - `SurveyUsed` — detected from `DelayLoopStarted` with "Using ... Survey/Map" label (informational only)
   - `Completed` — detected from `ScreenText(ImportantInfo, "...collected!...")` with parsed loot items and speed bonus detection
   - `MotherlodeCompleted` — detected via motherlode mining interaction tracking (map consumed → mining delay → loot attribution)
3. **ChatStatusParser** (`src-tauri/src/chat_status_parser.rs`) — Stateless parser that converts `[Status]` channel chat messages into structured `ChatStatusEvent`s (ItemGained, XpGained, LevelUp, CoinsLooted, CouncilsChanged, TreasureDistance, AnatomyResult, Summoned)
4. **SurveySessionTracker** (`src-tauri/src/survey_persistence.rs`) — Synchronous DB persistence:
   - Creates/updates `survey_session_stats` rows
   - Logs events to `survey_events` and loot to `survey_loot_items`
   - Auto-ends sessions when all completable maps are used
   - Finalizes sessions on auto-end: computes revenue, cost, profit, speed bonus count, survey types, and maps used
   - Corrects loot quantities from Chat.log cross-referencing (see [Loot Quantity Correction](#loot-quantity-correction))

### Survey Parser State Machine

1. `UsingSurvey` pending state waits for:
   - `ProcessMapFx` — locate, clear pending
   - `ItemDeleted` for survey map — transition to `AwaitingLoot`
   - `ScreenText` with "collected!" — emit `Completed` directly
   - Timeout (15 lines) — abort
2. `AwaitingLoot` pending state waits for:
   - `ScreenText` with "collected!" — emit `Completed`
   - Timeout (15 lines) — abort
3. **Crafting window tracking** — opens on "Surveying" `DelayLoopStarted`, records all `ItemStackChanged` (negative) and `ItemDeleted` during window, drains when survey map `ItemAdded` detected

### Motherlode Handling

- Motherlode surveys end in "Map" (vs "Survey" for regular)
- They don't produce `Completed` events (spawn a mineable node instead)
- Excluded from auto-end count (`completable_maps` only counts non-motherlode)
- Speed bonuses never apply to motherlode surveys

---

## Database Schema

All tables defined in the unified v1 migration (`src-tauri/src/db/migrations.rs`).

### `survey_types` — CDN-derived reference data

Populated during CDN refresh. Contains survey item metadata: name, zone, skill requirements, crafting cost, icon, category (mineral/mining), and motherlode flag.

### `survey_session_stats` — Pre-computed session summaries

Key columns: `id`, `name`, `notes`, `start_time`, `end_time`, `maps_started`, `surveys_located`, `surveys_completed`, XP gained (surveying/mining/geology), `total_revenue`, `total_cost`, `total_profit`, `profit_per_hour`, `elapsed_seconds`, `is_manual`, `speed_bonus_count`, `survey_types_used`, `maps_used_summary`.

**Finalization split:** The backend computes revenue/cost/profit/bonuses from DB data on session auto-end. The frontend patches in elapsed time (with pause accounting), XP gains, and manual flag via `patch_survey_session`.

### `survey_events` — Individual event log

Columns: `id`, `timestamp`, `session_id` (FK), `event_type` (`session_start`, `completed`, `map_crafted`, `survey_used`), `map_type`, `survey_type`, `speed_bonus_earned`.

### `survey_loot_items` — Individual loot drops

Columns: `id`, `event_id` (FK), `item_id`, `item_name`, `quantity`, `is_speed_bonus`, `is_primary`.

---

## Frontend

### Store (`src/stores/surveyStore.ts`)

- Manages active session display state (persistence is Rust-side)
- Tracks XP baselines and computes deltas for Surveying, Mining, Geology
- Tracks pause durations (only frontend knows these)
- On session end (auto or manual), calls `patch_survey_session` to write elapsed/XP/manual to DB
- Caches item vendor values; falls back to market price via `marketStore` if available

### Components (`src/components/Surveying/`)

- **SurveyView** — Main container with tab navigation
- **SessionTab** — Wraps active session UI
- **SessionCard** — Summary card with stats, XP, economics
- **SessionSidebar** — Controls (pause/end/reset), name/notes input, XP tracking
- **SurveyTypeAccordion** — Per-survey-type revenue/cost/profit breakdown
- **SurveyLootGrid** — Reusable loot grid with counts, percentages, rates/hour
- **SurveyLog** — Activity log showing map-crafted, survey-used, completed events
- **HistoricalTab** — Past sessions list with pre-computed summaries, expandable loot details
- **AnalyticsTab** — All-time speed bonus stats, per-type metrics, loot breakdown

### Frontend Events

| Event | Payload | Purpose |
|-------|---------|---------|
| `survey-event` | SurveyEvent (MapCrafted/SurveyUsed/Completed/MotherlodeCompleted) | Each parsed survey event |
| `survey-session-ended` | `session_id: number` | Backend session auto-ended, frontend should patch elapsed/XP |
| `survey-loot-correction` | LootCorrection `{item_name, old_quantity, new_quantity, delta}` | Chat.log corrected a loot quantity |
| `chat-status-event` | ChatStatusEvent | Structured Status channel events (replay only currently) |
| `skill-update` | SkillUpdate | XP changes (store tracks Surveying/Mining/Geology) |

---

## Loot Quantity Correction

Player.log's `ProcessAddItem` records quantity=1 for new item stacks (the real stack size is encoded in `UpdateItemCode`, but for new items added to inventory, the initial add is always 1). Chat.log's `[Status]` channel provides the real quantity: `"Gypsum x9 added to inventory."`.

### How It Works

1. **ChatStatusParser** parses `[Status]` messages into `ChatStatusEvent::ItemGained { item_name, quantity }` using display names
2. **SurveySessionTracker.correct_loot_from_chat_status()** finds the most recent matching loot row in the current session where `quantity < chat_quantity` and updates it
3. A `LootCorrection` is returned and emitted to the frontend as `survey-loot-correction`
4. The frontend's `handleLootCorrection` applies the delta to running totals

### Session Fallback

When a session auto-ends (all completable maps used), `current_session_id` is cleared. But chat corrections for the last motherlode may still be in-flight. The tracker maintains `last_session_id` as a fallback — corrections check `current_session_id.or(last_session_id)`.

### Name Matching Limitation

Player.log uses internal names (e.g., `MetalSlab7`) while Chat.log uses display names (e.g., `Astounding Metal Slab`). Corrections only work when both logs use the same name — this happens for some minerals (Gypsum, Paladium, Cinnabar, Iridium) but not for metal slabs. This is a known limitation.

---

## Dual-Log Replay

The replay engine (`src-tauri/src/replay.rs`) processes archived Player.log and Chat.log files together, interleaving events by UTC timestamp. This enables cross-referencing features (like loot correction) using historical logs.

### Interleaving Strategy

Both logs are parsed into `TimedEvent` variants, each carrying a UTC second. Events are stable-sorted by `(utc_second, source_order)`:

| source_order | Event Type | Rationale |
|:---:|---|---|
| 0 | ChatLogin | Timezone offset must be applied before processing anything in that second |
| 1 | PlayerLine | Player.log events (loot rows) must exist in DB before chat corrections arrive |
| 2 | ChatMessage | Chat corrections can reference loot rows persisted by PlayerLine events |

### Timezone Handling

- **Chat.log** timestamps are UTC (format: `YY-MM-DD HH:MM:SS`)
- **Player.log** timestamps are local time (format: `[HH:MM:SS]`, no date)
- The chat login line provides `Timezone Offset` (e.g., `-07:00:00` = UTC-7)
- UTC conversion: `utc = local_time + offset_seconds` (offset is negative for west-of-UTC)
- The base date is derived from the chat log filename (`Chat-YY-MM-DD.log`) or first chat message

### Tauri Command

`replay_dual_logs(player_log_path, chat_log_path)` — runs the replay on a blocking thread via `tokio::task::spawn_blocking`. Emits `replay-progress` events during processing and returns a `ReplayResult` summary. The UI for this is in `src/components/Settings/AdvancedSettings.vue`.

---

## Tauri Commands

| Command | Purpose |
|---------|---------|
| `get_all_survey_types` | List all survey types from CDN data |
| `patch_survey_session` | Patch frontend-known fields (elapsed, XP, manual) onto a finalized session |
| `update_survey_session` | Update user-provided name and notes |
| `get_historical_sessions` | Query pre-computed `survey_session_stats` |
| `get_speed_bonus_stats` | Aggregate speed bonus metrics (all-time or per-session) |
| `get_loot_breakdown` | Aggregate loot by item (all-time or per-session) |
| `get_survey_type_metrics` | Per-survey-type stats (all-time or per-session) |
| `replay_dual_logs` | Dual-log replay for offline cross-referencing analysis |



## Improvement Plan

### Survey Analytic Data

- Updated as survey sessions are completed. Calculating this in real time will cause the app to hang, and doesn't gain us any benefits.
- For each zone:
  - Speed Bonus:
    - Total items found across all surveys
    - Total count of each type of item found
    - Min, Max, Average of each item type found per speed bonus proc. (want to be able to produce quality tracking data. example: if aquamarine is the speed bonus, min ever seen is 1, max ever seen is 5, average seen is 2.6 over 567 sightings out of 6543 total speed bonuses)
    - Average value of speed bonus proc for that zone.
    - Since mineral and metal surveys have different speed bonus tables in each zone, the data will need to be split to track them both.
  - Each survey in zone:
    - Min/Max/Average cost of crafting that survey type.
    - Total count of each type of item found
    - Min, Max, Average of each item type found

### Running Surveys

- When we detect XP from crafting surveys, we should list how many surveys (assuming that xp amount per survey) would be required to level up. We don't need to persist this data anywhere, but can be useful to know while running
  - Can we do this for the geology/mining xp when completing them as well? I bet we can.
- If we accidentally attribute something the player loots/trades for/whatever to surveys, players need to be able to ignore it from that session to remove it.

### Loot Correction Improvements

- **Replay throttling:** The replay floods the Windows message queue (`PostMessage` limit ~10,000) when processing large log files. Needs a sleep/yield mechanism every N emitted events to pace the output. The replay runs on a blocking thread (`spawn_blocking`) so `std::thread::sleep` is the right tool.
- **Internal-to-display name resolution:** Metal slab corrections fail because Player.log records `MetalSlab7` but Chat.log says `Astounding Metal Slab`. Resolving internal→display names via game data would enable corrections for all items, not just those with matching names.
- **Buffered corrections for timing edge case:** Within the same UTC second, `ChatMessage` events (sort=2) arrive after `PlayerLine` events (sort=1). But a motherlode's `MotherlodeCompleted` is emitted on a LATER `PlayerLine` than the initial `AddItem`. This means a chat correction for `Gypsum x9` can arrive before the `Gypsum x1` loot row exists in the DB. A buffering system (queue corrections when no DB match, replay after loot persisted) would fix this. Attempted but reverted due to instability — needs a cleaner approach.

### Bugs

- Elapsed time calculation is broken — session shows correct start/end times but "0m 1s elapsed" for a ~1h39m session. Likely a timezone mismatch (UTC vs local) in the elapsed computation, or pause-duration accounting consuming the entire session duration.
- No "New Session" button on the Sessions tab after a session ends — no way to start a fresh session without resetting.

### Survey Summaries

- Need to make sure we're hooked up to the right data sources to provide the summaries for each completed session
- Don't load the details for the session until we expand the view
