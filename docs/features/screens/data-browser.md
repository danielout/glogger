# Data Browser Screen

## Overview

A multi-tab reference browser for exploring all CDN game data. Nine tabs cover the major entity types: Items, Skills, Abilities, Recipes, Quests, NPCs, Effects, Titles, and Treasure (TSys mods). All browsers share a consistent two-panel layout with search/filter on the left and detail view on the right.

## Architecture

### Files

**Frontend (Vue/TS):**
- `src/components/DataBrowser/DataBrowser.vue` — 9-tab container
- `src/components/DataBrowser/ItemSearch.vue` — items browser
- `src/components/DataBrowser/SkillBrowser.vue` — skills browser
- `src/components/DataBrowser/AbilityBrowser.vue` — abilities browser
- `src/components/DataBrowser/RecipeBrowser.vue` — recipes browser
- `src/components/DataBrowser/QuestBrowser.vue` — quests browser
- `src/components/DataBrowser/NpcBrowser.vue` — NPCs browser
- `src/components/DataBrowser/EffectBrowser.vue` — effects browser
- `src/components/DataBrowser/TitleBrowser.vue` — titles browser
- `src/components/DataBrowser/TsysBrowser.vue` — treasure system mods browser

**Store:**
- `gameDataStore` — CDN data loading, entity resolution, query methods, icon caching

**Backend (Rust):**
- `src-tauri/src/cdn_commands.rs` — all Tauri query commands (async, reading from shared `GameDataState`)
- `src-tauri/src/game_data/` — CDN JSON parsing into typed Rust structs with `raw_json` preservation

### Component Hierarchy

```
DataBrowser.vue                     — 9-tab container
├── ItemSearch.vue                  — items with advanced filters
├── SkillBrowser.vue                — skills with abilities
├── AbilityBrowser.vue              — abilities by skill
├── RecipeBrowser.vue               — recipes by skill
├── QuestBrowser.vue                — quests with area/sort filters
├── NpcBrowser.vue                  — NPCs with area filter
├── EffectBrowser.vue               — effects
├── TitleBrowser.vue                — titles with color rendering
└── TsysBrowser.vue                — treasure mods with skill filter + tier breakdown
```

## Shared Patterns

All browsers follow a consistent two-panel layout:

**Left Panel (~280px fixed width):**
- Search input (with optional loading spinner)
- Results list with keyboard navigation via `useKeyboard` composable
- Selected item highlighted with gold left border

**Right Panel (flex):**
- Detail view of selected entity
- Icon display (with loading state and fallback)
- Metadata sections specific to entity type
- Raw JSON view (expandable) at the bottom of every detail panel

**Common behaviors:**
- Keyboard navigation (arrow keys, Enter to select, scroll sync)
- Status banner while CDN data is loading
- Entity resolution via unified `resolve*` store methods
- Icons loaded asynchronously with caching

## Per-Tab Documentation

- [data-browser-items.md](data-browser/data-browser-items.md) — Items
- [data-browser-skills.md](data-browser/data-browser-skills.md) — Skills
- [data-browser-abilities.md](data-browser/data-browser-abilities.md) — Abilities
- [data-browser-recipes.md](data-browser/data-browser-recipes.md) — Recipes
- [data-browser-quests.md](data-browser/data-browser-quests.md) — Quests
- [data-browser-npcs.md](data-browser/data-browser-npcs.md) — NPCs
- [data-browser-effects.md](data-browser/data-browser-effects.md) — Effects
- [data-browser-titles.md](data-browser/data-browser-titles.md) — Titles
- [data-browser-treasure.md](data-browser/data-browser-treasure.md) — Treasure (TSys)

## Search & Filter Summary

| Browser | Search | Filters | Debounce |
|---------|--------|---------|----------|
| Items | Text (name) | Keywords, equipment slot, level range | 250ms |
| Skills | Text (name, description) | — | Computed |
| Abilities | Text + skill selector | Skill dropdown | 250ms (global) |
| Recipes | Text + skill selector | Skill dropdown | 250ms |
| Quests | Text (multi-field) | Area, cancellable, sort options | Computed |
| NPCs | Text (name, description) | Area | Computed |
| Effects | Text (name) | — | 250ms |
| Titles | Text (title, tooltip) | — | Computed |
| Treasure | Text (name, skill, prefix, suffix, slot, key) | Skill | 250ms |

## Data Source & Backend

All data comes from `gameDataStore` which manages the CDN data lifecycle. Data loads on app startup; browsers show loading/error banners until ready. Icons are cached per session in `store.iconPaths`.

### Backend State

All CDN data lives in a `GameDataState` struct behind `Arc<RwLock<>>` for thread-safe concurrent reads. Data is loaded once at startup from cached JSON files, then queried in-memory — no database round-trips for browsing.

### Cross-Type Indices

The `GameData` struct maintains prebuilt indices for efficient lookups:

| Index | Purpose |
|-------|---------|
| `item_name_index` | Item name → ID |
| `skill_name_index` | Skill name → ID |
| `recipes_by_skill` | Skill → recipe IDs |
| `recipes_producing_item` | Item ID → recipes that produce it |
| `recipes_using_item` | Item ID → recipes that consume it |
| `recipe_name_index` | Recipe name → ID |
| `npcs_by_skill` | Skill → NPC keys that train it |

### Cross-Entity Navigation

The `provideEntityNavigation` composable enables other parts of the app to navigate into the Data Browser targeting a specific entity. `DataBrowser.vue` watches a `navTarget` prop and maps entity types to tabs.

## Tauri Commands

### Items
- `get_item(id)` / `get_item_by_name(name)` / `search_items(query, limit?)` / `get_items_batch(ids)`

### Skills
- `get_all_skills()` / `get_skill_by_name(name)`

### Abilities
- `get_abilities_for_skill(skill)`

### Recipes
- `get_recipe_by_name(name)` / `search_recipes(query, limit?)` / `get_recipes_for_skill(skill)` / `get_recipes_for_item(item_id)` / `get_recipes_using_item(item_id)`

### Quests
- `get_all_quests()` / `search_quests(query)` / `get_quest_by_key(key)`

### NPCs
- `get_all_npcs()` / `search_npcs(query)` / `get_npcs_in_area(area)`

### Treasure (TSys)
- `get_all_tsys()` / `search_tsys(query, limit?)` / `get_tsys_profiles()`

### Icons
- `get_icon_path(icon_id)` — filesystem path; fetches from CDN if not cached

## CDN Data Lifecycle

1. **Startup:** `init_game_data()` compares cached version vs remote
2. **Download:** Fetches fresh data if stale; falls back to cache if offline
3. **Parse:** Loads all 27 JSON files into typed Rust structs with `raw_json` preservation
4. **Index:** Builds cross-type indices for efficient lookups
5. **Serve:** Frontend queries go through Tauri commands to in-memory data
6. **Refresh:** Users can force re-download via Settings > Game Data (CDN)

For CDN parsing details and field schemas, see `docs/architecture/cdn-data-parsing.md`.

## Key Design Decisions

- **Read-only reference tool** — the data browser is purely for exploring CDN data, not for editing. Player-specific data (known recipes, skill levels) is shown on the Character screen instead.
- **Inline components for cross-references** — `ItemInline`, `SkillInline`, etc. are used within detail views for clickable cross-references between entities.
- **Raw JSON always available** — every detail view includes an expandable raw JSON block for power users who want to see the full CDN data.
- **Sources panel** — Items, Abilities, Recipes, and Quests show a "Sources" panel listing where the entity can be obtained or what references it.
