# Unified Event Stream: Player.log + Chat.log

## Problem

The app currently has two independent log-watching pipelines:

1. **Player.log** → `PlayerLogWatcher` → `PlayerEventParser` → structured `PlayerEvent`s → features (survey parser, game state, etc.)
2. **Chat.log** → `ChatLogWatcher` → `ChatMessage`s → chat feature only (DB insert + frontend display)

These streams contain **complementary information** about the same game actions, but features can only subscribe to one. The chat log's `[Status]` channel carries data that Player.log doesn't:

| Status Pattern | What it tells us | Player.log gap it fills |
|---|---|---|
| `X xN added to inventory` | **Exact item quantity** for new stacks | `ProcessAddItem` has no quantity field |
| `X collected! Also found Y (speed bonus!)` | **Survey loot with quantities** | `ProcessScreenText("collected!")` has this, but motherlode mining doesn't |
| `You earned N XP in Skill` | **Per-action XP gain** | `ProcessUpdateSkill` gives cumulative XP, not per-action delta |
| `You earned N XP and reached level L in Skill!` | **Level-up events** | No equivalent in Player.log |
| `You searched the corpse and found N coins` | **Coin drops per corpse** | No equivalent |
| `You received N Councils` | **Council gains** (vendor, quest) | No equivalent |
| `You used N councils` | **Council spending** | No equivalent |
| `Stowed N items across N storages` | **Bulk storage confirmation** | Individual vault events exist but no summary |
| `The treasure is N meters from here` | **Survey triangulation distances** | Not in Player.log |
| `X was distilled into Y xN` | **Transmutation results** | Craftable from item events but tedious |
| `X was decomposed into Y xN and Z xN` | **Decomposition results** | Same |
| `Your friend X is now online/offline` | **Friend presence** | Not in Player.log |
| `X joined/left the hunting party` | **Group composition** | Not in Player.log |
| `Your actions have caused you to gain Favor with NPC!` | **Favor gain events** | Not in Player.log |
| `Ratkin Mugger stole N Councils!` | **Council losses** | Not in Player.log |
| `You botch the autopsy!` / `You bury the corpse.` | **Anatomy outcomes** | Not in Player.log |
| `You lost N.N favor with NPC` | **Favor loss** | Not in Player.log |
| World event announcements (invasions, raids) | **Server events** | Not in Player.log |
| `Summoned X xN` | **Crafting ingredient summoning output** | Partially from item events |

### Concrete example: Gypsum x9

A motherlode mining drop produces:
- **Player.log**: `ProcessAddItem(Gypsum(159455350), -1, True)` — we record quantity 1
- **Chat.log**: `[Status] Gypsum x9 added to inventory.` — actual quantity is 9

Without the chat Status stream, we permanently lose the real quantity for new item stacks.

## Proposed Architecture

### Core Concept: `ChatStatusEvent`

A new structured event enum, parsed from `[Status]` channel messages, analogous to `PlayerEvent`:

```rust
pub enum ChatStatusEvent {
    /// "X added to inventory" / "X xN added to inventory"
    ItemGained { timestamp, item_name: String, quantity: u32 },

    /// "X collected! Also found Y xN (speed bonus!)"
    SurveyCollected { timestamp, primary_item: String, primary_qty: u32, bonus_item: Option<String>, bonus_qty: Option<u32> },

    /// "You earned N XP in Skill."
    XpGained { timestamp, skill: String, amount: u32 },

    /// "You earned N XP and reached level L in Skill!"
    LevelUp { timestamp, skill: String, level: u32, xp: u32 },

    /// "You searched the corpse and found N coins."
    CoinsLooted { timestamp, amount: u32 },

    /// "You received N Councils." / "You used N councils."
    CouncilsChanged { timestamp, amount: i64 },

    /// "The treasure is N meters from here."
    TreasureDistance { timestamp, meters: u32 },

    /// "X was distilled into Y xN"
    Distilled { timestamp, input: String, output: String, quantity: u32 },

    /// "X was decomposed into Y xN and Z xN"
    Decomposed { timestamp, input: String, outputs: Vec<(String, u32)> },

    /// "You bury the corpse." / "You botch the autopsy!"
    AnatomyResult { timestamp, success: bool },

    /// "Your friend X is now online/offline"
    FriendPresence { timestamp, name: String, online: bool },

    /// "X joined/left the hunting party"
    PartyChange { timestamp, name: String, joined: bool },

    /// "Your actions have caused you to gain Favor with NPC!"
    FavorGained { timestamp, npc: String },

    /// "You lost N.N favor with NPC"
    FavorLost { timestamp, npc: String, amount: f32 },

    // ... extensible
}
```

### Where it lives

**New file: `src-tauri/src/chat_status_parser.rs`**

- Receives `ChatMessage` where `channel == "Status"`
- Parses message text into `ChatStatusEvent` variants via regex/string matching
- Stateless — each message → 0 or 1 events. Accumulation/correlation is left to the subscribing features.

### How features subscribe

The coordinator already processes both log streams. The change is:

1. `ChatLogWatcher` parses **all** channels (no exclusions at the watcher level)
2. Coordinator runs Status messages through `ChatStatusParser`
3. Features that need chat status data receive `ChatStatusEvent`s alongside `PlayerEvent`s

Two options for delivery:

**Option A: Parallel streams** — features subscribe to both `PlayerEvent` and `ChatStatusEvent` independently. Survey parser gets both, uses `ItemGained` to supplement `ItemAdded` quantities.

**Option B: Merged stream** — coordinator merges both into a single `GameEvent` enum. Features get one stream. Requires timestamp correlation.

Option A is simpler. Features that need both just receive both.

### Channel exclusion: parsing vs persistence

**Key decision:** The `excluded_channels` setting controls what the **chat log feature** persists to its database table — it does NOT gate parsing or data ingestion.

Currently `excluded_channels` is applied at parse time in `parse_chat_line()`, which means excluded channels (including Status) are dropped before they even become `LogEvent::ChatMessage`. This needs to change:

1. **`ChatLogWatcher`** parses with **no exclusions** — all channel messages flow through as `LogEvent::ChatMessage`
2. **Coordinator** becomes the routing layer — it sends Status messages to `ChatStatusParser`, sends all messages through watch rules, and only applies `excluded_channels` when calling `insert_chat_messages` for the chat log DB
3. `insert_chat_messages` already receives `excluded_channels` and filters there, so DB-level filtering is already in place — it just needs to be the **only** filter, not a redundant second one

This same pattern extends to `[Combat]` in the future — a combat parser would subscribe to Combat channel messages regardless of whether the user has Combat excluded from their chat log display.

### Timestamp strategy

Both log streams carry timestamps, but in different formats:

| Source | Format | Timezone | What's included |
|---|---|---|---|
| Player.log | `[HH:MM:SS]` | Local time | Time only — no date component |
| Chat.log | `YY-MM-DD HH:MM:SS` | UTC | Full date + time |
| Chat login line | `Timezone Offset -07:00:00` | — | Conversion factor between them |

**Decision: normalize everything to UTC internally.**

- **Player.log `[HH:MM:SS]`** — combine with current system date, then convert local → UTC using the known timezone offset
- **Chat.log timestamps** — already UTC, use as-is
- **Database storage** — all timestamps stored as UTC
- **Frontend display** — convert UTC → user's local time for display only
- **Timezone offset source** — auto-detect from system, with the chat login line's `Timezone Offset` as confirmation. Provide an advanced setting for manual override.

This is a cross-cutting prerequisite that touches DB schemas, all timestamp creation in Rust, and all timestamp display in Vue. Should be done before or in parallel with the event stream work.

#### Player.log timestamp details

The `parse_timestamp()` function in `parsers.rs` extracts `[HH:MM:SS]` as a string. Every `PlayerEvent` variant carries this as `timestamp: String`. The `GameStateManager` currently normalizes these to full datetimes by combining with `chrono::Local::now()` — this needs to switch to UTC conversion using the timezone offset.

Lines without timestamps (login announcements, chat log path, etc.) use `chrono::Local::now().naive_local()` in `log_watchers.rs` — these also need UTC conversion.

### Deduplication strategy

Some information appears in both streams:
- Item additions: `ProcessAddItem` + `[Status] X added to inventory`
- Skill XP: `ProcessUpdateSkill` + `[Status] You earned N XP in Skill`
- Survey completion: `ProcessScreenText("collected!")` + `[Status] X collected!`

Features should treat **Player.log as primary** (it has instance IDs, encoded values, entity references) and **Chat Status as supplementary** (it has quantities, human-readable names, per-action deltas). The pattern:

1. Player.log event triggers the feature logic
2. Chat Status event enriches it (e.g., corrects quantity from 1 → 9)

This means features don't need to deduplicate — they use different fields from each stream.

## Files to modify

1. **New: `src-tauri/src/chat_status_parser.rs`** — parser module
2. **`src-tauri/src/chat_parser.rs`** — decouple `excluded_channels` from `parse_chat_line` (move filtering to callers that need it)
3. **`src-tauri/src/log_watchers.rs`** — `ChatLogWatcher` parses all channels; stop passing excluded_channels to parse functions
4. **`src-tauri/src/coordinator.rs`** — route Status messages through ChatStatusParser; apply excluded_channels only at DB insert time
5. **`src-tauri/src/parsers.rs`** — update `parse_timestamp` to return proper UTC-aware types
6. **`src-tauri/src/game_state.rs`** — switch timestamp normalization from local to UTC
7. **`src-tauri/src/lib.rs`** — register new module
8. **Feature files** — opt-in to ChatStatusEvent as needed
9. **Frontend timestamp display** — UTC → local conversion helpers

## Implementation phases

### Phase 0: Timestamp normalization (prerequisite) — COMPLETE
- ~~Switch all internal timestamps to UTC~~
- ~~Store UTC in database~~
- ~~Add frontend helpers to convert UTC → local for display~~ (`useTimestamp.ts`)
- ~~Parse and store timezone offset from chat login line~~
- ~~Add advanced setting for manual timezone override~~

### Phase 1: Channel exclusion refactor + parser — COMPLETE
- ~~Decouple `excluded_channels` from parsing — move to persistence layer only~~
- ~~Build `ChatStatusParser`~~ with 8 event variants:
  - ~~`ItemGained`~~ (fixes motherlode quantity problem)
  - ~~`XpGained` / `LevelUp`~~
  - ~~`CoinsLooted` / `CouncilsChanged`~~
  - ~~`TreasureDistance`~~
  - ~~`AnatomyResult` / `Summoned`~~
- ~~Route ChatStatusEvents through coordinator → emit `"chat-status-event"` to frontend~~

See [`live-event-streams.md`](../architecture/live-event-streams.md) for the unified architecture reference.

### Phase 2: Feature adoption (next)
- Skill tracking uses per-action XP
- Economy tracking uses council events
- Survey tracker uses treasure distances for triangulation display
- Anatomy/transmutation/decomposition tracking
- Future: `[Combat]` parser for DPS/healing/kill tracking (separate parser, same subscription pattern)

## Resolved questions

1. **Should ChatStatusParser be stateful?** No — each Status message is self-contained. Features that subscribe handle their own accumulation and correlation.
2. **Should we parse [Combat] too?** Not in this workstream. A future combat log feature would follow the same pattern — a `CombatParser` subscribing to Combat channel messages via the coordinator.
3. **Should excluded_channels affect ChatStatusParser?** No. `excluded_channels` controls what the chat log feature persists to its DB table. Parsing and data ingestion always see all channels. Individual features decide what they persist.
4. **How to handle the UTC/local timestamp gap?** Normalize all timestamps to UTC internally. Player.log `[HH:MM:SS]` gets combined with system date and converted using the timezone offset. Chat.log is already UTC. Store UTC in DB. Convert to local time only for frontend display. Auto-detect timezone from system, with an advanced setting for manual override.
