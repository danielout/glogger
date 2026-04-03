# Economics — Surveying: Historical Tab

Parent: [economics-surveying.md](economics-surveying.md)

## Overview

The Historical tab lets users browse and review past survey sessions. It loads pre-computed session summaries from the database and supports expanding individual sessions to view detailed loot breakdowns. Session names and notes are editable inline.

## Files

- `src/components/Surveying/HistoricalTab.vue` — full tab implementation (self-contained)
- `src-tauri/src/db/player_commands_survey_events.rs` — `get_historical_sessions`, `get_loot_breakdown`, `update_survey_session` commands

## Layout

### Aggregate Stats Bar

A row of five summary cards across the top (visible when sessions exist):

| Card | Description |
|------|-------------|
| Sessions | Total number of historical sessions |
| Total Surveys | Sum of completions across all sessions |
| Total Profit | Aggregate profit across all sessions (green/red) |
| Avg Profit/Survey | Total profit divided by total surveys |
| Best Session | Highest profit-per-hour from any single session |

### Session List

Each session is a collapsible card showing a summary row and expandable detail view.

**Summary Row** (always visible):
- Expand/collapse arrow
- Editable session name (inline input)
- Start date/time
- Duration
- Survey types used (truncated label)
- Survey count
- Total profit (green/red)
- Profit per hour

**Expanded Detail** (on click):

Two-column layout:

*Left column (w-52):*
- **Stats** — duration, maps crafted, surveys completed
- **XP Gained** — Surveying, Mining, Geology XP (each with skill-specific color)
- **Economics** — revenue, cost, profit, profit/hour
- **Notes** — editable textarea, persisted via `update_survey_session`

*Right column:*
- **Survey Rewards** — primary loot table with columns: Item (`ItemInline`), Total quantity, Drop count. Sorted by quantity descending.
- **Speed Bonus** — bonus loot table (same columns) with speed bonus proc count header. Only shown when bonus loot exists.
- **Maps Used** — chip list of survey map types used in the session (parsed from `maps_used_summary` field).

## Data Loading

- Sessions load on mount via `get_historical_sessions` (limit 50, ordered by most recent)
- Loot data loads lazily on expand via `get_loot_breakdown` for the specific session ID
- Loot is cached client-side in a `Record<number, LootBreakdownEntry[]>` — expanding the same session twice doesn't re-fetch
- Manual refresh button re-fetches the session list

## Editing

Session names and notes are editable directly in the list:
- **Name** — inline `<input>` in the summary row; triggers `update_survey_session` on change
- **Notes** — `<textarea>` in the expanded detail; triggers `update_survey_session` on change

Both persist immediately to the database.
