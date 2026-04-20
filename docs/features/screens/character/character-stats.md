# Character — Stats

## Overview

The Stats tab manages character report imports (`/outputcharacter`) and provides browsable, comparable snapshots of a character's skills, stats, currencies, and recipe completions at various points in time.

## Architecture

### Files

**Backend (Rust):**
- `src-tauri/src/db/character_commands.rs` — import, snapshot queries, comparison

**Frontend (Vue/TS):**
- Inline in `src/components/Character/CharacterView.vue` (stats tab section)
- `src/components/Character/SnapshotList.vue` — snapshot selector
- `src/components/Character/SnapshotComparison.vue` — diff two snapshots
- `src/components/Character/SkillTable.vue` — skill levels from report
- `src/components/Character/StatsTable.vue` — combat/attribute stats
- `src/components/Character/CurrencyTable.vue` — currency holdings
- `src/components/Character/RecipeTable.vue` — known recipe completions
- `src/stores/characterStore.ts` — snapshot management, import

## How It Works

### Import Flow

1. Player runs `/outputcharacter` in-game, producing `Character_{Name}_{Server}.json`
2. User clicks "Import Report" (or auto-import detects the file)
3. Rust parses JSON, validates `Report == "CharacterSheet"`, inserts snapshot + all child data in a transaction
4. Deduplication: `ON CONFLICT DO NOTHING` on `(character_name, server_name, snapshot_timestamp)`
5. Import also seeds `game_state_skills`, `game_state_favor`, and other game state tables

### Snapshot Browsing

- Dropdown to select from all imported snapshots for the active character
- Each snapshot shows: skill levels/XP, combat stats, currencies, recipe completions
- Snapshot comparison: select two snapshots to see skill diffs (level changes, XP changes)

## Character Report Format

### Top-Level Fields

| Field | Type | Description |
|-------|------|-------------|
| `Character` | string | Character name |
| `ServerName` | string | Server name |
| `Timestamp` | string | ISO-ish: `"2026-03-08 14:56:31Z"` |
| `Report` | string | Always `"CharacterSheet"` |
| `Race` | string | Character race |
| `Skills` | object | Skill name → `{ Level, BonusLevels, XpTowardNextLevel, XpNeededForNextLevel, Abilities? }` |
| `RecipeCompletions` | object | Recipe internal name → completion count |
| `CurrentStats` | object | Stat key → numeric value |
| `Currencies` | object | Currency key → amount |
| `NPCs` | object | NPC key → `{ FavorLevel }` |
| `ActiveQuests` | string[] | Active quest internal names (stored in raw_json only) |

## Database Tables

- **`character_snapshots`** — one row per import (character_name, server_name, timestamp, race, raw_json). Unique on (character_name, server_name, snapshot_timestamp).
- **`character_skill_levels`** — per skill per snapshot (level, bonus_levels, xp_toward_next, xp_needed_for_next)
- **`character_npc_favor`** — per NPC per snapshot (npc_key, favor_level)
- **`character_recipe_completions`** — per recipe per snapshot (recipe_key, completions)
- **`character_stats`** — per stat per snapshot (stat_key, value as REAL)
- **`character_currencies`** — per currency per snapshot (currency_key, amount)

## Tauri Commands

| Command | Purpose |
|---------|---------|
| `import_character_report` | Parse JSON file, insert snapshot + all child data |
| `get_character_snapshots` | List snapshots for a character (ordered by timestamp DESC) |
| `get_snapshot_skills` | All skill levels for a snapshot |
| `get_snapshot_npc_favor` | All NPC favor for a snapshot |
| `get_snapshot_recipes` | All recipe completions for a snapshot |
| `get_snapshot_stats` | All stats for a snapshot |
| `get_snapshot_currencies` | All currencies for a snapshot |
| `get_characters` | List all known characters (distinct name+server pairs) |
| `compare_snapshots` | Compare skills between two snapshots → `Vec<SkillDiff>` |

## Report Stats (Live)

In addition to snapshot-based stats from `/outputcharacter`, the Stats tab displays **live report stats** parsed from in-game reports. When a player opens their behavior report or age report in-game, the content flows through `ProcessBook` → coordinator → `report_stats::parse_*` → `character_report_stats` table automatically.

### Data Sources

| Report | book_type | Stats Extracted |
|--------|-----------|-----------------|
| Age Report | `PlayerAge` | Created on, time played, deaths, attacks, kills, damage dealt/taken, VIP days |
| Behavior Report | `HelpScreen` | Challenge restrictions (with counts), behavior badges, food stats, killing stats by species, misc stats (time online, friends, burials, logins) |

### Storage

Stats are stored in `character_report_stats` as key-value pairs:

| Column | Type | Description |
|--------|------|-------------|
| `character_name` | TEXT | Character name |
| `server_name` | TEXT | Server name |
| `category` | TEXT | Group: `age`, `challenges`, `badges`, `food_stats`, `killing_stats`, `misc_stats` |
| `stat_name` | TEXT | Snake_case key, e.g. `killed_foes_any`, `time_spent_online` |
| `stat_value` | TEXT | Cleaned value: numbers stripped of commas, durations as-is, badges as `"true"` |
| `updated_at` | TEXT | When the report was last processed |

Upserts on `(character_name, server_name, category, stat_name)` — opening the report again updates values.

### UI

`ReportStatsSection.vue` displays below the snapshot tables on the Stats tab:
- Groups stats by category with section headers
- Filterable search across names, categories, and values
- Numbers formatted with locale separators, badges shown as checkmarks
- Shows "Last updated" timestamp
- Auto-refreshes when `report_stats` domain updates

### Key Files

- `src-tauri/src/report_stats.rs` — Parsers for PlayerAge and Behavior Report content
- `src-tauri/src/db/game_state_commands.rs` — `get_character_report_stats` query command
- `src/components/Character/ReportStatsSection.vue` — Frontend component

## Computed Stats

The Stats tab includes a **Computed Stats** card that calculates derived statistics by joining game state data with CDN recipe metadata. Auto-refreshes on `skills`, `recipes`, or `report_stats` domain updates, and also loads on mount.

### Stats Computed

**Skill Totals:**

| Stat | Source |
|------|--------|
| Total Level (base + bonus + synergy) | `game_state_skills` SUM |
| Total Base Levels | `game_state_skills` SUM(level) |
| Total Bonus Levels | `game_state_skills` SUM(bonus_levels) |
| Skills Known | `game_state_skills` COUNT |
| Total XP Earned | `game_state_skills` SUM(xp) |

**Rate Stats** (requires `/age` report opened in-game):

| Stat | Source |
|------|--------|
| Time Played | `character_report_stats` age/time_played, parsed from "X days Y hours" |
| XP / Hour | total_xp_earned ÷ hours_played |
| Kills / Hour | age/kills ÷ hours_played |
| Deaths / Hour | age/deaths ÷ hours_played |

**Crafting Activity:**

| Stat | Source |
|------|--------|
| Items Crafted | `game_state_recipes` JOIN `recipes` — excludes Lore, Teleportation, Augmentation, Transmutation |
| Items Distilled | Transmutation recipes excluding Repair |
| Items Deconstructed | Augmentation Decompose/Distill recipes |
| Times Teleported | Teleportation recipes excluding Bind |
| Dye Crafts | Dye Making skill or DyeRecipe keyword |
| Time Watching Bars Fill | SUM(completion_count × usage_delay) across all recipes |

**Crafting By Skill:** Per-skill breakdown showing total crafts and time spent, sorted by most crafted. Each row shows the skill name, craft count, and bar-fill duration.

**Attribute Ranges:** Tracks the min/max/current value of every attribute that has changed. Attributes that never changed (min == max) are filtered out. Displayed as color-coded min / current / max values. Requires migration v35 (`min_value`/`max_value` columns on `game_state_attributes`). Updated live as `ProcessSetAttributes` events arrive.

### Key Files

- `src-tauri/src/db/game_state_commands.rs` — `get_computed_stats` command, `ComputedStats` + `SkillCraftingBreakdown` + `AttributeExtreme` structs
- `src/components/Character/ComputedStatsCard.vue` — Frontend component

## Layout

The Stats tab uses a **3-column card layout** with internal scrolling per card:

| Column 1 | Column 2 | Column 3 |
|----------|----------|----------|
| Combat Stats (flex) | Skills (flex) | Computed Stats (flex) |
| Player Attributes (flex) | Currencies (30% max) | Character Report Stats (flex) |

## Edge Cases

| Case | Handling |
|------|----------|
| Duplicate import | Skip via UNIQUE constraint, `was_duplicate: true` |
| `XpNeededForNextLevel: -1` | Stored as-is (means max level) |
| Wrong `Report` type | Error returned |
| Empty `Skills`/`NPCs` | Valid — import with 0 entries |
| Recipe with 0 completions | Stored — tracks which recipes are known |
| Stat values with decimals | Stored as REAL |
