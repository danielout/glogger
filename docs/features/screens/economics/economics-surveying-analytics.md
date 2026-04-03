# Economics — Surveying: Analytics Tab

Parent: [economics-surveying.md](economics-surveying.md)

## Overview

The Analytics tab provides all-time aggregate survey statistics organized by zone. It includes an item cost calculator for planning material acquisition, speed bonus analysis, per-survey-type loot distributions, cross-zone comparisons, and speed bonus rate charts.

Players can export their survey data and import data from other players to build community knowledge. A data source toggle switches between viewing personal data only or combined data (personal + imported).

## Files

- `src/components/Surveying/AnalyticsTab.vue` — tab orchestrator, imports sub-components, export/import/toggle controls
- `src/components/Surveying/SurveyImportManager.vue` — modal for viewing and deleting imported data sets
- `src/components/Surveying/Analytics/ItemCostCalculator.vue` — item cost/efficiency calculator
- `src/components/Surveying/Analytics/SpeedBonusChart.vue` — bar chart of bonus rates by zone
- `src/components/Surveying/Analytics/CrossZoneComparison.vue` — zone comparison table
- `src-tauri/src/db/player_commands_survey_events.rs` — `get_speed_bonus_stats`, `get_zone_analytics`, `get_item_cost_analysis` commands
- `src-tauri/src/db/survey_sharing_commands.rs` — `export_survey_data`, `import_survey_data_from_file`, `get_survey_imports`, `delete_survey_import` commands

## Layout Order

1. Header + Data source toggle + Export/Import/Manage buttons + Refresh
2. All-Time Overview (summary card, label changes based on toggle)
3. **Item Cost Calculator** (prominent card)
4. Zone Accordions (existing per-zone breakdowns)
5. **Speed Bonus Rates by Zone** (accordion, shown when 2+ zones)
6. **Cross-Zone Comparison** (accordion, shown when 2+ zones)
7. Empty state
8. Import Manager modal (on demand)

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

## All-Time Overview

A summary card with five global metrics:

| Metric | Description |
|--------|-------------|
| Total Surveys | All-time survey completions |
| Bonuses Earned | Total speed bonus procs |
| Bonus Rate | Percentage of surveys that earned a speed bonus |
| Bonus Items | Total items received from speed bonuses |
| Zones Active | Number of distinct zones with survey data |

### Item Cost Calculator

Interactive tool for answering "how much does it cost to get X quantity of an item?"

**Controls:** Item dropdown (all items seen in survey loot) + quantity input + sort toggle (Cost / Time)

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

Each row includes a sub-detail showing primary vs speed bonus yield breakdown, proc rate, and sample size.

**Effective yield formula:** `primary_avg_per_completion + (bonus_avg_per_proc * speed_bonus_rate / 100)`

**Data source:** `get_item_cost_analysis` command — fetched on mount and when `includeImports` prop changes. All filtering/calculation done client-side.

### Zone Accordions

Expandable `AccordionSection` per zone (auto-expanded when only one zone). Within each zone, data split by category (mineral vs mining):

**Speed Bonus Items Table** (per category): Item, Total, Seen, Min, Max, Avg, Out of (procs)

**Per-Survey-Type Breakdown**: Survey type name, completions, crafting cost, and loot item table with min/max/avg per completion.

### Speed Bonus Rates by Zone

Bar chart (`VueUiXy`) comparing speed bonus rates across zones for the selected category (mineral/mining toggle). Zones sorted by bonus rate descending.

### Cross-Zone Comparison

Comparison table with one row per zone for the selected category:

| Column | Description |
|--------|-------------|
| Zone | Zone name |
| Surveys | Total completions |
| Bonus Rate | Speed bonus proc percentage |
| Avg Bonus Val | Average value per speed bonus proc |
| Avg Cost | Weighted average crafting cost per survey |
| Survey Types | Number of distinct survey types used |
| Profit/Survey | Expected bonus value per survey minus avg cost |

Sortable by any column. Profit is color-coded green/red.

## Data Loading

- All-Time Overview + Zone data loads on mount via two parallel `invoke` calls:
  - `get_speed_bonus_stats` (with `sessionId: null, includeImports: bool` for all-time)
  - `get_zone_analytics` (with `includeImports: bool`) — zone-grouped data passed as props to SpeedBonusChart and CrossZoneComparison
- Item Cost Calculator makes its own call to `get_item_cost_analysis` on mount and when `includeImports` changes
- Data source toggle triggers a full reload of all analytics
- Manual refresh button re-fetches the overview + zone data

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
