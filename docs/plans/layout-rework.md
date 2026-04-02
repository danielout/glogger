# Layout Rework

Reorganizing the app to flow better and look better. Also move to a philosophy of the user doesn't care where the data comes from or how we got it, just always try to display the most complete, up to date, and accurate info we can across the app. Sometimes "the most" will be somewhat stale, but it is the best we can do.

## Standardized Pane Layout System ✓ COMPLETE

A unified `PaneLayout` + `SidePane` component system replaces the ad-hoc per-screen flex layouts. **All screens and tabs now use PaneLayout, and it is the required standard for new screens.** See [layout-patterns.md](../architecture/layout-patterns.md) for full documentation and usage patterns.

**Components:** `PaneLayout.vue`, `SidePane.vue`, `usePaneResize.ts`

**Capabilities:**
- Full-height panes with independent scrolling
- Collapsible side panes with vertical title strip when collapsed
- Drag-to-resize side panes (double-click to reset)
- Per-screen width and collapse state persistence via `useViewPrefs`

**Migration status:**
- [x] NpcsScreen — left+center pane layout
- [x] Crafting SkillsTab — left+center pane layout
- [x] DashboardView — center-only PaneLayout
- [x] CharacterView — center-only PaneLayout
- [x] InventoryWrapper — center-only PaneLayout
- [x] CraftingView — center-only PaneLayout
- [x] EconomicsView — center-only PaneLayout
- [x] ChatView — center-only PaneLayout
- [x] DataBrowser — center-only PaneLayout
- [x] Settings — center-only PaneLayout
- [x] Crafting ProjectsTab — center-only PaneLayout (replaces calc-based height; internal 3-pane with custom resize retained)
- [x] ItemSearch — left pane (search/filters/results) + center (detail)
- [x] SkillBrowser — left pane (search/results) + center (detail)
- [x] AbilityBrowser — left pane (filter/search/results) + center (detail)
- [x] RecipeBrowser — left pane (filter/search/results) + center (detail)
- [x] QuestBrowser — left pane (search/filters/results) + center (detail)
- [x] NpcBrowser — left pane (filter/search/results) + center (detail)
- [x] EffectBrowser — left pane (search/results) + center (detail)
- [x] TitleBrowser — left pane (search/results) + center (detail)
- [x] TsysBrowser — left pane (search/filter/results) + center (detail)

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

Implemented. See [search.md](../features/screens/search.md).