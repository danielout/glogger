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

## Edge Cases

| Case | Handling |
|------|----------|
| Duplicate import | Skip via UNIQUE constraint, `was_duplicate: true` |
| `XpNeededForNextLevel: -1` | Stored as-is (means max level) |
| Wrong `Report` type | Error returned |
| Empty `Skills`/`NPCs` | Valid — import with 0 entries |
| Recipe with 0 completions | Stored — tracks which recipes are known |
| Stat values with decimals | Stored as REAL |
