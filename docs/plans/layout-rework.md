# Layout Rework

Reorganizing the app to flow better and look better. Also move to a philosophy of the user doesn't care where the data comes from or how we got it, just always try to display the most complete, up to date, and accurate info we can across the app. Sometimes "the most" will be somewhat stale, but it is the best we can do.

## Standardized Pane Layout System

A unified `PaneLayout` + `SidePane` component system replaces the ad-hoc per-screen flex layouts. See [layout-patterns.md](../architecture/layout-patterns.md) for full documentation.

**Components:** `PaneLayout.vue`, `SidePane.vue`, `usePaneResize.ts`

**Capabilities:**
- Full-height panes with independent scrolling
- Collapsible side panes with vertical title strip when collapsed
- Drag-to-resize side panes (double-click to reset)
- Per-screen width and collapse state persistence via `useViewPrefs`

**Migration status:**
- [x] NpcsScreen — migrated to PaneLayout
- [ ] Data Browser screens (ItemSearch, NpcBrowser, etc.)
- [ ] Crafting ProjectsTab (3-pane with existing resize logic to replace)
- [ ] Other screens as needed

## Top Menu Layout

Need to consolidate, remove, add, and change some of our menu items to have a better feel.

### Left Side
```
[Glogger | glogger [log status pips] |  Data   |
  Logo]  | [Character] on [server]   | Browser |
```

### Center Menu
- Dashboard
- Character
  - Skills
  - Stats
  - NPCs
  - Quests
  - Gourmand
  - Build Planner (Coming Soon!)
- Inventory
  - Vault Overview, Currencies, Live Inventory Tracker
  - Character Storage
  - Account-Wide Storage
- Crafting
  - Quick Calc
  - Projects
  - Work Order Helper
  - Cook's Helper
  - History
- Economics
  - Market Prices (default probably)
  - Farming Sessions
    - Active Session
    - Session History
  - Surveying
    - Session
    - Session History
    - Surveying Analytics
  - Stall Tracker (Coming Soon!)
- Chat Logs

### Right Side
- Search (goes to dedicated search page)
- Settings
- '?'

## Menu Bar Functionality

The menu bar is a fixed header with two rows:

1. **Primary nav row** — logo/identity block on the left, center navigation buttons, settings gear on the right. Always visible.
2. **Sub-tab row** — appears beneath the primary row when the active view has sub-tabs. Animates in/out with a slide-down flyout effect (max-height + opacity transition, 250ms ease-out).

Sub-tab definitions live centrally in `MenuBar.vue` (`viewTabs` map). Each view that has tabs receives the active tab as a prop from App.vue rather than managing its own tab state. This keeps tabs pinned in the header during scroll and gives a consistent, fast switching experience.

Keyboard tab cycling (Q/E, Shift+Arrow) is handled at the MenuBar level.

The content area padding transitions between `pt-20` (no tabs) and `pt-28` (with tabs) to match the header height change.

Settings uses its own internal sidebar navigation pattern and is not part of the sub-tab system.

## Work Required by Screen

### Need to be created

- stall tracker
- build planner
- search page

### Need to be updated

- inventory landing page with currencies, live inventory, etc.

### Need to be consolidated

- NPCs: bring together everything we can about NPCs in regards to the player. rep levels, storage, currency to purchase items remaining, etc.
- skills: bring together what we know from the game state and display it in a useful, informative way. game state should be seeded by latest report and updated from there, so we shouldn't need the report page anymore.


## Search

Search has two modes: a **quick search overlay** (Ctrl+F) for fast lookups without leaving the current view, and a **dedicated search page** for deep exploration. Additionally, Ctrl+I provides a focused **inventory search** shortcut.

### Quick Search Overlay (Ctrl+F)

A command-palette-style modal that appears centered over the current view. Does not navigate away — results are clickable and navigate to the relevant view/tab.

**Behavior:**
- Opens on Ctrl+F (or clicking "Search" in header). Escape or clicking outside dismisses it.
- Single text input with instant results (debounced ~150ms).
- Results grouped by category with a cap per category (e.g., 5 each) to keep it scannable.
- Keyboard navigable: arrow keys to move through results, Enter to select.
- Selecting a result navigates to the appropriate view and context (e.g., clicking an item opens Data Browser > Items with that item focused; clicking a vault item opens Inventory > Vaults).

**Result categories (in display order):**

1. **Your Items** — search `gameStateStore.ownedItemCounts` + `storageByVault` + `aggregateStore.inventoryByName`. Show item name, total count, where it is (backpack/vault name/character).
2. **Your Skills** — search `gameStateStore.skills`. Show skill name, level, XP.
3. **Your Recipes** — search `gameStateStore.recipes` (known recipes). Show recipe name, skill, completion count.
4. **Game Items** — search `gameDataStore.searchItems()`. Show name, keywords. Navigates to Data Browser > Items.
5. **Game Recipes** — search `gameDataStore.searchRecipes()`. Show name, skill, level req. Navigates to Data Browser > Recipes.
6. **NPCs** — search `gameDataStore.searchNpcs()`. Show name, area. Navigates to Data Browser > NPCs.
7. **Quests** — search `gameDataStore.searchQuests()`. Show name. Navigates to Data Browser > Quests.
8. **Market Values** — search `marketStore.values` by item name. Show item, price. Navigates to Economics > Market.

Player data categories ("Your ...") always appear above game data categories so the user sees their own stuff first.

**Implementation notes:**
- Build as a composable (`useQuickSearch`) that accepts a query string and returns grouped, capped results.
- The overlay component is rendered in App.vue (sibling to the main content area) so it's available from any view.
- Use `useKeyboard` to register the Ctrl+F binding globally.
- Each result carries a navigation action: `{ view: AppView, subTab?: string, context?: any }` that the overlay calls back into `navigateToView` + the entity navigation system.

### Inventory Search (Ctrl+I)

A specialized quick overlay that only searches items the player owns — across backpack, vaults, and all characters on the server.

**Behavior:**
- Opens on Ctrl+I. Same overlay style as quick search but with a different header/icon to indicate "Inventory Search."
- Single text input, instant results.
- Searches: `gameStateStore.inventory` (backpack), `gameStateStore.storage` (vaults), `aggregateStore.inventory` (all characters on server).
- Results show: item name, stack size, location (backpack / vault name / character name), and market value if known.
- Selecting a result navigates to Inventory view with the relevant tab focused.

### Dedicated Search Page

The full search page (the "Search" nav item). For when the user wants to do a broader, more detailed exploration.

**Layout:**
- Search bar at top (auto-focused on page load).
- Filter toggles beneath the search bar: checkboxes for which categories to include (Your Items, Game Items, Recipes, NPCs, Quests, etc.). All on by default.
- Results rendered in full detail below, grouped by category with no per-category cap.
- Each category section is collapsible.
- Results use the existing inline components (ItemInline, NpcInline, etc.) for hover tooltips and click-to-navigate.

**Differences from quick search:**
- No result cap — show everything that matches.
- Richer result display (more fields, sources, descriptions).
- Filter controls for narrowing by category.
- Persists as a real page so the user can scroll through large result sets.

### Navigation from Search Results

All search results need to carry enough context to navigate the user to the right place:

| Result Type | Navigates To | Context |
|---|---|---|
| Your item (backpack) | Inventory > Inventory | highlight item |
| Your item (vault) | Inventory > Vaults | expand vault, highlight item |
| Your item (other character) | Inventory > Vaults | show aggregate view |
| Your skill | Character > Skills | scroll to skill card |
| Your recipe | Crafting > Quick Calc | pre-fill recipe |
| Game item | Data Browser > Items | focus item in search |
| Game recipe | Data Browser > Recipes | focus recipe |
| NPC | Data Browser > NPCs | focus NPC |
| Quest | Data Browser > Quests | focus quest |
| Market value | Economics > Market | scroll to item |

This uses the existing `provideEntityNavigation` system for Data Browser targets, extended with a more general `navigateWithContext(view, subTab, context)` for player-data targets.