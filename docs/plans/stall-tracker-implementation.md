# Stall Tracker — Implementation Plan

This plan is the build guide for implementing the Stall Tracker feature on top
of the current `main` branch. The branch `PoC/ruby/stall-tracker` contains a
working version that we are **not** reusing — we are rebuilding against project
standards. The logic, data model, and UX shape below are derived from that
branch's commit history and the decisions made along the way.

See [features/screens/economics/economics-stall-tracker.md](../features/screens/economics/economics-stall-tracker.md)
for the user-facing feature documentation. This plan covers **how to build it**.

---

## Table of Contents

1. [Goals & non-goals](#1-goals--non-goals)
2. [Build order](#2-build-order)
3. [Phase 1 — Parser & data model](#3-phase-1--parser--data-model)
4. [Phase 2 — Live ingest](#4-phase-2--live-ingest)
5. [Phase 3 — Tauri commands (CRUD)](#5-phase-3--tauri-commands-crud)
6. [Phase 4 — Rust aggregations](#6-phase-4--rust-aggregations)
7. [Phase 5 — Frontend store & parent view](#7-phase-5--frontend-store--parent-view)
8. [Phase 6 — Sales tab](#8-phase-6--sales-tab)
9. [Phase 7 — Revenue tab](#9-phase-7--revenue-tab)
10. [Phase 8 — Inventory tab](#10-phase-8--inventory-tab)
11. [Phase 9 — Shop Log modal](#11-phase-9--shop-log-modal)
12. [Phase 10 — Import / Export / Clear](#12-phase-10--import--export--clear)
13. [Phase 11 — Character scoping & concurrency](#13-phase-11--character-scoping--concurrency)
14. [Phase 12 — Polish & empty states](#14-phase-12--polish--empty-states)
15. [Critical pitfalls](#15-critical-pitfalls)
16. [Test plan](#16-test-plan)

---

## 1. Goals & non-goals

**Goals:**
- Persist every parseable entry from every `PlayerShopLog` book the user opens
  in-game. Never drop data silently.
- Scale to **100k events** without UI freezes. Filtering/sorting/aggregation
  happen in Rust; the bridge only carries small, purpose-shaped payloads.
- Scope every operation to the active character. Multi-character accounts must
  never see mixed data.
- Round-trip: Export writes the exact in-game book format, so Import reads it
  back identically. Enables backups and benchmark scenarios.
- Clean empty, loading, and error states matching the project's UX standards.

**Non-goals:**
- **No chat.log cross-referencing.** Unlike Surveying, stall events are
  complete in Player.log — `ProcessBook` content carries the real data.
- No live-streaming of individual events. The book is the source of truth; the
  user has to open it for the data to flow.
- No editing of individual events. The ignore-flag is the only mutation.
- No cross-character aggregates (yet). Switch character if you want a
  different view.

## 2. Build order

The phases below are ordered so each one produces a runnable, testable
increment. **Do not skip ahead** — for example, the aggregation commands in
Phase 4 depend on `event_at` being populated, which only happens once Phase 1's
year resolver is in place.

```
Phase 1 ── Parser, year resolver, migrations                      [Rust only]
Phase 2 ── Live ingest via coordinator                            [Rust only]
Phase 3 ── List/stats/toggle/clear commands                       [Rust only]
Phase 4 ── Revenue + Inventory aggregations                       [Rust only]
Phase 5 ── Pinia store, parent view, EconomicsView wiring         [Vue]
Phase 6 ── Sales tab                                              [Vue]
Phase 7 ── Revenue tab                                            [Vue]
Phase 8 ── Inventory tab                                          [Vue]
Phase 9 ── Shop Log modal                                         [Vue]
Phase 10 ─ Import / Export / Clear wiring                         [Rust + Vue]
Phase 11 ─ Multi-character scoping, StallOpsLock                  [Rust + Vue]
Phase 12 ─ Polish pass: empty states, errors, filter resets       [Vue]
```

Each phase should end in a commit following the project's conventional commit
format: `feat(StallTracker): …`, `fix(StallTracker): …`, etc.

---

## 3. Phase 1 — Parser & data model

### 3.1 Files to create

| File | Purpose |
|---|---|
| `src-tauri/src/shop_log_parser.rs` | Regex parser, entry splitting, action dispatch. |
| `src-tauri/src/stall_year_resolver.rs` | Year inference + monotonic walk. |

### 3.2 `shop_log_parser.rs`

Six per-action regex patterns plus one splitter regex. `once_cell::Lazy` wraps
each so they compile once. The splitter matches the timestamp prefix with
multi-line mode:

```rust
static ENTRY_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?m)([A-Z][a-z]{2} [A-Z][a-z]{2} \d{1,2} \d{1,2}:\d{2}) - ").unwrap()
});
```

**Critical**: the `bought` regex uses `per (?P<quantity_unit>\d+)` to compute
`price_unit = price_unit_raw / quantity_unit`. Bulk configs like `"3000 per 2"`
must produce `price_unit = 1500.0`, not 3000.

**Critical**: ALL six action regexes (`bought`, `added`, `removed`, `visible`,
`configured`, `collected`) need to handle **optional space before the `xN`
suffix** (`"Barley Seeds x36"` *and* `"Barley Seedsx36"`). The original PoC
only handled the no-space form for `configured` and `visible`. The Phase 13
fixup added `xN` to `bought` after a real game log surfaced
`"MARCELA bought Aquamarine x5 at a cost of 750 per 1 = 3750"` —
without the extractor, every stacked sale became a phantom item
("Aquamarine x5", "Aquamarine x10", ...) that fragmented filter dropdowns
and aggregations. Use:

```
(?P<item>.+?) ?x?(?P<quantity>\d+)?
```

The trailing `?` on `?x?...` makes the quantity optional (single items show no
`xN`). The leading `?` on ` ?` allows an optional space.

For `bought`, the captured `xN` becomes the quantity directly. When `xN` is
absent, the quantity falls back to `total / corrected_unit_price`, which
preserves the bulk-pricing path. Either way the invariant
`price_unit * quantity == price_total` holds.

### 3.3 Entry splitting & indexing

```rust
pub fn parse_shop_log(
    title: &str,
    content: &str,
    log_timestamp: &str,
    base_year: i32,
) -> ShopLog {
    let content = content.replace("\\n", "\n");  // ProcessBook escapes newlines

    let matches: Vec<_> = ENTRY_RE.find_iter(&content).collect();
    let mut raw_entries: Vec<(&str, &str)> = Vec::new();
    for (i, m) in matches.iter().enumerate() {
        let timestamp = content[m.start()..m.end()].trim_end_matches(" - ").trim();
        let msg_start = m.end();
        let msg_end = matches.get(i + 1).map_or(content.len(), |n| n.start());
        let message = content[msg_start..msg_end].trim();
        if !message.is_empty() {
            raw_entries.push((timestamp, message));
        }
    }

    // ▼▼▼ CRITICAL ▼▼▼
    raw_entries.reverse();  // Oldest gets index 0 → stable across re-opens.
    // ▲▲▲ CRITICAL ▲▲▲

    let mut entries: Vec<ShopLogEntry> = raw_entries.iter().enumerate()
        .map(|(i, (ts, msg))| parse_entry(i as i64, ts, msg))
        .collect();

    let timestamps: Vec<&str> = entries.iter().map(|e| e.timestamp.as_str()).collect();
    let resolved = stall_year_resolver::resolve_timestamps_oldest_first(&timestamps, base_year);
    for (entry, event_at) in entries.iter_mut().zip(resolved) {
        entry.event_at = event_at;
    }

    // Advisory owner hint: first entry whose action is owner-only
    // (added/removed/configured/visible/collected). Used by the Import
    // command to distinguish friend-file / alt-file / bought-only cases.
    // The live-tailing path ignores this field — see §4.2.
    let owner = entries.iter()
        .find(|e| matches!(e.action.as_str(),
            "added" | "removed" | "configured" | "visible" | "collected"))
        .map(|e| e.player.clone());

    ShopLog { log_timestamp: log_timestamp.into(), title: title.into(), entries, owner }
}
```

**`ShopLog.owner` is advisory only.** The parser surfaces it as
`Option<String>` so the Import command can warn about cross-character files,
but the coordinator's live-tailing path does not use it (§4.2). Do not treat
it as a source of truth for row ownership.

**Why reverse-then-index matters:** game content arrives newest-first. If we
indexed in content order, entries would renumber every time the user re-opened
the log, and `INSERT OR IGNORE` would re-insert them as duplicates (the unique
key includes `entry_index`).

### 3.4 `stall_year_resolver.rs`

Two public functions:

```rust
pub fn resolve_timestamps_oldest_first(timestamps: &[&str], base_year: i32) -> Vec<Option<String>>;
pub fn base_year_for_live(oldest_ts: &str) -> i32;
```

`resolve_timestamps_oldest_first` walks forward, incrementing `year` when it
detects a backward month jump (Dec → Jan). `base_year_for_live` peeks at the
oldest entry and subtracts 1 from `now.year()` if the oldest entry's
`(month, day)` is in the *future* — meaning the book is still from last year.

(An earlier version of this plan included a `backfill_year` helper for a
hypothetical schema migration that adds `event_at` to rows pre-dating the
column. That migration never happened and the helper sat unused, so it was
deleted in commit `d2b6599`. Re-add from git history if a future schema
change actually needs it.)

Unit tests (all must pass before moving to Phase 2):

- single-year book resolves to the base year
- year-boundary book (Dec entries then Jan entries) bumps year correctly
- `base_year_for_live_in_past` (now = Apr 14, oldest = Apr 13 → 2026)
- `base_year_for_live_wraps_to_previous_year` (now = Jan 3 2026, oldest = Dec 28 → 2025)

### 3.5 Migration — `stall_events` table

Add a single new migration (the next available version number in the existing
chain) that creates `stall_events` in its final shape. The schema is the
endpoint of four iterative migrations in the PoC branch, but for a
fresh-build we only need one.

```sql
CREATE TABLE stall_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_timestamp TEXT NOT NULL,
    event_at TEXT,
    log_timestamp TEXT NOT NULL,
    log_title TEXT NOT NULL,
    action TEXT NOT NULL,
    player TEXT NOT NULL,
    owner TEXT,
    item TEXT,
    quantity INTEGER NOT NULL DEFAULT 1,
    price_unit REAL,
    price_total INTEGER,
    raw_message TEXT NOT NULL,
    entry_index INTEGER NOT NULL DEFAULT 0,
    ignored INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(event_timestamp, raw_message, entry_index)
);

CREATE INDEX idx_stall_events_action       ON stall_events(action);
CREATE INDEX idx_stall_events_created      ON stall_events(created_at DESC);
CREATE INDEX idx_stall_events_timestamp    ON stall_events(event_timestamp);
CREATE INDEX idx_stall_events_event_at     ON stall_events(event_at DESC);
CREATE INDEX idx_stall_events_action_event_at ON stall_events(action, event_at DESC);
CREATE INDEX idx_stall_events_player       ON stall_events(player);
CREATE INDEX idx_stall_events_item         ON stall_events(item);
```

Schema notes — each of these is load-bearing and needs to ship in this single
migration:

- **`UNIQUE(event_timestamp, raw_message, entry_index)`** — the `entry_index`
  component is essential. Without it, two identical events in the same minute
  (two buyers of the same item at the same price) collapse into one row
  because `event_timestamp` only has minute precision.
- **`entry_index`** — populated by the parser as the position within the book,
  oldest = 0. Stable across re-opens (see §3.3).
- **`event_at`** (nullable TEXT) — real ISO 8601 `"YYYY-MM-DD HH:MM:SS"`
  populated by the year resolver at insert time. All time-range queries and
  aggregations read this column rather than `event_timestamp`. Nullable only
  because the coordinator path could theoretically fail to resolve; in
  practice it should always be set for new rows.
- **`ignored`** — soft-mute flag, excluded from aggregations.
- **`owner`** — the character this row belongs to. Every read and mutation
  scopes on this (see Phase 11).
- **Index set** — the list above covers every hot access pattern:
  `idx_action` for the default list filter, `idx_event_at` for date-range
  sorts, `idx_action_event_at` for the `bought + ORDER BY event_at` query
  driving Sales, `idx_player` for buyer filters, `idx_item` for per-item
  drilldowns.

### 3.6 Unit tests (`shop_log_parser.rs`)

Mirror the PoC's test set — they're a regression harness for fiddly regex
edge cases:

- `test_parse_bought` / `test_parse_bought_with_quantity`
- `test_parse_added` / `test_parse_added_with_quantity`
- `test_parse_removed` / `test_parse_removed_with_quantity`
- `test_parse_configured` / `test_parse_configured_with_bulk_and_restriction`
- `test_parse_visible` / `test_parse_visible_with_bulk_and_restriction`
- `test_parse_visible_with_space_before_quantity` / `test_parse_configured_with_space_before_quantity`
- `test_parse_collected`
- `test_parse_unknown` (the "paid X Councils to hire" line — must land in `unknown`)
- `test_parse_shop_log_full` (end-to-end with owner detection)
- `test_parse_shop_log_with_escaped_newlines` (ProcessBook hands us `\\n`)
- `test_duplicate_entries_get_different_indices`
- `test_entry_indices_stable_across_reopens` (two opens, same old rows, different new prepended row)

---

## 4. Phase 2 — Live ingest

### 4.1 Where the hook lives

In `src-tauri/src/coordinator.rs`, inside the existing `LogEvent::PlayerEventParsed`
match arm. Add one branch that checks `PlayerEvent::BookOpened { book_type: "PlayerShopLog", … }`:

```rust
if let PlayerEvent::BookOpened { ref timestamp, ref title, ref content, ref book_type } = player_event {
    if book_type == "PlayerShopLog" {
        // The game only lets a player open their own shop log book, so
        // the active character IS the owner. Read it from settings and
        // stamp it onto every row unconditionally — we do NOT parse the
        // owner out of the book body in the live path. `ShopLog.owner`
        // is advisory and used only by the Import command (§10).
        let Some(active_character) = self.settings_manager.active_character_name() else {
            // No character loaded yet — defer. A real Player.log event
            // implies a character is active, so this branch is mostly a
            // belt-and-suspenders guard against startup ordering.
            return;
        };

        // Parse once to discover the oldest entry (cheap — regex-only),
        // then parse again with the real base year.
        let probe = shop_log_parser::parse_shop_log(title, content, timestamp, 1970);
        let base_year = probe.entries.first()
            .map(|e| stall_year_resolver::base_year_for_live(&e.timestamp))
            .unwrap_or_else(|| chrono::Local::now().year());
        let shop_log = shop_log_parser::parse_shop_log(title, content, timestamp, base_year);

        if !shop_log.entries.is_empty() {
            let inputs: Vec<_> = shop_log.entries.iter().map(|e| StallEventInput {
                event_timestamp: e.timestamp.clone(),
                event_at: e.event_at.clone(),
                log_timestamp: shop_log.log_timestamp.clone(),
                log_title: shop_log.title.clone(),
                action: e.action.clone(),
                player: e.player.clone(),
                owner: Some(active_character.clone()),  // authoritative
                item: e.item.clone(),
                quantity: e.quantity,
                price_unit: e.price_unit,
                price_total: e.price_total,
                raw_message: e.raw_message.clone(),
                entry_index: e.entry_index,
            }).collect();

            let ops_lock = self.app_handle.state::<StallOpsLock>();
            match insert_stall_events(&self.db_pool, &ops_lock, &inputs) {
                Ok(inserted) if inserted > 0 => {
                    self.app_handle.emit("stall-events-updated", inserted).ok();
                }
                Ok(_) => {}
                Err(e) => eprintln!("[coordinator] Failed to persist stall events: {e}"),
            }
        }
    }
}
```

**Why read the owner from settings instead of the book:** the game enforces
that a player can only open their own `PlayerShopLog` book, so the active
character is the book's owner by construction. Parsing it out of the book
body was a Ruby PoC pattern (the PoC had no concept of a current user) and
adds nothing here. Reading from `SettingsManager` is one field access, has no
failure modes (a real Player.log event implies a character is active), and
matches how every other feature in glogger resolves row ownership.

The exact API for reading the active character depends on the `SettingsManager`
shape on `main` — use whatever pattern the surrounding coordinator code
already uses for character-scoped writes. The snippet above assumes an
`active_character_name() -> Option<String>` accessor; adapt if the real API
differs.

**Why the double parse:** the year resolver needs the oldest entry's timestamp
before we know the base year. Parsing is regex-only and bounded by book size,
so the cost is negligible compared to the DB write it guards.

### 4.2 `insert_stall_events` helper

Not a Tauri command — a plain function called from the coordinator. Takes
`&DbPool`, `&StallOpsLock`, `&[StallEventInput]`, returns the newly-inserted
row count. Uses `INSERT OR IGNORE`, holds the ops lock for the whole batch.

### 4.3 Verification

Run the app, open the in-game shop log, verify rows appear in the
`stall_events` table. Re-open the same log — row count should not grow (unique
key prevents re-insertion). Add a new entry in-game (buy/list something),
re-open — only the new row should be inserted.

---

## 5. Phase 3 — Tauri commands (CRUD)

### 5.1 Filter shape

```rust
#[derive(Deserialize, Default, Debug)]
pub struct StallEventsFilters {
    pub owner: Option<String>,       // frontend ALWAYS passes active character
    pub action: Option<String>,
    pub player: Option<String>,
    pub item: Option<String>,
    pub date_from: Option<String>,   // "YYYY-MM-DD"
    pub date_to: Option<String>,
    pub include_ignored: Option<bool>,
}
```

Shared helper `build_filter_where(filters, force_action)` returns
`(String, Vec<Value>)`. It starts with `" WHERE 1=1"` so callers can append
conditions without caring about leading keywords. Empty strings are normalized
to `None` via `opt_nonempty()`.

Two surprising rules encoded in the helper:

1. **Default action filter excludes `'unknown'`** — list views never show
   unparseable entries. Callers that want to see unknowns pass
   `force_action = Some("unknown")`.
2. **Default `include_ignored` is `true`** — the list views *do* show ignored
   rows (greyed out). Aggregations pass `false` to exclude them.

### 5.2 Commands

| Command | Returns | Notes |
|---|---|---|
| `get_stall_events(params)` | `StallEventsPage { rows, total_count }` | Paginated. `limit.clamp(1, 10_000)`, `offset.max(0)`. Runs a `COUNT(*)` for `total_count`. |
| `get_stall_stats(filters)` | `StallStats { total_sales, total_revenue, unique_buyers, unique_items }` | Always forces `action = 'bought'` and `include_ignored = false`. Uses `COUNT(DISTINCT …)`. |
| `toggle_stall_event_ignored(id, ignored)` | `()` | Simple UPDATE. |
| `clear_stall_events(owner)` | `usize` | Requires non-empty owner. Holds `StallOpsLock`. |

### 5.3 Sort whitelist

`resolve_sort` returns a string like `"event_at DESC, event_at DESC, entry_index DESC, id DESC"`.
Whitelist the sortable columns — never interpolate user input into SQL:

```rust
let col = match sort_by.unwrap_or("event_at") {
    "event_at" | "event_timestamp" => "event_at",
    "player" => "player",
    "item" => "item",
    "action" => "action",
    "quantity" => "quantity",
    "price_unit" => "price_unit",
    "price_total" => "price_total",
    _ => "event_at",
};
```

Always append `event_at, entry_index, id` as stable tiebreakers so sort results
are deterministic across pagination boundaries.

### 5.4 `get_stall_filter_options(owner)`

Returns `{ buyers, players, items, dates, actions }`. `buyers` comes from
`DISTINCT player WHERE action = 'bought'`. `players` comes from `DISTINCT
player WHERE action != 'unknown'` (includes the stall owner). `dates` comes
from `substr(event_at, 1, 10)` ordered newest-first.

Cache the result in the store — refetch only on character switch, clear,
import, and the debounced `stall-events-updated` event.

---

## 6. Phase 4 — Rust aggregations

### 6.1 `stall_aggregations.rs`

Pure module. No DB access. Takes iterators of events, returns summary structs.
This is what makes the 100k-scale case work — the bridge only ships
`RevenueResult` / `InventoryResult`, not raw rows.

### 6.2 Revenue aggregation

```rust
pub enum Granularity { Daily, Weekly, Monthly }

pub struct RevenueResult {
    pub periods: Vec<RevenuePeriod>,           // sorted by period key
    pub items: Vec<String>,                    // sorted alphabetically
    pub cells: Vec<RevenueCell>,               // one per (item, period_key) non-zero
    pub row_totals: Vec<(String, i64)>,        // per item
    pub col_totals: Vec<(String, i64)>,        // per period_key
    pub grand_total: i64,
}

pub fn aggregate_revenue(
    events: impl IntoIterator<Item = RevenueEvent>,
    granularity: Granularity,
) -> RevenueResult;
```

Period key format:

- Daily → `"YYYY-MM-DD"`, label `"Apr 13"` (or `"Apr 13 2026"` if multi-year)
- Weekly → `"YYYY-Www"` (ISO week), label `"Apr 13 – Apr 19"` (+ year suffix if multi-year)
- Monthly → `"YYYY-MM"`, label `"Apr 2026"` (year always included)

Use `BTreeMap` for cells/periods/totals — iteration is sorted, and the keys
sort correctly lexicographically.

**Year-suffix detection:** before building labels, collect distinct
`event_at.date().year()` into a `HashSet`. If `len() > 1`, pass `show_year =
true` to the label formatter.

### 6.3 Inventory aggregation (tier stacks)

This is the most complex piece. Each item tracks a `Vec<PriceTier>` — a stack
of `(qty, price)` tuples. Events mutate the stack:

| Action | Stack mutation |
|---|---|
| `added` | If `total_qty < 0`, clear the stack first (negative-reset). Push a new tier `{ qty, price: None }`. |
| `visible` / `configured` | Walk tiers front-to-back, apply the new price to up to `e.quantity` units across unpriced tiers. Split tiers when the event covers fewer units than an unpriced tier contains. If no unpriced tier was found, overwrite the last tier's price (fallback). |
| `bought` | First try to remove from a tier with matching `price_unit` (deterministic), then fall back to removing from any tier front-to-back. If nothing matched, push a negative tier. |
| `removed` | LIFO: walk tiers back-to-front, remove from each. |

After each event, `retain(|t| t.qty != 0)` to prune zeroed tiers.

Final pass: collapse same-price tiers into one, drop non-positive tiers, sort
by price ascending.

### 6.4 Sales window logic

`aggregate_inventory(events, period_days)` takes a window parameter:

- **Positive finite** → take the **N most-recent distinct activity dates** as
  the window. Divisor for `avg_per_day` is `period_days`.
- **Zero / negative / ≥ 99999** → "all time". Window is every distinct date.
  Divisor is `max(distinct_date_count, 1)`.

**Why "distinct active dates" and not calendar days:** sparse logs with gaps
shouldn't have their per-day averages diluted. If a user only has data for 3
days across 2 weeks, "Last 7 days" should take the 3 most recent days and
divide by 7 (conservative forecast), not collapse to a single day just because
the calendar range spans 14 days.

`active_dates` is returned newest-first so the frontend can slice it for the
"Recently Sold Out in last N days" panel.

### 6.5 Aggregation commands

```rust
#[tauri::command] pub fn get_stall_revenue(db, params: RevenueParams) -> RevenueResult;
#[tauri::command] pub fn get_stall_inventory(db, params: InventoryParams) -> InventoryResult;
```

Both fetch rows from SQLite (filtering by `owner`, `NOT ignored`,
`item IS NOT NULL`, `event_at IS NOT NULL`), convert to the aggregation
module's event types, then delegate to the pure functions. Both return empty
results when `owner` is missing/empty.

**Critical ordering:** `get_stall_inventory` MUST `ORDER BY event_at ASC, id ASC`.
Without the `id` tiebreaker, `visible` events could be processed before
`added` events within the same minute (both have the same `event_at`), causing
prices to never apply to the correct tier.

### 6.6 Tests

- `revenue_daily_pivot` — 2 items across 2 days, grand total correct
- `revenue_monthly_collapses_days` — 30 events in April roll into one cell
- `inventory_add_then_sell` — add 5, visible at 4500g, sell 2 → qty=3, value=13500, period_sold=2
- `inventory_period_scoped_to_recent_active_dates` — sparse 3-day dataset, window=1 returns only latest day
- `inventory_period_spans_multiple_active_dates` — window=2 covers 2 most recent active days
- `inventory_all_time_uses_distinct_date_count_as_divisor` — 5 sold / 3 distinct dates ≈ 1.666 avg/day

---

## 7. Phase 5 — Frontend store & parent view

### 7.1 Pinia store

`src/stores/stallTrackerStore.ts` — reduced to shared state only:

```ts
export const useStallTrackerStore = defineStore('stallTracker', () => {
  const settingsStore = useSettingsStore()

  const stats = ref<StallStats | null>(null)
  const filterOptions = ref<StallFilterOptions>({
    buyers: [], players: [], items: [], dates: [], actions: []
  })
  const dataVersion = ref(0)  // monotonic: bump → tabs refetch

  const currentOwner = computed<string | null>(() => {
    const name = settingsStore.settings.activeCharacterName
    return name && name.length > 0 ? name : null
  })

  async function loadStats(filters?: StallStatsFilters) { /* scoped invoke */ }
  async function loadFilterOptions() { /* scoped invoke */ }
  async function toggleIgnored(id: number, ignored: boolean) { /* +dataVersion++ */ }
  async function clearAll(): Promise<number> { /* +dataVersion++, reload */ }

  watch(currentOwner, () => { dataVersion.value++; loadStats(); loadFilterOptions() })

  let coordTimer: ReturnType<typeof setTimeout> | null = null
  listen<number>('stall-events-updated', () => {
    if (coordTimer) clearTimeout(coordTimer)
    coordTimer = setTimeout(() => {
      dataVersion.value++
      loadStats()
      loadFilterOptions()
    }, 500)
  })

  return { stats, filterOptions, currentOwner, dataVersion,
           loadStats, loadFilterOptions, toggleIgnored, clearAll }
})
```

**Why `dataVersion` is a counter, not a boolean:** tabs use `watch(() =>
store.dataVersion, () => reload())`. A counter guarantees a new value on every
bump even if two mutations happen back-to-back.

**Why the 500ms debounce:** Player.log catch-up can replay many book events in
a burst during startup or after unlocking the PC. A single trailing refresh is
enough and avoids spamming the backend with identical `get_stall_stats` calls.

### 7.2 Startup wiring

In `startupStore.ts`, alongside `marketStore.loadAll()`:

```ts
await Promise.all([
  stallTrackerStore.loadStats(),
  stallTrackerStore.loadFilterOptions(),
])
```

Per-tab row data is fetched lazily when each tab mounts — don't preload it at
startup.

### 7.3 Parent view (`StallTrackerView.vue`)

```
┌────────────────────────────────────────────────────────────────────────────┐
│ [Sales] [Revenue] [Inventory]          Viewing: Deradon  [Shop Log] [Import]│
│                                                           [Export] [Clear] │
├────────────────────────────────────────────────────────────────────────────┤
│                                                                            │
│                      <active tab component>                                │
│                                                                            │
└────────────────────────────────────────────────────────────────────────────┘
```

- `TabBar` (from `Shared/TabBar.vue`) with three tabs: Sales, Revenue, Inventory.
- **Right-side action row** contains (from left to right):
  - `Viewing: <character>` — reactive on `store.currentOwner`, uses
    `text-entity-player` color token. Tooltip spells out the scoping contract.
  - Ephemeral `importMessage` text (5s auto-clear).
  - **Shop Log** button — gated on `hasData`, opens the modal.
  - **Import** button.
  - **Export** button — gated on `hasData && currentOwner`.
  - **Clear data** button — gated on `hasData && currentOwner`, red on hover.
- Below the tab bar, the active tab renders inside a flex container with
  `flex-1 min-h-0 overflow-auto` so tables scroll inside the view area.
- The Shop Log modal lives at the bottom of the template inside a
  `<Teleport v-if="shopLogMounted" to="body">`. See Phase 9 for details.

### 7.4 `EconomicsView.vue` wiring

The Economics view already has a `stall-tracker` tab stub. Replace its
`EmptyState` with `<StallTrackerView />`:

```vue
<template v-else-if="activeTab === 'stall-tracker'">
  <StallTrackerView />
</template>
```

---

## 8. Phase 6 — Sales tab

### 8.1 ASCII preview

```
┌──────────────────────────────────────────────────────────────────────────┐
│  TOTAL SALES    TOTAL REVENUE    UNIQUE BUYERS   UNIQUE ITEMS            │
│     1,247         18,439,500g         38              24                 │
├──────────────────────────────────────────────────────────────────────────┤
│ [From date ▼] – [To date ▼] [All buyers ▼] [All items ▼]                 │
│ Showing 500 of 1,247 sales    Clear filters                              │
├──────────────────────────────────────────────────────────────────────────┤
│    │ DATE               │ BUYER       │ ITEM               │  QTY │ UNIT │
│────┼────────────────────┼─────────────┼────────────────────┼──────┼──────┤
│ ⊘  │ Sat Apr 13 15:09   │ MrBonq      │ Quality Reins      │   1  │ 4,500│
│ ⊘  │ Sat Apr 13 15:08   │ AlestiarWolf│ Mystic Saddlebag   │   1  │40,000│
│ ⊘  │ Fri Apr 12 22:47   │ Zangariel   │ Orcish Spell Pouch │  12  │   450│
│ ○  │ Fri Apr 12 19:33   │ Kork        │ Nice Saddle        │   1  │ 4,000│  ← ignored (opacity 35)
│ ⊘  │ Fri Apr 12 14:13   │ Brynn       │ Amazing Horseshoes │   1  │ 6,000│
│ ⊘  │ …                  │ …           │ …                  │  …   │   …  │
├──────────────────────────────────────────────────────────────────────────┤
│             [ Load more (747 remaining) ]                                │
└──────────────────────────────────────────────────────────────────────────┘
```

(The TOTAL column is omitted in ASCII for space — it sits to the right of UNIT
and renders in the same gold accent.)

### 8.2 Behavior

- Stats header computes from `get_stall_stats(statsFilters)` — a **second invoke**,
  not from the current row slice. Stats reflect *all filtered rows*, not just
  the loaded page. This is what the commit message "reactive stats" refers to.
- Filter row uses `SearchableSelect` (a shared component — new, see
  `Shared/SearchableSelect.vue`). Each dropdown is populated from
  `store.filterOptions`.
- Date inputs use `DatePicker` (also new, see `Shared/DatePicker.vue`)
  rather than native `<input type="date">`. The native picker in WebView2
  doesn't dismiss on outside click and there's no clean DOM workaround
  that doesn't break in-picker month/year navigation. The custom popover
  uses the same teleport + position-flip + click-backdrop pattern as
  `SearchableSelect`, so it works consistently inside the Shop Log
  modal at `z-[60]` (the picker layers at `z-[70]`).
- `include_ignored: true` for the list query (user wants to see ignored rows),
  `false` implicit in `get_stall_stats` (stats exclude them).
- **Ignore toggle** (first column): `⊘` for active, `○` for ignored. Click
  triggers a `confirm()` dialog (kind: `'info'`). On accept, calls
  `store.toggleIgnored(id, !event.ignored)`, which bumps `dataVersion`, which
  triggers a reload.
- Page size 500. `loadMore()` appends the next page by calling
  `get_stall_events` with `offset = rows.length`.
- Sorting: click a column header to set `sortKey`, subsequent clicks flip
  direction. Default direction is `desc` for numerics/dates, `asc` for
  text columns (`player`, `item`).
- **Auto-swap inverted date range:** `watch([filterDateFrom, filterDateTo],
  ([from, to]) => { if (from && to && from > to) { swap } })`.
- **Reset filters on character switch:** `watch(() => store.currentOwner,
  resetFilters)`. Without this, a buyer filter from character A carries over
  to B and silently returns zero rows.
- Debounced filter-change reload (200ms) to avoid spamming invokes while the
  user is still typing in a dropdown.

### 8.3 Empty state

```
┌──────────────────────────────────────────┐
│                                          │
│            No sales recorded             │
│                                          │
│   Open your shop log book in-game to     │
│   start tracking, or use Import to load  │
│   an exported book file.                 │
│                                          │
└──────────────────────────────────────────┘
```

Use the shared `EmptyState` component with `variant="panel"`. Always mention
**Import** alongside live tailing so new users know the path exists.

---

## 9. Phase 7 — Revenue tab

### 9.1 ASCII preview (Daily)

```
┌──────────────────────────────────────────────────────────────────────────┐
│ [Daily] Weekly  Monthly     [From date ▼] – [To date ▼]                  │
│                             [All buyers ▼] [All items ▼]    Clear filters│
├────────────────────┬────────┬────────┬────────┬────────┬────────┬────────┤
│ ITEM               │ TOTAL  │ Apr 13 │ Apr 12 │ Apr 11 │ Apr 10 │ Apr 9  │ ← sticky header row
├────────────────────┼────────┼────────┼────────┼────────┼────────┼────────┤
│ Quality Reins      │ 18,000 │  4,500 │  9,000 │        │  4,500 │        │
│ Mystic Saddlebag   │ 80,000 │ 40,000 │ 40,000 │        │        │        │
│ Nice Saddle        │ 12,000 │        │  4,000 │  4,000 │  4,000 │        │
│ Orcish Spell Pouch │ 10,800 │  5,400 │        │        │        │  5,400 │
│ Amazing Horseshoes │  6,000 │        │        │  6,000 │        │        │
│ …                  │    …   │    …   │    …   │    …   │    …   │    …   │
├────────────────────┼────────┼────────┼────────┼────────┼────────┼────────┤
│ TOTAL              │126,800 │ 49,900 │ 57,000 │ 10,000 │  8,500 │  5,400 │ ← sticky footer row
└────────────────────┴────────┴────────┴────────┴────────┴────────┴────────┘
     ↑sticky                ↑sticky
     col                    col
```

### 9.2 Behavior

- Granularity toggle is three buttons inside a single bordered container. The
  active one gets `bg-accent-gold/20 text-accent-gold font-medium`.
- Filter row is identical to Sales (date range + buyer + item dropdowns).
- The table has **four sticky elements**:
  - `thead tr` — sticky top
  - `tfoot tr` — sticky bottom
  - `Item` column (200px fixed width) — sticky left, `z-20` in the header/footer, `z-10` in body
  - `Total` column — sticky at `left-[200px]`, same z-indexing
- Cells: use `BTreeMap` lookup `cells[(item, period_key)] → revenue`. Empty
  cells render as blank (`''`), not `0`, for readability.
- Grand total is the intersection of the sticky left+right and sticky bottom —
  visually pinned at the bottom-left corner.
- Row order: items sorted alphabetically (already sorted by `BTreeMap` in the
  aggregation). Column order: period keys sorted lexicographically
  (`YYYY-MM-DD` / `YYYY-Www` / `YYYY-MM` all sort correctly), then
  **reversed in the display layer** so the most recent period is the
  leftmost column after the sticky Total. Recent data is visible without
  horizontal scrolling. The cell lookup is keyed by `period_key` so the
  reversal is purely visual — totals are unaffected.
- Single invoke per change: `get_stall_revenue({ owner, granularity,
  date_from, date_to, buyer, item })`. No frontend pivoting.
- Same character-switch + inverted-date-range rules as Sales.

---

## 10. Phase 8 — Inventory tab

### 10.1 ASCII preview

```
┌──────────────────────────────────────────────────────────────────────────┐
│  ITEMS IN STOCK   EST. SHOP VALUE   SOLD   AVG DAILY REVENUE             │
│       17             238,500g        84        14,275g                   │
│                                                     Sales period [Last 7 ▼]│
├──────────────────────────────────────────────────────────────────────────┤
│ Inventory is estimated from shop log events. It may be incomplete        │ ← italic note
│ if older log data is missing.                                            │
├──────────────────────────────────────────────────────────────────────────┤
│ IN STOCK                                                                 │
│ ITEM                │ QTY │ PRICE    │ EST. VALUE │ SOLD │ AVG/DAY │ LAST│
├─────────────────────┼─────┼──────────┼────────────┼──────┼─────────┼─────┤
│ Quality Reins       │  4  │   4,500g │   18,000g  │  12  │   1.7   │Apr13│
│ Mystic Saddlebag    │  2  │  1×19,000│   59,000g  │   8  │   1.1   │Apr13│  ← multi-tier
│                     │     │  1×40,000│            │      │         │     │
│ Nice Saddle         │  3  │   4,000g │   12,000g  │   6  │   0.9   │Apr12│
│ Amazing Horseshoes  │  1  │   6,000g │    6,000g  │   1  │   0.1   │Apr10│
│ …                   │  …  │    …     │     …      │   …  │   …     │  …  │
├──────────────────────────────────────────────────────────────────────────┤
│ ▶ Recently Sold Out (3)  [Last 3 days ▼]                                 │
└──────────────────────────────────────────────────────────────────────────┘
```

When the "Recently Sold Out" accordion is expanded:

```
│ ▼ Recently Sold Out (3)  [Last 3 days ▼]                                 │
├──────────────────────┬────────────┬──────┬─────────┬────────────────────┤
│ ITEM                 │ LAST PRICE │ SOLD │ AVG/DAY │ LAST ACTIVITY      │
├──────────────────────┼────────────┼──────┼─────────┼────────────────────┤
│ Decent Horseshoes    │    3,500g  │   4  │   0.6   │ Apr 13             │  ← greyed
│ Orcish Spell Pouch   │      450g  │  12  │   1.7   │ Apr 12             │
│ Great Saddle         │    5,000g  │   2  │   0.3   │ Apr 11             │
```

### 10.2 Behavior

- Single invoke per change: `get_stall_inventory({ owner, period_days })`.
- Stats header recomputes in Rust: `estimated_value`, `total_sold` (window-scoped),
  `avg_daily_revenue` (window-scoped). `inStockItems.length` is computed on the
  frontend by filtering `items.filter(i => i.quantity > 0)`.
- **Sales period dropdown** — maps to `period_days`:
  | Label | Value |
  |---|---|
  | Last day | 1 |
  | Last 2 days | 2 |
  | Last 7 days | 7 |
  | Last 14 days | 14 |
  | Last 30 days | 30 |
  | All time | 100000 (sentinel) |
- Quantity is positional, **period-independent**. Changing the period only
  affects `period_sold`, `avg_per_day`, `period_revenue`. The stack state is
  always the full history.
- **Multi-tier price rendering:**

  ```vue
  <span v-if="item.price_tiers.length === 1">
    {{ formatPrice(item.price_tiers[0].price) }}
  </span>
  <span v-else-if="item.price_tiers.length > 1" class="flex flex-col items-end gap-0.5">
    <span v-for="(tier, idx) in item.price_tiers" :key="idx" class="text-xs">
      {{ tier.qty }}&times;{{ formatPrice(tier.price) }}
    </span>
  </span>
  ```

  This is the difference between "the item is listed at one price" (common) and
  "the user has the same item listed in multiple price tiers, e.g., 1 at 19k and
  1 at 40k". The aggregation merges same-price tiers before returning, so
  multi-row rendering only happens when there really are distinct tiers.
- **Recently Sold Out** accordion — `inStockItems.length`'s complement:
  items where `quantity === 0` AND `last_activity_at` falls in the active
  window. The window is computed from `active_dates.slice(0, soldOutDays)` —
  distinct dates, not calendar days, to match the backend semantics.
- **"Avg/Day" divisor** — always the full window size (1, 2, 7, 14, 30, or the
  all-time distinct-date-count), not just active days. Conservative forecasting.
- **Last price for sold-out items**: comes from the `InventoryItem.last_known_price`
  field tracked separately from the tier stack. The tier stack collapses
  to empty when an item sells out (`finalize_tiers` drops zero-quantity tiers),
  so a Recently Sold Out row would otherwise have no price to display.
  `last_known_price` updates on every `visible` / `configured` / `bought`
  event, so the most recent price the customer would have seen survives the
  collapse. Added in Phase 8 fixup.
- **Sortable columns** (both tables): both In Stock and Recently Sold Out
  have click-to-sort headers with their own independent sort state. In Stock
  defaults to `item asc`; Recently Sold Out defaults to `last_activity_at desc`
  so the most recently sold-out items appear first. Sorting happens
  frontend-only — inventory is computed all-at-once by Rust and the item
  count is bounded (typically <100 per character), so backend pagination
  isn't needed.
- Reload: `watch(salesPeriodDays, () => reload())`, `watch(() => store.dataVersion,
  () => reload())`.

### 10.3 Tier stack pitfalls (from the PoC)

- Same-timestamp events are sorted by `(event_at, id)`. Without the `id`
  tiebreaker, `visible` events could arrive before `added` events within the
  same minute and fail to price the tier.
- If `total_qty < 0` at the time an `added` event arrives, the stack is cleared
  first. This handles the case where the log is truncated and we've seen sales
  for items we never saw added — the fresh `added` resets the baseline.
- `bought` events first try to match a tier with the same `price_unit` before
  falling back to any tier. This correctly handles price drops mid-sale.
- Bulk configs like `"3000 per 2"` must compute `price_unit = 1500`, never
  `3000`. Do the division in the parser, not the aggregator.

---

## 11. Phase 9 — Shop Log modal

### 11.1 Why a modal, not a tab

The Shop Log view is a **maintenance view**. Most sessions only touch it to
ignore a misfire event or investigate why the inventory tab shows something
odd. Giving it the same prominence as Sales/Revenue/Inventory pushes a
rarely-used view into the user's attention every time they visit the screen.

### 11.2 ASCII preview

```
╔════════════════════════════════════════════════════════════════════════╗
║  Shop Log                                                           ×  ║
╟────────────────────────────────────────────────────────────────────────╢
║ [From date ▼] – [To date ▼] [All players ▼] [All actions ▼] [All items▼]║
║ Showing 500 of 2,341 entries    Clear filters                          ║
╟────────────────────────────────────────────────────────────────────────╢
║    │ DATE               │ PLAYER    │ ACTION   │ ITEM          │ QTY│G ║
║────┼────────────────────┼───────────┼──────────┼───────────────┼────┼──║
║ ⊘  │ Sat Apr 13 15:09   │ MrBonq    │ [bought] │ Quality Reins │  1 │4500║
║ ⊘  │ Sat Apr 13 14:13   │ Deradon   │[collected]│              │    │30500║
║ ⊘  │ Sat Apr 13 13:30   │ Deradon   │[configured]│ Nice Saddle │  1 │   ║
║ ⊘  │ Sat Apr 13 13:29   │ Deradon   │ [added]  │ Nice Saddle   │  1 │   ║
║ ⊘  │ Sat Apr 13 13:28   │ Deradon   │[visible] │ Nice Saddle   │  1 │4000║
║ ○  │ …                  │ …         │    …     │       …       │  …  │ …║
╟────────────────────────────────────────────────────────────────────────╢
║                 [ Load more (1841 remaining) ]                         ║
╚════════════════════════════════════════════════════════════════════════╝
                          ← dimmed backdrop click closes
```

Action badges use distinct colors:

| Action | Class hint |
|---|---|
| `bought` | green tint |
| `added` | blue tint |
| `removed` | red tint |
| `configured` | amber tint |
| `visible` | cyan tint |
| `collected` | gold tint |

(Exact tokens TBD by the styling pass — match existing badge patterns in the
app.)

### 11.3 Lazy mount + persistent state

The modal is expensive enough (its inner tab has full filter/sort/pagination
state) that we don't want to remount it on every close/reopen. Pattern:

```vue
<Teleport v-if="shopLogMounted" to="body">
  <div v-show="shopLogOpen" class="fixed inset-0 z-[60] …">
    <div class="absolute inset-0 bg-black/60" @click="shopLogOpen = false" />
    <div class="relative bg-surface-base … h-full max-h-[90vh]">
      <div class="flex items-center justify-between px-4 py-3 border-b">
        <h3>Shop Log</h3>
        <button class="text-text-secondary hover:text-text-primary …"
                @click="shopLogOpen = false">×</button>
      </div>
      <div class="flex-1 min-h-0 overflow-auto p-4">
        <StallShopLogTab />
      </div>
    </div>
  </div>
</Teleport>

<script setup>
const shopLogOpen = ref(false)
const shopLogMounted = ref(false)
function openShopLog() {
  shopLogMounted.value = true
  shopLogOpen.value = true
}
</script>
```

(An earlier draft wrapped the inner overlay in `<Transition name="modal" appear>`,
but the matching CSS classes never existed in any stylesheet so the transition
was a no-op. Removed in Phase 9 fixup. Add a transition only if the matching
CSS lands too.)

**Why `v-if` on the Teleport but `v-show` on the inner overlay:** the Teleport
doesn't mount until the first open (lazy). Once mounted, subsequent closes
only flip `v-show`, so the `<StallShopLogTab>` instance — and its internal
filter refs, sort state, page offset, and scroll position — persists across
open/close cycles within the session.

### 11.4 Escape handling

```ts
function onKeydown(e: KeyboardEvent) {
  if (e.key !== 'Escape' || !shopLogOpen.value) return
  const target = e.target as HTMLElement | null
  if (target && target.tagName === 'TEXTAREA') return
  if (target && target.tagName === 'INPUT' && (target as HTMLInputElement).type !== 'date') return
  shopLogOpen.value = false
}
```

The input-focus check is **critical**: the `SearchableSelect` dropdowns inside
the modal handle their own Escape-to-close (and stop propagation), so Escape
with a dropdown open should never reach the window listener. The text-input
exemption is belt-and-suspenders for any future text input added to the
modal.

`<input type="date">` is intentionally NOT exempted: native date pickers
don't have their own Escape behavior worth preserving, and users expect
Escape to dismiss the modal even from a focused date input.

### 11.5 z-index trap

The app-wide `MenuBar` is `fixed … z-50`. The modal MUST be at least `z-[60]`
or the menu bar will paint over its title bar and close button. Do not use
`z-40`. Confirmed by the `4aacb87` fix commit.

### 11.6 Close button visibility

Use `text-text-secondary` (≈5.5:1 contrast), not `text-text-dim` (≈3.2:1).
Hover brightens to `text-text-primary`. This matches the visibility of other
subtle chrome controls.

### 11.7 Sortable columns

All six list columns (Date, Player, Action, Item, Qty, Gold) have
click-to-sort headers with the same toggle behavior as the Sales tab.
Default sort is `event_at desc` to preserve the existing newest-first
list order. The backend's `get_stall_events` command already supports
all six sort keys via its whitelist (see §5.3), so this is a pure
frontend wire-up. Same request-token race protection as the Sales tab
prevents a header click during a `loadMore` from producing mixed-sort
rows. Added in Phase 13.

---

## 12. Phase 10 — Import / Export / Clear

### 12.1 Import

```rust
#[tauri::command]
pub fn import_shop_log_file(
    db: State<'_, DbPool>,
    ops_lock: State<'_, StallOpsLock>,
    path: String,
    current_owner: Option<String>,
) -> Result<ImportResult, String>
```

Returns:

```rust
pub struct ImportResult {
    pub total_entries: usize,
    pub new_entries: usize,
    pub effective_owner: Option<String>,
    pub owner_claimed: bool,
}
```

**Import is the one path that uses the parser's owner hint.** Unlike the live
coordinator (which stamps `owner = active_character` unconditionally because
the game guarantees the player can only open their own book), Import has to
handle files that could belong to anyone — the active character, a friend, an
alt, or an owner-less historical export. The `shop_log.owner` hint is how we
tell those cases apart.

Key rules:

1. **Year from filename first, `Local::now().year()` as fallback.**
   `year_from_filename()` scans for a 4-digit sequence between 2000 and 2099.
2. **`effective_owner = shop_log.owner.or(current_owner)`.** Three cases:
   - Book owner matches `current_owner` → normal import under the active
     character. No UI warning.
   - Book owner is a *different* character → stamp with the parsed owner so
     it lands under that character's view. The frontend detects the mismatch
     (parsed owner ≠ `store.currentOwner`) and surfaces a *"Switch character
     to view them"* message.
   - Book has no owner actions (`shop_log.owner.is_none()`) → fall back to
     `current_owner` so the rows stay visible under the active view, and set
     `owner_claimed = true`.
3. **`owner_claimed = shop_log.owner.is_none() && total_entries > 0`.** The UI
   uses this to show a "claimed for Deradon — book(s) did not identify an
   owner" message so users understand why the rows landed under their view.

The frontend tracks imports **per owner** across a multi-file import:

```ts
const entriesByOwner = new Map<string, number>()
for (const path of paths) {
  const result = await invoke<ImportResult>('import_shop_log_file', { path, currentOwner })
  const owner = result.effective_owner ?? '(unknown)'
  entriesByOwner.set(owner, (entriesByOwner.get(owner) ?? 0) + result.total_entries)
}
```

Four distinct messages:

| Case | Message |
|---|---|
| Zero entries | `"No shop log entries found in file. Is it an exported shop log book?"` |
| Mixed owners (current + others) | `"Imported 100 for Deradon, 134 for Alvida. Switch character to view entries for other owners."` |
| Only other owners | Same per-owner breakdown. |
| Owner-less claimed | `"Imported 134 entries (claimed for Deradon — book(s) did not identify an owner)."` |
| Normal | `"Imported 134 entries, 12 duplicates skipped"` |

### 12.2 Export

```rust
#[tauri::command]
pub fn export_shop_log_files(
    db: State<'_, DbPool>,
    directory: String,
    owner: Option<String>,
) -> Result<ExportResult, String>
```

- Requires non-empty `owner`.
- Fetches all non-unknown events for that owner.
- Groups by `(owner, year, month, day)` where year comes from `event_at`.
- **Sorts each group newest-first** by `(event_at desc, entry_index desc)` so
  the resulting file body matches the game's book format verbatim.
- Writes one file per group: `{sanitize(owner)}-shop-log-{YYYY}-{MM}-{DD}.txt`.
- Body format: `event_timestamp - raw_message` separated by `\n\n`, trailing `\n`.

The round-trip contract: any file produced by Export must be re-importable by
Import, landing identical rows (same unique keys → `INSERT OR IGNORE` skips
them all). Use this as a test case during development.

### 12.3 Clear

Native confirmation dialog:

```
┌─────────────────────── Clear Stall Data ───────────────────────┐
│                                                                │
│  This will delete all stall tracker data for Deradon. Other    │
│  characters are not affected.                                  │
│                                                                │
│  Consider using Export first — in-game shop log books only     │
│  hold recent history, so once this data is gone it may not     │
│  be recoverable from the game.                                 │
│                                                                │
│                                    [ Cancel ]  [ OK ]          │
└────────────────────────────────────────────────────────────────┘
```

Use Tauri's `confirm()` with `kind: 'warning'`. Name the character explicitly
and push users toward exporting first.

---

## 13. Phase 11 — Character scoping & concurrency

### 13.1 `StallOpsLock`

```rust
#[derive(Default)]
pub struct StallOpsLock(pub Mutex<()>);
```

Register as Tauri managed state in the setup hook:

```rust
app.manage(db::stall_tracker_commands::StallOpsLock::default());
```

Acquire inside `insert_stall_events` and `clear_stall_events` **around the DB
mutation only** — not around the whole command. Keep critical sections small.

The coordinator fetches the lock via
`app_handle.state::<StallOpsLock>()` at the insertion site.

**Why a mutex:** without it, a user clicking "Clear data" while the coordinator
is in the middle of ingesting a fresh live batch could leave the DB with a
partial batch written *after* the DELETE — invisible orphan rows scoped to a
character the user thought they cleared.

### 13.2 Scoping enforcement layers

1. **Backend reads** — return empty results when `owner` is missing/empty.
   Never SQL-filter with `WHERE owner = ''`; that's not the same as "no filter".
2. **Backend mutations** — return `"requires an owner"` error when owner is
   missing. `clear`, `export`, `seed` all enforce this.
3. **Frontend store** — `currentOwner` is a computed over
   `settingsStore.settings.activeCharacterName`. Empty strings → `null`.
4. **Frontend tabs** — `buildParams()` always threads `store.currentOwner`.
   `watch(() => store.currentOwner, resetFilters)` clears tab-local filter
   state on character switch.
5. **Frontend UI gating** — Clear and Export buttons are hidden unless
   `hasData && store.currentOwner`.

### 13.3 Character-switch refresh flow

```
user switches character
         │
         ▼
settingsStore.activeCharacterName changes
         │
         ▼
stallTrackerStore.currentOwner recomputes
         │
         ▼
watch(currentOwner) fires:
  • dataVersion++
  • loadStats()
  • loadFilterOptions()
         │
         ▼
each tab's watch(() => store.dataVersion) fires:
  • reload()
         │
         ▼
each tab's watch(() => store.currentOwner) fires:
  • resetFilters()
```

The two watches are independent so one can't starve the other. Filter reset
uses a direct `resetFilters()` call, not a `dataVersion`-piggybacked effect,
so the filter state is always consistent with the currently-loaded data.

---

## 14. Phase 12 — Polish & empty states

### 14.1 Empty state copy

Every tab mentions Import alongside the live-tailing flow:

> Open your shop log book in-game to start tracking, or use Import to load an
> exported book file.

New users with a friend's exported file shouldn't have to guess that Import
works.

### 14.2 Clear filters affordance

Each filterable tab (Sales, Revenue, Shop Log) grows a `Clear filters` link
next to the filter row, visible only when at least one filter is set:

```ts
function hasActiveFilters() {
  return !!(filterDateFrom.value || filterDateTo.value || filterBuyer.value || filterItem.value)
}
```

Inventory has no filter row — skip it there.

### 14.3 Error handling in toggles

`handleToggleIgnored` in both Sales and Shop Log tabs wraps `store.toggleIgnored`
in try/catch and surfaces failures via the existing `error` ref. Failed
toggles must not be silent.

### 14.4 Auto-swap inverted date ranges

All three filter-capable tabs:

```ts
watch([filterDateFrom, filterDateTo], ([from, to]) => {
  if (from && to && from > to) {
    filterDateFrom.value = to
    filterDateTo.value = from
  }
})
```

### 14.5 Stall benchmark helpers (dev only)

Expose on `window.stallBench` for DevTools-driven scale testing:

```ts
if (typeof window !== 'undefined') {
  ;(window as unknown as { stallBench: unknown }).stallBench = {
    seed: async (count: number) => { /* invoke seed_stall_events_dev */ },
    clear: async () => { /* invoke clear_stall_events */ },
    bump: () => { dataVersion.value++ },
  }
}
```

Use this to verify the 100k-scale target: `await stallBench.seed(100000)`
should complete in a few seconds, tab opens should remain sub-second.

---

## 15. Critical pitfalls

These are the traps the PoC hit that wasted real time. Front-load the
awareness:

1. **Newest-first book ordering.** Reverse before indexing. Every other
   decision about deduplication and round-trip depends on this.
2. **`UNIQUE(event_timestamp, raw_message)` is not enough.** Two buyers of the
   same item at the same price in the same minute collapse. Include
   `entry_index` in the unique key.
3. **Same-minute `visible` before `added`.** Sort inventory events by
   `(event_at, id)`, not just `event_at`. Without the `id` tiebreaker the
   visible event is processed first and the tier never gets priced.
4. **`per N` bulk pricing.** `"3000 per 2"` is `price_unit = 1500`, not 3000.
   Do the division in the parser, once.
5. **Year-boundary books.** Use the monotonic resolver — don't assume all
   entries are in the current year.
6. **Menu bar z-index.** Modal MUST be ≥ `z-[60]`. Menu bar is `z-50`.
7. **Input-focused Escape.** Don't swallow Escape inside inputs or
   SearchableSelect dropdowns break.
8. **Character scoping everywhere.** Read returns empty (not error) on
   missing owner. Mutations error (not empty). UI gates on
   `hasData && currentOwner`.
9. **500 row default limit.** Don't use an unbounded default. Paginate
   explicitly. The PoC's initial 500-row cap caused older dates to silently
   disappear from the Shop Log tab while they were still visible in Sales (a
   subset) — because all action types competed for the same 500 slots.
10. **Frontend aggregation doesn't scale.** At 100k events the IPC copy + Vue
    reactivity recomputes froze the UI for 13+ seconds. The entire aggregation
    pipeline lives in Rust for this reason. Do not skip the aggregation
    commands and reimplement them in JS "just to get something working."
11. **Owner-less imports.** Bought-only files need `current_owner` fallback.
    Without it, rows land with `owner = NULL` and are invisible under every
    character. The `owner_claimed` flag exists so the UI can explain what
    happened.
12. **Rusty tests live alongside the code.** The PoC's 208 passing Rust tests
    are the regression safety net. Don't skip writing them just because the
    feature "works in the browser."

## 16. Test plan

**Rust unit tests (must all pass before Phase 5):**

- `shop_log_parser` — 27 tests covering all action types, bulk pricing,
  escaped newlines, owner detection, entry index stability, same-minute
  duplicates, stacked listing `xN` extraction (bought + visible + configured),
  and empty / garbage content.
- `stall_year_resolver` — 5 tests covering single-year, year-boundary,
  `base_year_for_live` (past + wrap), and unparseable entry handling.
- `stall_aggregations` — 21 tests covering daily/weekly/monthly pivots,
  multi-year and ISO week year boundary labels, the full tier-stack state
  machine (add / visible / configured / bought / removed), period window
  scoping, all-time divisor, last_known_price survival, and the
  last_sold_at vs last_activity_at distinction.

**Manual integration tests:**

- **Live capture round trip.** Launch app, open in-game shop log, verify rows
  appear in Sales. Close book, reopen it — verify row count stable. Add a new
  entry in-game, reopen — only the new row appears.
- **Import round trip.** Export current data to a directory. Clear data.
  Import every exported file. Verify Sales stats match exactly.
- **Character switch isolation.** Log into character A, capture data. Switch
  to character B. Verify Sales/Revenue/Inventory/Shop Log all empty. Switch
  back. Verify A's data reappears untouched.
- **Bought-only import.** Import a file with only `bought` entries. Verify
  the "claimed for X" message appears and rows land under the active
  character.
- **Mixed-owner import.** Import multiple files where one has `owner = A`
  and another has `owner = B`. Verify the per-owner breakdown message.
- **Inventory negative-reset.** Sell an item that was never added (truncated
  log). Verify negative tier. Import a historic file that contains the
  missing `added`. Verify quantity resets cleanly on the next `added` after
  it goes negative.
- **Scale test.** `await stallBench.seed(100000)`. Verify tab opens in
  <1s, sub-tab switches are instant, filter dropdown population is
  instant, Revenue pivot for 100k events completes in <2s.

**Regression watchlist for future PRs:**

- Any change to `ENTRY_RE` or per-action regexes must run the full parser
  test suite.
- Any change to `aggregate_inventory` must run the tier-stack tests.
- Any change to the filter shape must verify `build_filter_where` still
  returns empty for missing owner.
- Any change to the Revenue pivot must verify year-suffix detection still
  fires for multi-year datasets.

---

## Appendix A — File tree delta

```
src-tauri/src/
├── shop_log_parser.rs                           (NEW)
├── stall_year_resolver.rs                       (NEW)
├── stall_aggregations.rs                        (NEW)
├── coordinator.rs                               (MODIFIED — add BookOpened branch)
├── lib.rs                                       (MODIFIED — register commands, StallOpsLock)
└── db/
    ├── migrations.rs                            (MODIFIED — add stall_events migration)
    └── stall_tracker_commands.rs                (NEW)

src/
├── types/stallTracker.ts                        (NEW)
├── stores/stallTrackerStore.ts                  (NEW)
├── stores/startupStore.ts                       (MODIFIED — load stall stats at startup)
└── components/
    ├── Economics/EconomicsView.vue              (MODIFIED — replace stall stub)
    └── StallTracker/                            (NEW directory)
        ├── StallTrackerView.vue
        ├── StallSalesTab.vue
        ├── StallRevenueTab.vue
        ├── StallInventoryTab.vue
        └── StallShopLogTab.vue

src/components/Shared/
├── SearchableSelect.vue                         (NEW — shared component, used here first)
└── DatePicker.vue                               (NEW — themed calendar popover, replaces native <input type="date">)

docs/
├── features/screens/economics/economics-stall-tracker.md    (NEW — feature doc)
├── features/screens/economics.md                (MODIFIED — update component hierarchy, add link)
├── plans/stall-tracker-implementation.md        (this file)
└── index.md                                     (MODIFIED — link the new feature doc + plan)
```

## Appendix B — Command registration (Tauri builder)

```rust
use db::stall_tracker_commands::{
    clear_stall_events, export_shop_log_files, get_stall_events,
    get_stall_filter_options, get_stall_inventory, get_stall_revenue,
    get_stall_stats, import_shop_log_file, seed_stall_events_dev,
    toggle_stall_event_ignored, StallOpsLock,
};

tauri::Builder::default()
    .setup(|app| {
        app.manage(StallOpsLock::default());
        // … other setup
        Ok(())
    })
    .invoke_handler(tauri::generate_handler![
        // … existing handlers
        get_stall_events,
        get_stall_stats,
        get_stall_revenue,
        get_stall_inventory,
        get_stall_filter_options,
        toggle_stall_event_ignored,
        clear_stall_events,
        import_shop_log_file,
        export_shop_log_files,
        seed_stall_events_dev,
    ])
```
