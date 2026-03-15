# Data Browser Reference

The Data Browser is a multi-tab interface for exploring Project: Gorgon's CDN game data. It supports browsing Items, Skills, Abilities, Recipes, Quests, and NPCs with search, filtering, and cross-entity navigation.

## Component Architecture

```
DataBrowser.vue (tab router)
├── ItemSearch.vue       — Item search and detail view
├── SkillBrowser.vue     — Skill browser with related abilities
├── AbilityBrowser.vue   — Skill-filtered ability browser
├── RecipeBrowser.vue    — Recipe browser with ingredients/results
├── QuestBrowser.vue     — Quest browser with advanced filtering
└── NpcBrowser.vue       — NPC browser with area filtering
```

**Location:** [`src/components/DataBrowser/`](../../src/components/DataBrowser/)

### DataBrowser.vue

The parent container that manages tab switching across the six data types. Accepts a `navTarget` prop (via the `provideEntityNavigation` composable) to support cross-entity navigation — clicking an entity reference in another part of the app automatically switches to the correct tab and selects the entity.

### Shared Layout Pattern

All browser components follow the same split-panel layout:

- **Left panel (fixed ~300px):** Search input, optional filter dropdowns, scrollable results list with count
- **Right panel (flexible):** Entity detail view with icon, metadata, sections, and keywords

Each browser also includes:
- Status banners for loading/error states
- Lazy icon loading with memoized paths via `getIconPath()`
- Keyword display

## Data Flow

```
Vue Component
  │ invoke()
  ▼
gameDataStore.ts (Pinia)
  │ invoke()
  ▼
cdn_commands.rs (Tauri commands)
  │ read lock
  ▼
GameDataState (Arc<RwLock<GameData>>)
  │
  ▼
In-memory HashMaps + indices
```

### Backend State

All CDN data lives in a `GameDataState` struct behind `Arc<RwLock<>>` for thread-safe concurrent reads. Data is loaded once at startup from cached JSON files, then queried in-memory — no database round-trips for browsing.

### Cross-Type Indices

The `GameData` struct maintains prebuilt indices for efficient lookups:

| Index | Type | Purpose |
|-------|------|---------|
| `item_name_index` | `HashMap<String, u32>` | Item name → ID |
| `skill_name_index` | `HashMap<String, u32>` | Skill name → ID |
| `recipes_by_skill` | `HashMap<String, Vec<u32>>` | Skill → recipe IDs |
| `recipes_producing_item` | `HashMap<u32, Vec<u32>>` | Item ID → recipes that produce it |
| `recipes_using_item` | `HashMap<u32, Vec<u32>>` | Item ID → recipes that consume it |
| `recipe_name_index` | `HashMap<String, u32>` | Recipe name → ID |
| `npcs_by_skill` | `HashMap<String, Vec<String>>` | Skill → NPC keys that train it |

## Tauri Commands

All commands are async and read from the shared `GameDataState`.

### Items
| Command | Returns | Notes |
|---------|---------|-------|
| `get_item(id)` | `Option<ItemInfo>` | Lookup by ID |
| `get_item_by_name(name)` | `Option<ItemInfo>` | Exact name match |
| `search_items(query, limit?)` | `Vec<ItemInfo>` | Case-insensitive substring, default limit 20 |
| `get_items_batch(ids)` | `HashMap<u32, ItemInfo>` | Bulk lookup for recipe ingredient/result resolution |

### Skills
| Command | Returns | Notes |
|---------|---------|-------|
| `get_all_skills()` | `Vec<SkillInfo>` | Sorted alphabetically |
| `get_skill_by_name(name)` | `Option<SkillInfo>` | Exact match |

### Abilities
| Command | Returns | Notes |
|---------|---------|-------|
| `get_abilities_for_skill(skill)` | `Vec<AbilityInfo>` | Sorted by level |

### Recipes
| Command | Returns | Notes |
|---------|---------|-------|
| `get_recipe_by_name(name)` | `Option<RecipeInfo>` | Exact match |
| `search_recipes(query, limit?)` | `Vec<RecipeInfo>` | Substring search, default limit 50 |
| `get_recipes_for_skill(skill)` | `Vec<RecipeInfo>` | Uses `recipes_by_skill` index |
| `get_recipes_for_item(item_id)` | `Vec<RecipeInfo>` | Recipes producing this item |
| `get_recipes_using_item(item_id)` | `Vec<RecipeInfo>` | Recipes consuming this item |

### Quests
| Command | Returns | Notes |
|---------|---------|-------|
| `get_all_quests()` | `Vec<QuestInfo>` | Sorted by DisplayName |
| `search_quests(query)` | `Vec<QuestInfo>` | Searches DisplayName and Description |
| `get_quest_by_key(key)` | `Option<QuestInfo>` | Lookup by internal key |

### NPCs
| Command | Returns | Notes |
|---------|---------|-------|
| `get_all_npcs()` | `Vec<NpcInfo>` | Sorted by name |
| `search_npcs(query)` | `Vec<NpcInfo>` | Searches name and description |
| `get_npcs_in_area(area)` | `Vec<NpcInfo>` | Filtered by area |

### Icons
| Command | Returns | Notes |
|---------|---------|-------|
| `get_icon_path(icon_id)` | `Result<String>` | Filesystem path; fetches from CDN if not cached |

## Game Data Store

**Location:** [`src/stores/gameDataStore.ts`](../../src/stores/gameDataStore.ts)

The Pinia store wraps all Tauri commands and manages:

- **Status tracking:** `loading` → `ready` / `error` / `empty`
- **Cache status:** Version info, item/skill counts via `CacheStatus` interface
- **Icon memoization:** `iconPaths` record caches resolved icon filesystem paths for the session
- **Event listeners:** Listens for `game-data-ready` and `game-data-error` events from Rust

Frontend components use `convertFileSrc()` from Tauri to convert filesystem paths to displayable URLs.

## Individual Browser Details

### ItemSearch

- Debounced search (250ms) to reduce API calls
- Shows "unobtainable" badges on items
- Detail view: icon, ID, value, stack size, description, keywords, effect descriptions

### SkillBrowser

- Loads and caches all skills on mount
- Real-time client-side filtering by name/description
- **Related Abilities section:** dynamically loads abilities for the selected skill, sorted by level

### AbilityBrowser

- Skill filter dropdown showing only skills that have abilities
- "All Skills" option with deduplication via Map
- Detail view: level, skill, damage type, description, keywords

### RecipeBrowser

- Skill filter dropdown showing only skills with recipes
- **Ingredients section:** stack size, item name (resolved via `getItemsBatch`), consume percentage
- **Results section:** stack size, item name, success percentage
- **XP Rewards:** skill, XP amount, first-time bonus if different
- **Prerequisites:** prerequisite recipe name if required

### QuestBrowser

- Advanced filter panel: Area dropdown, Sort options (Name/Level/Area), Cancellable filter
- Detail view parses the `raw` JSON field with helper functions for:
  - Requirements (QuestCompleted, MinFavorLevel, MinSkillLevel, ActiveCombatSkill)
  - Objectives with type display mapping
  - Rewards: favor, skill XP, items, loot profiles
  - Dialog text (PrefaceText, SuccessText)

### NpcBrowser

- Area filter dropdown with friendly area names
- **Trains Skills:** list of skills the NPC teaches
- **Favor Preferences:** sorted by preference value, color-coded by desire type (Love/Like/Dislike/Hate)
- **Favorite Gift Items:** list of loved items

## Integration Points

### App.vue

Renders `<DataBrowser>` when `currentView === "data-browser"` and passes a `navTarget` prop for cross-entity navigation.

### MenuBar.vue

Includes a "Data Browser" tab option alongside other app views (Skills, Surveying, Characters, Chat, Settings).

### Cross-Entity Navigation

The `provideEntityNavigation` composable enables other parts of the app to navigate into the Data Browser targeting a specific entity. `DataBrowser.vue` watches the `navTarget` prop and maps entity types to tabs via an `entityTypeToTab` dictionary.

## CDN Data Lifecycle

1. **Startup:** `init_game_data()` compares cached version vs remote version
2. **Download:** Fetches fresh data if stale; falls back to cache if offline
3. **Parse:** Loads all 27 JSON files into typed Rust structs with `raw_json` preservation
4. **Index:** Builds cross-type indices for efficient lookups
5. **Serve:** Frontend queries go through Tauri commands to in-memory data
6. **Refresh:** Users can force a full re-download via Settings > Game Data (CDN)

For CDN parsing details, field schemas, and how to add new typed fields, see [`cdn-data-parsing.md`](cdn-data-parsing.md).
