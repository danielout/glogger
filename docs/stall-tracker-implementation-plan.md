# Stall Tracker — First Iteration Implementation Plan

## Context

The Stall Tracker is a stubbed feature under Economics that currently shows "Coming soon." A Ruby PoC in `playground/ruby/stall-tracker/` demonstrates parsing Player.log shop log entries into structured events. The goal is to bring this to life with two tabs: **Sales** (bought events only) and **Shop Log** (all event types).

The Rust backend already parses `ProcessBook` calls into `BookOpened` events (with `book_type: "PlayerShopLog"`) and emits them via `player-events-batch`. We need to: (1) parse the content of those BookOpened events into individual shop log messages, (2) persist them to SQLite, and (3) display them in the frontend.

---

## Step 1: Rust — Shop Log Content Parser

**New file:** `src-tauri/src/shop_log_parser.rs`

Port the Ruby PoC's parsing logic to Rust. This module will:

1. **Split content into entries** — The `BookOpened.content` field contains multiple entries separated by timestamps. Use a regex equivalent to the Ruby `ENTRY_REGEX` to split into individual entries:
   - Pattern: `Day Month Date HH:MM - [message]`
   - Example: `"Sat Mar 28 15:39 - Deradon removed Decent Horseshoes from shop"`

2. **Parse each entry into a typed event** — Match each entry's message against patterns for the 6 event types (bought, added, removed, configured, visible, collected) + unknown fallback.

3. **Define a `ShopLogEntry` struct:**
   ```rust
   ShopLogEntry {
       timestamp: String,       // "Sat Mar 28 15:39"
       action: String,          // "bought", "added", "removed", "configured", "visible", "collected", "unknown"
       player: String,          // character name
       item: Option<String>,    // item name (None for "collected")
       quantity: i64,           // default 1
       price_unit: Option<f64>, // price per unit
       price_total: Option<i64>,// total gold
       raw_message: String,     // original entry text
   }
   ```

4. **Define a `ShopLog` struct** to represent one ProcessBook call:
   ```rust
   ShopLog {
       log_timestamp: String,   // from BookOpened.timestamp
       title: String,           // "Today's Shop Logs", etc.
       entries: Vec<ShopLogEntry>,
       owner: Option<String>,   // detected from first owner action
   }
   ```

5. **Expose a `parse_shop_log(title, content, timestamp) -> ShopLog` function.**

**Regex patterns** — Direct port from Ruby PoC (see `playground/ruby/stall-tracker/README.md` for all patterns).

**Register module** in `src-tauri/src/lib.rs` with `mod shop_log_parser;`

---

## Step 2: Rust — Database Schema (Migration v19)

**Modify:** `src-tauri/src/db/migrations.rs`

Add `migration_v19_stall_tracker` creating one table:

```sql
CREATE TABLE stall_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_timestamp TEXT NOT NULL,        -- parsed entry timestamp "Sat Mar 28 15:39"
    log_timestamp TEXT NOT NULL,          -- ProcessBook timestamp
    log_title TEXT NOT NULL,              -- "Today's Shop Logs" etc.
    action TEXT NOT NULL,                 -- bought/added/removed/configured/visible/collected/unknown
    player TEXT NOT NULL,                 -- character name performing the action
    owner TEXT,                           -- detected shop owner
    item TEXT,                            -- item name
    quantity INTEGER NOT NULL DEFAULT 1,
    price_unit REAL,                      -- price per unit
    price_total INTEGER,                  -- total gold
    raw_message TEXT NOT NULL,            -- original entry text for debugging
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(event_timestamp, raw_message)  -- deduplication
);

CREATE INDEX idx_stall_events_action ON stall_events(action);
CREATE INDEX idx_stall_events_created ON stall_events(created_at DESC);
CREATE INDEX idx_stall_events_timestamp ON stall_events(event_timestamp);
```

The `UNIQUE(event_timestamp, raw_message)` constraint handles deduplication (same approach as the Ruby PoC's `body + index` uniqueness, but using the timestamp + message text since the same message at the same time is a duplicate).

---

## Step 3: Rust — Tauri Commands

**New file:** `src-tauri/src/db/stall_tracker_commands.rs`

Following the pattern from `farming_commands.rs`:

### Commands:

1. **`save_stall_events(entries: Vec<StallEventInput>) -> Result<usize, String>`**
   - Batch INSERT OR IGNORE (dedup via unique constraint)
   - Called when BookOpened events with `book_type == "PlayerShopLog"` arrive
   - Returns count of new rows inserted

2. **`get_stall_sales(limit: Option<i64>, offset: Option<i64>) -> Result<Vec<StallEvent>, String>`**
   - Query `stall_events WHERE action = 'bought'` ordered by `event_timestamp DESC`
   - For the Sales tab

3. **`get_stall_log(limit: Option<i64>, offset: Option<i64>) -> Result<Vec<StallEvent>, String>`**
   - Query all `stall_events` ordered by `event_timestamp DESC`
   - For the Shop Log tab

4. **`get_stall_stats() -> Result<StallStats, String>`**
   - Returns summary: total sales count, total revenue, unique buyers, unique items sold
   - For a summary header on the Sales tab

### Types:

```rust
#[derive(Deserialize)]
struct StallEventInput {
    event_timestamp: String,
    log_timestamp: String,
    log_title: String,
    action: String,
    player: String,
    owner: Option<String>,
    item: Option<String>,
    quantity: i64,
    price_unit: Option<f64>,
    price_total: Option<i64>,
    raw_message: String,
}

#[derive(Serialize)]
struct StallEvent {
    id: i64,
    event_timestamp: String,
    log_timestamp: String,
    log_title: String,
    action: String,
    player: String,
    owner: Option<String>,
    item: Option<String>,
    quantity: i64,
    price_unit: Option<f64>,
    price_total: Option<i64>,
    raw_message: String,
    created_at: String,
}

#[derive(Serialize)]
struct StallStats {
    total_sales: i64,
    total_revenue: i64,
    unique_buyers: i64,
    unique_items: i64,
}
```

**Register** commands in `src-tauri/src/lib.rs` invoke_handler.

---

## Step 4: Rust — Coordinator Integration

**Modify:** `src-tauri/src/coordinator.rs`

In the section where `BookOpened` events are processed (currently they're just emitted as-is via `player-events-batch`):

1. When a `BookOpened` event has `book_type == "PlayerShopLog"`:
   - Call `shop_log_parser::parse_shop_log()` to parse the content
   - Call `save_stall_events()` to persist to DB
   - Emit a new `"stall-events-updated"` Tauri event to notify the frontend

This means stall events are parsed and stored in real-time as the player opens their shop log books in-game.

---

## Step 5: TypeScript Types

**New file:** `src/types/stallTracker.ts`

```typescript
export interface StallEvent {
  id: number
  event_timestamp: string
  log_timestamp: string
  log_title: string
  action: 'bought' | 'added' | 'removed' | 'configured' | 'visible' | 'collected' | 'unknown'
  player: string
  owner: string | null
  item: string | null
  quantity: number
  price_unit: number | null
  price_total: number | null
  raw_message: string
  created_at: string
}

export interface StallStats {
  total_sales: number
  total_revenue: number
  unique_buyers: number
  unique_items: number
}
```

---

## Step 6: Pinia Store

**New file:** `src/stores/stallTrackerStore.ts`

Following `marketStore.ts` pattern (simple CRUD, no session state):

### State:
- `sales: StallEvent[]` — bought events
- `shopLog: StallEvent[]` — all events
- `stats: StallStats | null`
- `loading: boolean`
- `error: string | null`

### Actions:
- `loadSales(limit?, offset?)` — invokes `get_stall_sales`
- `loadShopLog(limit?, offset?)` — invokes `get_stall_log`
- `loadStats()` — invokes `get_stall_stats`

### Event Listener:
- Listen to `"stall-events-updated"` and auto-refresh data

---

## Step 7: Frontend Components

### 7a. Replace stub in EconomicsView.vue

**Modify:** `src/components/Economics/EconomicsView.vue`

Replace the EmptyState with a new `StallTrackerView` component. Add sub-tabs within it.

### 7b. Stall Tracker View

**New file:** `src/components/StallTracker/StallTrackerView.vue`

Parent component with two internal tabs:
- **Sales** (default) — shows bought events
- **Shop Log** — shows all events

Uses a simple internal tab state (not MenuBar tabs — these are sub-sub-tabs).

### 7c. Sales Tab

**New file:** `src/components/StallTracker/StallSalesTab.vue`

- Summary stats header (total revenue, total sales, unique buyers, unique items)
- Table of bought events: date, buyer, item, quantity, unit price, total price
- Sorted by date descending (newest first)
- Uses the `stallTrackerStore.sales` data

### 7d. Shop Log Tab

**New file:** `src/components/StallTracker/StallShopLogTab.vue`

- Table of all events: date, player, action, item, quantity, price info
- Action column with color-coded badges (bought = green, added = blue, removed = red, etc.)
- Sorted by date descending
- Uses the `stallTrackerStore.shopLog` data

---

## Step 8: Startup Integration

**Modify:** `src/stores/startupStore.ts`

Add stall tracker store initialization to the startup sequence:
- Load existing stall events from DB during startup
- Register the `"stall-events-updated"` event listener

---

## File Summary

### New Files (8):
| File | Purpose |
|------|---------|
| `src-tauri/src/shop_log_parser.rs` | Parse shop log content into structured events |
| `src-tauri/src/db/stall_tracker_commands.rs` | Tauri commands for DB operations |
| `src/types/stallTracker.ts` | TypeScript type definitions |
| `src/stores/stallTrackerStore.ts` | Pinia store |
| `src/components/StallTracker/StallTrackerView.vue` | Parent view with tab switching |
| `src/components/StallTracker/StallSalesTab.vue` | Sales (bought events) table |
| `src/components/StallTracker/StallShopLogTab.vue` | Full shop log table |
| `playground/ruby/stall-tracker/README.md` | Ruby PoC documentation |

### Modified Files (5):
| File | Change |
|------|--------|
| `src-tauri/src/db/migrations.rs` | Add migration_v19 with `stall_events` table |
| `src-tauri/src/lib.rs` | Register `shop_log_parser` module + new Tauri commands |
| `src-tauri/src/coordinator.rs` | Handle BookOpened/PlayerShopLog → parse + persist + emit |
| `src/components/Economics/EconomicsView.vue` | Replace EmptyState with StallTrackerView |
| `src/stores/startupStore.ts` | Initialize stall tracker store + listen for updates |

---

## Implementation Order

1. **Parser** (`shop_log_parser.rs`) — core logic, testable in isolation with `cargo test`
2. **DB migration** — schema must exist before commands work
3. **Commands** (`stall_tracker_commands.rs`) — DB layer
4. **Coordinator integration** — wire parser + commands into the event pipeline
5. **Types** — TypeScript definitions
6. **Store** — frontend state management
7. **Components** — UI (view, sales tab, shop log tab)
8. **Startup integration** — wire everything together

---

## Verification

### Rust tests:
```bash
cd src-tauri && cargo test shop_log  # Parser unit tests
cd src-tauri && cargo test stall     # Command tests
```

### Manual E2E:
1. `npm run tauri dev`
2. In Project Gorgon, open your shop log book (Today's/Yesterday's Shop Logs)
3. Navigate to Economics → Stall Tracker
4. Sales tab should show bought events with prices
5. Shop Log tab should show all event types
6. Re-opening the shop log book should not create duplicates (dedup check)
