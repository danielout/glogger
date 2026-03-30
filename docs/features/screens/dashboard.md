# Dashboard Screen

## Overview

The dashboard is the landing screen — an at-a-glance view of the player's current session and server-wide progress. It supports two modes toggled by a button in the top-right: **Active Character** (live session data) and **All Characters on Server** (aggregate analytics).

## Architecture

### Files

**Frontend (Vue/TS):**
- `src/components/Dashboard/DashboardView.vue` — main container with view mode toggle
- `src/components/Dashboard/ContextBar.vue` — weather, combat/mount status, effects, currencies
- `src/components/Dashboard/ActiveSkillsWidget.vue` — live skill tracking cards
- `src/components/Dashboard/SessionWidget.vue` — session timer and status
- `src/components/Dashboard/StatusWidget.vue` — coordinator status
- `src/components/Dashboard/TransactionLog.vue` — recent inventory events
- `src/components/Dashboard/PlayerNotes.vue` — localStorage-backed checklist
- `src/components/Dashboard/AggregateView.vue` — server-wide analytics

**Stores:**
- `gameStateStore` — live session data (skills, inventory events, world state, currencies)
- `aggregateStore` — server-wide wealth, inventory, and skill data
- `coordinatorStore` — tailing status indicators

### Component Hierarchy

```
DashboardView.vue                — view mode toggle (Active / Aggregate)
├── Active Character View
│   ├── ContextBar               — weather, combat, mount, effects, currencies
│   ├── SkillCard grid           — live XP tracking per skill
│   └── Bottom row (2-col)
│       ├── TransactionLog       — recent inventory events (last 50)
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

### Transaction Log
Scrollable list of the most recent 50 inventory events from `gameStateStore.liveEventLog`:
- Color-coded by event type: green (added), red (removed), yellow (stack changed)
- Item names rendered via `ItemInline` for tooltips and navigation
- Contextual detail (sold, consumed, stored, etc.)

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
| Inventory events | `gameStateStore.liveEventLog` | Session (last 50, in-memory) |
| Player notes | `localStorage` | Persistent (browser) |
| Aggregate wealth/inventory/skills | `aggregateStore` → Tauri commands | On-demand from database |

## Key Design Decisions

- **Two-mode dashboard** — toggle between character-centric live view and server-wide analytics, rather than cramming both into one screen.
- **Bounded event log** — only keeps last 50 transactions to prevent memory bloat.
- **Lazy aggregate loading** — skills section collapsed by default; data fetched on demand rather than at startup.
- **Notes in localStorage** — player notes are intentionally not in the database since they're personal scratch-pad items, not game state.
