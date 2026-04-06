# Economics — Surveying: Analytics Tab

Parent: [economics-surveying.md](economics-surveying.md)

## Overview

The Analytics tab provides all-time aggregate survey statistics through a 3-panel PaneLayout. The left panel is a view navigator for switching between overview, per-zone, and per-survey-type views. The center panel renders the active view with multi-column card grids. The right panel houses the Item Cost Calculator for planning material acquisition.

Players can export their survey data and import data from other players to build community knowledge. A data source toggle switches between viewing personal data only or combined data (personal + imported).

## Files

### Orchestrator
- `src/components/Surveying/AnalyticsTab.vue` — PaneLayout orchestrator (`screen-key="survey-analytics"`), data loading, export/import logic, view state management

### Analytics Sub-Components (`src/components/Surveying/Analytics/`)
- `AnalyticsViewNav.vue` — left panel view selector (Overview, Zones, Survey Types)
- `AnalyticsHeader.vue` — center panel header with inline stats + controls
- `OverviewView.vue` — default center view (global stats, cross-zone comparison, all survey types table)
- `ZoneDetailView.vue` — per-zone center view composing card components
- `SurveyTypeDetailView.vue` — per-survey-type center view with cross-zone item data + charts
- `ZoneRewardsCard.vue` — item rewards table for a survey type
- `ZoneSpeedBonusCard.vue` — speed bonus items table for a category
- `SurveyTypeDistributionChart.vue` — VueUiXy bar chart of survey type completions
- `ProfitRateCard.vue` — expected profit/loss per survey per category
- `CrossZoneComparison.vue` — zone comparison table (reused in OverviewView)
- `ItemCostCalculator.vue` — item cost/efficiency calculator (in right pane)

### Supporting
- `src/components/Surveying/SurveyImportManager.vue` — modal for managing imported data sets
- `src/types/database.ts` — shared interfaces (`ZoneAnalytics`, `CategorySpeedBonusStats`, `SurveyTypeAnalytics`, etc.)

### Backend
- `src-tauri/src/db/player_commands_survey_events.rs` — `get_speed_bonus_stats`, `get_zone_analytics`, `get_item_cost_analysis` commands
- `src-tauri/src/db/survey_sharing_commands.rs` — `export_survey_data`, `import_survey_data_from_file`, `get_survey_imports`, `delete_survey_import` commands

## Layout

### 3-Panel PaneLayout

```
┌──────────────┬────────────────────────────────────┬──────────────────┐
│  Left Pane   │  Center Panel                      │  Right Pane      │
│  View Nav    │  Header: stats + controls           │  Item Cost       │
│              │  Active view (scrollable)           │  Calculator      │
│  • Overview  │                                    │                  │
│  • Zones     │  Multi-column card grid             │  (collapsible,   │
│  • Types     │                                    │   default open)  │
└──────────────┴────────────────────────────────────┴──────────────────┘
```

- **Left pane** (default 220px): View navigation with sections for Overview, Zones, and Survey Types. Selected view highlighted with standard list styling. Selection persisted via `useViewPrefs`.
- **Center panel**: Header bar (always visible) + scrollable view area. Views switch via `v-if` based on selected view key.
- **Right pane** (default 400px): Item Cost Calculator. Collapsible, defaults to open.

### View Keys
- `"overview"` — default view
- `"zone:<ZoneName>"` — e.g., `"zone:Eltibule"`
- `"surveytype:<SurveyTypeName>"` — e.g., `"surveytype:Eltibule Green Mineral Survey"`

### Center Panel Views

**Overview** — Global stat summary boxes (total surveys, bonus rate, bonus items, category split), Cross-Zone Comparison table, All Survey Types table. 2-column responsive grid.

**Zone Detail** — Zone summary stats row, then multi-column card grid (`grid-cols-1 xl:grid-cols-2 2xl:grid-cols-3`):
- ZoneRewardsCard per survey type (item loot tables with ItemInline)
- ZoneSpeedBonusCard per category with bonus data
- SurveyTypeDistributionChart (VueUiXy bar chart, shown when 2+ types)
- ProfitRateCard (expected profit/loss per category)

**Survey Type Detail** — Summary stats, per-zone item rewards tables, speed bonus context cards, item quantity chart (VueUiXy).

## Data Sharing

### Export

Exports all local survey data (sessions, events, loot) to a `.glogger-survey` JSON file. Uses the save dialog from `@tauri-apps/plugin-dialog`. Never re-exports imported data — only the user's own sessions.

### Import

Opens a `.glogger-survey` file via the open dialog. Creates a `survey_data_imports` record and inserts all sessions/events/loot with `import_id` set. Shows a toast with the import summary.

### Import Management

The "Imports (N)" button opens `SurveyImportManager.vue`, a modal listing all import batches. Each import shows label, source player, session/event counts, and import date. Users can remove individual imports — deletion cascades through sessions → events → loot.

### Data Source Toggle

A segmented control `[My Data] [All Data]` appears when imports exist. Default is "My Data". Switching triggers a reload of all analytics queries. The toggle state is passed to:
- `get_speed_bonus_stats` (via `includeImports` param)
- `get_zone_analytics` (via `includeImports` param)
- `get_item_cost_analysis` (via `includeImports` prop on ItemCostCalculator)

### Export File Format

```
{
  format: "glogger-survey-export",
  version: 1,
  metadata: { exported_at, exporter_name, session_count, event_count },
  sessions: [
    { start_time, end_time, maps_started, surveys_completed,
      total_revenue, total_cost, total_profit, profit_per_hour,
      elapsed_seconds, speed_bonus_count, survey_types_used, maps_used_summary,
      events: [
        { timestamp, event_type, map_type, survey_type, speed_bonus_earned,
          loot_items: [{ item_name, quantity, is_speed_bonus, is_primary }] }
      ] }
  ]
}
```

### Database Schema (Migration v16)

**`survey_data_imports` table:** Tracks import batches — id, label, source_player, session_count, event_count, imported_at.

**`survey_session_stats.import_id` column:** Nullable FK to `survey_data_imports(id)` with `ON DELETE CASCADE`. NULL = local data, non-NULL = imported.

## Item Cost Calculator

Interactive tool for answering "how much does it cost to get X quantity of an item?" Located in the right pane.

**Controls:** Item dropdown (all items seen in survey loot) + quantity input + sell price input + sort toggle (Cost / Time / Profit)

**Results table columns:**

| Column | Description |
|--------|-------------|
| Survey Type | Which survey drops this item |
| Zone | Zone the survey belongs to |
| Avg Yield | Effective yield per survey (primary + expected bonus contribution) |
| Needed | Number of surveys needed for desired quantity |
| Cost Each | Crafting cost per survey |
| Total Cost | Total gold needed |
| Est. Time | Estimated time based on session averages |
| Profit | Revenue minus total cost (when sell price set) |
| Profit/hr | Profit per hour (when sell price set and time data available) |

Each row includes a sub-detail showing primary vs speed bonus yield breakdown, proc rate, and sample size.

**Effective yield formula:** `primary_avg_per_completion + (bonus_avg_per_proc * speed_bonus_rate / 100)`

**Data source:** `get_item_cost_analysis` command — fetched on mount and when `includeImports` prop changes. All filtering/calculation done client-side.

## Data Loading

- AnalyticsTab loads all data on mount via two parallel `invoke` calls:
  - `get_speed_bonus_stats` (with `sessionId: null, includeImports: bool`)
  - `get_zone_analytics` (with `includeImports: bool`)
- Data is passed as props to all child view components — no additional backend calls per view
- Item Cost Calculator makes its own call to `get_item_cost_analysis` on mount and when `includeImports` changes
- Data source toggle triggers a full reload of all analytics
- Auto-reload when survey session is finalized (via `surveyStore.sessionFinalizedCounter` watcher)
- View selection persisted via `useViewPrefs("survey-analytics.view")` — falls back to "overview" if persisted view no longer exists in data

## Data Structures

### Zone Analytics

```
ZoneAnalytics[]
├── zone: string
├── speed_bonus_stats: CategorySpeedBonusStats[]
│   ├── category: "mineral" | "mining"
│   ├── total_surveys, speed_bonus_count, speed_bonus_rate, avg_bonus_value
│   └── item_stats: SpeedBonusItemStats[]
│       └── item_name, total_quantity, times_seen, total_procs, min/max/avg_per_proc
└── survey_type_stats: SurveyTypeAnalytics[]
    ├── survey_type, category, crafting_cost, total_completed
    └── item_stats: SurveyItemStats[]
        └── item_name, total_quantity, times_seen, min/max/avg_per_completion
```

### Item Source Analysis

```
ItemSourceAnalysis[]
├── item_name, survey_type, zone, category, crafting_cost
├── total_completions
├── primary_total_qty, primary_times_seen, primary_avg_per_completion
├── bonus_total_qty, bonus_times_seen, bonus_avg_per_proc
├── speed_bonus_rate
└── avg_seconds_per_survey (estimated from session averages)
```

### Survey Import Info

```
SurveyImportInfo[]
├── id, label, source_player
├── session_count, event_count
└── imported_at
```
