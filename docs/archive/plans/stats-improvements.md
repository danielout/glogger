# Character Stats Page Improvements

- Page doc: docs\features\screens\character\character-stats.md

## Done

### 3-Column Card Layout
Restructured the stats tab from a vertical dump to a 3-column card grid:
- Column 1: Skills (full height, filterable, sortable)
- Column 2: Combat Stats (top, flex) + Currencies (bottom, 30% max)
- Column 3: Computed Stats (top, shrink) + Report Stats (bottom, flex)

Each card has internal scrolling and headers.

### Computed Stats (v1)
Added `get_computed_stats` Tauri command and `ComputedStatsCard.vue` with a manual "Refresh" button:
- Total level / base levels / bonus levels / skills known
- Total XP earned across all skills
- Items crafted (excludes Lore, Teleportation, Augmentation, Transmutation skills)
- Items distilled (Transmutation minus Repair recipes)
- Items deconstructed (Augmentation Decompose/Distill)
- Times teleported (Teleportation minus Bind recipes)
- Dye crafts (Dye Making skill or DyeRecipe keyword)
- Total time watching crafting bars fill (completion_count × UsageDelay from CDN)

### Rate Stats
Cross-references `character_report_stats` (from /age report) with computed data:
- Time Played (parsed from "X days Y hours" format)
- XP per hour (total XP earned ÷ hours played)
- Kills per hour (from /age kills ÷ hours played)
- Deaths per hour (from /age deaths ÷ hours played)
- Gracefully hidden when /age report hasn't been opened in-game yet

### Per-Skill Crafting Breakdowns
Groups recipe completions by CDN skill name, showing:
- Total crafted count per skill
- Total bar-fill time per skill (from UsageDelay)
- Sorted by most crafted first

### Attribute Ranges (Min/Max Tracking)
- Migration v35 adds `min_value`/`max_value` columns to `game_state_attributes`
- The attribute upsert handler now tracks lifetime min and max alongside current value
- Computed stats includes an "Attribute Ranges" section showing attributes that have changed
- Displayed as color-coded: blue (min) / gold (current) / green (max)
- Only attributes where min != max are shown (filters out constants)

### Auto-Refresh
- Computed stats now auto-load on mount (no manual click needed on first visit)
- Auto-refreshes when `skills`, `recipes`, `report_stats`, or `attributes` game-state domains update
- Manual "Refresh" button still available for on-demand recomputation
