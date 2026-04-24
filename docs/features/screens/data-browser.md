# Data Browser

## Overview

A multi-tab reference browser for exploring all CDN game data, presented as a **popup overlay** that can be opened from any screen. Eleven tabs cover the major entity types: Items, Skills, Abilities, Recipes, Quests, NPCs, Enemies, Effects, Lorebooks, Titles, and Treasure (TSys mods). The overlay uses a three-panel layout: search/filter list on the left (from each browser's own PaneLayout), detail view in the center, and a sidebar on the right with History, Favorites, and Pinned tabs.

**Opening the Data Browser:** Click "Data Browser" in the nav bar, press `Ctrl+D`, or click any entity inline link (ItemInline, SkillInline, etc.) to open the overlay targeting that entity.

## Architecture

### Files

**Frontend (Vue/TS):**
- `src/components/DataBrowser/DataBrowserOverlay.vue` — teleported overlay shell with type selector tabs
- `src/components/DataBrowser/DataBrowserSidebar.vue` — right panel with History/Favorites/Pinned tabs
- `src/components/DataBrowser/ItemSearch.vue` — items browser
- `src/components/DataBrowser/SkillBrowser.vue` — skills browser
- `src/components/DataBrowser/AbilityBrowser.vue` — abilities browser
- `src/components/DataBrowser/RecipeBrowser.vue` — recipes browser
- `src/components/DataBrowser/QuestBrowser.vue` — quests browser
- `src/components/DataBrowser/NpcBrowser.vue` — NPCs browser
- `src/components/DataBrowser/EnemyBrowser.vue` — enemies (AI) browser
- `src/components/DataBrowser/EffectBrowser.vue` — effects browser
- `src/components/DataBrowser/LoreBrowser.vue` — lorebooks browser (book reader with category filter)
- `src/components/DataBrowser/TitleBrowser.vue` — titles browser
- `src/components/DataBrowser/TsysBrowser.vue` — treasure system mods browser

**Stores:**
- `src/stores/dataBrowserStore.ts` — overlay state (open/close, active type), favorites, history, persistence
- `src/stores/gameDataStore.ts` — CDN data loading, entity resolution, query methods, icon caching
- `src/stores/referenceShelfStore.ts` — pinned entities (used by the Pinned tab)

**Backend (Rust):**
- `src-tauri/src/cdn_commands.rs` — all Tauri query commands (async, reading from shared `GameDataState`)
- `src-tauri/src/game_data/` — CDN JSON parsing into typed Rust structs with `raw_json` preservation

### Component Hierarchy

```
DataBrowserOverlay.vue              — teleported overlay, type tabs, ESC/Ctrl+D
├── ItemSearch.vue                  — items with advanced filters
├── SkillBrowser.vue                — skills with abilities
├── AbilityBrowser.vue              — abilities by skill
├── RecipeBrowser.vue               — recipes by skill
├── QuestBrowser.vue                — quests with area/sort filters
├── NpcBrowser.vue                  — NPCs with area filter
├── EnemyBrowser.vue                — enemies (AI) with strategy filter
├── EffectBrowser.vue               — effects
├── LoreBrowser.vue                 — lorebooks with category filter + book reader
├── TitleBrowser.vue                — titles with color rendering
├── TsysBrowser.vue                 — treasure mods with skill filter + tier breakdown
└── DataBrowserSidebar.vue          — History / Favorites / Pinned tabs
```

## Layout

The overlay is teleported to `<body>` with `position: fixed; inset: 1rem` (nearly full screen). It contains:

1. **Top bar** — type selector tabs (Items, Skills, Abilities, ...) + close button + ESC hint
2. **Body** — flex row:
   - **Browser area** (flex-1): each browser's own PaneLayout provides the left panel (search/filters/list) and center panel (detail view). Browsers use `v-if`/`v-show` (visited set pattern) to preserve filter state across tab switches.
   - **Sidebar** (w-72, fixed): three tabs for History, Favorites, and Pinned entities.

## Sidebar Tabs

### History
- Automatically populated when any entity is selected in any browser
- Shows the most recent N entries (default 30, configurable in Settings > Advanced)
- Each entry shows entity type badge, label, and relative timestamp
- Click navigates to that entity within the overlay

### Favorites
- Every browser detail view has a star (&#x2605;) button to toggle favorites
- Searchable by name, filterable by entity type chips
- Persisted across sessions in `settings.viewPreferences.dataBrowser`

### Pinned
- Mirrors the Reference Shelf pins (from `referenceShelfStore`)
- Click navigates within the overlay
- Individual unpin buttons + "Clear all pins" action

## Shared Patterns

All browsers follow a consistent two-panel layout (via their own PaneLayout):

**Left Panel (~360px fixed width):**
- Search input (with optional loading spinner)
- Results list with keyboard navigation via `useKeyboard` composable
- Selected item highlighted with gold left border

**Right Panel (flex):**
- Detail view of selected entity
- Icon display (with loading state and fallback)
- Favorite star button + close button in header
- Metadata sections specific to entity type
- Raw JSON view at the bottom of every detail panel (hidden by default; enable via Settings > Advanced > "Show Raw JSON in Data Browser")

**Common behaviors:**
- Keyboard navigation (arrow keys, Enter to select, scroll sync)
- Status banner while CDN data is loading
- Entity resolution via unified `resolve*` store methods
- Icons loaded asynchronously with caching
- History tracking on entity selection
- Favorite toggle in detail header

## Per-Tab Documentation

- [data-browser-items.md](data-browser/data-browser-items.md) — Items
- [data-browser-skills.md](data-browser/data-browser-skills.md) — Skills
- [data-browser-abilities.md](data-browser/data-browser-abilities.md) — Abilities
- [data-browser-recipes.md](data-browser/data-browser-recipes.md) — Recipes
- [data-browser-quests.md](data-browser/data-browser-quests.md) — Quests
- [data-browser-npcs.md](data-browser/data-browser-npcs.md) — NPCs
- [data-browser-enemies.md](data-browser/data-browser-enemies.md) — Enemies (AI)
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
| Enemies | Text (key, comment, abilities) | Strategy | Computed |
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
| `npc_favor_by_item_name` | Item name (lowercase) → NPCs with favor preference |
| `npc_favor_by_keyword` | Item keyword → NPCs with favor preference |
| `quests_by_npc` | NPC key → quest keys (via FavorNpc) |
| `quests_by_work_order_skill` | Skill → work order quest keys |
| `recipes_by_ingredient_keyword` | Item keyword → recipe IDs accepting that keyword |

### Cross-Entity Navigation

The `provideEntityNavigation` composable enables other parts of the app to open the Data Browser overlay targeting a specific entity. `App.vue` calls `dataBrowserStore.open(tab)` and sets a `navTarget` on the `DataBrowserOverlay` ref, which routes to the correct browser tab. Clicking any entity inline link (ItemInline, SkillInline, etc.) opens the overlay and navigates to the entity.

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

### Enemies (AI)
- `get_all_enemies()` / `search_enemies(query)` / `get_enemy(key)`

### Treasure (TSys)
- `get_all_tsys()` / `search_tsys(query, limit?)` / `get_tsys_profiles()`

### Cross-References
- `get_npcs_wanting_item(item_id)` — NPCs with favor preference for item (by name and keyword match)
- `get_npcs_training_skill(skill)` — NPCs that train a given skill
- `get_quests_for_npc(npc_key)` — quests associated with an NPC (via FavorNpc)
- `get_quests_for_skill(skill)` — work order quests for a skill
- `get_recipes_for_keyword(keyword)` — recipes accepting keyword-based ingredients

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
- **Inline components for cross-references** — `ItemInline`, `SkillInline`, `RecipeInline`, `QuestInline`, `NpcInline`, `AreaInline` are used within detail views for clickable cross-references between entities. Each supports tooltips on hover and click-to-navigate.
- **Bidirectional cross-references** — browsers show both forward and reverse relationships. For example, Items show "Produced By" (recipes → item), "Used In" (item → recipes), "NPC Favor" (item → NPCs who want it), and "Could Fill Keyword Slots In" (item keywords → keyword-based recipe ingredients). Skills show "Trained By" (NPCs), "Recipes", and "Work Order Quests".
- **Raw JSON opt-in** — every detail view includes a raw JSON block, but it is hidden by default. Users can enable it via Settings > Advanced > "Show Raw JSON in Data Browser" (`showRawJsonInDataBrowser` in `settingsStore`).
- **Sources panel** — Items, Abilities, Recipes, and Quests show a "Sources" panel listing where the entity can be obtained or what references it.
