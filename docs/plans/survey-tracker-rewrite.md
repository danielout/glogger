# Survey Tracker Rewrite (Phase 5 of Item Provenance Overhaul)

> This is the implementation plan for the survey tracker rewrite called out as Phase 5 in [item-provenance-overhaul.md](item-provenance-overhaul.md). Read that first for the broader provenance pipeline this plugs into.

## Why a rewrite

The existing `survey_parser.rs` + `survey_persistence.rs` rely on `ProcessScreenText` regex scraping ("X collected! Also found Y x3 (speed bonus!)") which has been called out as the "weakest" tracker in the app — fragile, prone to format drift, and missing classes of events entirely (it doesn't handle multihit nodes at all today). With the unified provenance pipeline (Phases 1–4) in place, every gain event already carries `provenance.source` identifying whether it came from a `SurveyMapUse` or `Mining` activity. The survey tracker should be a thin consumer of that pipeline, not its own parser.

This is a **complete nuke and pave** — the existing `survey_parser.rs`, `survey_persistence.rs`, and frontend survey screen logic are deleted and replaced. Beta users will be briefly affected; they were warned.

## Design choices (decided)

- **A3: Time-proximity stitching** — `Mining` and `SurveyMapUse` stay separate `ActivitySource`s in the parser. The new aggregator stitches them at write time by detecting "mining context opened within N seconds of a survey map being consumed" and chaining the resulting gains to the originating survey use.
- **Schema** — keep `survey_sessions` (header) and `survey_uses` (one row per map use); drop `survey_loot_items` in favor of querying `item_transactions` filtered by `source_kind` and `source_details->>'survey_use_id'`.
- **Session lifecycle**:
  - Start: manual button OR detected survey-map crafting OR detected first survey-map use
  - End for crafting-started sessions: when the count of consumed survey-map uses equals the count of crafted survey-maps (i.e., player used everything they crafted)
  - End for the other two starts: manual end button only
- **Multihit window state lives in DB** (not in-memory) so 30-minute windows survive app restarts. Basic and Motherlode windows are short enough to keep in-memory.
- **No legacy / no parallel.** Old code is deleted. Survey screen rewritten from scratch.
- **Backend-owned state** (per CLAUDE.md rule) — tracker survives screen navigation.

## Architecture

### Module layout

New directory `src-tauri/src/survey/` replaces the two top-level files:

```
src-tauri/src/survey/
├── mod.rs               # Public surface: SurveyTracker, commands
├── aggregator.rs        # Subscribes to PlayerEvents, stitches survey↔mining attribution
├── session.rs           # Session lifecycle (start/end logic, idle handling)
├── multihit_state.rs    # DB-backed open-multihit-node tracking
├── persistence.rs       # All DB I/O (rows in/out, no business logic)
└── tests.rs             # Integration tests using the paired log datasets
```

Files removed: `src-tauri/src/survey_parser.rs`, `src-tauri/src/survey_persistence.rs`.

### Data flow

```
Player.log ──► PlayerEventParser ──► PlayerEvent { provenance } ──┐
                                                                   ├──► SurveyTracker (in coordinator)
Chat.log ────► ChatStatusParser ──► ChatStatusEvent ──────────────┘         │
                                                                            │
                                                                            ▼
                                                                  Stitches survey use ↔ mining
                                                                            │
                                                                            ▼
                                                              SQLite: survey_sessions,
                                                                       survey_uses,
                                                                       open_multihit_nodes,
                                                                       item_transactions
                                                                            │
                                                                            ▼
                                                              Tauri commands ──► Frontend (read-only)
```

The frontend never owns tracker state. It calls commands like `survey_tracker_status()`, `survey_tracker_start_session()`, `survey_tracker_end_session()`, `survey_tracker_recent_uses(limit)`. Live updates flow via existing event emits (`survey-session-updated`, etc.).

### A3 stitching: how survey↔mining attribution works

The parser cannot know that a mining context is "the loot from a motherlode map I used 4 seconds ago". That cross-context inference is too survey-specific to live in the generic parser. It belongs in the aggregator.

Mechanism:

1. Parser emits `ItemDeleted` for a survey map (with `DeleteContext::Consumed`, derived later from kind classification).
2. Aggregator sees the deletion, looks up the map's `SurveyKind`, allocates a `survey_use_id`, and writes a `survey_uses` row with status `pending_loot`.
3. Aggregator pushes a "pending attribution" record into its own in-memory state: `(survey_use_id, kind, map_internal_name, expires_at)`.
4. Subsequent `Mining` activity contexts that open within the kind's grace window (Motherlode: 60s of survey use; Multihit: same) get their gains tagged with this `survey_use_id` in `item_transactions.source_details`.
5. For Multihit: the open-mining-entity is recorded in `open_multihit_nodes` with the originating `survey_use_id`. When that mining context emits gains, they chain through. Window closes per the rules in [survey-mechanics.md](../architecture/survey-mechanics.md).
6. For Basic: there is no separate Mining context — gains drop directly into the `SurveyMapUse` window. The aggregator just attaches `survey_use_id` to gains whose `provenance.source == SurveyMapUse`.

The parser's `ItemProvenance` taxonomy doesn't change. The aggregator augments `source_details` with `survey_use_id` when persisting transactions, which is the grouping key for "all loot from this survey use" queries.

## Schema (migration v26)

```sql
-- New table: per-session header
CREATE TABLE survey_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    character_name TEXT NOT NULL,
    server_name TEXT NOT NULL,
    started_at TEXT NOT NULL,
    ended_at TEXT,
    -- 'manual' | 'crafting' | 'first_use'
    start_trigger TEXT NOT NULL,
    -- only set when start_trigger='crafting': how many maps were crafted in
    -- this session (auto-end when consumed_count == crafted_count)
    crafted_count INTEGER,
    consumed_count INTEGER NOT NULL DEFAULT 0,
    notes TEXT
);
CREATE INDEX idx_survey_sessions_char ON survey_sessions(character_name, server_name);
CREATE INDEX idx_survey_sessions_active ON survey_sessions(ended_at) WHERE ended_at IS NULL;

-- New table: one row per survey-map use
CREATE TABLE survey_uses (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id INTEGER REFERENCES survey_sessions(id) ON DELETE SET NULL,
    character_name TEXT NOT NULL,
    server_name TEXT NOT NULL,
    used_at TEXT NOT NULL,
    map_internal_name TEXT NOT NULL,
    map_display_name TEXT NOT NULL,
    -- 'basic' | 'motherlode' | 'multihit'
    kind TEXT NOT NULL,
    -- area where used (live-tracked area at use time; canonical for disambiguation)
    area TEXT,
    -- 'pending_loot' | 'completed' | 'aborted' | 'unknown'
    -- pending_loot: window still open, more loot may arrive
    -- completed: window closed cleanly with at least one loot row
    -- aborted: window closed without loot (e.g., motherlode despawned, multihit timeout with nothing)
    -- unknown: shouldn't happen; defensive default
    status TEXT NOT NULL DEFAULT 'pending_loot',
    -- denormalized convenience: total loot quantity attributed to this use.
    -- updated as item_transactions are inserted; not the source of truth.
    loot_qty INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX idx_survey_uses_session ON survey_uses(session_id);
CREATE INDEX idx_survey_uses_status ON survey_uses(status) WHERE status = 'pending_loot';

-- New table: multihit nodes the player has spawned and started mining.
-- Survives app restart so a long mining session isn't lost.
CREATE TABLE open_multihit_nodes (
    -- entity_id from ProcessStartInteraction; per-area unique
    node_entity_id INTEGER NOT NULL,
    character_name TEXT NOT NULL,
    server_name TEXT NOT NULL,
    survey_use_id INTEGER NOT NULL REFERENCES survey_uses(id) ON DELETE CASCADE,
    opened_at TEXT NOT NULL,
    last_hit_at TEXT NOT NULL,
    PRIMARY KEY (character_name, server_name, node_entity_id)
);
CREATE INDEX idx_open_multihit_nodes_use ON open_multihit_nodes(survey_use_id);
```

**Decision at implementation time** (made during the build): the old tables (`survey_session_stats`, `survey_events`, `survey_loot_items`) are **not** dropped in v26. They stay alongside the new tables during cutover so the legacy survey screen and its supporting Tauri commands (`patch_survey_session`, `save_survey_session_stats`, `get_survey_sessions`, etc.) keep working until the frontend rewrite lands. A follow-up migration (v27) drops them once the new frontend is verified end-to-end. Reasoning: the surface area of the legacy commands turned out larger than expected — `db/player_commands.rs` has ~10 commands wired into the existing screen, and pulling those out cleanly belongs in the same change as the frontend rewrite, not in the schema migration.

`item_transactions.source_details` for gains attributed to a survey gets a `survey_use_id` field added to the JSON blob. No schema change needed — the column is `TEXT` JSON.

## Per-kind logic (window semantics)

| Kind | Window opens | Window closes | Stitching |
|---|---|---|---|
| Basic | Map consumed (`ItemDeleted`) | Same tick (loot arrives in same batch) | Direct `SurveyMapUse` provenance on gains |
| Motherlode | Map consumed | First Mining context that opens within 60s, runs to completion (`EndInteraction` or 30s mining timeout); window closes at end of that single mining cycle | Map consumption → records pending attribution → next Mining context within 60s adopts the `survey_use_id` |
| Multihit | Map consumed | (a) Player starts mining a *different* `entity_id`, OR (b) 30 minutes elapsed since last hit on this node | Same as Motherlode initially, but the `entity_id` of the first Mining context is recorded in `open_multihit_nodes` and held until close conditions hit |

The "60s grace" between map consumption and mining-context-open covers the player walking from where they used the map to where the node spawned.

For Multihit specifically: every time a Mining context fires on a node tracked in `open_multihit_nodes`, the gains from that cycle are attributed to the same `survey_use_id` and `last_hit_at` is updated. When the player starts mining a different entity, the previous open multihit row is closed (status → completed, removed from `open_multihit_nodes`). The 30-minute fallback runs as a periodic sweep (every minute or so) on the table.

## Session lifecycle implementation

State machine, kept simple:

```
                        ┌──────────────────────────┐
                        │   no active session      │
                        └────┬──────┬──────┬───────┘
                             │      │      │
              user clicks    │      │      │  first survey use detected
              "start session"│      │      │  (no active session)
                             │      │      │
                             ▼      │      ▼
                  ┌──────────────┐  │  ┌──────────────┐
                  │ active(manual)│  │  │ active(first_use)│
                  └──────┬───────┘  │  └──────┬─────────┘
                         │          │         │
                         │ user clicks "end"  │ user clicks "end"
                         │          │         │
                         ▼          │         ▼
                  ┌──────────────────────────┐
                  │   ended (final stats)    │
                  └──────────────────────────┘
                                │
                                │ first crafted-survey detected
                                │ (no active session)
                                ▼
                  ┌──────────────────────────┐
                  │  active(crafting,        │
                  │   crafted_count=1)       │
                  └──────────┬───────────────┘
                             │
                             │ each subsequent crafted survey: crafted_count++
                             │ each consumed survey: consumed_count++
                             │
                             │ when consumed_count >= crafted_count
                             │ AND no pending_loot uses remain
                             ▼
                  ┌──────────────────────────┐
                  │   ended (auto, final)    │
                  └──────────────────────────┘
```

A session header is allocated at start. Each survey use writes a row with `session_id` set to the active session. End-of-session writes `ended_at` and final aggregates.

The crafting-triggered session counts open `pending_loot` uses too — auto-end shouldn't fire while a multihit window is still open from a use within the session.

## Tauri commands (frontend API)

Read-only-from-frontend pattern. All commands are async, return JSON-serializable shapes.

```
survey_tracker_status() -> {
    active_session: Option<SessionSummary>,
    open_multihit_nodes: Vec<MultihitSummary>,
}

survey_tracker_start_session() -> SessionId    # starts a manual session, errors if one is active
survey_tracker_end_session(id) -> ()           # closes the session, computes final stats
survey_tracker_active_session() -> Option<Session>
survey_tracker_recent_sessions(limit) -> Vec<Session>
survey_tracker_session_detail(id) -> {
    session: Session,
    uses: Vec<SurveyUse>,
    loot_summary: HashMap<item_name, total_qty>,  # derived from item_transactions
}
```

Live updates emitted from the aggregator via existing pattern:

- `survey-session-started`, `survey-session-ended`, `survey-session-updated`
- `survey-use-recorded`, `survey-use-completed`
- `multihit-node-opened`, `multihit-node-closed`

Frontend stores subscribe to these and refresh affected views.

## Frontend rewrite

Today the survey screen has component-local state (timers, partial tallies, etc.) — this is what causes loss-on-navigation. Rewrite:

- All state comes from `survey_tracker_status()` polled / pushed via events
- "Start Session" / "End Session" buttons issue commands; UI updates via emitted events
- Loot tally pulled from `survey_tracker_session_detail(id)` — a single backend query
- Component unmount loses nothing; remount calls `status()` and rehydrates

## Validation strategy ✅ IMPLEMENTED

Three paired datasets with hand-recorded ground truth (`results.txt`):

- `test_data/surveyLogs/100x-serbcrystal-withring/` — 100 Basic surveys
- `test_data/surveyLogs/100x-eltmetal-ringandpick/` — 100 Basic surveys (Eltibule)
- `test_data/surveyLogs/50x-povusmarvelous-ringandpick/` — 50 Multihit surveys (Povus)

Tests live in `src-tauri/src/survey/replay_tests.rs`. Each loads the full CDN `items.json` snapshot, replays the paired logs through the production pipeline (PlayerEventParser → SurveySessionAggregator → in-memory SQLite with full migration history), and validates:

1. **Per-kind use counts** — the 50x-povus dataset must produce ≈50 Multihit-kind uses; the 100x-* datasets must produce ≈100 Basic-kind uses. Catches misclassification regressions.
2. **Pending-loot bound** — most uses must reach a terminal status (completed/aborted), not languish in pending_loot.
3. **Loot summary query produces non-trivial totals** — proves the json_extract join from `item_transactions` to `survey_uses` works end-to-end.

The tests are gated behind `#[ignore]` because they're slow (full CDN load + 30k+ player.log lines per dataset). Run with `cargo test --lib survey::replay_tests -- --ignored`.

**Current results** (six datasets, `npm run survey-test`):

| Dataset | Uses | Kind | Qty Accuracy | Items exact | Items ±2 | Items >2 off |
|---|---|---|---|---|---|---|
| 50x-povus-marvelous (multihit) | 50 | 50 Multihit | 99.4% | 9/12 | 3/12 | 0/12 |
| 100x-eltmetal (basic, ring+pick) | 100 | 100 Basic | **100.0%** | 17/17 | 0/17 | 0/17 |
| 100x-serbcrystal (basic, with ring) | 100 | 100 Basic | **100.0%** | 17/17 | 0/17 | 0/17 |
| 100x-serbcrystal (basic, vanilla) | 100 | 100 Basic | **100.0%** | 15/15 | 0/15 | 0/15 |
| 100x-eltblue (basic, ring+pick) | 100 | 100 Basic | **100.0%** | 19/19 | 0/19 | 0/19 |
| 100x-eltcrystal (basic, vanilla) | 100 | 100 Basic | **100.0%** | 24/24 | 0/24 | 0/24 |
| **Overall** | | | **99.9%** | **101/104** | **3/104** | **0/104** |

The 3 remaining ±1 misses are all in the multihit (Povus) dataset — see the per-item table below. Basic survey accuracy is 100% across all 5 datasets (92 items).

Two ground-truth errors in `results.txt` were corrected:
- `100x-serbcrystal-vanilla`: Fluorite was 48 → corrected to 13 (only 13 `[Status]` entries in the chat log; 48 was likely a transcription error from Bloodstone which is also 48).
- `100x-eltblue-ringandpick`: Sardonyx was 3 → corrected to 7 (player counted speed-bonus occurrences but missed the `x2` multipliers on 3 of 4 entries).

The 50x-povus dataset's `results.txt` is special: the user hand-isolated only loot that came from survey nodes during data capture, so it's authoritative ground truth for what survey-loot attribution should produce. The pipeline produces:

| Item (results.txt) | Expected | Pipeline | Match |
|---|---:|---:|:---:|
| Marvelous Metal Slab | 177 | 177 | ✓ |
| Pebbles | 84 | 84 | ✓ |
| Orichalcum | 47 | 47 | ✓ |
| Paladium | 36 | 36 | ✓ |
| Molybdenum | 40 | 40 | ✓ |
| Extraordinary Metal Slab | 34 | 34 | ✓ |
| Superb Metal Slab | 35 | 35 | ✓ |
| Flinty Rock | 15 | 15 | ✓ |
| Perfectly Round Pebble | 4 | 4 | ✓ |
| Gold Nugget | 15 | 14 | -1 |
| Expert-Quality Metal Slab | 1 | 0 | -1 |
| Amazing Metal Slab | 1 | 0 | -1 |

### Mining-vs-passive tie-breaker (the unlock)

Reaching this match rate required adding a tie-breaker to `compute_provenance` for the multi-context overlap case. The 50x-povus dataset has the player constantly killing mob guards between mining swings — every kill opens a CorpseSearch context that overlaps with the active Mining context. Without disambiguation, ~40% of mining loot landed as `Uncertain` and never chained.

**The rule**: when multiple ActivityContexts are active and `Mining` is one of them AND every other active context is `CorpseSearch | VendorBrowsing | StorageBrowsing` (i.e., passive screens that aren't themselves item-producing tasks), pick `Mining` with `Probable` confidence.

Justification: mining is an explicit active task (delay loop swinging a pick); the passive screens are just open windows. Real-world data shows ~99% of these overlapping gains belong to the active mining cycle. Confidence is `Probable` (not `Confident`) so downstream code can still distinguish "single-context attribution" from "tie-breaker outcome."

### Deferred basic-gain attribution (the second unlock)

Basic survey accuracy was initially ~33–52% across four basic-survey datasets because **the game emits primary loot before the survey map deletion**:

1. `ProcessAddItem` (primary loot) — arrives with `SurveyMapUse` provenance
2. `ProcessDeleteItem` (map consumed) — creates the pending use
3. `ProcessAddItem` (speed-bonus loot) — attributed normally

The aggregator created pending uses on `DeleteItem` (step 2), so the primary gain from step 1 found no pending use and was silently dropped. Only speed-bonus items (step 3) were captured.

**Fix**: `attribute_basic_gain` now buffers gains in `deferred_basic_gains` when no pending Basic use exists. When `handle_survey_consumed` fires and creates the pending use, it drains the buffer (counting quantity, updating timestamps) and retroactively patches `item_transactions` rows to inject the `survey_use_id` via `retroactively_tag_unlinked_survey_transactions`.

### Chat-seeded initial_quantity on ItemAdded (the third unlock)

The remaining ~5% gap came from chat-seeded multi-quantity gains (e.g., "Fluorite x3 added to inventory"). When chat seeding worked correctly, the parser set the stack to 3 and the server's `ProcessUpdateItemCode` saw the stack was already correct — no `ItemStackChanged` event fired. But `ItemAdded` hardcoded qty=1 in both the test harness and `GameStateManager`, so the extra 2 were silently lost.

**Fix**: Added `initial_quantity: u32` field to `PlayerEvent::ItemAdded`. The parser populates it from the chat-seeded value (or 1 as fallback). `GameStateManager::process_events_batch` and `persist_for_test` now use `initial_quantity` instead of hardcoding 1. This brought basic-survey accuracy from ~95% to **100.0%** across all 5 basic datasets.


## Open questions to resolve during implementation

- **Map consumption detection**: today we infer survey-map deletion via `ItemDeleted` + survey_types lookup. With `survey_kind()` available, the aggregator can do this directly. Cleaner: extend `DeleteContext` taxonomy with `Consumed { reason: SurveyUse }`? Or just do the lookup in the aggregator? Current preference: lookup in aggregator, no parser change.
- **Crafted-count edge case**: what if the player crafts 50 maps, uses 30, then closes the app for a week? The session stays open forever. Solution: idle timeout (e.g., 6 hours since last activity in this session) auto-ends a crafting session as `ended (idle)`.
- **Cross-session multihit**: a multihit window can outlive a session if the player opens a new session between hits. Decide: does the loot attribute to the session that was active when the *first hit* happened, or to whichever session is active *now*? Current preference: the first-hit session — once attributed, the link doesn't change.
- **Area dropdown in survey_uses**: do we want to record the `area` token from the map name, the live tracked area, or both? Live tracked area is canonical (covers the user using a Gazluk map while standing in Gazluk vs Ilmari). Plan: store live area, internal-name area is fallback only if live unknown.

## Implementation order (working sequence)

1. Schema migration v26 (tables + drop old)
2. Move/delete `KnownSurveyType` & related types into the new module structure (everything used by the parser today)
3. Build `multihit_state.rs` (DB-backed) — easy first because it's pure CRUD
4. Build `persistence.rs` — session/use insert + update helpers
5. Build `aggregator.rs` — the meat: subscribe to events, maintain in-memory short-window state, write to DB
6. Build `session.rs` — lifecycle state machine
7. Wire into coordinator (replace the existing survey_parser/survey_persistence references)
8. Tauri commands
9. Frontend rewrite (separate work-stream once backend is solid)
10. Integration tests + validation
11. Delete `survey_parser.rs` and `survey_persistence.rs`
12. Docs

Steps 1–8 are backend-only and can land before 9 starts. Frontend rewrite needs the commands to exist but doesn't block the backend correctness work.

## Phase 5 status ✅ COMPLETE

All three survey tabs now run on the new ledger. Legacy code has been removed.

### Final state

- **Session tab** → `SurveyTrackerView.vue` consuming `useSurveyTrackerStore` over `survey_tracker_*` commands. Backend-owned state, full multihit support, attribution wired through.
- **Historical tab** → `HistoricalTab.vue` on `survey_tracker_historical_sessions(limit)`. Expandable per-session summary with notes editor and delete action (`survey_tracker_update_session_notes`, `survey_tracker_delete_session`).
- **Analytics tab** → `AnalyticsTab.vue` on `survey_tracker_analytics()`. Three sub-views (Zones, Survey Types, Items) derived from `item_transactions` joined to `survey_uses` via `source_details->>'survey_use_id'`.

All three tabs are **read-only views over the new ledger** — `item_transactions` + `survey_sessions` + `survey_uses`. Every gain attributed to a survey use carries the link via `source_details->>'survey_use_id'`.

### Legacy removal (landed together)

- Deleted: `src-tauri/src/survey_parser.rs`, `src-tauri/src/survey_persistence.rs`, `db/player_commands_survey_events.rs`, `db/survey_sharing_commands.rs`.
- Deleted: `src/stores/surveyStore.ts` and 9 legacy components (SessionTab, SessionCard, SessionSidebar, SurveyImportManager, SurveyLog, SurveyLootGrid, SurveyTypeAccordion, SurveyView, and the entire `Analytics/` subdirectory).
- Coordinator no longer dispatches `LogEvent::SurveyParsed`; `PlayerLogWatcher` no longer takes a `known_surveys` parameter.
- `db/player_commands.rs` trimmed of all legacy survey-session-stats commands.
- Migration **v27** drops `survey_loot_items`, `survey_events`, `survey_session_stats`, `survey_imports`.
- `survey_types` (CDN reference table) is retained — still useful raw material, still wiped and reloaded on each CDN update.


