# Unified Search System

Scryfall-inspired structured search across all entity types with a single query syntax.

## Current State

Three search layers exist, each implemented differently:
- **Quick Search** (`useQuickSearch.ts`): searches 7 categories via parallel Tauri commands, name-only, caps at 5 results
- **Search View**: full-page version of quick search, cap=50
- **11 Data Browser searches**: each reimplements search UI with different fields and patterns

Key problems: quick search misses 6 entity types, item search is name-only everywhere, no structured query syntax, each browser reinvents search UI, no relevance ranking.

## Search Syntax

Scryfall-inspired with PG adaptations. Colon handling: filter keys are from a known set (`type`, `skill`, `area`, `level`, etc.) — any colon not preceded by a known key is treated as literal text (handles "Knife: Precision Slash").

```
Plain text           → fuzzy/substring match across all fields
"quoted text"        → exact phrase match
type:item            → restrict to entity type
skill:Sword          → entities associated with a skill
area:Serbule         → entities in a zone
level:30-50          → level range
keyword:Food         → item keyword
has:recipe           → items with recipes
-keyword:NotObtainable → exclusion
```

Multiple filters AND together.

## Architecture

**Rust backend search engine** — single `unified_search` Tauri command replacing 4+ parallel commands. Iterates all entity collections in GameDataState. Relevance scoring: exact name=100, starts-with=80, contains=60, description=40, keyword=20.

**Frontend composable** (`useUnifiedSearch.ts`) — calls backend for game data, searches player data client-side from Pinia stores, merges results.

**Query parser** (`SearchParser.ts`) — shared between frontend (filter chips, autocomplete) and backend (query execution).

## What's Searchable

| Type | Fields |
|------|--------|
| Items | name, description, keywords, effect_descs, food_desc, equip_slot |
| NPCs | name, desc, area |
| Skills | name |
| Quests | name, description, area, NPC, objectives |
| Recipes | name, skill, ingredients, result items |
| Abilities | base_name, tier names, descriptions, skill |
| Effects | name, desc |
| Enemies | key, comment, strategy, abilities |
| Areas | friendly_name |
| + Lorebooks, Titles, TSys, Market Values, Player Inventory/Skills |

## Phases

### Phase 1: Foundation (highest value)
- Rust `unified_search.rs` — query parser + search across ALL types
- `useUnifiedSearch.ts` composable replacing `useQuickSearch`
- Wire into QuickSearchOverlay as drop-in replacement

### Phase 2: Enhanced UI
- Syntax highlighting in search input
- Filter autocomplete dropdowns
- Bidirectional filter chips
- Entity type tabs in results

### Phase 3: Migrate Data Browsers
- Shared `SearchPanel.vue` component
- Each browser switches from custom search to SearchPanel with pre-applied `type:` filter
- Migration order: simple browsers first (Skills, Titles) → complex last (Items)

### Phase 4: Advanced
- Search history, saved searches
- Fuzzy matching
- Cross-entity search
- Rayon parallelism / inverted index if needed

## Key Files

- [useQuickSearch.ts](../../src/composables/useQuickSearch.ts) — current composable to replace
- [QuickSearchOverlay.vue](../../src/components/Search/QuickSearchOverlay.vue) — primary UI consumer
- [cdn_commands.rs](../../src-tauri/src/cdn_commands.rs) — existing search_* commands to unify
- [ItemSearch.vue](../../src/components/DataBrowser/ItemSearch.vue) — most complex browser search
