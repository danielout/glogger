# Unified Search System

Scryfall-inspired structured search across all entity types with a single query syntax. Ref: https://scryfall.com/docs/syntax

## Current State

Phase 1 is complete. The unified search engine replaces the old `useQuickSearch` composable (which fired 4 parallel Tauri commands for 7 categories) with a single `unified_search` Tauri command that searches 11 game data entity types in one pass, plus client-side player data search.

Both the **Quick Search Overlay** (Ctrl+F) and the **Dedicated Search Page** now use the unified engine. In-app syntax help is shown in both UIs when the search input is empty.

The 11 **Data Browser** search UIs still use their own individual search commands and client-side filtering. These are the primary targets for Phase 3 migration.

## Search Syntax

Scryfall-inspired with PG adaptations. Colon handling: filter keys are from a known set (`type`, `skill`, `area`, `level`, etc.) — any colon not preceded by a known key is treated as literal text (handles "Knife: Precision Slash").

```
Plain text           → substring match across all fields
"quoted text"        → exact phrase match
type:item            → restrict to entity type
skill:Sword          → entities associated with a skill
area:Serbule         → entities in a zone
level:30-50          → level range
keyword:Food         → item keyword
has:recipe           → items with recipes
slot:MainHand        → equipment slot
name:sword           → restrict match to name field only
-keyword:NotObtainable → exclusion (negate any filter with -)
```

Multiple filters AND together.

## Architecture

**Rust backend search engine** — single `unified_search` Tauri command. Iterates all entity collections in GameDataState under one read lock. Relevance scoring: exact name=100, starts-with=80, contains=60, description/secondary=40.

**Frontend composable** (`useUnifiedSearch.ts`) — calls backend for game data, searches player data client-side from Pinia stores (backpack, vaults, aggregate inventory, skills, market values), merges and orders results.

**Query parser** — Rust-side in `unified_search.rs`. Tokenizes input, recognizes quoted phrases, known filter keys, and negation prefixes. Returns a `ParsedQuery` struct used by all per-type search functions.

## What's Searchable

| Type | Fields Searched |
|------|-----------------|
| Items | name, description, keywords, effect_descs, food_desc |
| Recipes | name, description, skill, ingredient names, result item names |
| NPCs | name, description, area |
| Quests | name, description, location, favor NPC |
| Skills | name, description |
| Abilities | base_name, tier names, tier descriptions, skill |
| Effects | name, description |
| Enemies | key, comment, strategy |
| Areas | friendly_name |
| Titles | title, tooltip |
| Lorebooks | title, text, category |
| Your Items | item name (client-side, backpack + vaults + aggregate) |
| Your Skills | skill name (client-side) |
| Market Values | item name (client-side) |

## Phases

### Phase 1: Foundation ✓ DONE
- Rust `unified_search.rs` — query parser + search across ALL 11 entity types with relevance scoring
- `useUnifiedSearch.ts` composable replacing `useQuickSearch` (single Tauri command + client-side player data)
- Wire into QuickSearchOverlay + SearchView as drop-in replacement
- Supported syntax: plain text, "quoted phrases", type:, skill:, area:, level:, keyword:, has:, slot:, name:, negation (-keyword:)
- In-app syntax help in both overlay and search page empty states
- Feature documentation at [docs/features/screens/search.md](../features/screens/search.md)

### Phase 2: Enhanced UI
- Syntax highlighting in search input
- Filter autocomplete dropdowns
- Bidirectional filter chips
- Entity type tabs in results

### Phase 3: Migrate Data Browsers ✓ DONE (8 of 11 browsers)
Created a **TypeScript query parser** ([SearchParser.ts](../../src/utils/SearchParser.ts)) and a **shared composable** ([useDataBrowserSearch.ts](../../src/composables/useDataBrowserSearch.ts)) that applies unified search syntax client-side to each browser's preloaded data.

**Approach:** Browsers keep their existing data loading patterns (preload-all or Tauri search) but replace manual `query.filter()` logic with the composable. Dropdown filters compose as pre-filters via computed refs.

**Migrated:**
- SkillBrowser — text-only, direct swap
- TitleBrowser — text-only, direct swap
- LoreBrowser — category dropdown as pre-filter + unified text search
- EnemyBrowser — strategy dropdown as pre-filter + unified text search
- NpcBrowser — area dropdown as pre-filter + unified text search with `area:` filter support
- RecipeBrowser — skill dropdown pre-filter + unified text (skill-selected mode); Tauri search (all-skills mode)
- QuestBrowser — area/NPC/cancellable dropdowns as pre-filter + unified text + sort
- AbilityBrowser — skill dropdown + monster toggle; client-side unified search (skill-selected), Tauri search (all-skills)

**Not migrated (staying as-is):**
- EffectBrowser — lazy Tauri search, no preloaded data to filter client-side
- TsysBrowser — TSys not in unified search entity types
- ItemSearch — most complex (5 filter dimensions), deferred to Phase 3b

### Phase 3b: Remaining Browsers
- ~~ItemSearch.vue~~ ✓ DONE — search bar now accepts `keyword:`, `level:`, `slot:`, `"quoted phrases"`, and `-keyword:` syntax, merged with dropdown filters
- TsysBrowser — needs `tsys` as a searchable entity type in unified_search.rs
- EffectBrowser — could preload all effects to enable client-side filtering

### Phase 4: Advanced
- Search history, saved searches
- Fuzzy matching
- Cross-entity search (e.g., "show recipes that produce items with keyword:Food")
- Rayon parallelism / inverted index if needed

## Integration Audit

Every component with a search bar, filter input, or large list — assessed for unified search migration.

### Data Browser Components (Phase 3 targets)

These are the primary migration targets. Each currently has its own search UI that could be replaced with a shared `SearchPanel` component pre-configured with `type:` filter.

| Component | Current Search | Filters Beyond Text | Difficulty | Notes |
|-----------|---------------|---------------------|------------|-------|
| [SkillBrowser.vue](../../src/components/DataBrowser/SkillBrowser.vue) | Client-side `.filter()` on name + description | None | **Easy** | Straightforward swap — text-only search, no extra filters |
| [TitleBrowser.vue](../../src/components/DataBrowser/TitleBrowser.vue) | Client-side `.filter()` on title + tooltip | None | **Easy** | Same as skills — text-only |
| [LoreBrowser.vue](../../src/components/DataBrowser/LoreBrowser.vue) | Client-side `.filter()` on title + text + location | Category dropdown | **Easy** | Would need a `category:` filter in the parser, or keep the dropdown alongside |
| [EnemyBrowser.vue](../../src/components/DataBrowser/EnemyBrowser.vue) | Client-side `.filter()` on key + comment + abilities | Strategy dropdown | **Easy** | Strategy filter could map to a query filter or stay as dropdown |
| [EffectBrowser.vue](../../src/components/DataBrowser/EffectBrowser.vue) | Tauri `search_effects` | None | **Easy** | Text-only, direct replacement |
| [NpcBrowser.vue](../../src/components/DataBrowser/NpcBrowser.vue) | Client-side `.filter()` + Tauri `search_npcs` | Area dropdown | **Medium** | `area:` filter already supported in unified search |
| [QuestBrowser.vue](../../src/components/DataBrowser/QuestBrowser.vue) | Client-side `.filter()` | Area, NPC, cancellable toggle, sort-by dropdown | **Medium** | Multiple filter dimensions; area/NPC covered by unified search, but cancellable toggle and sort-by are quest-specific |
| [RecipeBrowser.vue](../../src/components/DataBrowser/RecipeBrowser.vue) | Tauri `search_recipes` | Skill dropdown | **Medium** | `skill:` filter already supported; would need to handle skill-based category tabs |
| [AbilityBrowser.vue](../../src/components/DataBrowser/AbilityBrowser.vue) | Client-side `.filter()` | Skill dropdown, monster abilities toggle | **Medium** | `skill:` filter supported; monster toggle needs a `has:` or custom filter |
| [TsysBrowser.vue](../../src/components/DataBrowser/TsysBrowser.vue) | Tauri `search_tsys` | Skill filter | **Medium** | TSys not currently in unified search; would need a new `tsys` entity type |
| [ItemSearch.vue](../../src/components/DataBrowser/ItemSearch.vue) | Tauri `search_items` + SearchParser | Keyword dropdown, equip slot, level range, armor type, effect text | ✓ **Done** | Search bar accepts `keyword:`, `level:`, `slot:`, `"phrases"`, `-keyword:` syntax merged with dropdown filters |

### Crafting Components

| Component | Current Search | Difficulty | Notes |
|-----------|---------------|------------|-------|
| [QuickCalcTab.vue](../../src/components/Crafting/QuickCalcTab.vue) | Tauri `search_recipes` (debounced) | **Medium** | Could use `type:recipe` unified search, but needs recipe selection UX (not navigation) |
| [ProjectRecipePanel.vue](../../src/components/Crafting/ProjectRecipePanel.vue) | Tauri `search_recipes` (cap=20) | **Medium** | Same as QuickCalc — search-to-select, not search-to-navigate |
| [CooksHelperTab.vue](../../src/components/Crafting/CooksHelperTab.vue) | Store-driven filter | **Low priority** | Specialized food filter with skill pills, availability filter, sort — very domain-specific |
| [DynamicItemsTab.vue](../../src/components/Crafting/DynamicItemsTab.vue) | Client-side `.filter()` on keyword-matched items | **Low priority** | Keyword slot picker, checkbox selection — specialized UX |
| [DynamicIngredientPicker.vue](../../src/components/Crafting/DynamicIngredientPicker.vue) | Client-side `.filter()` on keyword-matched items | **Low priority** | Popup picker, deduplication — specialized |

### Character / Build Planner

| Component | Current Search | Difficulty | Notes |
|-----------|---------------|------------|-------|
| [NpcFilterPanel.vue](../../src/components/Character/NpcFilterPanel.vue) | Client-side filter on NPC list | **Low priority** | Multi-dimension filter (group by, sort, favor tier, area, services) — very specialized character view |
| [SlotItemPicker.vue](../../src/components/Character/BuildPlanner/SlotItemPicker.vue) | Tauri `search_items` with filters | **Medium** | Similar to ItemSearch — equip slot, skill, armor type, level range, effect text. Could use unified search but needs slot pre-filtering |
| [GlobalModSearch.vue](../../src/components/Character/BuildPlanner/GlobalModSearch.vue) | Client-side `.filter()` across equipped mods | **Not applicable** | Searches the player's equipped build mods, not game data — stays client-side |

### Other Searches

| Component | Current Search | Difficulty | Notes |
|-----------|---------------|------------|-------|
| [ChatSearchView.vue](../../src/components/Chat/ChatSearchView.vue) | Tauri `get_chat_messages` with `from:` / `in:` syntax | **Not applicable** | Searches chat DB, not game data. Already has its own query parser with `from:` and `in:` filters |
| [MarketView.vue](../../src/components/Market/MarketView.vue) | Tauri `search_items` for adding items | **Low priority** | Item picker for market value entry — could use unified search but it's a minor UX |
| [SearchableSelect.vue](../../src/components/Shared/SearchableSelect.vue) | Client-side `.filter()` | **Not applicable** | Generic dropdown component — stays as-is |
| [GiftWatcherWidget.vue](../../src/components/Dashboard/widgets/GiftWatcherWidget.vue) | Tauri `search_npcs` | **Low priority** | NPC picker for widget config — narrow scope |
| [CriticalResourcesConfig.vue](../../src/components/Dashboard/widgets/CriticalResourcesConfig.vue) | Tauri `search_items` | **Low priority** | Item picker for widget config — narrow scope |

### Recommended Migration Order

1. **Easy wins** — SkillBrowser, TitleBrowser, LoreBrowser, EnemyBrowser, EffectBrowser (5 components, minimal filters, straightforward swap)
2. **Medium browsers** — NpcBrowser, QuestBrowser, RecipeBrowser, AbilityBrowser (4 components, have dropdowns that map to existing unified search filters)
3. **Complex** — ItemSearch (most filters), TsysBrowser (needs new entity type)
4. **Crafting pickers** — QuickCalcTab, ProjectRecipePanel (search-to-select pattern, not search-to-navigate)
5. **Deferred** — ChatSearch (separate DB), build planner pickers (specialized), widget configs (narrow scope)

## Key Files

- [unified_search.rs](../../src-tauri/src/unified_search.rs) — Rust search engine with query parser, relevance scoring, 11 entity type searchers
- [SearchParser.ts](../../src/utils/SearchParser.ts) — TypeScript port of the query parser for client-side use
- [useUnifiedSearch.ts](../../src/composables/useUnifiedSearch.ts) — frontend composable for QuickSearch/SearchView (replaces useQuickSearch)
- [useDataBrowserSearch.ts](../../src/composables/useDataBrowserSearch.ts) — frontend composable for data browsers (client-side filtering with unified syntax)
- [useQuickSearch.ts](../../src/composables/useQuickSearch.ts) — legacy composable (still exists, no longer imported)
- [QuickSearchOverlay.vue](../../src/components/Search/QuickSearchOverlay.vue) — primary UI consumer
- [SearchView.vue](../../src/components/Search/SearchView.vue) — full-page search view
- [cdn_commands.rs](../../src-tauri/src/cdn_commands.rs) — existing search_* commands (still used by EffectBrowser, RecipeBrowser All mode, AbilityBrowser All mode)
- [ItemSearch.vue](../../src/components/DataBrowser/ItemSearch.vue) — most complex browser search (Phase 3b target)
- [search.md](../features/screens/search.md) — feature documentation
