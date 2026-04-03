# Dashboard Screen

## Overview

The dashboard is the landing screen — an at-a-glance view of the player's current session and server-wide progress. It supports two modes toggled by a button in the top-right: **Active Character** (live session data) and **All Characters on Server** (aggregate analytics).

## Architecture

### Files

**Frontend (Vue/TS):**
- `src/components/Dashboard/DashboardView.vue` — main container with view mode toggle
- `src/components/Dashboard/ContextBar.vue` — weather, combat/mount status, effects, currencies
- `src/components/Dashboard/ActivityFeed.vue` — reusable activity feed card (used by all five feed cards)
- `src/components/Dashboard/CurrentZone.vue` — current area with friendly NPCs and favor ranks
- `src/components/Dashboard/PlayerNotes.vue` — localStorage-backed checklist
- `src/components/Dashboard/AggregateView.vue` — server-wide analytics

**Stores:**
- `gameStateStore` — live session data (skills, inventory events, activity feeds, world state, currencies)
- `aggregateStore` — server-wide wealth, inventory, and skill data
- `coordinatorStore` — tailing status indicators

### Component Hierarchy

```
DashboardView.vue                — view mode toggle (Active / Aggregate)
├── Active Character View
│   ├── ContextBar               — weather, combat, mount, effects, currencies
│   ├── SkillCard grid           — live XP tracking per skill
│   ├── Activity row (3-col)
│   │   ├── ActivityFeed         — Items Incoming (loot, crafts, summoned)
│   │   ├── ActivityFeed         — Items Outgoing (sold, stored, consumed)
│   │   └── ActivityFeed         — Councils (vendor gold, loot, transactions)
│   └── Bottom row (3-col)
│       ├── CurrentZone          — current area, friendly NPCs, favor ranks
│       ├── ActivityFeed         — Favor Changes (NPC favor deltas)
│       └── PlayerNotes          — simple checklist (localStorage)
└── Aggregate View
    └── AggregateView            — server-wide stats across all characters
```

## Active Character View

### Context Bar
Single horizontal row of real-time environmental and character state:
- Current zone weather
- Combat/mount status indicators
- Active named effects count
- Non-zero currency balances

### Live Skill Tracking
Grid of `SkillCard` components for skills that have gained XP during the current session:
- Skill name and level (with bonus level breakdown)
- XP gained and XP/hour rate
- Levels gained during session
- Progress bar toward next level
- Estimated time to next level

Empty state: "No skill updates yet." when no XP has been gained.

### Activity Feeds

Five activity feed cards replace the old single transaction log, each showing a focused stream of events. All use the reusable `ActivityFeed.vue` component.

**Items Incoming** (green dot) — items gained during the session:
- Chat status only: `ItemGained` and `Summoned` events (Player.log item events are excluded to avoid double-counting — the same pickup fires both `ItemAdded` and `ItemStackChanged` in Player.log alongside `ItemGained` in chat)
- Item names rendered via `ItemInline` for tooltips and navigation

**Items Outgoing** (red dot) — items lost during the session:
- Player.log only: `ItemDeleted` (with context: sold, stored, consumed) and `ItemStackChanged` (negative delta)
- Chat status has no item removal events, so Player.log is the sole source
- Item names rendered via `ItemInline`

**Councils** (yellow dot) — gold/council currency changes:
- Chat status: `CouncilsChanged` (received/spent) and `CoinsLooted` (corpse search)
- Player.log: `VendorSold` events (sale price)
- Shows signed running total

**Favor Changes** (purple dot) — NPC favor deltas:
- Player.log: `FavorChanged` events (gifts, quest rewards, etc.)
- NPC names rendered via `NpcInline` for tooltips and navigation
- Shows signed running total

Each feed shows up to 30 entries with a summary footer (entry count + total amount).

### Current Zone
Shows the player's current area and lists all friendly NPCs located there:
- Area name displayed via `AreaInline`
- NPC list loaded from CDN data via `getNpcsInArea()`, each rendered with `NpcInline` for click-to-navigate
- Favor rank badge next to each NPC (from `gameStateStore.favorByNpc`), color-coded by tier
- Updates reactively when the player transitions to a new zone

### Player Notes
Simple checklist persisted to `localStorage` (key: `glogger-player-notes`):
- Add new notes
- Toggle checkbox completion
- Clear all completed notes

## Aggregate View

Server-wide analytics across all characters, loaded on-demand via `aggregateStore.loadAll()`:

### Wealth Summary
Three summary cards:
- **Total Currencies** — sum of all currency holdings across characters
- **Inventory Market Value** — estimated value based on market prices
- **Grand Total** — combined

### Per-Character Wealth Breakdown
Table showing currencies + inventory market value per character.

### Combined Inventory
Searchable table of all items across all characters with per-character quantity breakdown.

### Skills Across Characters
Collapsible, searchable table showing skill levels as a matrix (rows = skills, columns = characters). Collapsed by default for performance.

## Data Sources

| Data | Source | Persistence |
|------|--------|-------------|
| Weather, combat, mount | `gameStateStore.world` | Session (in-memory) |
| Currencies | `gameStateStore.currencies` | Session (in-memory) |
| Skill tracking | `gameStateStore.sessionSkillList` | Session (in-memory) |
| Items incoming/outgoing | `gameStateStore.itemsIncoming` / `itemsOutgoing` | Session (last 30, in-memory) |
| Council changes | `gameStateStore.councilChanges` | Session (last 30, in-memory) |
| Favor changes | `gameStateStore.favorChanges` | Session (last 30, in-memory) |
| Current area | `gameStateStore.world.area` | Persistent (database, `game_state_area`) |
| Area NPCs | `gameDataStore.getNpcsInArea()` → CDN data | CDN cache |
| NPC favor ranks | `gameStateStore.favorByNpc` | Persistent (database) |
| Chat status events | Backend `chat-status-event` → `gameStateStore` listener | Session (in-memory) |
| Item transactions | `item_transactions` table (v15 migration) | Persistent (database) |
| Player notes | `localStorage` | Persistent (browser) |
| Aggregate wealth/inventory/skills | `aggregateStore` → Tauri commands | On-demand from database |

## Key Design Decisions

- **Two-mode dashboard** — toggle between character-centric live view and server-wide analytics, rather than cramming both into one screen.
- **Split activity feeds** — five focused cards instead of one mixed transaction log, making it easy to scan specific categories at a glance.
- **Single source per feed** — each feed uses exactly one event source to avoid double-counting. Items Incoming uses chat status exclusively (correct quantities in one event); Items Outgoing uses Player.log exclusively (chat has no removal events); Councils uses both (non-overlapping: Player.log has VendorSold, chat has CouncilsChanged/CoinsLooted); Favor uses Player.log exclusively.
- **Reusable ActivityFeed component** — all five cards use the same component with props for dot color, entity links (ItemInline/NpcInline), signed totals, and empty states.
- **Bounded feeds** — each feed keeps last 30 entries to prevent memory bloat.
- **Lazy aggregate loading** — skills section collapsed by default; data fetched on demand rather than at startup.
- **Notes in localStorage** — player notes are intentionally not in the database since they're personal scratch-pad items, not game state.
