# Item Provenance Overhaul

## Motivation

Today we parse inventory gains two ways:

1. `ProcessAddItem` / `ProcessUpdateItemCode` / `ProcessDeleteItem` from Player.log — precise instance identity, but `AddItem` has no quantity (always reports "1"), and we can only know *that* an item entered, not *why*.
2. `ProcessScreenText(ImportantInfo, "X collected! Also found Y…")` for surveys — fragile text scraping that breaks when the game changes the format.

Downstream features (survey tracker, future mining stats, kill-loot stats, crafting yield analysis) need to answer the question **"where did this item come from?"** and today the answer is often wrong, imprecise, or missing.

The chat log's `[Status] X added to inventory` messages give us authoritative quantities for most gain events, and the player.log's interaction/delay-loop state tells us what the player was *doing* at the moment of the gain. Correlating the two gives us **provenance** — the ability to attribute every item gain to its source context.

## Evidence from paired test logs

A dry-run correlation analyzer (`scripts/correlation_report.cjs`) was run across five paired chat/player.log datasets. Findings:

- With a real display-name → internal-name resolver (CDN `items.json`), chat ↔ player.log match rates are **89–99%** across all tested datasets.
- Chat `[Status]` tallies match the player's hand-recorded survey results (`results.txt`) exactly for most items, off by small counts only for the most-frequent items (likely hand-count imprecision).
- Remaining mismatches all fit one pattern: **`ProcessAddItem` was seeded to stack_size=1 but the real initial stack was N** (vendor multi-buy, stack split, first pickup of N).

The current `ProcessScreenText` survey-loot parser is the weakest link — it misses edge cases, breaks on format drift, and is a separate code path from the general inventory pipeline. Unifying around chat `[Status]` events eliminates this entirely.

## Goals

1. **Attach a source context** to every item gain event so downstream features can filter/aggregate by source.
2. **Use chat `[Status]` as the authoritative quantity source**, with player.log as the structural source (instance IDs, identity, deletion context).
3. **Retire ProcessScreenText-based survey loot parsing** in favor of correlating `[Status]` events to active survey contexts.
4. **Be honest about uncertainty** — prefer `UnknownSource` entries over confidently-wrong attributions.
5. **Keep zero data loss** — the existing `instance_registry`, `stack_sizes`, and `pending_deletes` state machinery keeps working.

## Non-goals

- Absolute inventory baselines on login (still requires JSON export; see `memory/project_inventory_baseline.md`).
- Back-filling historical transactions with provenance (only applies going forward).
- 100% attribution coverage — there will always be a long tail of ambiguous or context-less gains, and that's fine.

## Observations that shape the design

### The game's log emitter is inconsistent

- `ProcessStartInteraction` sometimes includes a helpful name like `"MiningNodeFromSurvey9"` and sometimes just `""`. Motherlode nodes in particular are frequently nameless.
- `ProcessFirstEverInteraction` fires unpredictably — sometimes multiple times in a session, sometimes never, with no clear trigger. **Ignore entirely.** It is not a reliable signal and any feature that depends on it will break.
- `"Mining..."` vs `"Mining ..."` (trailing space variation) show up across different log ages; both refer to the same action.
- `TalkScreen` bodies for corpses sometimes include "skinned … obtained X" phrases (results of skinning/autopsy only), but **not** the normal loot drops from the corpse.

**Consequence**: we must lean on *reliable* signals (delay-loop action_type and label) and treat *nice-to-have* signals (interaction names, first-ever-interaction metadata) as pure enrichment. No feature should depend on the nice-to-haves being present.

### Rapid / overlapping actions are real

In the 50x-povusmarvelous-ringandpick session, the player repeatedly mined nodes while being attacked. Same-second sequences like `AddItem + EndInteraction(old) + StartInteraction(new) + DoDelayLoop(new)` are common. Corpse searches can start before a mining context explicitly closes.

**Consequence**: contexts must be tracked as a *stack* with timeouts, not an exclusive slot. Gain attribution must handle ambiguity explicitly rather than arbitrarily picking a winner.

### Provenance is more than "survey vs. mining"

We have known gain sources beyond the obvious:

- Survey map use (as primary or "speed bonus" secondary loot)
- Mining from a node (named motherlode node, or nameless world node)
- Corpse loot (from a kill's search corpse interaction)
- Corpse skinning / autopsy / butchering results
- Vendor purchase
- Storage withdrawal
- Crafting output (cooking, brewing, distilling, lumber chopping, etc.)
- Quest / NPC reward
- Ground pickup
- **Unknown** — and there will be more of these than we'd like

## Proposed architecture

### Concept: Activity Context Stack

A new parser subsystem maintains a **stack of active player activities** reconstructed from the player.log line stream. Each entry describes something the player is currently doing that could be a source of inventory gains.

```rust
pub enum ActivitySource {
    Mining { node_entity_id: Option<u32>, node_name: Option<String> },
    SurveyMapUse { survey_map_internal_name: Option<String> },
    SurveyMapCraft,
    GeneralCraft { recipe_label: String, action_type: String },
    CorpseSearch { entity_id: u32, corpse_name: String },
    VendorBrowsing { npc_entity_id: u32, npc_name: Option<String> },
    StorageBrowsing { vault_owner_entity_id: u32, vault_name: String },
    // Future: additional source categories as they're discovered
}

pub struct ActivityContext {
    pub source: ActivitySource,
    pub started_at: DateTime<Utc>,
    pub expected_duration_ms: Option<u32>, // from DoDelayLoop
    pub close_deadline: DateTime<Utc>,     // started_at + duration + slack
    pub entity_id: Option<u32>,            // for matching EndInteraction
}
```

The parser keeps `Vec<ActivityContext>` as state. New contexts are pushed on relevant signals; contexts are closed on explicit end events or on timeout-expiry during line processing.

### Signals that open/close contexts

| Signal | Action |
|---|---|
| `DoDelayLoop(duration, ChopLumber, "Mining…")` | Open `Mining { node_entity_id: <from recent StartInteraction if any>, node_name: <optional> }` with duration |
| `DoDelayLoop(duration, Unset, "Using <X> Survey")` | Open `SurveyMapUse { survey_map_internal_name: derived from label }` |
| `DoDelayLoop(5, UseTeleportationCircle, "Surveying")` | Open `SurveyMapCraft` |
| `DoDelayLoop(duration, <other>, <label>)` | Open `GeneralCraft { recipe_label, action_type }` |
| `TalkScreen(id, "Search Corpse of X", ..., Corpse)` | Open `CorpseSearch { entity_id: id, corpse_name: "X" }` |
| `VendorScreen(npc_id, ...)` | Open `VendorBrowsing { npc_entity_id: npc_id, npc_name: <from recent NPC_ interaction> }` |
| `ShowStorageVault(npc_id, _, name, ...)` | Open `StorageBrowsing { vault_owner_entity_id: npc_id, vault_name: name }` |
| `EndInteraction(id)` | Close any context whose `entity_id == id` |
| Line timestamp exceeds `ctx.close_deadline` | Close that context (timeout) |
| `Error: Usage Aborted` in chat log | Close most recent craft-type context (covered separately, see below) |
| Area change (login, zone change) | Close all contexts |

### Signals treated as enrichment only (never required)

- `ProcessStartInteraction` name — when non-empty, attach to currently-open Mining / other ambiguous context
- `TalkScreen` body text containing "skinned"/"obtained" — attach as hint to `CorpseSearch` context (for skinning/autopsy/butchering results only, not normal kill loot)
- Chat `[Error] Usage Aborted` — closes open survey/craft contexts before timeout

### Gain event attribution

When an item gain event fires (either chat `[Status] ItemGained` or player.log `ItemAdded`/`ItemStackChanged` / `RemoveFromStorageVault`), assign a `provenance`:

```rust
pub enum ItemProvenance {
    Attributed {
        source: ActivitySource,
        confidence: AttributionConfidence,
    },
    Uncertain {
        candidates: Vec<ActivitySource>,
    },
    UnknownSource,
}

pub enum AttributionConfidence {
    Confident,   // exactly one matching context, item type consistent
    Probable,    // one context, item type unconfirmed against loot table
    Weak,        // multiple contexts, best guess
}
```

Attribution algorithm:
1. Collect all active contexts at the gain's timestamp (after closing any that have expired).
2. If **zero** contexts active → `UnknownSource`.
3. If **one** context active → `Attributed { source: that, confidence: Confident }`.
4. If **multiple** contexts active:
   - If exactly one has an item-type rule that matches the gained item (e.g., `Mining` context and item is in mining loot table) → `Attributed { confidence: Confident }`.
   - Otherwise, if one context was *very recently* started (within 1s) relative to the gain → `Attributed { confidence: Probable }`.
   - Otherwise → `Uncertain { candidates }`.

Item-type rules can be loose initially ("is this item a craft output? a loot item?") and tightened over time. Mostly the logic should fall through to `Uncertain` cleanly rather than guessing.

### Chat-primary quantity seeding

Separately from provenance, we adopt chat `[Status]` as the authoritative quantity source when available:

- Chat `ItemGained` events are held in a short-lived buffer (~2 second window) keyed by display name.
- On `ProcessAddItem(item, -1, True)` (genuine new stack), the parser consults the buffer for a matching chat `ItemGained` within the time window:
  - If found: seed stack to the chat qty, consume that buffer entry.
  - If not found: seed to 1 as today.
- On storage withdrawals (`slot >= 0`): keep using `RemoveFromStorageVault` quantity (already correct).
- On ambiguous matches (multiple candidate AddItems within the window, multiple chat entries): greedy assign, record `UnknownSource` or `Uncertain` if we can't resolve.

Chat-seeded stacks are marked so their initial quantity is trusted; the first subsequent `UpdateItemCode` on the same instance produces a correct delta instead of the current off-by-N artifacts. The chat-seeded quantity is also carried on the `ItemAdded` event as `initial_quantity` so downstream consumers (`GameStateManager`, test harnesses) record the correct gain quantity instead of hardcoding 1.

### Downstream changes

**`item_transactions` table** (v17 migration, follows current schema):
- Add `source_kind TEXT` column (e.g., `"mining"`, `"survey_map_use"`, `"corpse_search"`, `"vendor_purchase"`, `"storage_withdrawal"`, `"craft_output"`, `"unknown"`, `"uncertain"`).
- Add `source_details TEXT` column (JSON blob — node name, NPC name, recipe label, candidate list for uncertain, etc.).
- Add `confidence TEXT` column (`"confident"`, `"probable"`, `"weak"`, `"n/a"`).

**Survey tracker** (`src-tauri/src/survey_parser.rs`):
- Rewrite to subscribe to enriched `ItemGained` events where `provenance.source == SurveyMapUse`.
- Group gains by survey invocation (one `SurveyMapUse` context = one survey).
- Remove the ProcessScreenText scraping path entirely once the new pipeline is validated.

**Coordinator** (`src-tauri/src/coordinator.rs`):
- Chat `ItemGained` events flow into the parser's chat-quantity buffer.
- Enriched gain events (with provenance) replace the current raw event stream for downstream consumers.

**Frontend**:
- `liveEventLog` / transaction views expose `source_kind` so users can filter "show me only mining gains" etc.

## Phased rollout

Each phase is independently testable and shippable. None of them require the next to be useful.

### Phase 1 — Activity Context stack (parser-internal, no consumer changes) ✅ COMPLETE

Build the `ActivityContext` stack in `PlayerEventParser`. Wire up context open/close on the signals listed above. Add tests using the paired test logs to verify:
- Correct context is active at known timestamps
- Contexts close on explicit end and on timeout
- Rapid-action scenarios (mining interrupted by combat, then corpse search) produce correct stacks
- No consumer code is changed yet — we're just building parser state

**Implemented:** `ActivitySource` enum, `ActivityContext` struct, activity stack with push/expire/close helpers, dispatch handlers for `ProcessTalkScreen`/`ProcessVendorScreen`/`ProcessShowStorageVault`, `classify_delay_loop` mapping labels to sources. 14 new tests covering named/unnamed mining nodes, label variants, survey map use/craft, corpse search, vendor/storage browsing, overlapping contexts, timeouts, and explicit/non-matching end interactions. See [docs/architecture/player-event-parser.md](../architecture/player-event-parser.md#activity-context-stack).

### Phase 2 — Attach provenance to gain events ✅ COMPLETE

Add `ItemProvenance` to `PlayerEvent::ItemAdded`, `ItemStackChanged`, `StorageWithdrawal`. Populate from the context stack. Emit as part of the existing event payload. Downstream code ignores it for now.

**Implemented:** `ItemProvenance` enum (`Attributed`, `Uncertain`, `UnknownSource`, `NotApplicable`) and `AttributionConfidence` enum in the Rust parser. `compute_provenance()` helper on `PlayerEventParser`. Field added to all three gain events. Storage withdrawals attribute to matching `StorageBrowsing` context preferentially. Session-load `AddItem` and non-positive stack deltas map to `NotApplicable`. TS types in `src/types/playerEvents.ts` mirror the Rust shape exactly. 9 new tests covering unknown/confident/uncertain/not-applicable cases, stack-change positive delta, storage withdrawal attribution, and post-timeout behavior. Frontend `vue-tsc --noEmit` build passes. See [docs/architecture/player-event-parser.md#item-events](../architecture/player-event-parser.md#item-events).

### Phase 3 — Chat-primary quantity seeding ✅ COMPLETE

Add chat quantity buffer to parser. Modify `parse_add_item` to consult the buffer. Update tests. Retain the existing `correct_stack_from_chat` reactive correction as a fallback safety net for cases where chat arrives outside the window.

**Implemented:** `PendingChatGain` buffer on `PlayerEventParser`, `feed_chat_gain()` public method, `consume_chat_gain_for_add_item()` helper with ±2s match window and 10s buffer lifetime. `parse_add_item` now seeds from chat quantity when a match is found, falls back to 1 otherwise. `PlayerLogWatcher::feed_chat_gain()` exposes the buffer to the coordinator. Coordinator resolves chat display names → CDN internal names before pushing into the buffer (chat says "Rubywall Crystal" but player.log uses "RedCrystal" — resolution happens where CDN access exists). 7 new tests: seed-from-chat, fallback-to-1, out-of-window, consume-on-first-match, multiple-buffered, cross-item-no-match, storage-withdrawal-no-consume. The existing reactive `correct_stack_from_chat` remains as a safety net.

### Phase 4 — Transaction ledger enrichment ✅ COMPLETE

Add `source_kind`, `source_details`, `confidence` columns via v25 migration. Populate from `ItemProvenance` on every transaction insert. Update existing tests to assert new columns.

**Implemented:** Migration **v25** (not v17 — see `db/migrations.rs` for the actual sequence; v17 was already taken when this plan was written) adds three nullable columns to `item_transactions`: `source_kind`, `source_details` (JSON), `confidence`, plus an index on `source_kind`. `ItemProvenance::to_columns()` projects into the three values with a stable taxonomy (`"mining"`, `"survey_map_use"`, `"survey_map_craft"`, `"general_craft"`, `"corpse_search"`, `"vendor_browsing"`, `"storage_browsing"`, `"uncertain"`, `"unknown"`, `"not_applicable"`). `GameStateManager::record_transaction` takes an optional `ProvenanceColumns` parameter. All four call sites updated: `ItemAdded` (gain, populated), `ItemDeleted` (not a gain, None), `StorageDeposit` (internal move, None), `StorageWithdrawal` (gain, populated). Chat-sourced rows in the coordinator remain `source_kind = NULL` to avoid double-counting in per-source aggregates (the correlated player_log row carries provenance). 6 new tests for `to_columns()` across all provenance variants including the unnamed-mining-node edge case.

### Phase 5 — Survey tracker rewrite ✅ COMPLETE

Reimplemented as a complete nuke-and-pave. New `src-tauri/src/survey/` module (aggregator + persistence + multihit_state + commands + replay_tests) subscribes to the provenance-enriched pipeline. Legacy `survey_parser.rs` / `survey_persistence.rs` and their frontend have been removed. Migration v27 drops `survey_loot_items` / `survey_events` / `survey_session_stats` / `survey_imports`.

All three frontend tabs (Session, Historical, Analytics) run on the new ledger as read-only views over `item_transactions` joined to `survey_uses` via `source_details->>'survey_use_id'`.

**See the dedicated implementation plan: [survey-tracker-rewrite.md](survey-tracker-rewrite.md).**

**Surveys are three behaviorally distinct kinds** — see [survey-mechanics.md](../architecture/survey-mechanics.md). Basic (single-tick, speed-bonus eligible), Motherlode (one mining cycle), Multihit (many hits on one node, closes on different-entity mining or 30-min timeout). Motherlode/Multihit attribution chains a `SurveyMapUse` context to subsequent `Mining` contexts via `survey_use_id` in `source_details`.

Speed-bonus tracking deferred — `parse_loot_items` in `parsers.rs` is retained (dead-code-gated) for reuse when that lands.

### Phase 6 — New downstream features (time permitting)

Now that provenance is in the transaction ledger, new features become trivial:
- Mining node yield stats per node type
- Vendor purchase history with total spend
- Kill loot breakdown by mob type
- Crafting yield analysis per recipe
- "Unknown source" and "Uncertain source" diagnostic reports — useful for discovering new signal patterns we haven't accounted for yet

## Uncertainty and unknown sources

The `UnknownSource` and `Uncertain` buckets will be larger than we'd like at first. This is fine and expected — **honest "I don't know" data is far more useful than confidently-wrong labels**. Over time:

- We'll gather more paired test logs from players to expose gain patterns we haven't seen yet.
- Each time we discover a reliable signal for a currently-unknown source, we add it to the context detection rules.
- The `source_details` JSON preserves raw context candidates even for `Uncertain` entries, so future logic can re-classify historical rows.

## Risks and open questions

- **How much does the `Unknown` bucket actually contain after Phase 2 is live?** — we expect some surprises; the bucket will tell us what's missing.
- **Does chat quantity seeding break any existing tests?** — the parser tests that manually seed `stack_sizes` and the survey tracker tests will both need updates. Already caught some of these during previous work.
- **Multi-player loot (grouped kills)** — if another player is in the group, do we see chat `[Status]` for their loot? Needs paired test data with group play to validate.
- **Compound recipes producing multiple outputs at once** — crafting that outputs several items simultaneously; does each fire its own chat status? Needs validation.
- **Stall / other shop transactions** — not currently in scope; should they get a source_kind? TBD.

## Reference: investigation artifacts

- `scripts/correlation_report.cjs` — paired-log correlation analyzer; run with `--all` for aggregate summary, single-dataset mode for detailed unmatched-entry lists.
- `scripts/analyze_chat_correlation.cjs` — earlier, simpler variant; kept for reference.
- `test_data/inventory/` — controlled 17-action session with known ground truth in `sample-overview.txt`.
- `test_data/surveyLogs/*/` — three survey sessions (100x-serbcrystal, 100x-eltmetal, 50x-povusmarvelous) each with `results.txt` hand-recorded totals.
- `docs/samples/chat+logCombos/Gazluk-Motherlodes-and-Nodes.log` — mixed mining/combat/survey session, useful for rapid-action and unnamed-interaction cases.

## What this does *not* change

- The `instance_registry`, `stack_sizes`, `pending_deletes` machinery in the parser — these continue to work exactly as today. Provenance is layered on top, not a replacement.
- JSON import of inventory exports — still the only way to establish accurate absolute baselines.
- The `DeleteContext` taxonomy (`StorageTransfer`, `VendorSale`, `Consumed`, `Unknown`) on `ItemDeleted` — this is about exits from inventory, not entries. Stays as-is. (Could be unified conceptually in the future but isn't blocking.)
