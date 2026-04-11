# Search

Search provides two interfaces for finding anything in the app: a **quick search overlay** (Ctrl+F) and a **dedicated search page**.

## Components

- **`QuickSearchOverlay.vue`** (`src/components/Search/QuickSearchOverlay.vue`) — Command-palette modal, teleported to body. Keyboard navigable (arrow keys + Enter), grouped results with per-category cap (default 5), escape/click-outside to dismiss.
- **`SearchView.vue`** (`src/components/Search/SearchView.vue`) — Full search page with category filter toggles, collapsible result sections, no per-category cap. Uses `PaneLayout`.
- **`useQuickSearch`** (`src/composables/useQuickSearch.ts`) — Core composable shared by both components. Accepts a query ref and optional cap, returns grouped `SearchCategory[]` and a loading flag. Debounces via watcher with a version counter to discard stale results.

## Quick Search Overlay (Ctrl+F)

A command-palette-style modal that appears centered over the current view. Does not navigate away — results are clickable and navigate to the relevant view/tab.

- Opens on **Ctrl+F** globally (listener registered in `App.vue`). Escape or clicking outside dismisses.
- Single text input with instant results grouped by category, capped at 5 per category.
- Results use shared inline components (`ItemInline`, `SkillInline`, `RecipeInline`, `NpcInline`, `QuestInline`) for hover tooltips. Click propagation is stopped on the inline wrapper so the overlay's own navigation handler fires instead of the inline's built-in entity nav.
- Keyboard: arrow keys to move through results (flat index across categories), Enter to select.

## Dedicated Search Page

The "Search" nav item in the menu bar. For broader, more detailed exploration.

- Search bar auto-focused on page load.
- Filter toggles (pill buttons) for each category — all enabled by default. Shows match count per category.
- Results grouped by category in collapsible cards, no per-category cap (uses cap=50).
- Results use shared inline components with icons for richer display.
- Clicking a result navigates to the appropriate view via the same `handleSearchNavigate` handler in `App.vue`. For entity-type results (items, skills, NPCs, etc.), this opens the Data Browser overlay rather than switching views.

## Search Categories

Player data (synchronous from store refs) always appears above game data (async Tauri invocations):

| Category | Data Source | Detail Shown |
|---|---|---|
| Your Items | `gameStateStore.ownedItemCounts`, `storageByVault`, `aggregateStore.inventory` | Count, location (Backpack / vault NPC name / character name), market value |
| Your Skills | `gameStateStore.skills` | Level, XP |
| Game Items | `gameDataStore.searchItems()` | Keywords |
| Game Recipes | `gameDataStore.searchRecipes()` | Skill, level requirement |
| NPCs | `gameDataStore.searchNpcs()` | Area name |
| Quests | `gameDataStore.searchQuests()` | Displayed location |
| Market Values | `marketStore.values` | Market price |

### Your Items Search Priority

Items are deduplicated across three sources with this priority:

1. **Backpack / live inventory** — from `ownedItemCounts` (merged DB inventory + live tracking)
2. **Vault storage** — from `storageByVault`, with stacks aggregated per item and vault keys resolved to NPC friendly names via `storageVaultsByKey`
3. **Other characters** — from `aggregateStore.inventory`, showing which characters hold the item

## Navigation from Search Results

Results carry a `SearchNavigation` object with `view`, `subTab`, and optionally `entityType` + `entityId`.

- **Entity results** (Game Items, Game Recipes, NPCs, Quests): routed through the entity navigation system — switches to Data Browser with the correct tab and passes a nav target.
- **Player data results** (Your Items, Your Skills, Market Values): navigates to the target view and activates the sub-tab directly.

Navigation is handled by `handleSearchNavigate` in `App.vue`, which covers both paths.

## Future Work

- **Ctrl+I inventory search** — specialized overlay searching only player-owned items across backpack, vaults, and all characters.
