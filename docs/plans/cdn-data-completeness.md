# CDN Data Completeness Plan

## Problem

Our CDN parsers are dropping significant amounts of game data. We have 27 CDN JSON files containing rich game data, but our Rust parsers only capture a fraction of the available fields. This limits what features we can build (e.g., Gourmand tracking needs `FoodDesc` and `SkillReqs` from items, which are read but discarded).

## Current State

### Parser Categories

**Typed parsers (selective fields — data loss):**
- `items.rs` — 10 of 45 fields parsed (35 dropped)
- `skills.rs` — 9 of 29 fields (20 dropped)
- `abilities.rs` — 7 of 72 fields (65 dropped)
- `recipes.rs` — 20 of 42 fields (22 dropped, including `ResultEffects` at 59%)

**Raw JSON pass-throughs (no data loss, no typed access):**
- `effects.rs`, `quests.rs`, `npcs.rs`, `areas.rs`, `attributes.rs`, `ai.rs`, `xp_tables.rs`, `advancement_tables.rs`, `ability_keywords.rs`, `ability_dynamic.rs`, `directed_goals.rs`, `item_uses.rs`, `landmarks.rs`, `lorebooks.rs`, `player_titles.rs`, `sources.rs`, `storage_vaults.rs`, `tsys.rs`

**Database persistence:** Only items, skills, abilities, recipes, npcs, and quests are persisted to SQLite. The rest live only in memory.

### Reference Data

Full field inventories for all 27 CDN files are captured in:
- `docs/reference/cdn-field-schemas.json` — every field, type, frequency, and sample values
- `docs/reference/cdn-gap-analysis.json` — per-file comparison of CDN fields vs parser coverage
- `docs/samples/CDN-full-examples/` — complete CDN data snapshots for offline reference

## Strategy

### Approach: Typed fields for what we use, raw JSON preserved for everything

Rather than trying to type every field across all 45+ CDN schemas (which would be a massive upfront effort and fragile to CDN updates), we should:

1. **Always preserve the full raw JSON** — every parser should store the original `serde_json::Value` alongside typed fields, so we never lose data
2. **Add typed fields incrementally** — as features need specific fields, promote them from raw JSON to typed struct fields
3. **Database stores raw JSON + indexed typed columns** — SQLite tables get a `raw_json TEXT` column, plus typed columns for fields we query/filter on

This means:
- No data loss ever (raw JSON is the source of truth)
- Typed fields give us compile-time safety and efficient queries for the fields we actually use
- New features can access any field immediately via raw JSON, then we can promote hot fields to typed columns later

### Database Schema Pattern

```sql
-- Example: items table evolution
CREATE TABLE items (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    -- Typed columns for fields we query/filter on
    food_desc TEXT,
    equip_slot TEXT,
    internal_name TEXT,
    -- Full raw JSON for everything else
    raw_json TEXT NOT NULL
);
```

## Implementation Phases

### Phase 1: Items (highest immediate value)

Items has the most dropped fields (35) and blocks the Gourmand feature.

**1a. Update `ItemInfo` struct** in `src-tauri/src/game_data/items.rs`:
- Add `raw_json: serde_json::Value` to preserve everything
- Add typed fields for high-value data:
  - `food_desc: Option<String>` — food type/level (575 items)
  - `skill_reqs: Option<HashMap<String, u32>>` — skill gates (5,877 items)
  - `internal_name: Option<String>` — internal ID (100% of items)
  - `equip_slot: Option<String>` — equipment slot (2,959 items)
  - `num_uses: Option<u32>` — usage count (7,170 items)
  - `behaviors: Option<Vec<serde_json::Value>>` — usage behaviors (8,041 items)
  - `bestow_recipes: Option<Vec<String>>` — recipe grants (683 items)
  - `bestow_ability: Option<String>` — ability grants (537 items)
  - `bestow_quest: Option<String>` — quest grants (1,324 items)
  - `bestow_title: Option<u32>` — title grants (552 items)
  - `craft_points: Option<u32>` — crafting XP (2,471 items)
  - `crafting_target_level: Option<u32>` — crafting level (2,469 items)
  - `tsys_profile: Option<String>` — crafting system (2,469 items)

**1b. Update database schema** — add columns to `items` table + `raw_json TEXT`

**1c. Update `cdn_persistence.rs`** — persist new fields

**1d. Update TypeScript types** — `src/types/gameData/items.ts`

**1e. Update frontend commands** — expose new fields through existing Tauri commands

### Phase 2: Skills & Recipes (combat/crafting features)

**Skills** (20 fields dropped):
- Add `raw_json` preservation
- Typed: `combat: bool`, `max_bonus_levels: u32`, `parents: Vec<String>`, `advancement_hints: Option<Value>`, `guest_level_cap: u32`

**Recipes** (22 fields dropped):
- Add `raw_json` preservation
- Typed: `result_effects: Vec<String>`, `usage_delay: Option<f32>`, `reward_skill_xp_drop_off_level: Option<u32>`, `sort_skill: Option<String>`

### Phase 3: Abilities (combat feature enablement)

65 of 72 fields dropped. Key additions:
- `raw_json` preservation
- Typed: `damage_type`, `reset_time`, `target`, `prerequisite`, `is_harmless`, `animation`, `special_info`, `works_underwater`, `works_while_falling`

### Phase 4: Remaining typed parsers

Promote high-value fields from the raw JSON pass-through parsers as features demand them. These are already preserving full data, so this is just adding typed convenience fields.

Priority order based on feature value:
1. **Effects** — combat tooltips, buff display
2. **NPCs** — position data (`Pos` field), service listings
3. **Quests** — objectives, rewards, requirements
4. **Storage Vaults** — slot counts, requirements
5. **Player Titles** — title display
6. **Attributes** — display rules, labels, icons

### Phase 5: Database persistence for remaining types

Currently only 6 types are persisted to SQLite. As we need offline/queryable access to other types, add persistence for them. No rush — in-memory from CDN cache works fine for now.

## Keeping Schemas Up to Date

The Python extraction script (`docs/reference/cdn-field-schemas.json`) should be re-run periodically when the CDN version bumps. The script lives at the repo root and can be run with:

```bash
python scripts/extract_cdn_schemas.py --input docs/samples/CDN-full-examples/ --output docs/reference/cdn-field-schemas.json
```

(Script to be created from the ad-hoc extraction we did — currently exists only as a one-off.)

## Migration Note

Per CLAUDE.md: we're in early prototyping with no backward compat concerns. The v1 migration should be replaced wholesale — drop and recreate all tables with the expanded schema. No migration path needed.
